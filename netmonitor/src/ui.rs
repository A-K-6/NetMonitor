use crate::app::{App, Column};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Clear, Sparkline, Chart, Axis, Dataset, GraphType},
    text::{Line, Span},
    symbols,
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.size();

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
    
    let mut header_text = format!(
        "UP: {:.1} KB/s | DOWN: {:.1} KB/s",
        up_kbs, down_kbs
    );

    if !app.alerts.is_empty() {
        if let Some(last) = app.alerts.back() {
            header_text.push_str(&format!(" | ALERT: {} ({}) > {} KB/s", last.process_name, last.pid, last.threshold));
        }
    }

    let header_info = Paragraph::new(header_text)
        .style(if !app.alerts.is_empty() { Style::default().fg(Color::Red) } else { Style::default().fg(Color::Cyan) })
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM).title("NetMonitor"));
    f.render_widget(header_info, header_chunks[0]);

    // Sparklines
    let up_data: Vec<u64> = app.history_up.iter().cloned().collect();
    let sparkline_up = Sparkline::default()
        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM).title("Upload"))
        .data(&up_data)
        .style(Style::default().fg(Color::Green));
    f.render_widget(sparkline_up, header_chunks[1]);

    let down_data: Vec<u64> = app.history_down.iter().cloned().collect();
    let sparkline_down = Sparkline::default()
        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT).title("Download"))
        .data(&down_data)
        .style(Style::default().fg(Color::Yellow));
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
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Searching (Enter/Esc to stop)"));
        f.render_widget(search_bar, main_chunks[0]);
    }

    // Main table
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().fg(Color::White);

    let header_cells = ["PID", "NAME", "UP (KB/s)", "DOWN (KB/s)", "TOTAL (KB)"]
        .iter()
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
                Cell::from(Line::from(text).alignment(Alignment::Right)).style(Style::default().fg(Color::Blue))
            } else {
                Cell::from(text).style(Style::default().fg(Color::Blue))
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
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let cells = vec![
            Cell::from(item.pid.to_string()).style(base_style),
            Cell::from(item.name.clone()).style(base_style),
            Cell::from(Line::from(format!("{:.2}", up)).alignment(Alignment::Right)).style(if exceeded { base_style } else { Style::default().fg(Color::Green) }),
            Cell::from(Line::from(format!("{:.2}", down)).alignment(Alignment::Right)).style(if exceeded { base_style } else { Style::default().fg(Color::Yellow) }),
            Cell::from(Line::from(format!("{:.2}", total)).alignment(Alignment::Right)).style(base_style),
        ];
        Row::new(cells).height(1).bottom_margin(0)
    }).collect();

    // Check if terminal is small
    let widths = if size.width < 60 {
        [
            Constraint::Percentage(20),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(0),
            Constraint::Percentage(0),
        ]
    } else {
        [
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ]
    };

    let table = Table::new(rows, widths)
        .header(table_header)
        .block(Block::default().borders(Borders::ALL).title("Processes"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, main_chunks[1], &mut app.table_state);

    // Footer
    let footer_text = if app.is_filtering {
        "Type to filter | Enter/Esc: Finish".to_string()
    } else if app.show_graph {
        format!("Tab: Cycle Range ({}) | g/Esc: Close", app.graph_time_range.label())
    } else if app.show_threshold_dialog {
        "Enter: Set Threshold (KB/s) | Esc: Cancel".to_string()
    } else if let Some(msg) = &app.status_message {
        format!("STATUS: {} | Press any key to clear", msg)
    } else {
        "q: Quit | k: Kill | s: Sort | /: Filter | Enter: Details | g: Graph | a: Alert | A: Alerts | ?: Help".to_string()
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);

    // Kill Dialog
    if app.show_kill_dialog {
        let area = centered_rect(60, 20, size);
        let pid_to_kill = app.table_state.selected().and_then(|i| app.process_data.get(i)).map(|p| p.pid);
        let text = format!("\nAre you sure you want to kill PID {:?}?\n\n(y)es / (n)o", pid_to_kill);
        let dialog = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Confirm Kill").style(Style::default().fg(Color::Red)))
            .alignment(Alignment::Center);
        f.render_widget(Clear, area);
        f.render_widget(dialog, area);
    }

    // Detail View
    if app.show_detail {
        if let Some(i) = app.table_state.selected() {
            if let Some(row) = app.process_data.get(i) {
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
                    .block(Block::default().borders(Borders::ALL).title("Process Info").style(Style::default().fg(Color::Cyan)));

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
                ]).style(Style::default().fg(Color::Blue)).height(1);

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
                        ]).height(1)
                    }).collect(),
                    None => vec![Row::new(vec![Cell::from("No active connections detected").style(Style::default().fg(Color::DarkGray))])],
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
                .block(Block::default().borders(Borders::ALL).title("Active Connections"));
                
                f.render_widget(Clear, area);
                f.render_widget(info, chunks[0]);
                f.render_widget(conn_table, chunks[1]);

                let footer = Paragraph::new("Press Enter to close")
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(footer, chunks[2]);
            }
        }
    }

    // Graph View
    if app.show_graph {
        render_graph_view(f, app, size);
    }

    // Help Overlay
    if app.show_help {
        render_help_overlay(f, size);
    }

    // Threshold Dialog
    if app.show_threshold_dialog {
        let area = centered_rect(40, 20, size);
        let pid = app.table_state.selected().and_then(|i| app.process_data.get(i)).map(|p| p.pid);
        let text = format!("\nSet bandwidth threshold for PID {:?} (KB/s):\n\n {}_ ", pid, app.threshold_input);
        let dialog = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Set Threshold").style(Style::default().fg(Color::Yellow)))
            .alignment(Alignment::Center);
        f.render_widget(Clear, area);
        f.render_widget(dialog, area);
    }

    // Alerts Overlay
    if app.show_alerts {
         render_alerts_overlay(f, app, size);
    }
}

fn render_alerts_overlay(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(80, 80, size);
    f.render_widget(Clear, area);

    let rows: Vec<Row> = app.alerts.iter().rev().map(|a| {
        Row::new(vec![
            Cell::from(a.timestamp.format("%H:%M:%S").to_string()),
            Cell::from(a.pid.to_string()),
            Cell::from(a.process_name.clone()),
            Cell::from(format!("{} KB/s", a.value)),
            Cell::from(format!("{} KB/s", a.threshold)),
        ])
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(20),
        Constraint::Percentage(10),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .header(Row::new(vec!["Time", "PID", "Process", "Value", "Threshold"])
        .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Red))
        .bottom_margin(1))
    .block(Block::default().borders(Borders::ALL).title(" Recent Alerts (Press A or Esc to close) "));

    f.render_widget(table, area);
}

fn render_help_overlay(f: &mut Frame, size: Rect) {
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
        Row::new(vec![Cell::from("?"), Cell::from("Toggle this help screen")]),
        Row::new(vec![Cell::from("Up/Down"), Cell::from("Navigate process table")]),
        Row::new(vec![Cell::from("Tab"), Cell::from("Cycle graph time range (when in graph view)")]),
    ];

    let help_table = Table::new(help_text, [Constraint::Percentage(30), Constraint::Percentage(70)])
        .header(Row::new(vec![Cell::from("Key"), Cell::from("Action")])
            .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Magenta))
            .bottom_margin(1))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Help / Keybindings ")
            .style(Style::default().fg(Color::Magenta))
            .border_style(Style::default().fg(Color::Magenta)));

    f.render_widget(help_table, area);
}

fn render_graph_view(f: &mut Frame, app: &App, size: Rect) {
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
        .style(Style::default().fg(Color::Green))
        .data(&app.graph_data_up);

    let down_dataset = Dataset::default()
        .name("Download (KB/s)")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Yellow))
        .data(&app.graph_data_down);

    let max_up = app.graph_data_up.iter().map(|(_, v)| *v).fold(0.0, f64::max);
    let max_down = app.graph_data_down.iter().map(|(_, v)| *v).fold(0.0, f64::max);
    let max_y = (max_up.max(max_down) * 1.2).max(10.0);

    let x_bounds = [0.0, app.graph_time_range.to_seconds() as f64];
    
    let chart = Chart::new(vec![up_dataset, down_dataset])
        .block(Block::default().borders(Borders::ALL).title(title))
        .x_axis(Axis::default()
            .title("Time (s ago)")
            .style(Style::default().fg(Color::Gray))
            .bounds(x_bounds)
            .labels(vec![
                Span::styled(format!("{:.0}", x_bounds[1]), Style::default()),
                Span::styled("0", Style::default()),
            ]))
        .y_axis(Axis::default()
            .title("KB/s")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0, max_y])
            .labels(vec![
                Span::styled("0", Style::default()),
                Span::styled(format!("{:.1}", max_y / 2.0), Style::default()),
                Span::styled(format!("{:.1}", max_y), Style::default()),
            ]));

    f.render_widget(chart, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
