use crate::config::Config;
use crate::core::{Collector, MonitoringService, Resolver};
use crate::process::ProcessContext;
use crate::theme::{Theme, ThemeType};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        self.status_message = None;

        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                    self.show_help = false;
                }
                _ => {}
            }
        } else if self.show_alerts {
            match key.code {
                KeyCode::Esc | KeyCode::Char('A') | KeyCode::Char('q') => {
                    self.show_alerts = false;
                }
                _ => {}
            }
        } else if self.show_historical_dialog {
            match key.code {
                KeyCode::Up => self.previous_historical_range(),
                KeyCode::Down => self.next_historical_range(),
                KeyCode::Esc | KeyCode::Char('H') => {
                    self.show_historical_dialog = false;
                }
                _ => {}
            }
        } else if self.show_threshold_dialog {
            match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    self.threshold_input.push(c);
                }
                KeyCode::Backspace => {
                    self.threshold_input.pop();
                }
                KeyCode::Enter => {
                    if let Some(i) = self.table_state.selected() {
                        if let Some(row) = self.process_data.get(i) {
                            if let Ok(val) = self.threshold_input.parse::<u64>() {
                                if val > 0 {
                                    self.monitoring
                                        .enforcement
                                        .set_threshold(crate::core::Pid(row.pid), val);
                                    self.status_message = Some(format!(
                                        "Set threshold for {} to {} KB/s",
                                        row.name, val
                                    ));
                                } else {
                                    self.monitoring
                                        .enforcement
                                        .remove_threshold(crate::core::Pid(row.pid));
                                    self.status_message =
                                        Some(format!("Removed threshold for {}", row.name));
                                }
                            }
                        }
                    }
                    self.show_threshold_dialog = false;
                    self.threshold_input.clear();
                }
                KeyCode::Esc => {
                    self.show_threshold_dialog = false;
                    self.threshold_input.clear();
                }
                _ => {}
            }
        } else if self.show_throttle_dialog {
            match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    self.throttle_input.push(c);
                }
                KeyCode::Backspace => {
                    self.throttle_input.pop();
                }
                KeyCode::Enter => {
                    if let Some(i) = self.table_state.selected() {
                        if let Some(row) = self.process_data.get(i) {
                            if let Ok(val) = self.throttle_input.parse::<u64>() {
                                if val > 0 {
                                    let _ = self.monitoring.enforcement.set_throttle(
                                        &mut self.monitoring.collector,
                                        crate::core::Pid(row.pid),
                                        val,
                                    );
                                    self.status_message =
                                        Some(format!("Throttled {} to {} KB/s", row.name, val));
                                } else {
                                    let _ = self.monitoring.enforcement.clear_throttle(
                                        &mut self.monitoring.collector,
                                        crate::core::Pid(row.pid),
                                    );
                                    self.status_message =
                                        Some(format!("Removed throttle for {}", row.name));
                                }
                            }
                        }
                    }
                    self.show_throttle_dialog = false;
                    self.throttle_input.clear();
                }
                KeyCode::Esc => {
                    self.show_throttle_dialog = false;
                    self.throttle_input.clear();
                }
                _ => {}
            }
        } else if self.show_theme_dialog {
            match key.code {
                KeyCode::Up => self.previous_theme(),
                KeyCode::Down => self.next_theme(),
                KeyCode::Enter => {
                    self.apply_theme();
                    self.show_theme_dialog = false;
                }
                KeyCode::Esc | KeyCode::Char('t') => {
                    self.show_theme_dialog = false;
                }
                _ => {}
            }
        } else if self.is_filtering {
            match key.code {
                KeyCode::Char(c) => {
                    self.filter_text.push(c);
                }
                KeyCode::Backspace => {
                    self.filter_text.pop();
                }
                KeyCode::Esc | KeyCode::Enter => {
                    self.is_filtering = false;
                }
                _ => {}
            }
        } else if self.show_kill_dialog {
            match key.code {
                KeyCode::Char('y') => {
                    if let Some(i) = self.table_state.selected() {
                        if let Some(row) = self.process_data.get(i) {
                            unsafe {
                                if libc::kill(row.pid as libc::pid_t, libc::SIGKILL) == 0 {
                                    self.status_message = Some(format!("Killed PID {}", row.pid));
                                } else {
                                    self.status_message =
                                        Some(format!("Failed to kill PID {}", row.pid));
                                }
                            }
                        }
                    }
                    self.show_kill_dialog = false;
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.show_kill_dialog = false;
                }
                _ => {}
            }
        } else if self.show_graph {
            match key.code {
                KeyCode::Esc | KeyCode::Char('g') | KeyCode::Char('q') => {
                    self.show_graph = false;
                }
                KeyCode::Char('l') => {
                    self.graph_scale_log = !self.graph_scale_log;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    if self.historical_view_mode {
                        self.historical_view_mode = false;
                        self.status_message = Some("Exited Historical View".to_string());
                    } else {
                        self.is_running = false;
                    }
                }
                KeyCode::Char(' ') if self.view_mode == ViewMode::ProcessTable => {
                    if let Some(i) = self.table_state.selected() {
                        if let Some(row) = self.process_data.get(i) {
                            if self.selected_pids.contains(&row.pid) {
                                self.selected_pids.remove(&row.pid);
                            } else {
                                self.selected_pids.insert(row.pid);
                            }
                        }
                    }
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.is_running = false;
                }
                KeyCode::Char('H') => {
                    if self.historical_view_mode {
                        self.historical_view_mode = false;
                        self.status_message = Some("Exited Historical View".to_string());
                    } else {
                        self.show_historical_dialog = true;
                    }
                }
                KeyCode::Char('/') | KeyCode::Char('f') => {
                    self.is_filtering = true;
                }
                KeyCode::Down => self.next(),
                KeyCode::Up => self.previous(),
                KeyCode::Enter => {
                    self.show_detail = !self.show_detail;
                }
                KeyCode::Char('g') => {
                    self.show_graph = true;
                }
                KeyCode::Char('k') if self.table_state.selected().is_some() => {
                    self.show_kill_dialog = true;
                }
                KeyCode::Char('a') if self.table_state.selected().is_some() => {
                    self.show_threshold_dialog = true;
                    self.threshold_input.clear();
                }
                KeyCode::Char('b') if self.table_state.selected().is_some() => {
                    self.show_throttle_dialog = true;
                    self.throttle_input.clear();
                }
                KeyCode::Char('A') => {
                    self.show_alerts = !self.show_alerts;
                }
                KeyCode::Char('t') => {
                    self.show_theme_dialog = !self.show_theme_dialog;
                }
                KeyCode::F(1) => {
                    self.view_mode = ViewMode::Dashboard;
                }
                KeyCode::F(2) => {
                    self.view_mode = ViewMode::ProcessTable;
                }
                KeyCode::F(3) => {
                    self.view_mode = ViewMode::Alerts;
                }
                KeyCode::Tab => {
                    self.view_mode = match self.view_mode {
                        ViewMode::Dashboard => ViewMode::ProcessTable,
                        ViewMode::ProcessTable => ViewMode::Alerts,
                        ViewMode::Alerts => ViewMode::Dashboard,
                    };
                }
                KeyCode::BackTab => {
                    self.view_mode = match self.view_mode {
                        ViewMode::Dashboard => ViewMode::Alerts,
                        ViewMode::ProcessTable => ViewMode::Dashboard,
                        ViewMode::Alerts => ViewMode::ProcessTable,
                    };
                }
                KeyCode::Char('s') => {
                    // Cycle sort columns
                    let next_col = match self.sort_column {
                        Column::Pid => Column::Name,
                        Column::Name => Column::Context,
                        Column::Context => Column::Up,
                        Column::Up => Column::Down,
                        Column::Down => Column::Total,
                        Column::Total => Column::Pid,
                    };
                    self.toggle_sort(next_col);
                }
                KeyCode::Char('c') => {
                    self.show_context = !self.show_context;
                    self.status_message = Some(format!(
                        "Context view: {}",
                        if self.show_context {
                            "Enabled"
                        } else {
                            "Disabled"
                        }
                    ));
                }
                KeyCode::Char('?') | KeyCode::Char('h') => {
                    self.show_help = true;
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collector::MockCollector;
    use crate::core::services::identity::MockResolver;
    use crate::core::services::MonitoringService;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn create_test_app() -> App<MockCollector, MockResolver> {
        let collector = MockCollector::new();
        let resolver = MockResolver::new();
        let monitoring = MonitoringService::new(
            collector,
            crate::core::services::IdentityService::new(resolver),
        );
        let config = Config::default();
        App::new(monitoring, HashMap::new(), config)
    }

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_app_view_transitions() {
        let mut app = create_test_app();
        assert_eq!(app.view_mode, ViewMode::Dashboard);

        app.handle_key_event(key_event(KeyCode::Tab));
        assert_eq!(app.view_mode, ViewMode::ProcessTable);

        app.handle_key_event(key_event(KeyCode::F(3)));
        assert_eq!(app.view_mode, ViewMode::Alerts);
    }

    #[test]
    fn test_app_dialog_toggles() {
        let mut app = create_test_app();

        app.handle_key_event(key_event(KeyCode::Char('?')));
        assert!(app.show_help);

        app.handle_key_event(key_event(KeyCode::Esc));
        assert!(!app.show_help);

        app.handle_key_event(key_event(KeyCode::Char('t')));
        assert!(app.show_theme_dialog);
    }

    #[test]
    fn test_app_filtering_state() {
        let mut app = create_test_app();

        app.handle_key_event(key_event(KeyCode::Char('/')));
        assert!(app.is_filtering);

        app.handle_key_event(key_event(KeyCode::Char('t')));
        app.handle_key_event(key_event(KeyCode::Char('e')));
        app.handle_key_event(key_event(KeyCode::Char('s')));
        app.handle_key_event(key_event(KeyCode::Char('t')));
        assert_eq!(app.filter_text, "test");

        app.handle_key_event(key_event(KeyCode::Enter));
        assert!(!app.is_filtering);
    }
}
