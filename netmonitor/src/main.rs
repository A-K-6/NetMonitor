mod process;
mod app;
mod tui;
mod ui;
mod geoip;
mod protocol;
mod dns;
mod db;

use app::{App, Column, ProcessRow, TimeRange};
use aya::maps::HashMap;
use aya::programs::KProbe;
use aya::Ebpf;
use aya_log::EbpfLogger;
use caps::{CapSet, Capability, has_cap};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use netmonitor_common::{TrafficStats, ConnectionKey};
use process::ProcessResolver;
use std::env;
use std::time::{Duration, Instant};
use log::{error, info};
use db::DbManager;
use chrono::Utc;

fn check_caps() -> Result<(), anyhow::Error> {
    let required = [Capability::CAP_BPF, Capability::CAP_NET_ADMIN];
    for &cap in &required {
        if !has_cap(None, CapSet::Effective, cap).unwrap_or(false) {
            error!("Missing capability: {:?}. Try 'sudo setcap cap_net_admin,cap_bpf=ep {}'", cap, env::current_exe()?.display());
            return Err(anyhow::anyhow!("Insufficient permissions"));
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Check capabilities before loading
    if let Err(e) = check_caps() {
        return Err(e);
    }

    // Bump RLIMIT_MEMLOCK to allow BPF programs to load
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        eprintln!("Failed to increase rlimit RLIMIT_MEMLOCK: {}", ret);
    }

    // Load the BPF program
    let mut bpf = Ebpf::load_file("target/bpfel-unknown-none/release/netmonitor-ebpf")?;

    if let Err(e) = EbpfLogger::init(&mut bpf) {
        eprintln!("failed to initialize eBPF logger: {}", e);
    }

    let program: &mut KProbe = bpf.program_mut("tcp_sendmsg").unwrap().try_into()?;
    program.load()?;
    program.attach("tcp_sendmsg", 0)?;

    let recv_program: &mut KProbe = bpf.program_mut("tcp_cleanup_rbuf").unwrap().try_into()?;
    recv_program.load()?;
    recv_program.attach("tcp_cleanup_rbuf", 0)?;

    // Load UDP probes
    let udp_send: &mut KProbe = bpf.program_mut("udp_sendmsg").expect("udp_sendmsg not found").try_into()?;
    udp_send.load()?;
    udp_send.attach("udp_sendmsg", 0)?;

    let udp_recv: &mut KProbe = bpf.program_mut("udp_recvmsg").expect("udp_recvmsg not found").try_into()?;
    udp_recv.load()?;
    udp_recv.attach("udp_recvmsg", 0)?;

    // Load RAW probes
    let raw_send: &mut KProbe = bpf.program_mut("raw_sendmsg").expect("raw_sendmsg not found").try_into()?;
    raw_send.load()?;
    raw_send.attach("raw_sendmsg", 0)?;

    let raw_recv: &mut KProbe = bpf.program_mut("raw_recvmsg").expect("raw_recvmsg not found").try_into()?;
    raw_recv.load()?;
    raw_recv.attach("raw_recvmsg", 0)?;

    let stats_map: HashMap<_, u32, TrafficStats> = HashMap::try_from(bpf.take_map("TRAFFIC_STATS").unwrap())?;
    let connections_map: HashMap<_, ConnectionKey, TrafficStats> = 
        HashMap::try_from(bpf.take_map("CONNECTIONS").unwrap())?;

    let mut db = DbManager::new("netmonitor.db")?;
    let historical_data = db.load_historical_stats()?;
    info!("Loaded historical stats for {} processes", historical_data.len());

    let mut resolver = ProcessResolver::new(Duration::from_secs(10));
    let mut terminal = tui::Tui::new()?;
    let mut app = App::new(historical_data);

    let mut last_tick = Instant::now();
    let mut last_db_flush = Instant::now();
    let tick_rate = Duration::from_millis(1000);
    let db_flush_rate = Duration::from_secs(60);

    // Track deltas for DB flushing
    let mut db_deltas: std::collections::HashMap<u32, (u64, u64)> = std::collections::HashMap::new();

    while app.is_running {
        terminal.draw(|f| ui::render(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if let Some(event) = terminal.handle_events(timeout)? {
            if let Event::Key(key) = event {
                // Clear status message on any key press
                app.status_message = None;

                if app.show_help {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                            app.show_help = false;
                        }
                        _ => {}
                    }
                } else if app.show_alerts {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('A') | KeyCode::Char('q') => {
                            app.show_alerts = false;
                        }
                        _ => {}
                    }
                } else if app.show_threshold_dialog {
                    match key.code {
                        KeyCode::Char(c) if c.is_digit(10) => {
                            app.threshold_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.threshold_input.pop();
                        }
                        KeyCode::Enter => {
                            if let Some(i) = app.table_state.selected() {
                                if let Some(row) = app.process_data.get(i) {
                                    if let Ok(val) = app.threshold_input.parse::<u64>() {
                                        if val > 0 {
                                            app.thresholds.insert(row.pid, val);
                                            app.status_message = Some(format!("Set threshold for {} to {} KB/s", row.name, val));
                                        } else {
                                            app.thresholds.remove(&row.pid);
                                            app.status_message = Some(format!("Removed threshold for {}", row.name));
                                        }
                                    }
                                }
                            }
                            app.show_threshold_dialog = false;
                            app.threshold_input.clear();
                        }
                        KeyCode::Esc => {
                            app.show_threshold_dialog = false;
                            app.threshold_input.clear();
                        }
                        _ => {}
                    }
                } else if app.is_filtering {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.filter_text.push(c);
                        }
                        KeyCode::Backspace => {
                            app.filter_text.pop();
                        }
                        KeyCode::Esc | KeyCode::Enter => {
                            app.is_filtering = false;
                        }
                        _ => {}
                    }
                } else if app.show_kill_dialog {
                    match key.code {
                        KeyCode::Char('y') => {
                            if let Some(i) = app.table_state.selected() {
                                if let Some(row) = app.process_data.get(i) {
                                    unsafe {
                                        if libc::kill(row.pid as libc::pid_t, libc::SIGKILL) == 0 {
                                            app.status_message = Some(format!("Killed PID {}", row.pid));
                                        } else {
                                            app.status_message = Some(format!("Failed to kill PID {}", row.pid));
                                        }
                                    }
                                }
                            }
                            app.show_kill_dialog = false;
                        }
                        KeyCode::Char('n') | KeyCode::Esc => {
                            app.show_kill_dialog = false;
                        }
                        _ => {}
                    }
                } else if app.show_graph {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('g') | KeyCode::Char('q') => {
                            app.show_graph = false;
                        }
                        KeyCode::Tab => {
                            app.graph_time_range = match app.graph_time_range {
                                TimeRange::TenMinutes => TimeRange::OneHour,
                                TimeRange::OneHour => TimeRange::TwentyFourHours,
                                TimeRange::TwentyFourHours => TimeRange::TenMinutes,
                            };
                            // Refetch data
                            if let Some(i) = app.table_state.selected() {
                                if let Some(row) = app.process_data.get(i) {
                                    if let Ok(history) = db.get_traffic_history(row.pid, app.graph_time_range) {
                                        let start_ts = (Utc::now() - chrono::Duration::seconds(app.graph_time_range.to_seconds())).timestamp() as f64;
                                        app.graph_data_up = history.iter().map(|(dt, up, _)| (dt.timestamp() as f64 - start_ts, *up as f64 / 1024.0)).collect();
                                        app.graph_data_down = history.iter().map(|(dt, _, down)| (dt.timestamp() as f64 - start_ts, *down as f64 / 1024.0)).collect();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.is_running = false;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.is_running = false;
                        }
                        KeyCode::Char('/') | KeyCode::Char('f') => {
                            app.is_filtering = true;
                        }
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Enter => {
                            app.show_detail = !app.show_detail;
                        }
                        KeyCode::Char('g') => {
                            if let Some(i) = app.table_state.selected() {
                                if let Some(row) = app.process_data.get(i) {
                                    app.show_graph = true;
                                    // Fetch data
                                    if let Ok(history) = db.get_traffic_history(row.pid, app.graph_time_range) {
                                        let start_ts = (Utc::now() - chrono::Duration::seconds(app.graph_time_range.to_seconds())).timestamp() as f64;
                                        app.graph_data_up = history.iter().map(|(dt, up, _)| (dt.timestamp() as f64 - start_ts, *up as f64 / 1024.0)).collect();
                                        app.graph_data_down = history.iter().map(|(dt, _, down)| (dt.timestamp() as f64 - start_ts, *down as f64 / 1024.0)).collect();
                                    }
                                }
                            }
                        }
                        KeyCode::Char('k') => {
                            if app.table_state.selected().is_some() {
                                app.show_kill_dialog = true;
                            }
                        }
                        KeyCode::Char('s') => {
                            // Cycle sort columns
                            let next_col = match app.sort_column {
                                Column::Pid => Column::Name,
                                Column::Name => Column::Up,
                                Column::Up => Column::Down,
                                Column::Down => Column::Total,
                                Column::Total => Column::Pid,
                            };
                            app.toggle_sort(next_col);
                        }
                        KeyCode::Char('?') | KeyCode::Char('h') => {
                            app.show_help = true;
                        }
                        KeyCode::Char('a') => {
                            if app.table_state.selected().is_some() {
                                app.show_threshold_dialog = true;
                                app.threshold_input.clear();
                            }
                        }
                        KeyCode::Char('A') => {
                            app.show_alerts = !app.show_alerts;
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let mut current_total_up = 0;
            let mut current_total_down = 0;

            // Update stats
            for result in stats_map.iter() {
                if let Ok((pid, stats)) = result {
                    let name = resolver.get_process_name(pid);
                    let hist = app.process_history.entry(pid).or_insert(ProcessRow {
                        pid,
                        name: name.clone(),
                        up_bytes: 0,
                        down_bytes: 0,
                        total_bytes: 0,
                        last_up_bytes: 0,
                        last_down_bytes: 0,
                    });

                    if hist.name == "unknown" && name != "unknown" {
                        hist.name = name;
                    }

                    let up_delta = stats.bytes_sent.saturating_sub(hist.last_up_bytes);
                    let down_delta = stats.bytes_recv.saturating_sub(hist.last_down_bytes);

                    hist.up_bytes = up_delta;
                    hist.down_bytes = down_delta;
                    hist.total_bytes += up_delta + down_delta;
                    
                    hist.last_up_bytes = stats.bytes_sent;
                    hist.last_down_bytes = stats.bytes_recv;

                    current_total_up += up_delta;
                    current_total_down += down_delta;

                    // Update deltas for DB
                    let entry = db_deltas.entry(pid).or_insert((0, 0));
                    entry.0 += up_delta;
                    entry.1 += down_delta;

                    // Alert check
                    if let Some(&threshold) = app.thresholds.get(&pid) {
                        let current_rate = (up_delta + down_delta) / 1024; // KB/s (assuming 1Hz)
                        if current_rate > threshold {
                            let alert = app::Alert {
                                timestamp: Utc::now(),
                                pid,
                                process_name: hist.name.clone(),
                                value: current_rate,
                                threshold,
                            };
                            app.alerts.push_back(alert);
                            if app.alerts.len() > app::MAX_HISTORY {
                                app.alerts.pop_front();
                            }
                            app.status_message = Some(format!("ALERT: {} exceeded threshold!", hist.name));
                        }
                    }
                }
            }

            // Periodic DB Flush
            if last_db_flush.elapsed() >= db_flush_rate {
                let mut batch = Vec::new();
                for (pid, (up, down)) in db_deltas.drain() {
                    if up > 0 || down > 0 {
                        let name = app.process_history.get(&pid).map(|p| p.name.clone()).unwrap_or_else(|| "unknown".to_string());
                        batch.push((pid, name, up, down));
                    }
                }
                if !batch.is_empty() {
                    if let Err(e) = db.flush_batch(&batch) {
                        error!("Failed to flush to DB: {}", e);
                    }
                }
                last_db_flush = Instant::now();
            }

            // Update connections
            app.connections.clear();
            for result in connections_map.iter() {
                if let Ok((key, stats)) = result {
                    use std::net::{Ipv4Addr, IpAddr};
                    let dst_addr = Ipv4Addr::from(u32::from_be(key.dst_ip));
                    let src_ip = Ipv4Addr::from(u32::from_be(key.src_ip)).to_string();
                    let dst_ip_addr = IpAddr::V4(dst_addr);
                    let dst_ip = dst_addr.to_string();
                    
                    let (country, isp) = geoip::RESOLVER.resolve(dst_ip_addr);
                    let service = protocol::RESOLVER.resolve(key.proto, key.dst_port);
                    
                    // Get cached hostname or trigger resolution
                    let hostname = match dns::RESOLVER.get_cached(dst_ip_addr) {
                        Some(h) => h,
                        None => {
                            // Spawn background resolution if not in cache
                            tokio::spawn(async move {
                                dns::RESOLVER.resolve(dst_ip_addr).await;
                            });
                            None
                        }
                    };

                    let conn_info = app::ConnectionInfo {
                        proto: key.proto,
                        src_ip,
                        src_port: key.src_port,
                        dst_ip,
                        dst_port: key.dst_port,
                        up_bytes: stats.bytes_sent,
                        down_bytes: stats.bytes_recv,
                        country,
                        isp,
                        hostname,
                        service,
                    };
                    app.connections.entry(key.pid).or_default().push(conn_info);
                }
            }

            // 2. Clear current process_data and populate from history with filter
            app.process_data.clear();
            let filter_lower = app.filter_text.to_lowercase();

            for row in app.process_history.values() {
                if app.filter_text.is_empty() || row.name.to_lowercase().contains(&filter_lower) {
                    app.process_data.push(row.clone());
                }
            }

            app.total_upload = current_total_up;
            app.total_download = current_total_down;

            // Update global history
            app.history_up.push_back(current_total_up);
            app.history_down.push_back(current_total_down);
            if app.history_up.len() > app::MAX_HISTORY {
                app.history_up.pop_front();
                app.history_down.pop_front();
            }

            app.sort_data();
            last_tick = Instant::now();
        }
    }

    // Final DB Flush
    let mut batch = Vec::new();
    for (pid, (up, down)) in db_deltas.drain() {
        if up > 0 || down > 0 {
            let name = app.process_history.get(&pid).map(|p| p.name.clone()).unwrap_or_else(|| "unknown".to_string());
            batch.push((pid, name, up, down));
        }
    }
    if !batch.is_empty() {
        if let Err(e) = db.flush_batch(&batch) {
            error!("Final DB flush failed: {}", e);
        } else {
            info!("Final DB flush completed");
        }
    }

    Ok(())
}
