use crate::config::Config;
use crate::core::{
    Collector, MonitoringService, Resolver,
};
use crate::process::ProcessContext;
use crate::theme::{Theme, ThemeType};
use dark_light::Mode;
use ratatui::widgets::{ListState, TableState};
use std::collections::{HashMap, HashSet, VecDeque};

pub const MAX_HISTORY: usize = 100;

#[derive(PartialEq, Clone, Copy)]
pub enum Column {
    Pid,
    Name,
    Context,
    Up,
    Down,
    Total,
}

#[derive(Clone)]
pub struct ProcessRow {
    pub pid: u32,
    pub name: String,
    pub context: ProcessContext,
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
    pub country: String,
    pub isp: String,
    pub hostname: Option<String>,
    pub service: String,
}

#[derive(Clone)]
pub struct Alert {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub pid: u32,
    pub process_name: String,
    pub value: u64,     // KB/s
    pub threshold: u64, // KB/s
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TimeRange {
    TenMinutes,
    OneHour,
    TwentyFourHours,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum HistoricalRange {
    LastHour,
    Last4Hours,
    Last24Hours,
}

impl HistoricalRange {
    pub fn all() -> Vec<Self> {
        vec![
            HistoricalRange::LastHour,
            HistoricalRange::Last4Hours,
            HistoricalRange::Last24Hours,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            HistoricalRange::LastHour => "Last 1 Hour",
            HistoricalRange::Last4Hours => "Last 4 Hours",
            HistoricalRange::Last24Hours => "Last 24 Hours",
        }
    }

    pub fn to_seconds(self) -> i64 {
        match self {
            HistoricalRange::LastHour => 3600,
            HistoricalRange::Last4Hours => 14400,
            HistoricalRange::Last24Hours => 86400,
        }
    }
}

impl TimeRange {
    pub fn to_seconds(self) -> i64 {
        match self {
            TimeRange::TenMinutes => 600,
            TimeRange::OneHour => 3600,
            TimeRange::TwentyFourHours => 86400,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            TimeRange::TenMinutes => "10m",
            TimeRange::OneHour => "1h",
            TimeRange::TwentyFourHours => "24h",
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ViewMode {
    Dashboard,
    ProcessTable,
    Alerts,
}

pub struct GraphSeries {
    pub pid: u32,
    pub name: String,
    pub data_up: Vec<(f64, f64)>,
    pub data_down: Vec<(f64, f64)>,
}

pub struct App<C: Collector, R: Resolver> {
    pub monitoring: MonitoringService<C, R>,
    pub view_mode: ViewMode,
    pub process_data: Vec<ProcessRow>,
    pub total_upload: u64,
    pub total_download: u64,
    pub session_upload: u64,
    pub session_download: u64,
    pub table_state: TableState,
    pub sort_column: Column,
    pub sort_desc: bool,
    pub is_running: bool,
    pub show_kill_dialog: bool,
    pub show_detail: bool,
    pub show_graph: bool,
    pub show_help: bool,
    pub show_threshold_dialog: bool,
    pub show_throttle_dialog: bool,
    pub show_alerts: bool,
    pub show_theme_dialog: bool,
    pub show_context: bool,
    pub theme_state: ListState,
    pub current_theme: Theme,
    pub current_theme_type: ThemeType,
    pub threshold_input: String,
    pub throttle_input: String,
    pub alerts: VecDeque<Alert>,
    pub graph_time_range: TimeRange,
    pub graph_series: Vec<GraphSeries>,
    pub graph_scale_log: bool,
    pub selected_pids: HashSet<u32>,
    pub filter_text: String,
    pub is_filtering: bool,
    pub status_message: Option<String>,
    pub process_history: HashMap<u32, ProcessRow>,
    pub history_up: VecDeque<u64>,
    pub history_down: VecDeque<u64>,
    pub connections: HashMap<u32, Vec<ConnectionInfo>>, // PID -> List of connections
    pub historical_view_mode: bool,
    pub historical_start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub historical_end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub show_historical_dialog: bool,
    pub historical_range_state: ListState,
    pub historical_data: Vec<ProcessRow>,
    // Dashboard data
    pub protocol_stats: HashMap<u32, (u64, u64)>, // Proto -> (Up, Down)
    pub country_stats: HashMap<String, (u64, u64)>, // Country -> (Up, Down)
    pub config: Config,
}

impl<C: Collector, R: Resolver> App<C, R> {
    pub fn new(
        monitoring: MonitoringService<C, R>,
        historical_data: HashMap<u32, ProcessRow>,
        config: Config,
    ) -> Self {
        let mut theme_state = ListState::default();
        let themes = ThemeType::all();
        let theme_idx = themes
            .iter()
            .position(|t| *t == config.ui.theme)
            .unwrap_or(0);
        theme_state.select(Some(theme_idx));

        let mut historical_range_state = ListState::default();
        historical_range_state.select(Some(0));

        let current_theme_type = config.ui.theme;
        let current_theme = if current_theme_type == ThemeType::Auto {
            match dark_light::detect() {
                Mode::Dark => Theme::from_type(ThemeType::Default),
                Mode::Light => Theme::from_type(ThemeType::Terminal),
                Mode::Default => Theme::from_type(ThemeType::Default),
            }
        } else {
            Theme::from_type(current_theme_type)
        };

        let view_mode = match config.ui.default_view.as_str() {
            "Dashboard" => ViewMode::Dashboard,
            "ProcessTable" | "Table" => ViewMode::ProcessTable,
            "Alerts" => ViewMode::Alerts,
            _ => ViewMode::Dashboard,
        };

        Self {
            monitoring,
            view_mode,
            process_data: Vec::new(),
            total_upload: 0,
            total_download: 0,
            session_upload: 0,
            session_download: 0,
            table_state: TableState::default(),
            sort_column: Column::Up,
            sort_desc: true,
            is_running: true,
            show_kill_dialog: false,
            show_detail: false,
            show_graph: config.ui.show_graph,
            show_help: false,
            show_threshold_dialog: false,
            show_throttle_dialog: false,
            show_alerts: false,
            show_theme_dialog: false,
            show_context: false,
            theme_state,
            current_theme,
            current_theme_type,
            threshold_input: String::new(),
            throttle_input: String::new(),
            alerts: VecDeque::with_capacity(MAX_HISTORY),
            graph_time_range: TimeRange::TenMinutes,
            graph_series: Vec::new(),
            graph_scale_log: false,
            selected_pids: HashSet::new(),
            filter_text: String::new(),
            is_filtering: false,
            status_message: None,
            process_history: historical_data,
            history_up: VecDeque::with_capacity(MAX_HISTORY),
            history_down: VecDeque::with_capacity(MAX_HISTORY),
            connections: HashMap::new(),
            historical_view_mode: false,
            historical_start_time: None,
            historical_end_time: None,
            show_historical_dialog: false,
            historical_range_state,
            historical_data: Vec::new(),
            protocol_stats: HashMap::new(),
            country_stats: HashMap::new(),
            config,
        }
    }
}

impl<C: Collector, R: Resolver> App<C, R> {
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

    pub fn next_theme(&mut self) {
        let i = match self.theme_state.selected() {
            Some(i) => {
                let themes = ThemeType::all();
                if i >= themes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.theme_state.select(Some(i));
    }

    pub fn previous_theme(&mut self) {
        let i = match self.theme_state.selected() {
            Some(i) => {
                let themes = ThemeType::all();
                if i == 0 {
                    themes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.theme_state.select(Some(i));
    }

    pub fn apply_theme(&mut self) {
        if let Some(i) = self.theme_state.selected() {
            let themes = ThemeType::all();
            if let Some(t_type) = themes.get(i) {
                self.current_theme_type = *t_type;
                self.config.ui.theme = *t_type;
                if *t_type == ThemeType::Auto {
                    self.current_theme = match dark_light::detect() {
                        Mode::Dark => Theme::from_type(ThemeType::Default),
                        Mode::Light => Theme::from_type(ThemeType::Terminal),
                        Mode::Default => Theme::from_type(ThemeType::Default),
                    };
                } else {
                    self.current_theme = Theme::from_type(*t_type);
                }
            }
        }
    }

    pub fn next_historical_range(&mut self) {
        let i = match self.historical_range_state.selected() {
            Some(i) => {
                let ranges = HistoricalRange::all();
                if i >= ranges.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.historical_range_state.select(Some(i));
    }

    pub fn previous_historical_range(&mut self) {
        let i = match self.historical_range_state.selected() {
            Some(i) => {
                let ranges = HistoricalRange::all();
                if i == 0 {
                    ranges.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.historical_range_state.select(Some(i));
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
        let data = if self.historical_view_mode {
            &mut self.historical_data
        } else {
            &mut self.process_data
        };

        data.sort_by(|a, b| {
            let ordering = match self.sort_column {
                Column::Pid => a.pid.cmp(&b.pid),
                Column::Name => a.name.cmp(&b.name),
                Column::Context => a.context.label().cmp(&b.context.label()),
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
