use ratatui::widgets::TableState;
use std::collections::{HashMap, VecDeque};

pub const MAX_HISTORY: usize = 100;

#[derive(PartialEq, Clone, Copy)]
pub enum Column {
    Pid,
    Name,
    Up,
    Down,
    Total,
}

#[derive(Clone)]
pub struct ProcessRow {
    pub pid: u32,
    pub name: String,
    pub up_bytes: u64,
    pub down_bytes: u64,
    pub total_bytes: u64,
    pub last_up_bytes: u64,
    pub last_down_bytes: u64,
}

#[derive(Clone)]
pub struct ConnectionInfo {
    pub proto: u32,
    pub src_ip: String,
    pub src_port: u16,
    pub dst_ip: String,
    pub dst_port: u16,
    pub up_bytes: u64,
    pub down_bytes: u64,
}

pub struct App {
    pub process_data: Vec<ProcessRow>,
    pub total_upload: u64,
    pub total_download: u64,
    pub table_state: TableState,
    pub sort_column: Column,
    pub sort_desc: bool,
    pub is_running: bool,
    pub show_kill_dialog: bool,
    pub show_detail: bool,
    pub filter_text: String,
    pub is_filtering: bool,
    pub status_message: Option<String>,
    pub process_history: HashMap<u32, ProcessRow>,
    pub history_up: VecDeque<u64>,
    pub history_down: VecDeque<u64>,
    pub connections: HashMap<u32, Vec<ConnectionInfo>>, // PID -> List of connections
}

impl App {
    pub fn new() -> Self {
        Self {
            process_data: Vec::new(),
            total_upload: 0,
            total_download: 0,
            table_state: TableState::default(),
            sort_column: Column::Up,
            sort_desc: true,
            is_running: true,
            show_kill_dialog: false,
            show_detail: false,
            filter_text: String::new(),
            is_filtering: false,
            status_message: None,
            process_history: HashMap::new(),
            history_up: VecDeque::with_capacity(MAX_HISTORY),
            history_down: VecDeque::with_capacity(MAX_HISTORY),
            connections: HashMap::new(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.process_data.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.process_data.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn toggle_sort(&mut self, col: Column) {
        if self.sort_column == col {
            self.sort_desc = !self.sort_desc;
        } else {
            self.sort_column = col;
            self.sort_desc = true;
        }
        self.sort_data();
    }

    pub fn sort_data(&mut self) {
        self.process_data.sort_by(|a, b| {
            let ordering = match self.sort_column {
                Column::Pid => a.pid.cmp(&b.pid),
                Column::Name => a.name.cmp(&b.name),
                Column::Up => a.up_bytes.cmp(&b.up_bytes),
                Column::Down => a.down_bytes.cmp(&b.down_bytes),
                Column::Total => a.total_bytes.cmp(&b.total_bytes),
            };
            if self.sort_desc {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }
}
