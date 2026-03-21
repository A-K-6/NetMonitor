use crate::app::{App, Column, HistoricalRange};
use chrono::Utc;
use crate::theme::ThemeType;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Clear, Sparkline, Chart, Axis, Dataset, GraphType, List, ListItem},
    text::{Line, Span},
    symbols,
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.size();
    let theme = app.current_theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Main
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header Area
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Info
            Constraint::Percentage(30), // Up Sparkline
            Constraint::Percentage(30), // Down Sparkline
        ])
        .split(chunks[0]);

    // Header Info
    let up_kbs = app.total_upload as f64 / 1024.0;
    let down_kbs = app.total_download as f64 / 1024.0;
    
    let mut header_text = if app.historical_view_mode {
        let start = app.historical_start_time.unwrap_or(Utc::now());
        let end = app.historical_end_time.unwrap_or(Utc::now());
        format!(
            "HISTORICAL: {} - {} | UP: {:.1} KB | DOWN: {:.1} KB",
            start.format("%H:%M:%S"), end.format("%H:%M:%S"), up_kbs, down_kbs
        )
    } else {
        format!(
            "UP: {:.1} KB/s | DOWN: {:.1} KB/s",
            up_kbs, down_kbs
        )
    };

    if !app.alerts.is_empty() {
        if let Some(last) = app.alerts.back() {
            header_text.push_str(&format!(" | ALERT: {} ({}) > {} KB/s", last.process_name, last.pid, last.threshold));
        }
    }

    let header_info = Paragraph::new(header_text)
        .style(if !app.alerts.is_empty() { Style::default().fg(theme.alert_fg) } else { Style::default().fg(theme.header_fg) })
        .alignment(Alignment::Left)
        .block(Block::default()
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .title("NetMonitor")
            .border_style(Style::default().fg(theme.border_fg)));
    f.render_widget(header_info, header_chunks[0]);

    // Sparklines
    let up_data: Vec<u64> = app.history_up.iter().cloned().collect();
    let sparkline_up = Sparkline::default()
        .block(Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .title("Upload")
            .border_style(Style::default().fg(theme.border_fg)))
        .data(&up_data)
        .style(Style::default().fg(theme.upload_fg));
    f.render_widget(sparkline_up, header_chunks[1]);

    let down_data: Vec<u64> = app.history_down.iter().cloned().collect();
    let sparkline_down = Sparkline::default()
        .block(Block::default()
            .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
            .title("Download")
            .border_style(Style::default().fg(theme.border_fg)))
        .data(&down_data)
        .style(Style::default().fg(theme.download_fg));
    f.render_widget(sparkline_down, header_chunks[2]);

    // Main Content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            if app.is_filtering { Constraint::Length(3) } else { Constraint::Length(0) },
            Constraint::Min(0),
        ])
        .split(chunks[1]);

    // Search Bar
    if app.is_filtering {
        let search_text = format!(" Filter: {}_ ", app.filter_text);
        let search_bar = Paragraph::new(search_text)
            .style(Style::default().fg(theme.download_fg))
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Searching (Enter/Esc to stop)")
                .border_style(Style::default().fg(theme.border_fg)));
        f.render_widget(search_bar, main_chunks[0]);
    }

    match app.view_mode {
        crate::app::ViewMode::ProcessTable => render_process_table(f, app, main_chunks[1]),
        crate::app::ViewMode::Dashboard => render_dashboard(f, app, main_chunks[1]),
    }

    // Footer
    let footer_text = if app.is_filtering {
        "Type to filter | Enter/Esc: Finish".to_string()
    } else if app.show_graph {
        format!("Tab: Cycle Range ({}) | g/Esc: Close", app.graph_time_range.label())
    } else if app.show_historical_dialog {
        "Up/Down: Cycle | Enter: Select | Esc/H: Close".to_string()
    } else if app.show_threshold_dialog {
        "Enter: Set Threshold (KB/s) | Esc: Cancel".to_string()
    } else if app.show_theme_dialog {
        "Up/Down: Cycle | Enter: Apply | Esc/t: Close".to_string()
    } else if let Some(msg) = &app.status_message {
        format!("STATUS: {} | Press any key to clear", msg)
    } else if app.historical_view_mode {
        "q/Esc/H: Exit Historical | s: Sort | /: Filter | Enter: Details | ?: Help".to_string()
    } else if app.view_mode == crate::app::ViewMode::Dashboard {
        "Tab: Switch View | q: Quit | t: Theme | ?: Help".to_string()
    } else {
        "Tab: Switch View | q: Quit | k: Kill | s: Sort | /: Filter | Enter: Details | g: Graph | H: History | a: Alert | A: Alerts | t: Theme | ?: Help".to_string()
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(theme.status_fg))
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_fg)));
    f.render_widget(footer, chunks[2]);

    // Kill Dialog
    if app.show_kill_dialog {
        let area = centered_rect(60, 20, size);
        let pid_to_kill = app.table_state.selected().and_then(|i| app.process_data.get(i)).map(|p| p.pid);
        let text = format!("\nAre you sure you want to kill PID {:?}?\n\n(y)es / (n)o", pid_to_kill);
        let dialog = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Confirm Kill")
                .style(Style::default().fg(theme.alert_fg))
                .border_style(Style::default().fg(theme.alert_fg)))
            .alignment(Alignment::Center);
        f.render_widget(Clear, area);
        f.render_widget(dialog, area);
    }

    // Detail View
    if app.show_detail {
        render_detail_view(f, app, size);
    }

    // Graph View
    if app.show_graph {
        render_graph_view(f, app, size);
    }

    // Help Overlay
    if app.show_help {
        render_help_overlay(f, app, size);
    }

    // Threshold Dialog
    if app.show_threshold_dialog {
        let area = centered_rect(40, 20, size);
        let pid = app.table_state.selected().and_then(|i| app.process_data.get(i)).map(|p| p.pid);
        let text = format!("\nSet bandwidth threshold for PID {:?} (KB/s):\n\n {}_ ", pid, app.threshold_input);
        let dialog = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Set Threshold")
                .style(Style::default().fg(theme.download_fg))
                .border_style(Style::default().fg(theme.border_fg)))
            .alignment(Alignment::Center);
        f.render_widget(Clear, area);
        f.render_widget(dialog, area);
    }

    // Alerts Overlay
    if app.show_alerts {
         render_alerts_overlay(f, app, size);
    }

    // Theme Dialog
    if app.show_theme_dialog {
        render_theme_dialog(f, app, size);
    }

    // Historical Range Dialog
    if app.show_historical_dialog {
        render_historical_dialog(f, app, size);
    }
}

fn render_process_table(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.current_theme;
    let selected_style = Style::default()
        .fg(theme.highlight_fg)
        .bg(theme.highlight_bg)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(theme.row_fg);

    let up_label = if app.historical_view_mode { "UP (KB)" } else { "UP (KB/s)" };
    let down_label = if app.historical_view_mode { "DOWN (KB)" } else { "DOWN (KB/s)" };

    let header_cells = ["PID", "NAME", up_label, down_label, "TOTAL (KB)"]
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            let col = match i {
                0 => Column::Pid,
                1 => Column::Name,
                2 => Column::Up,
                3 => Column::Down,
                _ => Column::Total,
            };
            let mut text = h.to_string();
            if app.sort_column == col {
                text.insert_str(0, if app.sort_desc { "↓ " } else { "↑ " });
            }
            if i >= 2 {
                Cell::from(Line::from(text).alignment(Alignment::Right)).style(Style::default().fg(theme.header_fg))
            } else {
                Cell::from(text).style(Style::default().fg(theme.header_fg))
            }
        });
    let table_header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> = app.process_data.iter().map(|item| {
        let up = item.up_bytes as f64 / 1024.0;
        let down = item.down_bytes as f64 / 1024.0;
        let total = item.total_bytes as f64 / 1024.0;

        let threshold = app.thresholds.get(&item.pid);
        let exceeded = threshold.map_or(false, |&t| (up + down) > t as f64);

        let base_style = if exceeded {
            Style::default().fg(theme.alert_fg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.row_fg)
        };

        let cells = vec![
            Cell::from(item.pid.to_string()).style(base_style),
            Cell::from(item.name.clone()).style(base_style),
            Cell::from(Line::from(format!("{:.2}", up)).alignment(Alignment::Right)).style(if exceeded { base_style } else { Style::default().fg(theme.upload_fg) }),
            Cell::from(Line::from(format!("{:.2}", down)).alignment(Alignment::Right)).style(if exceeded { base_style } else { Style::default().fg(theme.download_fg) }),
            Cell::from(Line::from(format!("{:.2}", total)).alignment(Alignment::Right)).style(base_style),
        ];
        Row::new(cells).height(1).bottom_margin(0)
    }).collect();

    let widths = get_column_widths(f.size());

    let table = Table::new(rows, widths)
        .header(table_header)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(if app.historical_view_mode { "Processes (Historical)" } else { "Processes" })
            .border_style(Style::default().fg(theme.border_fg)))
        .highlight_style(selected_style)
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.current_theme;
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Top aggregates
            Constraint::Min(0),    // Bottom details
        ])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Traffic Info
            Constraint::Percentage(50), // Protocol Breakdown
        ])
        .split(chunks[0]);

    // Traffic Aggregates
    let up_kbs = app.total_upload as f64 / 1024.0;
    let down_kbs = app.total_download as f64 / 1024.0;
    let traffic_text = vec![
        Line::from(vec![
            Span::styled("Total Upload:   ", Style::default().fg(theme.header_fg)),
            Span::styled(format!("{:.1} KB/s", up_kbs), Style::default().fg(theme.upload_fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Total Download: ", Style::default().fg(theme.header_fg)),
            Span::styled(format!("{:.1} KB/s", down_kbs), Style::default().fg(theme.download_fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("System Status:  ", Style::default().fg(theme.header_fg)),
            Span::styled("Monitoring Active", Style::default().fg(theme.status_fg)),
        ]),
        Line::from(vec![
            Span::styled("Alerts Triggered: ", Style::default().fg(theme.header_fg)),
            Span::styled(app.alerts.len().to_string(), Style::default().fg(if app.alerts.is_empty() { theme.status_fg } else { theme.alert_fg })),
        ]),
    ];
    let traffic_block = Paragraph::new(traffic_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" System Summary ")
            .border_style(Style::default().fg(theme.border_fg)));
    f.render_widget(traffic_block, top_chunks[0]);

    // Protocol Distribution
    let mut protos: Vec<ListItem> = app.protocol_stats.iter().map(|(proto, (up, down))| {
        let name = match proto {
            6 => "TCP",
            17 => "UDP",
            1 => "ICMP",
            _ => "OTHER",
        };
        let total = (up + down) as f64 / 1024.0;
        ListItem::new(format!("{:<6} {:>10.1} KB total", name, total))
            .style(Style::default().fg(theme.row_fg))
    }).collect();
    if protos.is_empty() {
        protos.push(ListItem::new("No active traffic data").style(Style::default().fg(theme.status_fg)));
    }
    let proto_list = List::new(protos)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Protocol Distribution (Lifetime) ")
            .border_style(Style::default().fg(theme.border_fg)));
    f.render_widget(proto_list, top_chunks[1]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Top Talkers
            Constraint::Percentage(50), // Geo Summary
        ])
        .split(chunks[1]);

    // Top Talkers (Mini Table)
    let mut talkers = app.process_history.values().collect::<Vec<_>>();
    talkers.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
    let talker_rows: Vec<Row> = talkers.iter().take(10).map(|p| {
        Row::new(vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.name.clone()),
            Cell::from(Line::from(format!("{:.1} KB", p.total_bytes as f64 / 1024.0)).alignment(Alignment::Right)),
        ]).style(Style::default().fg(theme.row_fg))
    }).collect();
    let talker_table = Table::new(talker_rows, [
        Constraint::Length(8),
        Constraint::Min(0),
        Constraint::Length(15),
    ])
    .header(Row::new(vec!["PID", "Process", "Total Traffic"]).style(Style::default().fg(theme.header_fg)))
    .block(Block::default()
        .borders(Borders::ALL)
        .title(" Top Talkers ")
        .border_style(Style::default().fg(theme.border_fg)));
    f.render_widget(talker_table, bottom_chunks[0]);

    // Geo Summary
    let mut countries: Vec<(String, u64)> = app.country_stats.iter()
        .map(|(c, (up, down))| (c.clone(), up + down))
        .collect();
    countries.sort_by(|a, b| b.1.cmp(&a.1));
    let country_rows: Vec<Row> = countries.iter().take(10).map(|(c, total)| {
        Row::new(vec![
            Cell::from(if c.is_empty() { "Local/Unknown".to_string() } else { c.clone() }),
            Cell::from(Line::from(format!("{:.1} KB", *total as f64 / 1024.0)).alignment(Alignment::Right)),
        ]).style(Style::default().fg(theme.row_fg))
    }).collect();
    let country_table = Table::new(country_rows, [
        Constraint::Min(0),
        Constraint::Length(15),
    ])
    .header(Row::new(vec!["Country", "Total Traffic"]).style(Style::default().fg(theme.header_fg)))
    .block(Block::default()
        .borders(Borders::ALL)
        .title(" Top Destinations ")
        .border_style(Style::default().fg(theme.border_fg)));
    f.render_widget(country_table, bottom_chunks[1]);
}


pub fn get_table_rect(size: Rect, is_filtering: bool) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Main
            Constraint::Length(3), // Footer
        ])
        .split(size);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            if is_filtering { Constraint::Length(3) } else { Constraint::Length(0) },
            Constraint::Min(0),
        ])
        .split(chunks[1]);

    main_chunks[1]
}

pub fn get_footer_rect(size: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Main
            Constraint::Length(3), // Footer
        ])
        .split(size);
    chunks[2]
}

pub fn get_column_widths(size: Rect) -> Vec<Constraint> {
    if size.width < 60 {
        vec![
            Constraint::Percentage(20),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(0),
            Constraint::Percentage(0),
        ]
    } else {
        vec![
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ]
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_theme_dialog(f: &mut Frame, app: &mut App, size: Rect) {
    let area = centered_rect(30, 40, size);
    f.render_widget(Clear, area);

    let theme = &app.current_theme;
    let themes = ThemeType::all();
    let items: Vec<ListItem> = themes.iter().map(|t| {
        ListItem::new(t.name())
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Select Theme ")
            .border_style(Style::default().fg(theme.header_fg)))
        .highlight_style(Style::default()
            .fg(theme.highlight_fg)
            .bg(theme.highlight_bg)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.theme_state);
}

fn render_detail_view(f: &mut Frame, app: &App, size: Rect) {
    if let Some(i) = app.table_state.selected() {
        if let Some(row) = app.process_data.get(i) {
            let theme = &app.current_theme;
            let area = centered_rect(80, 70, size);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8), // Info
                    Constraint::Min(0),    // Connections Table
                    Constraint::Length(3), // Footer
                ])
                .split(area);

            // Info Paragraph
            let info_text = vec![
                format!("PID:        {}", row.pid),
                format!("Name:       {}", row.name),
                "".to_string(),
                format!("Upload:     {:.2} KB/s", row.up_bytes as f64 / 1024.0),
                format!("Download:   {:.2} KB/s", row.down_bytes as f64 / 1024.0),
                format!("Total Size: {:.2} KB", row.total_bytes as f64 / 1024.0),
            ].join("\n");
            let info = Paragraph::new(info_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Process Info")
                    .style(Style::default().fg(theme.header_fg))
                    .border_style(Style::default().fg(theme.border_fg)));

            // Connections Table
            let conns = app.connections.get(&row.pid);
            let conn_header = Row::new(vec![
                Cell::from("PROTO"),
                Cell::from("SERVICE"),
                Cell::from("LOCAL ADDR"),
                Cell::from("REMOTE HOST/IP"),
                Cell::from("GEO"),
                Cell::from("ISP"),
                Cell::from("UP (KB)"),
                Cell::from("DOWN (KB)"),
            ]).style(Style::default().fg(theme.header_fg)).height(1);

            let conn_rows: Vec<Row> = match conns {
                Some(list) => list.iter().map(|c| {
                    let proto = match c.proto {
                        6 => "TCP",
                        17 => "UDP",
                        1 => "ICMP",
                        _ => "RAW",
                    };
                    let remote = if let Some(host) = &c.hostname {
                        format!("{} ({})", host, c.dst_ip)
                    } else {
                        format!("{}:{}", c.dst_ip, c.dst_port)
                    };
                    Row::new(vec![
                        Cell::from(proto),
                        Cell::from(c.service.clone()),
                        Cell::from(format!("{}:{}", c.src_ip, c.src_port)),
                        Cell::from(remote),
                        Cell::from(c.country.clone()),
                        Cell::from(c.isp.clone()),
                        Cell::from(Line::from(format!("{:.1}", c.up_bytes as f64 / 1024.0)).alignment(Alignment::Right)),
                        Cell::from(Line::from(format!("{:.1}", c.down_bytes as f64 / 1024.0)).alignment(Alignment::Right)),
                    ]).height(1).style(Style::default().fg(theme.row_fg))
                }).collect(),
                None => vec![Row::new(vec![Cell::from("No active connections detected").style(Style::default().fg(theme.status_fg))])],
            };

            let conn_table = Table::new(conn_rows, [
                Constraint::Length(6),
                Constraint::Length(10),
                Constraint::Percentage(15),
                Constraint::Percentage(25),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
            ])
            .header(conn_header)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Active Connections")
                .border_style(Style::default().fg(theme.border_fg)));
            
            f.render_widget(Clear, area);
            f.render_widget(info, chunks[0]);
            f.render_widget(conn_table, chunks[1]);

            let footer = Paragraph::new("Press Enter to close")
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_fg))
                    .style(Style::default().fg(theme.status_fg)));
            f.render_widget(footer, chunks[2]);
        }
    }
}

fn render_alerts_overlay(f: &mut Frame, app: &App, size: Rect) {
    let theme = &app.current_theme;
    let area = centered_rect(80, 80, size);
    f.render_widget(Clear, area);

    let rows: Vec<Row> = app.alerts.iter().rev().map(|a| {
        Row::new(vec![
            Cell::from(a.timestamp.format("%H:%M:%S").to_string()),
            Cell::from(a.pid.to_string()),
            Cell::from(a.process_name.clone()),
            Cell::from(format!("{} KB/s", a.value)),
            Cell::from(format!("{} KB/s", a.threshold)),
        ]).style(Style::default().fg(theme.row_fg))
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(20),
        Constraint::Percentage(10),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .header(Row::new(vec!["Time", "PID", "Process", "Value", "Threshold"])
        .style(Style::default().add_modifier(Modifier::BOLD).fg(theme.alert_fg))
        .bottom_margin(1))
    .block(Block::default()
        .borders(Borders::ALL)
        .title(" Recent Alerts (Press A or Esc to close) ")
        .border_style(Style::default().fg(theme.alert_fg)));

    f.render_widget(table, area);
}

fn render_help_overlay(f: &mut Frame, app: &App, size: Rect) {
    let theme = &app.current_theme;
    let area = centered_rect(60, 65, size);
    f.render_widget(Clear, area);

    let help_text = vec![
        Row::new(vec![Cell::from("q / Esc"), Cell::from("Quit / Back")]),
        Row::new(vec![Cell::from("k"), Cell::from("Kill selected process")]),
        Row::new(vec![Cell::from("a"), Cell::from("Set bandwidth threshold for selected process")]),
        Row::new(vec![Cell::from("A"), Cell::from("View recent alerts log")]),
        Row::new(vec![Cell::from("s"), Cell::from("Cycle sort column (Pid -> Name -> Up -> Down -> Total)")]),
        Row::new(vec![Cell::from("/"), Cell::from("Filter by process name")]),
        Row::new(vec![Cell::from("Enter"), Cell::from("Deep-dive into process connections")]),
        Row::new(vec![Cell::from("g"), Cell::from("Traffic history graph (requires historical data)")]),
        Row::new(vec![Cell::from("t"), Cell::from("Theme selector")]),
        Row::new(vec![Cell::from("?"), Cell::from("Toggle this help screen")]),
        Row::new(vec![Cell::from("Up/Down"), Cell::from("Navigate process table / menus")]),
        Row::new(vec![Cell::from("Tab"), Cell::from("Cycle graph time range (when in graph view)")]),
    ];

    let help_table = Table::new(help_text, [Constraint::Percentage(30), Constraint::Percentage(70)])
        .header(Row::new(vec![Cell::from("Key"), Cell::from("Action")])
            .style(Style::default().add_modifier(Modifier::BOLD).fg(theme.help_fg))
            .bottom_margin(1))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Help / Keybindings ")
            .style(Style::default().fg(theme.row_fg))
            .border_style(Style::default().fg(theme.help_fg)));

    f.render_widget(help_table, area);
}

fn render_graph_view(f: &mut Frame, app: &App, size: Rect) {
    let theme = &app.current_theme;
    let area = centered_rect(90, 80, size);
    f.render_widget(Clear, area);

    let selected_proc = app.table_state.selected()
        .and_then(|i| app.process_data.get(i));

    let title = match selected_proc {
        Some(p) => format!(" Traffic History: {} (PID: {}) - [{}] ", p.name, p.pid, app.graph_time_range.label()),
        None => " Traffic History ".to_string(),
    };

    let up_dataset = Dataset::default()
        .name("Upload (KB/s)")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.upload_fg))
        .data(&app.graph_data_up);

    let down_dataset = Dataset::default()
        .name("Download (KB/s)")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.download_fg))
        .data(&app.graph_data_down);

    let max_up = app.graph_data_up.iter().map(|(_, v)| *v).fold(0.0, f64::max);
    let max_down = app.graph_data_down.iter().map(|(_, v)| *v).fold(0.0, f64::max);
    let max_y = (max_up.max(max_down) * 1.2).max(10.0);

    let x_bounds = [0.0, app.graph_time_range.to_seconds() as f64];
    
    let chart = Chart::new(vec![up_dataset, down_dataset])
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(theme.border_fg))
            .style(Style::default().fg(theme.row_fg)))
        .x_axis(Axis::default()
            .title("Time (s ago)")
            .style(Style::default().fg(theme.status_fg))
            .bounds(x_bounds)
            .labels(vec![
                Span::styled(format!("{:.0}", x_bounds[1]), Style::default()),
                Span::styled("0", Style::default()),
            ]))
        .y_axis(Axis::default()
            .title("KB/s")
            .style(Style::default().fg(theme.status_fg))
            .bounds([0.0, max_y])
            .labels(vec![
                Span::styled("0", Style::default()),
                Span::styled(format!("{:.1}", max_y / 2.0), Style::default()),
                Span::styled(format!("{:.1}", max_y), Style::default()),
            ]));

    f.render_widget(chart, area);
}

fn render_historical_dialog(f: &mut Frame, app: &mut App, size: Rect) {
    let area = centered_rect(30, 40, size);
    f.render_widget(Clear, area);

    let theme = &app.current_theme;
    let ranges = HistoricalRange::all();
    let items: Vec<ListItem> = ranges.iter().map(|r| {
        ListItem::new(r.label())
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Select Time Frame ")
            .border_style(Style::default().fg(theme.header_fg)))
        .highlight_style(Style::default()
            .fg(theme.highlight_fg)
            .bg(theme.highlight_bg)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.historical_range_state);
}
