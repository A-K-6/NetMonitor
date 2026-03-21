mod process;
mod app;
mod tui;
mod ui;
mod theme;
mod geoip;
mod protocol;
mod dns;
mod db;

use app::{App, Column, ProcessRow, TimeRange, HistoricalRange};
use aya::maps::HashMap;
use aya::programs::KProbe;
use aya::Ebpf;
use aya_log::EbpfLogger;
use caps::{CapSet, Capability, has_cap};
use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind, MouseButton};
use netmonitor_common::{TrafficStats, ConnectionKey};
use process::ProcessResolver;
use ratatui::layout::{Direction, Layout, Margin, Rect};
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
            match event {
                Event::Key(key) => {
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
                    } else if app.show_historical_dialog {
                        match key.code {
                            KeyCode::Up => app.previous_historical_range(),
                            KeyCode::Down => app.next_historical_range(),
                            KeyCode::Enter => {
                                if let Some(i) = app.historical_range_state.selected() {
                                    let ranges = HistoricalRange::all();
                                    if let Some(range) = ranges.get(i) {
                                        let end = Utc::now();
                                        let start = end - chrono::Duration::seconds(range.to_seconds());
                                        
                                        match db.get_aggregated_stats(start, end) {
                                            Ok(stats) => {
                                                app.historical_data = stats.into_values().collect();
                                                app.historical_view_mode = true;
                                                app.historical_start_time = Some(start);
                                                app.historical_end_time = Some(end);
                                                app.status_message = Some(format!("Historical View: {}", range.label()));
                                                app.sort_data();
                                            }
                                            Err(e) => {
                                                app.status_message = Some(format!("Error fetching stats: {}", e));
                                            }
                                        }
                                    }
                                }
                                app.show_historical_dialog = false;
                            }
                            KeyCode::Esc | KeyCode::Char('H') => {
                                app.show_historical_dialog = false;
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
                    } else if app.show_theme_dialog {
                        match key.code {
                            KeyCode::Up => app.previous_theme(),
                            KeyCode::Down => app.next_theme(),
                            KeyCode::Enter => {
                                app.apply_theme();
                                app.show_theme_dialog = false;
                            }
                            KeyCode::Esc | KeyCode::Char('t') => {
                                app.show_theme_dialog = false;
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
                                if app.historical_view_mode {
                                    app.historical_view_mode = false;
                                    app.status_message = Some("Exited Historical View".to_string());
                                } else {
                                    app.is_running = false;
                                }
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.is_running = false;
                            }
                            KeyCode::Char('H') => {
                                if app.historical_view_mode {
                                    app.historical_view_mode = false;
                                    app.status_message = Some("Exited Historical View".to_string());
                                } else {
                                    app.show_historical_dialog = true;
                                }
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
                            KeyCode::Char('t') => {
                                app.show_theme_dialog = !app.show_theme_dialog;
                            }
                            KeyCode::F(1) => {
                                app.view_mode = app::ViewMode::Dashboard;
                            }
                            KeyCode::F(2) => {
                                app.view_mode = app::ViewMode::ProcessTable;
                            }
                            KeyCode::F(3) => {
                                app.view_mode = app::ViewMode::Alerts;
                            }
                            KeyCode::Tab => {
                                app.view_mode = match app.view_mode {
                                    app::ViewMode::Dashboard => app::ViewMode::ProcessTable,
                                    app::ViewMode::ProcessTable => app::ViewMode::Alerts,
                                    app::ViewMode::Alerts => app::ViewMode::Dashboard,
                                };
                            }
                            KeyCode::BackTab => {
                                app.view_mode = match app.view_mode {
                                    app::ViewMode::Dashboard => app::ViewMode::Alerts,
                                    app::ViewMode::ProcessTable => app::ViewMode::Dashboard,
                                    app::ViewMode::Alerts => app::ViewMode::ProcessTable,
                                };
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            if app.show_theme_dialog {
                                app.previous_theme();
                            } else {
                                app.previous();
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if app.show_theme_dialog {
                                app.next_theme();
                            } else {
                                app.next();
                            }
                        }
                        MouseEventKind::Down(MouseButton::Left) => {
                            app.status_message = None;
                            let size = terminal.size().unwrap_or_default();
                            
                            if app.show_theme_dialog {
                                let area = ui::centered_rect(30, 40, size);
                                if mouse.column >= area.x && mouse.column < area.x + area.width && mouse.row >= area.y && mouse.row < area.y + area.height {
                                    let content_y = area.y + 1;
                                    if mouse.row >= content_y && mouse.row < area.y + area.height - 1 {
                                        let idx = (mouse.row - content_y) as usize;
                                        if idx < theme::ThemeType::all().len() {
                                            app.theme_state.select(Some(idx));
                                        }
                                    }
                                }
                            } else if app.show_kill_dialog {
                                let area = ui::centered_rect(60, 20, size);
                                if mouse.column >= area.x && mouse.column < area.x + area.width && mouse.row >= area.y && mouse.row < area.y + area.height {
                                    if mouse.row == area.y + 4 {
                                        let text = "(y)es / (n)o";
                                        let start_x = area.x + (area.width.saturating_sub(text.len() as u16)) / 2;
                                        if mouse.column >= start_x && mouse.column < start_x + 5 {
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
                                        } else if mouse.column >= start_x + 8 && mouse.column < start_x + 12 {
                                            app.show_kill_dialog = false;
                                        }
                                    }
                                }
                            } else if !app.show_help && !app.show_alerts && !app.show_graph && !app.show_threshold_dialog && !app.show_detail {
                                // Check Tab click
                                let tab_rect = Rect::new(0, 3, size.width, 3);
                                if mouse.row >= tab_rect.y && mouse.row < tab_rect.y + tab_rect.height {
                                    let tab_width = size.width / 3;
                                    if mouse.column < tab_width {
                                        app.view_mode = app::ViewMode::Dashboard;
                                    } else if mouse.column < tab_width * 2 {
                                        app.view_mode = app::ViewMode::ProcessTable;
                                    } else {
                                        app.view_mode = app::ViewMode::Alerts;
                                    }
                                } else if app.view_mode == app::ViewMode::ProcessTable {
                                    let table_rect = ui::get_table_rect(size, app.is_filtering);
                                    if mouse.column >= table_rect.x && mouse.column < table_rect.x + table_rect.width && mouse.row >= table_rect.y && mouse.row < table_rect.y + table_rect.height {
                                        if mouse.row == table_rect.y + 1 {
                                            // Header clicked
                                            let inner_rect = table_rect.inner(&Margin { vertical: 1, horizontal: 1 });
                                            let content_x = inner_rect.x + 3; // Shift by ">> " symbol
                                            let content_width = inner_rect.width.saturating_sub(3);
                                            let content_rect = Rect::new(content_x, inner_rect.y, content_width, 1);
                                            
                                            let widths = ui::get_column_widths(size);
                                            let col_rects = Layout::default()
                                                .direction(Direction::Horizontal)
                                                .constraints(widths)
                                                .split(content_rect);
                                            
                                            for (i, col_rect) in col_rects.iter().enumerate() {
                                                if mouse.column >= col_rect.x && mouse.column < col_rect.x + col_rect.width {
                                                    let col = match i {
                                                        0 => Column::Pid,
                                                        1 => Column::Name,
                                                        2 => Column::Up,
                                                        3 => Column::Down,
                                                        _ => Column::Total,
                                                    };
                                                    app.toggle_sort(col);
                                                    break;
                                                }
                                            }
                                        } else {
                                            let content_y = table_rect.y + 3; // border + header
                                            if mouse.row >= content_y && mouse.row < table_rect.y + table_rect.height - 1 {
                                                let offset = app.table_state.offset();
                                                let row_idx = (mouse.row - content_y) as usize + offset;
                                                if row_idx < app.process_data.len() {
                                                    app.table_state.select(Some(row_idx));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Check Footer click (only if no dialog is open)
                            if !app.show_help && !app.show_alerts && !app.show_kill_dialog && !app.show_graph && !app.show_threshold_dialog && !app.show_detail && !app.show_theme_dialog && !app.show_historical_dialog {
                                let footer_rect = ui::get_footer_rect(size);
                                if mouse.row >= footer_rect.y && mouse.row < footer_rect.y + footer_rect.height {
                                    let text = if app.historical_view_mode {
                                        "q/Esc/H: Exit Historical | s: Sort | /: Filter | Enter: Details | ?: Help".to_string()
                                    } else {
                                        match app.view_mode {
                                            app::ViewMode::Dashboard => "Tab/F1-F3: Switch | q: Quit | t: Theme | ?: Help".to_string(),
                                            app::ViewMode::ProcessTable => "Tab/F1-F3: Switch | q: Quit | k: Kill | s: Sort | /: Filter | Enter: Details | g: Graph | H: History | a: Alert | t: Theme | ?: Help".to_string(),
                                            app::ViewMode::Alerts => "Tab/F1-F3: Switch | q: Quit | t: Theme | ?: Help".to_string(),
                                        }
                                    };
                                    let start_x = footer_rect.x + (footer_rect.width.saturating_sub(text.len() as u16)) / 2;
                                    
                                    if app.historical_view_mode {
                                        if mouse.column >= start_x && mouse.column < start_x + 9 {
                                            app.historical_view_mode = false;
                                            app.status_message = Some("Exited Historical View".to_string());
                                        } else if mouse.column >= start_x + 12 && mouse.column < start_x + 19 {
                                            let next_col = match app.sort_column {
                                                Column::Pid => Column::Name,
                                                Column::Name => Column::Up,
                                                Column::Up => Column::Down,
                                                Column::Down => Column::Total,
                                                Column::Total => Column::Pid,
                                            };
                                            app.toggle_sort(next_col);
                                        } else if mouse.column >= start_x + 22 && mouse.column < start_x + 31 {
                                            app.is_filtering = true;
                                            app.filter_text.clear();
                                        } else if mouse.column >= start_x + 34 && mouse.column < start_x + 48 {
                                            if app.table_state.selected().is_some() {
                                                app.show_detail = true;
                                            }
                                        } else if mouse.column >= start_x + 51 && mouse.column < start_x + 58 {
                                            app.show_help = true;
                                        }
                                    } else {
                                        match app.view_mode {
                                            app::ViewMode::Dashboard | app::ViewMode::Alerts => {
                                                if mouse.column >= start_x && mouse.column < start_x + 15 {
                                                    // Toggle View via Tab logic (or just cycle)
                                                    app.view_mode = match app.view_mode {
                                                        app::ViewMode::Dashboard => app::ViewMode::ProcessTable,
                                                        app::ViewMode::Alerts => app::ViewMode::Dashboard,
                                                        _ => app::ViewMode::Dashboard,
                                                    };
                                                } else if mouse.column >= start_x + 18 && mouse.column < start_x + 25 {
                                                    return Ok(());
                                                } else if mouse.column >= start_x + 28 && mouse.column < start_x + 36 {
                                                    app.show_theme_dialog = !app.show_theme_dialog;
                                                } else if mouse.column >= start_x + 39 && mouse.column < start_x + 46 {
                                                    app.show_help = true;
                                                }
                                            }
                                            app::ViewMode::ProcessTable => {
                                                if mouse.column >= start_x && mouse.column < start_x + 15 {
                                                    app.view_mode = app::ViewMode::Alerts;
                                                } else if mouse.column >= start_x + 18 && mouse.column < start_x + 25 {
                                                    return Ok(());
                                                } else if mouse.column >= start_x + 28 && mouse.column < start_x + 35 {
                                                    if app.table_state.selected().is_some() {
                                                        app.show_kill_dialog = true;
                                                    }
                                                } else if mouse.column >= start_x + 38 && mouse.column < start_x + 45 {
                                                    let next_col = match app.sort_column {
                                                        Column::Pid => Column::Name,
                                                        Column::Name => Column::Up,
                                                        Column::Up => Column::Down,
                                                        Column::Down => Column::Total,
                                                        Column::Total => Column::Pid,
                                                    };
                                                    app.toggle_sort(next_col);
                                                } else if mouse.column >= start_x + 48 && mouse.column < start_x + 57 {
                                                    app.is_filtering = true;
                                                    app.filter_text.clear();
                                                } else if mouse.column >= start_x + 60 && mouse.column < start_x + 74 {
                                                    if app.table_state.selected().is_some() {
                                                        app.show_detail = true;
                                                    }
                                                } else if mouse.column >= start_x + 77 && mouse.column < start_x + 85 {
                                                    if let Some(i) = app.table_state.selected() {
                                                        if let Some(row) = app.process_data.get(i) {
                                                            app.show_graph = true;
                                                            if let Ok(history) = db.get_traffic_history(row.pid, app.graph_time_range) {
                                                                let start_ts = (Utc::now() - chrono::Duration::seconds(app.graph_time_range.to_seconds())).timestamp() as f64;
                                                                app.graph_data_up = history.iter().map(|(dt, up, _)| (dt.timestamp() as f64 - start_ts, *up as f64 / 1024.0)).collect();
                                                                app.graph_data_down = history.iter().map(|(dt, _, down)| (dt.timestamp() as f64 - start_ts, *down as f64 / 1024.0)).collect();
                                                            }
                                                        }
                                                    }
                                                } else if mouse.column >= start_x + 88 && mouse.column < start_x + 98 {
                                                    app.show_historical_dialog = true;
                                                } else if mouse.column >= start_x + 101 && mouse.column < start_x + 109 {
                                                    if app.table_state.selected().is_some() {
                                                        app.show_threshold_dialog = true;
                                                        app.threshold_input.clear();
                                                    }
                                                } else if mouse.column >= start_x + 112 && mouse.column < start_x + 120 {
                                                    app.show_theme_dialog = !app.show_theme_dialog;
                                                } else if mouse.column >= start_x + 123 && mouse.column < start_x + 130 {
                                                    app.show_help = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                        }
                        _ => {}
                    }
                }
                _ => {}
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
            if !app.historical_view_mode {
                app.connections.clear();
                app.protocol_stats.clear();
                app.country_stats.clear();
                for result in connections_map.iter() {
                    if let Ok((key, stats)) = result {
                        use std::net::{Ipv4Addr, IpAddr};
                        let dst_addr = Ipv4Addr::from(u32::from_be(key.dst_ip));
                        let src_ip = Ipv4Addr::from(u32::from_be(key.src_ip)).to_string();
                        let dst_ip_addr = IpAddr::V4(dst_addr);
                        let dst_ip = dst_addr.to_string();
                        
                        let (country, isp) = geoip::RESOLVER.resolve(dst_ip_addr);
                        let service = protocol::RESOLVER.resolve(key.proto, key.dst_port);
                        
                        // Aggregate protocol/country stats
                        let p_stats = app.protocol_stats.entry(key.proto).or_insert((0, 0));
                        p_stats.0 += stats.bytes_sent;
                        p_stats.1 += stats.bytes_recv;

                        let c_stats = app.country_stats.entry(country.clone()).or_insert((0, 0));
                        c_stats.0 += stats.bytes_sent;
                        c_stats.1 += stats.bytes_recv;

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
            }

            // 2. Clear current process_data and populate from history/historical_data with filter
            app.process_data.clear();
            let filter_lower = app.filter_text.to_lowercase();

            if app.historical_view_mode {
                for row in &app.historical_data {
                    if app.filter_text.is_empty() || row.name.to_lowercase().contains(&filter_lower) {
                        app.process_data.push(row.clone());
                    }
                }
            } else {
                for row in app.process_history.values() {
                    if app.filter_text.is_empty() || row.name.to_lowercase().contains(&filter_lower) {
                        app.process_data.push(row.clone());
                    }
                }
            }

            if !app.historical_view_mode {
                app.total_upload = current_total_up;
                app.total_download = current_total_down;

                // Update global history
                app.history_up.push_back(current_total_up);
                app.history_down.push_back(current_total_down);
                if app.history_up.len() > app::MAX_HISTORY {
                    app.history_up.pop_front();
                    app.history_down.pop_front();
                }
            } else {
                // In historical mode, total_upload/download for the header
                // will reflect the sum of the filtered historical data
                app.total_upload = app.process_data.iter().map(|p| p.up_bytes).sum();
                app.total_download = app.process_data.iter().map(|p| p.down_bytes).sum();
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
