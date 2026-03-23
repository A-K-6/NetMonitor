use crate::app::ProcessRow;
use crate::core::collector::Collector;
use crate::core::types::{Bytes, Pid};
use crate::db::DbManager;
use anyhow::Result;
use chrono::{DateTime, Utc};
use netmonitor_common::{ConnectionKey, TrafficStats};
use std::collections::{HashMap, VecDeque};

pub trait Storage {
    fn save_stats(&mut self, pid: Pid, up: Bytes, down: Bytes) -> Result<()>;
    fn load_historical(
        &mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<HashMap<Pid, ProcessRow>>;
}

pub struct DbStorage {
    inner: DbManager,
}

impl DbStorage {
    pub fn new(path: &str) -> Result<Self> {
        Ok(Self {
            inner: DbManager::new(path)?,
        })
    }
}

// I'll need to update DbManager or bridge it.
// For now let's assume simple bridge or update later.

pub struct TrafficService {
    current_stats: HashMap<Pid, TrafficStats>,
    connections: HashMap<ConnectionKey, TrafficStats>,
    history_up: VecDeque<Bytes>,
    history_down: VecDeque<Bytes>,
    total_upload: Bytes,
    total_download: Bytes,
    max_history: usize,
}

impl TrafficService {
    pub fn new() -> Self {
        Self {
            current_stats: HashMap::new(),
            connections: HashMap::new(),
            history_up: VecDeque::with_capacity(100),
            history_down: VecDeque::with_capacity(100),
            total_upload: Bytes(0),
            total_download: Bytes(0),
            max_history: 100,
        }
    }

    pub fn update(&mut self, collector: &mut dyn Collector) -> Result<()> {
        let stats = collector.collect_stats()?;
        let connections = collector.collect_connections()?;
        // ... rest of logic

        let mut current_up = 0;
        let mut current_down = 0;

        for traffic in stats.values() {
            current_up += traffic.bytes_sent;
            current_down += traffic.bytes_recv;
        }

        self.total_upload = Bytes(current_up);
        self.total_download = Bytes(current_down);

        self.history_up.push_back(self.total_upload);
        self.history_down.push_back(self.total_download);

        if self.history_up.len() > self.max_history {
            self.history_up.pop_front();
            self.history_down.pop_front();
        }

        self.current_stats = stats.into_iter().map(|(k, v)| (Pid(k), v)).collect();
        self.connections = connections;
        Ok(())
    }

    pub fn get_stats(&self) -> &HashMap<Pid, TrafficStats> {
        &self.current_stats
    }

    pub fn get_connections(&self) -> &HashMap<ConnectionKey, TrafficStats> {
        &self.connections
    }

    pub fn total_upload(&self) -> Bytes {
        self.total_upload
    }

    pub fn total_download(&self) -> Bytes {
        self.total_download
    }
}
