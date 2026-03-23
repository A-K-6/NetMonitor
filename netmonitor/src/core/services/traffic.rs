use crate::app::ProcessRow;
use crate::core::collector::Collector;
use crate::core::types::{Bytes, Pid};
use crate::db::DbManager;
use anyhow::Result;
use chrono::{DateTime, Utc};
use netmonitor_common::{ConnectionKey, TrafficStats};
use std::collections::{HashMap, VecDeque};

#[allow(dead_code)]
pub trait Storage {
    fn save_stats(&mut self, pid: Pid, up: Bytes, down: Bytes) -> Result<()>;
    fn load_historical(
        &mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<HashMap<Pid, ProcessRow>>;
}

#[allow(dead_code)]
pub struct DbStorage {
    inner: DbManager,
}

#[allow(dead_code)]
impl DbStorage {
    pub fn new(path: &str) -> Result<Self> {
        Ok(Self {
            inner: DbManager::new(path).map_err(|e| anyhow::anyhow!(e))?,
        })
    }

    pub fn inner_mut(&mut self) -> &mut DbManager {
        &mut self.inner
    }
}

impl Storage for DbStorage {
    fn save_stats(&mut self, pid: Pid, up: Bytes, down: Bytes) -> Result<()> {
        self.inner
            .flush_batch(&[(pid.0, "".to_string(), up.0, down.0)])
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn load_historical(
        &mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<HashMap<Pid, ProcessRow>> {
        self.inner
            .get_aggregated_stats(start, end)
            .map(|stats| stats.into_iter().map(|(k, v)| (Pid(k), v)).collect())
            .map_err(|e| anyhow::anyhow!(e))
    }
}

// I'll need to update DbManager or bridge it.
// For now let's assume simple bridge or update later.

pub struct TrafficService {
    current_stats: HashMap<Pid, TrafficStats>,
    previous_stats: HashMap<Pid, TrafficStats>,
    connections: HashMap<ConnectionKey, TrafficStats>,
    history_up: VecDeque<Bytes>,
    history_down: VecDeque<Bytes>,
    total_upload: Bytes,     // Current rate (diff)
    total_download: Bytes,   // Current rate (diff)
    session_upload: Bytes,   // Lifetime total (since app start)
    session_download: Bytes, // Lifetime total (since app start)
    max_history: usize,
    last_update: Option<std::time::Instant>,
}

impl TrafficService {
    pub fn new() -> Self {
        Self {
            current_stats: HashMap::new(),
            previous_stats: HashMap::new(),
            connections: HashMap::new(),
            history_up: VecDeque::with_capacity(100),
            history_down: VecDeque::with_capacity(100),
            total_upload: Bytes(0),
            total_download: Bytes(0),
            session_upload: Bytes(0),
            session_download: Bytes(0),
            max_history: 100,
            last_update: None,
        }
    }

    pub fn update(&mut self, collector: &mut dyn Collector) -> Result<()> {
        let stats = collector.collect_stats()?;
        let connections = collector.collect_connections()?;

        self.previous_stats = std::mem::take(&mut self.current_stats);

        let mut total_up_diff = 0;
        let mut total_down_diff = 0;

        for (pid, curr) in &stats {
            let prev_up = self
                .previous_stats
                .get(&Pid(*pid))
                .map(|s| s.bytes_sent)
                .unwrap_or(0);
            let prev_down = self
                .previous_stats
                .get(&Pid(*pid))
                .map(|s| s.bytes_recv)
                .unwrap_or(0);
            total_up_diff += curr.bytes_sent.saturating_sub(prev_up);
            total_down_diff += curr.bytes_recv.saturating_sub(prev_down);
        }

        self.total_upload = Bytes(total_up_diff);
        self.total_download = Bytes(total_down_diff);

        self.session_upload.0 += total_up_diff;
        self.session_download.0 += total_down_diff;

        self.history_up.push_back(self.total_upload);
        self.history_down.push_back(self.total_download);

        if self.history_up.len() > self.max_history {
            self.history_up.pop_front();
            self.history_down.pop_front();
        }

        self.current_stats = stats.into_iter().map(|(k, v)| (Pid(k), v)).collect();
        self.connections = connections;
        self.last_update = Some(std::time::Instant::now());
        Ok(())
    }

    pub fn get_process_rates(&self, pid: Pid) -> (Bytes, Bytes) {
        if let Some(curr) = self.current_stats.get(&pid) {
            let prev_up = self
                .previous_stats
                .get(&pid)
                .map(|s| s.bytes_sent)
                .unwrap_or(0);
            let prev_down = self
                .previous_stats
                .get(&pid)
                .map(|s| s.bytes_recv)
                .unwrap_or(0);

            let up_diff = curr.bytes_sent.saturating_sub(prev_up);
            let down_diff = curr.bytes_recv.saturating_sub(prev_down);

            // Since our snapshot frequency can vary, we might want to scale by time.
            // But usually this represents the bytes since last update. We'll return the raw diff.
            // If strictly bytes-per-second is needed, we would divide by elapsed time.
            (Bytes(up_diff), Bytes(down_diff))
        } else {
            (Bytes(0), Bytes(0))
        }
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

    pub fn session_upload(&self) -> Bytes {
        self.session_upload
    }

    pub fn session_download(&self) -> Bytes {
        self.session_download
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collector::MockCollector;

    #[test]
    fn test_traffic_service_rates() {
        let mut mock = MockCollector::new();
        let mut service = TrafficService::new();

        let mut stats = HashMap::new();
        stats.insert(
            123,
            TrafficStats {
                bytes_sent: 1000,
                packets_sent: 10,
                bytes_recv: 2000,
                packets_recv: 20,
            },
        );
        mock.set_mock_stats(stats);

        service.update(&mut mock).unwrap();

        let (up_rate, down_rate) = service.get_process_rates(Pid(123));
        assert_eq!(up_rate.0, 1000);
        assert_eq!(down_rate.0, 2000);

        // Update again with higher values
        let mut stats2 = HashMap::new();
        stats2.insert(
            123,
            TrafficStats {
                bytes_sent: 1500,
                packets_sent: 15,
                bytes_recv: 2500,
                packets_recv: 25,
            },
        );
        mock.set_mock_stats(stats2);

        service.update(&mut mock).unwrap();

        let (up_rate2, down_rate2) = service.get_process_rates(Pid(123));
        assert_eq!(up_rate2.0, 500); // 1500 - 1000
        assert_eq!(down_rate2.0, 500); // 2500 - 2000
    }
}
