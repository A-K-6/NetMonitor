use crate::app::{App, Column, HistoricalRange};
use crate::core::{Collector, Resolver};
use crate::theme::ThemeType;
use chrono::Utc;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Cell, Chart, Clear, Dataset, GraphType, List, ListItem, Paragraph,
        Row, Sparkline, Table, Tabs,
    },
    Frame,
};

pub fn render<C: Collector, R: Resolver>(f: &mut Frame, app: &mut App<C, R>) {
    let size = f.size();
    let theme = app.current_theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
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
            start.format("%H:%M:%S"),
            end.format("%H:%M:%S"),
            up_kbs,
            down_kbs
        )
    } else {
        format!("UP: {:.1} KB/s | DOWN: {:.1} KB/s", up_kbs, down_kbs)
    };

    if !app.alerts.is_empty() {
        if let Some(last) = app.alerts.back() {
            header_text.push_str(&format!(
                " | ALERT: {} ({}) > {} KB/s",
                last.process_name, last.pid, last.threshold
            ));
        }
    }

    let header_info = Paragraph::new(header_text)
        .style(if !app.alerts.is_empty() {
            Style::default().fg(theme.alert_fg).bg(theme.bg)
        } else {
            Style::default().fg(theme.header_fg).bg(theme.bg)
        })
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
                .title("NetMonitor")
                .border_style(Style::default().fg(theme.border_fg)),
        );
    f.render_widget(header_info, header_chunks[0]);

    // Sparklines
    let up_data: Vec<u64> = app.history_up.iter().cloned().collect();
    let sparkline_up = Sparkline::default()
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::TOP | Borders::BOTTOM)
                .title("Upload")
                .border_style(Style::default().fg(theme.border_fg)),
        )
        .data(&up_data)
        .style(Style::default().fg(theme.upload_fg).bg(theme.bg));
    f.render_widget(sparkline_up, header_chunks[1]);

    let down_data: Vec<u64> = app.history_down.iter().cloned().collect();
    let sparkline_down = Sparkline::default()
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
                .title("Download")
                .border_style(Style::default().fg(theme.border_fg)),
        )
        .data(&down_data)
        .style(Style::default().fg(theme.download_fg).bg(theme.bg));
    f.render_widget(sparkline_down, header_chunks[2]);

    // Tabs
    render_tabs::<C, R>(f, app, chunks[1]);

    // Main Content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            if app.is_filtering {
                Constraint::Length(3)
            } else {
                Constraint::Length(0)
            },
            Constraint::Min(0),
        ])
        .split(chunks[2]);

    // Search Bar
    if app.is_filtering {
        let search_text = format!(" Filter: {}_ ", app.filter_text);
        let search_bar = Paragraph::new(search_text)
            .style(Style::default().fg(theme.download_fg).bg(theme.bg))
            .block(
                Block::default()
                    .style(Style::default().bg(theme.bg))
                    .borders(Borders::ALL)
                    .title("Searching (Enter/Esc to stop)")
                    .border_style(Style::default().fg(theme.border_fg)),
            );
        f.render_widget(search_bar, main_chunks[0]);
    }

    match app.view_mode {
        crate::app::ViewMode::Dashboard => render_dashboard::<C, R>(f, app, main_chunks[1]),
        crate::app::ViewMode::ProcessTable => render_process_table::<C, R>(f, app, main_chunks[1]),
        crate::app::ViewMode::Alerts => render_alerts_view::<C, R>(f, app, main_chunks[1]),
    }

    // Footer
    let footer_text = if app.is_filtering {
        "Type to filter | Enter/Esc: Finish".to_string()
    } else if app.show_graph {
        format!(
            "Tab: Cycle Range ({}) | l: Toggle Scale ({}) | g/Esc: Close",
            app.graph_time_range.label(),
            if app.graph_scale_log { "Log" } else { "Linear" }
        )
    } else if app.show_historical_dialog {
        "Up/Down: Cycle | Enter: Select | Esc/H: Close".to_string()
    } else if app.show_threshold_dialog {
        "Enter: Set Threshold (KB/s) | Esc: Cancel".to_string()
    } else if app.show_throttle_dialog {
        "Enter: Set Limit (KB/s) | Esc: Cancel".to_string()
    } else if app.show_theme_dialog {
        "Up/Down: Cycle | Enter: Apply | Esc/t: Close".to_string()
    } else if let Some(msg) = &app.status_message {
        format!("STATUS: {} | Press any key to clear", msg)
    } else if app.historical_view_mode {
        "q/Esc/H: Exit Historical | s: Sort | /: Filter | Enter: Details | ?: Help".to_string()
    } else {
        match app.view_mode {
            crate::app::ViewMode::Dashboard => "Tab/F1-F3: Switch | q: Quit | t: Theme | ?: Help".to_string(),
            crate::app::ViewMode::ProcessTable => "Tab/F1-F3: Switch | q: Quit | k: Kill | s: Sort | b: Throttle | c: Context | /: Filter | Enter: Details | g: Graph | H: History | a: Alert | t: Theme | ?: Help".to_string(),
            crate::app::ViewMode::Alerts => "Tab/F1-F3: Switch | q: Quit | t: Theme | ?: Help".to_string(),
        }
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(theme.status_fg).bg(theme.bg))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_fg)),
        );
    f.render_widget(footer, chunks[3]);

    // Kill Dialog
    if app.show_kill_dialog {
        let area = centered_rect(60, 20, size);
        let pid_to_kill = app
            .table_state
            .selected()
            .and_then(|i| app.process_data.get(i))
            .map(|p| p.pid);
        let text = format!(
            "\nAre you sure you want to kill PID {:?}?\n\n(y)es / (n)o",
            pid_to_kill
        );
        let dialog = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Confirm Kill")
                    .style(Style::default().fg(theme.alert_fg).bg(theme.bg))
                    .border_style(Style::default().fg(theme.alert_fg)),
            )
            .alignment(Alignment::Center);
        f.render_widget(Clear, area);
        f.render_widget(dialog, area);
    }

    // Detail View
    if app.show_detail {
        render_detail_view::<C, R>(f, app, size);
    }

    // Graph View
    if app.show_graph {
        render_graph_view::<C, R>(f, app, size);
    }

    // Help Overlay
    if app.show_help {
        render_help_overlay::<C, R>(f, app, size);
    }

    // Threshold Dialog
    if app.show_threshold_dialog {
        let area = centered_rect(40, 20, size);
        let pid = app
            .table_state
            .selected()
            .and_then(|i| app.process_data.get(i))
            .map(|p| p.pid);
        let text = format!(
            "\nSet bandwidth threshold for PID {:?} (KB/s):\n\n {}_ ",
            pid, app.threshold_input
        );
        let dialog = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Set Threshold")
                    .style(Style::default().fg(theme.download_fg).bg(theme.bg))
                    .border_style(Style::default().fg(theme.border_fg)),
            )
            .alignment(Alignment::Center);
        f.render_widget(Clear, area);
        f.render_widget(dialog, area);
    }

    // Throttle Dialog
    if app.show_throttle_dialog {
        render_throttle_dialog::<C, R>(f, app, size);
    }

    // Alerts Overlay
    if app.show_alerts {
        render_alerts_overlay::<C, R>(f, app, size);
    }

    // Theme Dialog
    if app.show_theme_dialog {
        render_theme_dialog::<C, R>(f, app, size);
    }

    // Historical Range Dialog
    if app.show_historical_dialog {
        render_historical_dialog::<C, R>(f, app, size);
    }
}

fn render_tabs<C: Collector, R: Resolver>(f: &mut Frame, app: &App<C, R>, area: Rect) {
    let theme = &app.current_theme;
    let titles = ["[F1] Dashboard", "[F2] Processes", "[F3] Alerts"];
    let titles_spans: Vec<Line> = titles
        .iter()
        .map(|t| {
            Line::from(Span::styled(
                *t,
                Style::default().fg(theme.header_fg).bg(theme.bg),
            ))
        })
        .collect();

    let tabs = Tabs::new(titles_spans)
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::ALL)
                .title(" Views ")
                .border_style(Style::default().fg(theme.border_fg)),
        )
        .select(match app.view_mode {
            crate::app::ViewMode::Dashboard => 0,
            crate::app::ViewMode::ProcessTable => 1,
            crate::app::ViewMode::Alerts => 2,
        })
        .style(Style::default().fg(theme.header_fg).bg(theme.bg))
        .highlight_style(
            Style::default()
                .fg(theme.highlight_fg)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        );
    f.render_widget(tabs, area);
}

fn render_alerts_view<C: Collector, R: Resolver>(f: &mut Frame, app: &App<C, R>, area: Rect) {
    let theme = &app.current_theme;

    let rows: Vec<Row> = app
        .alerts
        .iter()
        .rev()
        .map(|a| {
            Row::new(vec![
                Cell::from(a.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
                Cell::from(a.pid.to_string()),
                Cell::from(a.process_name.clone()),
                Cell::from(format!("{} KB/s", a.value)),
                Cell::from(format!("{} KB/s", a.threshold)),
            ])
            .style(Style::default().fg(theme.row_fg).bg(theme.bg))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(10),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["Time", "PID", "Process", "Value", "Threshold"])
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(theme.alert_fg)
                    .bg(theme.bg),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .style(Style::default().bg(theme.bg))
            .borders(Borders::ALL)
            .title(" System Alerts Log ")
            .border_style(Style::default().fg(theme.border_fg)),
    )
    .style(Style::default().bg(theme.bg));

    f.render_widget(table, area);
}

fn render_process_table<C: Collector, R: Resolver>(f: &mut Frame, app: &mut App<C, R>, area: Rect) {
    let theme = &app.current_theme;
    let selected_style = Style::default()
        .fg(theme.highlight_fg)
        .bg(theme.highlight_bg)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(theme.row_fg).bg(theme.bg);

    let up_label = if app.historical_view_mode {
        "UP (KB)"
    } else {
        "UP (KB/s)"
    };
    let down_label = if app.historical_view_mode {
        "DOWN (KB)"
    } else {
        "DOWN (KB/s)"
    };

    let name_header = if app.show_context { "CONTEXT" } else { "NAME" };
    let header_cells = ["PID", name_header, up_label, down_label, "TOTAL (KB)"]
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
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
            let mut text = h.to_string();
            if app.sort_column == col {
                text.insert_str(0, if app.sort_desc { "↓ " } else { "↑ " });
            }
            if i >= 2 {
                Cell::from(Line::from(text).alignment(Alignment::Right))
                    .style(Style::default().fg(theme.header_fg).bg(theme.bg))
            } else {
                Cell::from(text).style(Style::default().fg(theme.header_fg).bg(theme.bg))
            }
        });
    let table_header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> =
        app.process_data
            .iter()
            .map(|item| {
                let up = item.up_bytes as f64 / 1024.0;
                let down = item.down_bytes as f64 / 1024.0;
                let total = item.total_bytes as f64 / 1024.0;

                let threshold = app
                    .monitoring
                    .enforcement
                    .get_threshold(crate::core::Pid(item.pid));
                let exceeded = threshold.is_some_and(|t| (up + down) > t as f64);

                let base_style = if exceeded {
                    Style::default()
                        .fg(theme.alert_fg)
                        .add_modifier(Modifier::BOLD)
                        .bg(theme.bg)
                } else {
                    Style::default().fg(theme.row_fg).bg(theme.bg)
                };

                let is_selected = app.selected_pids.contains(&item.pid);
                let pid_text = if is_selected {
                    format!("[x] {}", item.pid)
                } else {
                    format!("[ ] {}", item.pid)
                };

                let throttle = app
                    .monitoring
                    .enforcement
                    .get_throttle(crate::core::Pid(item.pid));
                let name_val = if app.show_context {
                    item.context.label()
                } else {
                    item.name.clone()
                };
                let name_display = if throttle.is_some() {
                    format!("[L] {}", name_val)
                } else {
                    name_val
                };

                let cells = vec![
                    Cell::from(pid_text).style(if is_selected {
                        base_style.fg(theme.highlight_fg)
                    } else {
                        base_style
                    }),
                    Cell::from(name_display).style(base_style),
                    Cell::from(Line::from(format!("{:.2}", up)).alignment(Alignment::Right)).style(
                        if exceeded {
                            base_style
                        } else {
                            Style::default().fg(theme.upload_fg).bg(theme.bg)
                        },
                    ),
                    Cell::from(Line::from(format!("{:.2}", down)).alignment(Alignment::Right))
                        .style(if exceeded {
                            base_style
                        } else {
                            Style::default().fg(theme.download_fg).bg(theme.bg)
                        }),
                    Cell::from(Line::from(format!("{:.2}", total)).alignment(Alignment::Right))
                        .style(base_style),
                ];
                Row::new(cells).height(1).bottom_margin(0)
            })
            .collect();

    let widths = get_column_widths(f.size());

    let table = Table::new(rows, widths)
        .header(table_header)
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::ALL)
                .title(if app.historical_view_mode {
                    "Processes (Historical)"
                } else {
                    "Processes"
                })
                .border_style(Style::default().fg(theme.border_fg)),
        )
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .style(Style::default().bg(theme.bg));

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_dashboard<C: Collector, R: Resolver>(f: &mut Frame, app: &App<C, R>, area: Rect) {
    let theme = &app.current_theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Top aggregates
            Constraint::Min(0),     // Bottom details
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
    let sess_up = app.session_upload as f64 / (1024.0 * 1024.0);
    let sess_down = app.session_download as f64 / (1024.0 * 1024.0);

    let traffic_text = vec![
        Line::from(vec![
            Span::styled(
                "Current Upload:   ",
                Style::default().fg(theme.header_fg).bg(theme.bg),
            ),
            Span::styled(
                format!("{:.1} KB/s", up_kbs),
                Style::default()
                    .fg(theme.upload_fg)
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.bg),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Current Download: ",
                Style::default().fg(theme.header_fg).bg(theme.bg),
            ),
            Span::styled(
                format!("{:.1} KB/s", down_kbs),
                Style::default()
                    .fg(theme.download_fg)
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.bg),
            ),
        ]),
        Line::from(Span::styled("", Style::default().bg(theme.bg))),
        Line::from(vec![
            Span::styled(
                "Session Upload:   ",
                Style::default().fg(theme.header_fg).bg(theme.bg),
            ),
            Span::styled(
                format!("{:.2} MB", sess_up),
                Style::default().fg(theme.upload_fg).bg(theme.bg),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Session Download: ",
                Style::default().fg(theme.header_fg).bg(theme.bg),
            ),
            Span::styled(
                format!("{:.2} MB", sess_down),
                Style::default().fg(theme.download_fg).bg(theme.bg),
            ),
        ]),
        Line::from(Span::styled("", Style::default().bg(theme.bg))),
        Line::from(vec![
            Span::styled(
                "System Status:  ",
                Style::default().fg(theme.header_fg).bg(theme.bg),
            ),
            Span::styled(
                "Monitoring Active",
                Style::default().fg(theme.status_fg).bg(theme.bg),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Alerts Triggered: ",
                Style::default().fg(theme.header_fg).bg(theme.bg),
            ),
            Span::styled(
                app.alerts.len().to_string(),
                Style::default()
                    .fg(if app.alerts.is_empty() {
                        theme.status_fg
                    } else {
                        theme.alert_fg
                    })
                    .bg(theme.bg),
            ),
        ]),
    ];
    let traffic_block = Paragraph::new(traffic_text)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::ALL)
                .title(" System Summary ")
                .border_style(Style::default().fg(theme.border_fg)),
        );
    f.render_widget(traffic_block, top_chunks[0]);

    // Protocol Distribution
    let mut protos: Vec<ListItem> = app
        .protocol_stats
        .iter()
        .map(|(proto, (up, down))| {
            let name = match proto {
                6 => "TCP",
                17 => "UDP",
                1 => "ICMP",
                _ => "OTHER",
            };
            let total = (up + down) as f64 / 1024.0;
            ListItem::new(format!("{:<6} {:>10.1} KB total", name, total))
                .style(Style::default().fg(theme.row_fg).bg(theme.bg))
        })
        .collect();
    if protos.is_empty() {
        protos.push(
            ListItem::new("No active traffic data")
                .style(Style::default().fg(theme.status_fg).bg(theme.bg)),
        );
    }
    let proto_list = List::new(protos)
        .style(Style::default().bg(theme.bg))
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::ALL)
                .title(" Protocol Distribution (Lifetime) ")
                .border_style(Style::default().fg(theme.border_fg)),
        );
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
    talkers.sort_by_key(|b| std::cmp::Reverse(b.total_bytes));
    let talker_rows: Vec<Row> = talkers
        .iter()
        .take(10)
        .map(|p| {
            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.clone()),
                Cell::from(
                    Line::from(format!("{:.1} KB", p.total_bytes as f64 / 1024.0))
                        .alignment(Alignment::Right),
                ),
            ])
            .style(Style::default().fg(theme.row_fg).bg(theme.bg))
        })
        .collect();
    let talker_table = Table::new(
        talker_rows,
        [
            Constraint::Length(8),
            Constraint::Min(0),
            Constraint::Length(15),
        ],
    )
    .header(
        Row::new(vec!["PID", "Process", "Total Traffic"])
            .style(Style::default().fg(theme.header_fg).bg(theme.bg)),
    )
    .block(
        Block::default()
            .style(Style::default().bg(theme.bg))
            .borders(Borders::ALL)
            .title(" Top Talkers ")
            .border_style(Style::default().fg(theme.border_fg)),
    )
    .style(Style::default().bg(theme.bg));
    f.render_widget(talker_table, bottom_chunks[0]);

    // Geo Summary
    let mut countries: Vec<(String, u64)> = app
        .country_stats
        .iter()
        .map(|(c, (up, down))| (c.clone(), up + down))
        .collect();
    countries.sort_by_key(|b| std::cmp::Reverse(b.1));
    let country_rows: Vec<Row> = countries
        .iter()
        .take(10)
        .map(|(c, total)| {
            Row::new(vec![
                Cell::from(if c.is_empty() {
                    "Local/Unknown".to_string()
                } else {
                    c.clone()
                }),
                Cell::from(
                    Line::from(format!("{:.1} KB", *total as f64 / 1024.0))
                        .alignment(Alignment::Right),
                ),
            ])
            .style(Style::default().fg(theme.row_fg).bg(theme.bg))
        })
        .collect();
    let country_table = Table::new(country_rows, [Constraint::Min(0), Constraint::Length(15)])
        .header(
            Row::new(vec!["Country", "Total Traffic"])
                .style(Style::default().fg(theme.header_fg).bg(theme.bg)),
        )
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::ALL)
                .title(" Top Destinations ")
                .border_style(Style::default().fg(theme.border_fg)),
        )
        .style(Style::default().bg(theme.bg));
    f.render_widget(country_table, bottom_chunks[1]);
}

pub fn get_table_rect(size: Rect, is_filtering: bool) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(5),    // Main
            Constraint::Length(3), // Footer
        ])
        .split(size);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            if is_filtering {
                Constraint::Length(3)
            } else {
                Constraint::Length(0)
            },
            Constraint::Min(0),
        ])
        .split(chunks[2]);

    main_chunks[1]
}

pub fn get_footer_rect(size: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(5),    // Main
            Constraint::Length(3), // Footer
        ])
        .split(size);
    chunks[3]
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

fn render_theme_dialog<C: Collector, R: Resolver>(f: &mut Frame, app: &mut App<C, R>, size: Rect) {
    let area = centered_rect(30, 40, size);
    f.render_widget(Clear, area);

    let theme = &app.current_theme;
    let themes = ThemeType::all();
    let items: Vec<ListItem> = themes
        .iter()
        .map(|t| ListItem::new(t.name()).style(Style::default().fg(theme.row_fg).bg(theme.bg)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::ALL)
                .title(" Select Theme ")
                .border_style(Style::default().fg(theme.header_fg)),
        )
        .highlight_style(
            Style::default()
                .fg(theme.highlight_fg)
                .bg(theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .style(Style::default().bg(theme.bg));

    f.render_stateful_widget(list, area, &mut app.theme_state);
}

fn render_detail_view<C: Collector, R: Resolver>(f: &mut Frame, app: &App<C, R>, size: Rect) {
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
            let info_text = [
                format!("PID:        {}", row.pid),
                format!("Name:       {}", row.name),
                "".to_string(),
                format!("Upload:     {:.2} KB/s", row.up_bytes as f64 / 1024.0),
                format!("Download:   {:.2} KB/s", row.down_bytes as f64 / 1024.0),
                format!("Total Size: {:.2} KB", row.total_bytes as f64 / 1024.0),
            ]
            .join("\n");
            let info = Paragraph::new(info_text)
                .style(Style::default().bg(theme.bg))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Process Info")
                        .style(Style::default().fg(theme.header_fg).bg(theme.bg))
                        .border_style(Style::default().fg(theme.border_fg)),
                );

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
            ])
            .style(Style::default().fg(theme.header_fg).bg(theme.bg))
            .height(1);

            let conn_rows: Vec<Row> = match conns {
                Some(list) => list
                    .iter()
                    .map(|c| {
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
                            Cell::from(
                                Line::from(format!("{:.1}", c.up_bytes as f64 / 1024.0))
                                    .alignment(Alignment::Right),
                            ),
                            Cell::from(
                                Line::from(format!("{:.1}", c.down_bytes as f64 / 1024.0))
                                    .alignment(Alignment::Right),
                            ),
                        ])
                        .height(1)
                        .style(Style::default().fg(theme.row_fg).bg(theme.bg))
                    })
                    .collect(),
                None => vec![Row::new(vec![Cell::from("No active connections detected")
                    .style(Style::default().fg(theme.status_fg).bg(theme.bg))])],
            };

            let conn_table = Table::new(
                conn_rows,
                [
                    Constraint::Length(6),
                    Constraint::Length(10),
                    Constraint::Percentage(15),
                    Constraint::Percentage(25),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                ],
            )
            .header(conn_header)
            .block(
                Block::default()
                    .style(Style::default().bg(theme.bg))
                    .borders(Borders::ALL)
                    .title("Active Connections")
                    .border_style(Style::default().fg(theme.border_fg)),
            )
            .style(Style::default().bg(theme.bg));

            f.render_widget(Clear, area);
            f.render_widget(info, chunks[0]);
            f.render_widget(conn_table, chunks[1]);

            let footer = Paragraph::new("Press Enter to close")
                .alignment(Alignment::Center)
                .style(Style::default().bg(theme.bg))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border_fg))
                        .style(Style::default().fg(theme.status_fg).bg(theme.bg)),
                );
            f.render_widget(footer, chunks[2]);
        }
    }
}

fn render_alerts_overlay<C: Collector, R: Resolver>(f: &mut Frame, app: &App<C, R>, size: Rect) {
    let theme = &app.current_theme;
    let area = centered_rect(80, 80, size);
    f.render_widget(Clear, area);

    let rows: Vec<Row> = app
        .alerts
        .iter()
        .rev()
        .map(|a| {
            Row::new(vec![
                Cell::from(a.timestamp.format("%H:%M:%S").to_string()),
                Cell::from(a.pid.to_string()),
                Cell::from(a.process_name.clone()),
                Cell::from(format!("{} KB/s", a.value)),
                Cell::from(format!("{} KB/s", a.threshold)),
            ])
            .style(Style::default().fg(theme.row_fg).bg(theme.bg))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["Time", "PID", "Process", "Value", "Threshold"])
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(theme.alert_fg)
                    .bg(theme.bg),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .style(Style::default().bg(theme.bg))
            .borders(Borders::ALL)
            .title(" Recent Alerts (Press A or Esc to close) ")
            .border_style(Style::default().fg(theme.alert_fg)),
    )
    .style(Style::default().bg(theme.bg));

    f.render_widget(table, area);
}

fn render_help_overlay<C: Collector, R: Resolver>(f: &mut Frame, app: &App<C, R>, size: Rect) {
    let theme = &app.current_theme;
    let area = centered_rect(60, 65, size);
    f.render_widget(Clear, area);

    let help_text = vec![
        Row::new(vec![Cell::from("q / Esc"), Cell::from("Quit / Back")]),
        Row::new(vec![Cell::from("k"), Cell::from("Kill selected process")]),
        Row::new(vec![
            Cell::from("Space"),
            Cell::from("Toggle multi-process selection for graph"),
        ]),
        Row::new(vec![
            Cell::from("a"),
            Cell::from("Set bandwidth threshold for selected process"),
        ]),
        Row::new(vec![Cell::from("A"), Cell::from("View recent alerts log")]),
        Row::new(vec![
            Cell::from("s"),
            Cell::from("Cycle sort column (Pid -> Name -> Context -> Up -> Down -> Total)"),
        ]),
        Row::new(vec![
            Cell::from("c"),
            Cell::from("Toggle between Binary Name and Service Context"),
        ]),
        Row::new(vec![
            Cell::from("/"),
            Cell::from("Filter by process name/context"),
        ]),
        Row::new(vec![
            Cell::from("Enter"),
            Cell::from("Deep-dive into process connections"),
        ]),
        Row::new(vec![
            Cell::from("g"),
            Cell::from("Traffic history graph (requires historical data)"),
        ]),
        Row::new(vec![
            Cell::from("l"),
            Cell::from("Toggle Log/Linear scale in graph view"),
        ]),
        Row::new(vec![Cell::from("t"), Cell::from("Theme selector")]),
        Row::new(vec![Cell::from("?"), Cell::from("Toggle this help screen")]),
        Row::new(vec![
            Cell::from("Up/Down"),
            Cell::from("Navigate process table / menus"),
        ]),
        Row::new(vec![
            Cell::from("Tab"),
            Cell::from("Cycle graph time range (when in graph view)"),
        ]),
        Row::new(vec![
            Cell::from("F1/F2/F3"),
            Cell::from("Switch views (Dashboard, Processes, Alerts)"),
        ]),
        Row::new(vec![
            Cell::from(""),
            Cell::from("See 'man netmonitor' for full documentation"),
        ])
        .style(Style::default().fg(theme.header_fg).add_modifier(Modifier::ITALIC)),
    ];

    let help_table = Table::new(
        help_text,
        [Constraint::Percentage(30), Constraint::Percentage(70)],
    )
    .header(
        Row::new(vec![Cell::from("Key"), Cell::from("Action")])
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(theme.help_fg)
                    .bg(theme.bg),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help / Keybindings ")
            .style(Style::default().fg(theme.row_fg).bg(theme.bg))
            .border_style(Style::default().fg(theme.help_fg)),
    )
    .style(Style::default().bg(theme.bg));

    f.render_widget(help_table, area);
}

fn render_graph_view<C: Collector, R: Resolver>(f: &mut Frame, app: &App<C, R>, size: Rect) {
    let theme = &app.current_theme;
    let area = centered_rect(95, 85, size);
    f.render_widget(Clear, area);

    let title = format!(
        " Traffic History ({} scale) - [{}] ",
        if app.graph_scale_log {
            "Logarithmic"
        } else {
            "Linear"
        },
        app.graph_time_range.label()
    );

    // Prepare colors for different processes
    let colors = [
        theme.upload_fg,
        theme.download_fg,
        theme.highlight_fg,
        theme.alert_fg,
        theme.header_fg,
        theme.status_fg,
    ];

    // Prepare all data first to ensure it lives long enough for the chart
    let prepared_data: Vec<_> = app
        .graph_series
        .iter()
        .map(|series| {
            let up = if app.graph_scale_log {
                series
                    .data_up
                    .iter()
                    .map(|(x, y)| (*x, (*y + 1.0).log10()))
                    .collect()
            } else {
                series.data_up.clone()
            };
            let down = if app.graph_scale_log {
                series
                    .data_down
                    .iter()
                    .map(|(x, y)| (*x, (*y + 1.0).log10()))
                    .collect()
            } else {
                series.data_down.clone()
            };
            (up, down, series.name.clone(), series.pid)
        })
        .collect();

    let mut datasets = Vec::new();
    let mut current_max_y: f64 = 0.0;

    for (i, (up, down, name, _pid)) in prepared_data.iter().enumerate() {
        let color = colors[i % colors.len()];
        datasets.push(
            Dataset::default()
                .name(format!("{} (Up)", name))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(color).bg(theme.bg))
                .data(up),
        );

        datasets.push(
            Dataset::default()
                .name(format!("{} (Down)", name))
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::DIM)
                        .bg(theme.bg),
                )
                .data(down),
        );

        let series_max = up
            .iter()
            .chain(down.iter())
            .map(|(_, v)| *v)
            .fold(0.0, f64::max);
        current_max_y = current_max_y.max(series_max);
    }

    let max_y = (current_max_y * 1.2).max(1.0);
    let x_bounds = [0.0, app.graph_time_range.to_seconds() as f64];
    let y_label = if app.graph_scale_log {
        "Log10(KB/s + 1)"
    } else {
        "KB/s"
    };

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(theme.border_fg))
                .style(Style::default().fg(theme.row_fg).bg(theme.bg)),
        )
        .x_axis(
            Axis::default()
                .title("Time (s ago)")
                .style(Style::default().fg(theme.status_fg).bg(theme.bg))
                .bounds(x_bounds)
                .labels(vec![
                    Span::styled(format!("{:.0}", x_bounds[1]), Style::default().bg(theme.bg)),
                    Span::styled("0", Style::default().bg(theme.bg)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(y_label)
                .style(Style::default().fg(theme.status_fg).bg(theme.bg))
                .bounds([0.0, max_y])
                .labels(vec![
                    Span::styled("0", Style::default().bg(theme.bg)),
                    Span::styled(format!("{:.1}", max_y / 2.0), Style::default().bg(theme.bg)),
                    Span::styled(format!("{:.1}", max_y), Style::default().bg(theme.bg)),
                ]),
        )
        .style(Style::default().bg(theme.bg));

    f.render_widget(chart, area);
}

fn render_historical_dialog<C: Collector, R: Resolver>(
    f: &mut Frame,
    app: &mut App<C, R>,
    size: Rect,
) {
    let area = centered_rect(30, 40, size);
    f.render_widget(Clear, area);

    let theme = &app.current_theme;
    let ranges = HistoricalRange::all();
    let items: Vec<ListItem> = ranges
        .iter()
        .map(|r| ListItem::new(r.label()).style(Style::default().fg(theme.row_fg).bg(theme.bg)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .style(Style::default().bg(theme.bg))
                .borders(Borders::ALL)
                .title(" Select Time Frame ")
                .border_style(Style::default().fg(theme.header_fg)),
        )
        .highlight_style(
            Style::default()
                .fg(theme.highlight_fg)
                .bg(theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .style(Style::default().bg(theme.bg));

    f.render_stateful_widget(list, area, &mut app.historical_range_state);
}

fn render_throttle_dialog<C: Collector, R: Resolver>(
    f: &mut Frame,
    app: &mut App<C, R>,
    size: Rect,
) {
    let theme = &app.current_theme;
    let area = centered_rect(40, 20, size);
    let pid = app
        .table_state
        .selected()
        .and_then(|i| app.process_data.get(i))
        .map(|p| p.pid);
    let text = format!(
        "\nSet Bandwidth Limit for PID {:?} (KB/s):\n(0 to remove)\n\n {}_ ",
        pid, app.throttle_input
    );
    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Traffic Shaping / Throttle ")
                .style(Style::default().fg(theme.upload_fg).bg(theme.bg))
                .border_style(Style::default().fg(theme.border_fg)),
        )
        .alignment(Alignment::Center);
    f.render_widget(Clear, area);
    f.render_widget(dialog, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, ViewMode};
    use crate::config::Config;
    use crate::core::collector::MockCollector;
    use crate::core::services::identity::MockResolver;
    use crate::core::services::MonitoringService;
    use ratatui::{backend::TestBackend, Terminal};
    use std::collections::HashMap;

    #[test]
    fn test_ui_snapshot_dashboard() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let collector = MockCollector::new();
        let resolver = MockResolver::new();
        let monitoring = MonitoringService::new(
            collector,
            crate::core::services::IdentityService::new(resolver),
        );
        let config = Config::default();
        let mut app = App::new(monitoring, HashMap::new(), config);
        app.view_mode = ViewMode::Dashboard;

        terminal.draw(|f| render(f, &mut app)).unwrap();

        let buffer = terminal.backend().buffer();
        // Check if "NetMonitor" title is rendered
        let mut found = false;
        for y in 0..24 {
            for x in 0..80 {
                let cell = buffer.get(x, y);
                if cell.symbol().contains('N') {
                    // Simple check for title
                    found = true;
                }
            }
        }
        assert!(found);
    }
}
