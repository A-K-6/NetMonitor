mod app;
mod config;
mod core;
mod db;
mod dns;
mod export;
mod geoip;
mod process;
mod protocol;
mod theme;
mod tui;
mod ui;

use app::{App, Column, HistoricalRange, TimeRange};
use caps::{has_cap, CapSet, Capability};
use chrono::Utc;
use clap::Parser;
use config::Config;
use core::collector::Collector;
use core::services::MonitoringLoop;
use crossterm::event::{Event, KeyCode, MouseButton, MouseEventKind};
use daemonize::Daemonize;
use db::DbManager;
use export::Formatter;
use log::{error, info, warn};
use ratatui::layout::{Direction, Layout, Margin, Rect};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Run in headless mode (no TUI)
    #[arg(long)]
    headless: bool,

    /// Run as a background daemon
    #[arg(short, long)]
    daemon: bool,

    /// Path to the PID file (daemon mode only)
    #[arg(long)]
    pid_file: Option<PathBuf>,

    /// Path to the SQLite database
    #[arg(long)]
    db_path: Option<PathBuf>,

    /// Output format (json, csv, plain)
    #[arg(short, long, default_value = "plain")]
    output: String,

    /// Interval between snapshots in seconds (headless/daemon mode only)
    #[arg(short, long, default_value = "1")]
    interval: u64,

    /// Number of snapshots to take before exiting (headless mode only)
    #[arg(short = 'n', long)]
    count: Option<usize>,

    /// Path to a file to log snapshots (headless mode only)
    #[arg(long)]
    log_file: Option<PathBuf>,

    /// Verify traffic accuracy in a temporary network namespace
    #[arg(long)]
    verify_accuracy: bool,
}

fn resolve_db_path(db_arg: Option<PathBuf>, daemon: bool) -> PathBuf {
    if let Some(path) = db_arg {
        return path;
    }

    if daemon {
        let path = PathBuf::from("/var/lib/netmonitor/netmonitor.db");
        if let Some(parent) = path.parent() {
            if parent.exists() {
                return path;
            }
        }
    }

    // Default to local directory if /var/lib doesn't exist or not in daemon mode
    PathBuf::from("netmonitor.db")
}

fn check_caps() -> Result<(), anyhow::Error> {
    let required = [Capability::CAP_BPF, Capability::CAP_NET_ADMIN];
    for &cap in &required {
        if !has_cap(None, CapSet::Effective, cap).unwrap_or(false) {
            error!(
                "Missing capability: {:?}. Try 'sudo setcap cap_net_admin,cap_bpf=ep {}'",
                cap,
                env::current_exe()?.display()
            );
            return Err(anyhow::anyhow!("Insufficient permissions"));
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = Args::parse();
    let (config, config_path) = Config::load(args.config);

    // Check capabilities before loading
    check_caps()?;

    // Bump RLIMIT_MEMLOCK to allow BPF programs to load
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        warn!("Failed to increase rlimit RLIMIT_MEMLOCK: {}", ret);
    }

    // Load the BPF program via AyaCollector
    let collector = match core::collector::AyaCollector::new() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to initialize eBPF collector: {}", e);
            return Err(e);
        }
    };

    let resolver = core::services::identity::LocalResolver::new(Duration::from_secs(10));
    let identity_service = core::services::IdentityService::new(resolver);
    let monitoring = core::services::MonitoringService::new(collector, identity_service);

    let db_path = resolve_db_path(args.db_path, args.daemon);
    let mut db = DbManager::new(&db_path)?;
    let historical_data = db.load_historical_stats()?;
    info!(
        "Loaded historical stats for {} processes from {}",
        historical_data.len(),
        db_path.display()
    );

    if args.verify_accuracy {
        println!("Verification mode active. Monitoring for 3 seconds...");
        let mut monitoring = monitoring;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(3) {
            monitoring.collector.collect_stats()?;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        // Report stats
        let stats = monitoring.collector.collect_stats()?;
        for (pid, stat) in stats {
            if stat.bytes_sent > 0 || stat.bytes_recv > 0 {
                println!(
                    "PID {}: Sent {} bytes, Recv {} bytes",
                    pid, stat.bytes_sent, stat.bytes_recv
                );
            }
        }
        return Ok(());
    }

    if args.daemon {
        let mut daemonize = Daemonize::new()
            .pid_file(
                args.pid_file
                    .unwrap_or_else(|| PathBuf::from("/run/netmonitor/netmonitor.pid")),
            )
            .chown_pid_file(true)
            .working_directory("/tmp");

        if let Some(log_path) = &args.log_file {
            let stdout = fs::File::create(log_path)?;
            let stderr = fs::File::create(log_path)?;
            daemonize = daemonize.stdout(stdout).stderr(stderr);
        }

        match daemonize.start() {
            Ok(_) => {
                info!("NetMonitor daemon started.");
                let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

                // Signal Handling for Daemon
                let mut sigterm =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
                let mut sigint =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
                let mut sighup =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?;

                let shutdown_tx_clone = shutdown_tx.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        _ = sigterm.recv() => { info!("SIGTERM received"); },
                        _ = sigint.recv() => { info!("SIGINT received"); },
                    }
                    let _ = shutdown_tx_clone.send(());
                });

                tokio::spawn(async move {
                    loop {
                        let _ = sighup.recv().await;
                        info!("SIGHUP received, reloading config (not fully implemented yet)");
                        // TODO: Implement config reload logic
                    }
                });

                let mut monitoring_loop =
                    MonitoringLoop::new(monitoring, db, config, Duration::from_secs(args.interval));

                monitoring_loop.run(shutdown_rx, None, args.count).await?;
                return Ok(());
            }
            Err(e) => {
                error!("Failed to daemonize: {}", e);
                return Err(e.into());
            }
        }
    }

    if args.headless {
        let formatter: Box<dyn Formatter> = match args.output.as_str() {
            "json" => Box::new(export::JsonFormatter),
            "csv" => Box::new(export::CsvFormatter {
                include_header: true,
            }),
            _ => Box::new(export::PlainFormatter),
        };

        let output_writer: Box<dyn std::io::Write + Send> = if let Some(path) = args.log_file {
            Box::new(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?,
            )
        } else {
            Box::new(std::io::stdout())
        };

        println!(
            "NetMonitor running in headless mode (output: {}, interval: {}s).",
            args.output, args.interval
        );

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                let _ = shutdown_tx.send(());
            }
        });

        let mut monitoring_loop =
            MonitoringLoop::new(monitoring, db, config, Duration::from_secs(args.interval));

        monitoring_loop
            .run(shutdown_rx, Some((formatter, output_writer)), args.count)
            .await?;
        return Ok(());
    }

    let mut terminal = tui::Tui::new()?;
    let mut app = App::new(monitoring, historical_data, config);

    if let Some(path) = &config_path {
        if path.exists() {
            app.status_message = Some(format!("Loaded config from {}", path.display()));
        } else {
            app.status_message = Some(format!("Created new config at {}", path.display()));
        }
    }

    let mut last_tick = Instant::now();
    let mut last_db_flush = Instant::now();
    let tick_rate = Duration::from_millis(app.config.ui.refresh_rate);
    let db_flush_rate = Duration::from_secs(60);

    // Track deltas for DB flushing
    let mut db_deltas: std::collections::HashMap<u32, (u64, u64)> =
        std::collections::HashMap::new();

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

                    if app.show_historical_dialog && key.code == KeyCode::Enter {
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
                                        app.status_message =
                                            Some(format!("Historical View: {}", range.label()));
                                        app.sort_data();
                                    }
                                    Err(e) => {
                                        app.status_message =
                                            Some(format!("Error fetching stats: {}", e));
                                    }
                                }
                            }
                        }
                        app.show_historical_dialog = false;
                    } else if (app.show_graph && key.code == KeyCode::Tab)
                        || (!app.show_graph && key.code == KeyCode::Char('g'))
                    {
                        if key.code == KeyCode::Tab {
                            app.graph_time_range = match app.graph_time_range {
                                TimeRange::TenMinutes => TimeRange::OneHour,
                                TimeRange::OneHour => TimeRange::TwentyFourHours,
                                TimeRange::TwentyFourHours => TimeRange::TenMinutes,
                            };
                        } else {
                            app.show_graph = true;
                        }

                        // Fetch data for all selected PIDs
                        app.graph_series.clear();
                        let mut pids_to_fetch =
                            app.selected_pids.iter().cloned().collect::<Vec<_>>();
                        if pids_to_fetch.is_empty() {
                            if let Some(i) = app.table_state.selected() {
                                if let Some(row) = app.process_data.get(i) {
                                    pids_to_fetch.push(row.pid);
                                }
                            }
                        }

                        for pid in pids_to_fetch {
                            if let Ok(history) = db.get_traffic_history(pid, app.graph_time_range) {
                                let name = app
                                    .process_history
                                    .get(&pid)
                                    .map(|p| p.name.clone())
                                    .unwrap_or_else(|| "unknown".to_string());
                                let start_ts = (Utc::now()
                                    - chrono::Duration::seconds(app.graph_time_range.to_seconds()))
                                .timestamp() as f64;

                                app.graph_series.push(app::GraphSeries {
                                    pid,
                                    name,
                                    data_up: history
                                        .iter()
                                        .map(|(dt, up, _)| {
                                            (dt.timestamp() as f64 - start_ts, *up as f64 / 1024.0)
                                        })
                                        .collect(),
                                    data_down: history
                                        .iter()
                                        .map(|(dt, _, down)| {
                                            (
                                                dt.timestamp() as f64 - start_ts,
                                                *down as f64 / 1024.0,
                                            )
                                        })
                                        .collect(),
                                });
                            }
                        }
                    } else {
                        app.handle_key_event(key);
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
                                if mouse.column >= area.x
                                    && mouse.column < area.x + area.width
                                    && mouse.row >= area.y
                                    && mouse.row < area.y + area.height
                                {
                                    let content_y = area.y + 1;
                                    if mouse.row >= content_y
                                        && mouse.row < area.y + area.height - 1
                                    {
                                        let idx = (mouse.row - content_y) as usize;
                                        if idx < theme::ThemeType::all().len() {
                                            app.theme_state.select(Some(idx));
                                        }
                                    }
                                }
                            } else if app.show_kill_dialog {
                                let area = ui::centered_rect(60, 20, size);
                                if mouse.column >= area.x
                                    && mouse.column < area.x + area.width
                                    && mouse.row >= area.y
                                    && mouse.row < area.y + area.height
                                    && mouse.row == area.y + 4
                                {
                                    let text = "(y)es / (n)o";
                                    let start_x =
                                        area.x + (area.width.saturating_sub(text.len() as u16)) / 2;
                                    if mouse.column >= start_x && mouse.column < start_x + 5 {
                                        if let Some(i) = app.table_state.selected() {
                                            if let Some(row) = app.process_data.get(i) {
                                                unsafe {
                                                    if libc::kill(
                                                        row.pid as libc::pid_t,
                                                        libc::SIGKILL,
                                                    ) == 0
                                                    {
                                                        app.status_message =
                                                            Some(format!("Killed PID {}", row.pid));
                                                    } else {
                                                        app.status_message = Some(format!(
                                                            "Failed to kill PID {}",
                                                            row.pid
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                        app.show_kill_dialog = false;
                                    } else if mouse.column >= start_x + 8
                                        && mouse.column < start_x + 12
                                    {
                                        app.show_kill_dialog = false;
                                    }
                                }
                            } else if !app.show_help
                                && !app.show_alerts
                                && !app.show_graph
                                && !app.show_threshold_dialog
                                && !app.show_throttle_dialog
                                && !app.show_detail
                            {
                                // Check Tab click
                                let tab_rect = Rect::new(0, 3, size.width, 3);
                                if mouse.row >= tab_rect.y
                                    && mouse.row < tab_rect.y + tab_rect.height
                                {
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
                                    if mouse.column >= table_rect.x
                                        && mouse.column < table_rect.x + table_rect.width
                                        && mouse.row >= table_rect.y
                                        && mouse.row < table_rect.y + table_rect.height
                                    {
                                        if mouse.row == table_rect.y + 1 {
                                            // Header clicked
                                            let inner_rect = table_rect.inner(&Margin {
                                                vertical: 1,
                                                horizontal: 1,
                                            });
                                            let content_x = inner_rect.x + 3; // Shift by ">> " symbol
                                            let content_width = inner_rect.width.saturating_sub(3);
                                            let content_rect = Rect::new(
                                                content_x,
                                                inner_rect.y,
                                                content_width,
                                                1,
                                            );

                                            let widths = ui::get_column_widths(size);
                                            let col_rects = Layout::default()
                                                .direction(Direction::Horizontal)
                                                .constraints(widths)
                                                .split(content_rect);

                                            for (i, col_rect) in col_rects.iter().enumerate() {
                                                if mouse.column >= col_rect.x
                                                    && mouse.column < col_rect.x + col_rect.width
                                                {
                                                    let col = match i {
                                                        0 => Column::Pid,
                                                        1 => {
                                                            if app.show_context {
                                                                Column::Context
                                                            } else {
                                                                Column::Name
                                                            }
                                                        }
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
                                            if mouse.row >= content_y
                                                && mouse.row < table_rect.y + table_rect.height - 1
                                            {
                                                let offset = app.table_state.offset();
                                                let row_idx =
                                                    (mouse.row - content_y) as usize + offset;
                                                if row_idx < app.process_data.len() {
                                                    app.table_state.select(Some(row_idx));
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Check Footer click (only if no dialog is open)
                            if !app.show_help
                                && !app.show_alerts
                                && !app.show_kill_dialog
                                && !app.show_graph
                                && !app.show_threshold_dialog
                                && !app.show_throttle_dialog
                                && !app.show_detail
                                && !app.show_theme_dialog
                                && !app.show_historical_dialog
                            {
                                let footer_rect = ui::get_footer_rect(size);
                                if mouse.row >= footer_rect.y
                                    && mouse.row < footer_rect.y + footer_rect.height
                                {
                                    let text = if app.historical_view_mode {
                                        "q/Esc/H: Exit Historical | s: Sort | /: Filter | Enter: Details | ?: Help".to_string()
                                    } else {
                                        match app.view_mode {
                                            app::ViewMode::Dashboard => "Tab/F1-F3: Switch | q: Quit | t: Theme | ?: Help".to_string(),
                                            app::ViewMode::ProcessTable => "Tab/F1-F3: Switch | q: Quit | k: Kill | s: Sort | /: Filter | Enter: Details | g: Graph | H: History | a: Alert | t: Theme | ?: Help".to_string(),
                                            app::ViewMode::Alerts => "Tab/F1-F3: Switch | q: Quit | t: Theme | ?: Help".to_string(),
                                        }
                                    };
                                    let start_x = footer_rect.x
                                        + (footer_rect.width.saturating_sub(text.len() as u16)) / 2;

                                    if app.historical_view_mode {
                                        if mouse.column >= start_x && mouse.column < start_x + 9 {
                                            app.historical_view_mode = false;
                                            app.status_message =
                                                Some("Exited Historical View".to_string());
                                        } else if mouse.column >= start_x + 12
                                            && mouse.column < start_x + 19
                                        {
                                            let next_col = match app.sort_column {
                                                Column::Pid => Column::Name,
                                                Column::Name => Column::Context,
                                                Column::Context => Column::Up,
                                                Column::Up => Column::Down,
                                                Column::Down => Column::Total,
                                                Column::Total => Column::Pid,
                                            };
                                            app.toggle_sort(next_col);
                                        } else if mouse.column >= start_x + 22
                                            && mouse.column < start_x + 31
                                        {
                                            app.is_filtering = true;
                                            app.filter_text.clear();
                                        } else if mouse.column >= start_x + 34
                                            && mouse.column < start_x + 48
                                        {
                                            if app.table_state.selected().is_some() {
                                                app.show_detail = true;
                                            }
                                        } else if mouse.column >= start_x + 51
                                            && mouse.column < start_x + 58
                                        {
                                            app.show_help = true;
                                        }
                                    } else {
                                        match app.view_mode {
                                            app::ViewMode::Dashboard | app::ViewMode::Alerts => {
                                                if mouse.column >= start_x
                                                    && mouse.column < start_x + 15
                                                {
                                                    // Toggle View via Tab logic (or just cycle)
                                                    app.view_mode = match app.view_mode {
                                                        app::ViewMode::Dashboard => {
                                                            app::ViewMode::ProcessTable
                                                        }
                                                        app::ViewMode::Alerts => {
                                                            app::ViewMode::Dashboard
                                                        }
                                                        _ => app::ViewMode::Dashboard,
                                                    };
                                                } else if mouse.column >= start_x + 18
                                                    && mouse.column < start_x + 25
                                                {
                                                    return Ok(());
                                                } else if mouse.column >= start_x + 28
                                                    && mouse.column < start_x + 36
                                                {
                                                    app.show_theme_dialog = !app.show_theme_dialog;
                                                } else if mouse.column >= start_x + 39
                                                    && mouse.column < start_x + 46
                                                {
                                                    app.show_help = true;
                                                }
                                            }
                                            app::ViewMode::ProcessTable => {
                                                if mouse.column >= start_x
                                                    && mouse.column < start_x + 15
                                                {
                                                    app.view_mode = app::ViewMode::Alerts;
                                                } else if mouse.column >= start_x + 18
                                                    && mouse.column < start_x + 25
                                                {
                                                    return Ok(());
                                                } else if mouse.column >= start_x + 28
                                                    && mouse.column < start_x + 35
                                                {
                                                    if app.table_state.selected().is_some() {
                                                        app.show_kill_dialog = true;
                                                    }
                                                } else if mouse.column >= start_x + 38
                                                    && mouse.column < start_x + 45
                                                {
                                                    let next_col = match app.sort_column {
                                                        Column::Pid => Column::Name,
                                                        Column::Name => Column::Context,
                                                        Column::Context => Column::Up,
                                                        Column::Up => Column::Down,
                                                        Column::Down => Column::Total,
                                                        Column::Total => Column::Pid,
                                                    };
                                                    app.toggle_sort(next_col);
                                                } else if mouse.column >= start_x + 48
                                                    && mouse.column < start_x + 59
                                                {
                                                    if app.table_state.selected().is_some() {
                                                        app.show_throttle_dialog = true;
                                                        app.throttle_input.clear();
                                                    }
                                                } else if mouse.column >= start_x + 62
                                                    && mouse.column < start_x + 71
                                                {
                                                    app.is_filtering = true;
                                                    app.filter_text.clear();
                                                } else if mouse.column >= start_x + 74
                                                    && mouse.column < start_x + 88
                                                {
                                                    if app.table_state.selected().is_some() {
                                                        app.show_detail = true;
                                                    }
                                                } else if mouse.column >= start_x + 91
                                                    && mouse.column < start_x + 99
                                                {
                                                    if let Some(i) = app.table_state.selected() {
                                                        if let Some(row) = app.process_data.get(i) {
                                                            app.show_graph = true;
                                                            app.graph_series.clear();
                                                            let mut pids_to_fetch = app
                                                                .selected_pids
                                                                .iter()
                                                                .cloned()
                                                                .collect::<Vec<_>>();
                                                            if pids_to_fetch.is_empty() {
                                                                pids_to_fetch.push(row.pid);
                                                            }
                                                            for pid in pids_to_fetch {
                                                                if let Ok(history) = db
                                                                    .get_traffic_history(
                                                                        pid,
                                                                        app.graph_time_range,
                                                                    )
                                                                {
                                                                    let name = app
                                                                        .process_history
                                                                        .get(&pid)
                                                                        .map(|p| p.name.clone())
                                                                        .unwrap_or_else(|| {
                                                                            "unknown".to_string()
                                                                        });
                                                                    let start_ts = (Utc::now()
                                                                        - chrono::Duration::seconds(
                                                                            app.graph_time_range
                                                                                .to_seconds(),
                                                                        ))
                                                                    .timestamp()
                                                                        as f64;
                                                                    app.graph_series
                                                                        .push(app::GraphSeries {
                                                                        pid,
                                                                        name,
                                                                        data_up: history
                                                                            .iter()
                                                                            .map(|(dt, up, _)| {
                                                                                (
                                                                                    dt.timestamp()
                                                                                        as f64
                                                                                        - start_ts,
                                                                                    *up as f64
                                                                                        / 1024.0,
                                                                                )
                                                                            })
                                                                            .collect(),
                                                                        data_down: history
                                                                            .iter()
                                                                            .map(|(dt, _, down)| {
                                                                                (
                                                                                    dt.timestamp()
                                                                                        as f64
                                                                                        - start_ts,
                                                                                    *down as f64
                                                                                        / 1024.0,
                                                                                )
                                                                            })
                                                                            .collect(),
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else if mouse.column >= start_x + 102
                                                    && mouse.column < start_x + 112
                                                {
                                                    app.show_historical_dialog = true;
                                                } else if mouse.column >= start_x + 115
                                                    && mouse.column < start_x + 123
                                                {
                                                    if app.table_state.selected().is_some() {
                                                        app.show_threshold_dialog = true;
                                                        app.threshold_input.clear();
                                                    }
                                                } else if mouse.column >= start_x + 126
                                                    && mouse.column < start_x + 134
                                                {
                                                    app.show_theme_dialog = !app.show_theme_dialog;
                                                } else if mouse.column >= start_x + 137
                                                    && mouse.column < start_x + 144
                                                {
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
            match app.monitoring.snapshot(
                app.config.network.dns_resolution,
                app.config.network.geo_ip_enabled,
            ) {
                Ok(snapshot) => {
                    app.total_upload = snapshot.total_up.0;
                    app.total_download = snapshot.total_down.0;
                    app.session_upload = snapshot.session_up.0;
                    app.session_download = snapshot.session_down.0;

                    // Update global history for graphs
                    app.history_up.push_back(snapshot.total_up.0);
                    app.history_down.push_back(snapshot.total_down.0);
                    if app.history_up.len() > app::MAX_HISTORY {
                        app.history_up.pop_front();
                        app.history_down.pop_front();
                    }

                    for proc in snapshot.processes {
                        let hist =
                            app.process_history
                                .entry(proc.pid.0)
                                .or_insert(app::ProcessRow {
                                    pid: proc.pid.0,
                                    name: proc.name.clone(),
                                    context: proc.context.clone(),
                                    up_bytes: 0,
                                    down_bytes: 0,
                                    total_bytes: 0,
                                    last_up_bytes: 0,
                                    last_down_bytes: 0,
                                });

                        let up_delta = proc.up_rate.0;
                        let down_delta = proc.down_rate.0;

                        hist.up_bytes = up_delta;
                        hist.down_bytes = down_delta;
                        hist.total_bytes += up_delta + down_delta;
                        // Use proc.total.0 for consistency if we wanted, but we keep total_bytes as sum over app lifetime
                        hist.last_up_bytes = proc.up.0;
                        hist.last_down_bytes = proc.down.0;

                        // Update deltas for DB
                        let entry = db_deltas.entry(proc.pid.0).or_insert((0, 0));
                        entry.0 += up_delta;
                        entry.1 += down_delta;

                        // Alert check
                        let threshold = app
                            .monitoring
                            .enforcement
                            .get_threshold(core::Pid(proc.pid.0))
                            .or_else(|| app.config.alerts.processes.get(&hist.name).cloned())
                            .unwrap_or(app.config.alerts.default_threshold);

                        let current_rate = (up_delta + down_delta) / 1024;
                        if current_rate > threshold && threshold > 0 {
                            app.alerts.push_back(app::Alert {
                                timestamp: snapshot.timestamp,
                                pid: proc.pid.0,
                                process_name: hist.name.clone(),
                                value: current_rate,
                                threshold,
                            });
                            if app.alerts.len() > app::MAX_HISTORY {
                                app.alerts.pop_front();
                            }
                        }
                    }

                    app.protocol_stats = snapshot
                        .protocol_stats
                        .into_iter()
                        .map(|(k, v)| (k, (v.0 .0, v.1 .0)))
                        .collect();
                    app.country_stats = snapshot
                        .country_stats
                        .into_iter()
                        .map(|(k, v)| (k, (v.0 .0, v.1 .0)))
                        .collect();

                    app.connections.clear();
                    for (pid, conns) in snapshot.connections {
                        app.connections.insert(
                            pid,
                            conns
                                .into_iter()
                                .map(|c| app::ConnectionInfo {
                                    proto: c.proto,
                                    src_ip: c.src_ip,
                                    src_port: c.src_port,
                                    dst_ip: c.dst_ip,
                                    dst_port: c.dst_port,
                                    up_bytes: c.up.0,
                                    down_bytes: c.down.0,
                                    country: c.country,
                                    isp: c.isp,
                                    hostname: c.hostname,
                                    service: c.service,
                                })
                                .collect(),
                        );
                    }
                }
                Err(e) => error!("Snapshot failed: {}", e),
            }

            // Periodic DB Flush
            if last_db_flush.elapsed() >= db_flush_rate {
                let mut batch = Vec::new();
                for (pid, (up, down)) in db_deltas.drain() {
                    if up > 0 || down > 0 {
                        let name = app
                            .process_history
                            .get(&pid)
                            .map(|p| p.name.clone())
                            .unwrap_or_else(|| "unknown".to_string());
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

            // Populate process_data for UI (filtering)
            app.process_data.clear();
            let filter_lower = app.filter_text.to_lowercase();

            if app.historical_view_mode {
                for row in &app.historical_data {
                    if app.filter_text.is_empty()
                        || row.name.to_lowercase().contains(&filter_lower)
                        || row.context.label().to_lowercase().contains(&filter_lower)
                    {
                        app.process_data.push(row.clone());
                    }
                }
            } else {
                for row in app.process_history.values() {
                    if app.filter_text.is_empty()
                        || row.name.to_lowercase().contains(&filter_lower)
                        || row.context.label().to_lowercase().contains(&filter_lower)
                    {
                        app.process_data.push(row.clone());
                    }
                }
            }

            if app.historical_view_mode {
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
            let name = app
                .process_history
                .get(&pid)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "unknown".to_string());
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

    // Save config on exit
    if let Some(path) = config_path {
        if let Err(e) = app.config.save(&path) {
            error!("Failed to save config: {}", e);
        } else {
            info!("Config saved to {}", path.display());
        }
    }

    Ok(())
}
