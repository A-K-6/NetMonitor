use crate::app::{App, Column};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Clear},
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

    // Header
    let up_kbs = app.total_upload as f64 / 1024.0;
    let down_kbs = app.total_download as f64 / 1024.0;
    
    let header_text = format!(
        "UP: {:.2} KB/s | DOWN: {:.2} KB/s | Kernel: eBPF CO-RE",
        up_kbs, down_kbs
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("NetMonitor"));
    f.render_widget(header, chunks[0]);

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
            Cell::from(text).style(Style::default().fg(Color::Blue))
        });
    let table_header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> = app.process_data.iter().map(|item| {
        let up = item.up_bytes as f64 / 1024.0;
        let down = item.down_bytes as f64 / 1024.0;
        let total = item.total_bytes as f64 / 1024.0;

        let cells = vec![
            Cell::from(item.pid.to_string()),
            Cell::from(item.name.clone()),
            Cell::from(format!("{:.2}", up)).style(Style::default().fg(Color::Green)),
            Cell::from(format!("{:.2}", down)).style(Style::default().fg(Color::Yellow)),
            Cell::from(format!("{:.2}", total)),
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
        "Type to filter | Enter/Esc: Finish"
    } else {
        "q: Quit | k: Kill | s: Sort | /: Filter | Enter: Deep-dive"
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
        let text = format!("Are you sure you want to kill PID {:?}? (y/n)", pid_to_kill);
        let dialog = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Confirm Kill").style(Style::default().fg(Color::Red)))
            .alignment(Alignment::Center);
        f.render_widget(Clear, area);
        f.render_widget(dialog, area);
    }
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
