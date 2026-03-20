mod process;
mod app;
mod tui;
mod ui;

use app::{App, Column, ProcessRow};
use aya::maps::HashMap;
use aya::programs::KProbe;
use aya::Ebpf;
use aya_log::EbpfLogger;
use caps::{CapSet, Capability, has_cap};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use netmonitor_common::TrafficStats;
use process::ProcessResolver;
use std::env;
use std::time::{Duration, Instant};
use log::error;

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

    let stats_map: HashMap<_, u32, TrafficStats> = HashMap::try_from(bpf.map_mut("TRAFFIC_STATS").unwrap())?;

    let mut resolver = ProcessResolver::new(Duration::from_secs(10));
    let mut terminal = tui::Tui::new()?;
    let mut app = App::new();

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(1000);

    while app.is_running {
        terminal.draw(|f| ui::render(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if let Some(event) = terminal.handle_events(timeout)? {
            if let Event::Key(key) = event {
                if app.is_filtering {
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
                                if let Some(_row) = app.process_data.get(i) {
                                    // Simulated kill
                                }
                            }
                            app.show_kill_dialog = false;
                        }
                        KeyCode::Char('n') | KeyCode::Esc => {
                            app.show_kill_dialog = false;
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
                        KeyCode::Char('k') => app.show_kill_dialog = true,
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
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let mut current_total_up = 0;
            let mut current_total_down = 0;

            // 1. Update history with latest from BPF map
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

                    // Ensure name is up to date (it might change from "unknown" to something else)
                    if hist.name == "unknown" && name != "unknown" {
                        hist.name = name;
                    }

                    // Calculate deltas (rates)
                    // If eBPF values are total since start, delta is current - last
                    let up_delta = stats.bytes_sent.saturating_sub(hist.last_up_bytes);
                    let down_delta = stats.bytes_recv.saturating_sub(hist.last_down_bytes);

                    hist.up_bytes = up_delta;
                    hist.down_bytes = down_delta;
                    hist.total_bytes = stats.bytes_sent + stats.bytes_recv;
                    
                    hist.last_up_bytes = stats.bytes_sent;
                    hist.last_down_bytes = stats.bytes_recv;

                    current_total_up += up_delta;
                    current_total_down += down_delta;
                }
            }

            // 2. Clear current process_data and populate from history with filter
            app.process_data.clear();
            let filter_lower = app.filter_text.to_lowercase();

            for row in app.process_history.values() {
                if app.filter_text.is_empty() || row.name.to_lowercase().contains(&filter_lower) {
                    // Only show rows that have activity or are specifically filtered?
                    // Let's show everything that matches the filter.
                    app.process_data.push(row.clone());
                }
            }

            app.total_upload = current_total_up;
            app.total_download = current_total_down;
            app.sort_data();

            last_tick = Instant::now();
        }
    }

    Ok(())
}
