use rusqlite::{params, Connection, Result};
use std::path::Path;
use chrono::{Utc, DateTime};
use crate::app::{ProcessRow, TimeRange};
use std::collections::HashMap;

pub struct DbManager {
    conn: Connection,
}

impl DbManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let manager = Self { conn };
        manager.initialize()?;
        Ok(manager)
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS processes (
                pid INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                first_seen DATETIME NOT NULL,
                last_seen DATETIME NOT NULL,
                total_up INTEGER DEFAULT 0,
                total_down INTEGER DEFAULT 0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS traffic_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pid INTEGER NOT NULL,
                timestamp DATETIME NOT NULL,
                up_bytes INTEGER NOT NULL,
                down_bytes INTEGER NOT NULL,
                FOREIGN KEY(pid) REFERENCES processes(pid)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn get_traffic_history(&self, pid: u32, range: TimeRange) -> Result<Vec<(DateTime<Utc>, u64, u64)>> {
        let now = Utc::now();
        let seconds = range.to_seconds();
        let start_time = now - chrono::Duration::seconds(seconds);
        
        // Determine bucket size in seconds
        let bucket_size = match range {
            TimeRange::TenMinutes => 10,       // 10s buckets
            TimeRange::OneHour => 60,         // 1m buckets
            TimeRange::TwentyFourHours => 900, // 15m buckets
        };

        let start_ts = start_time.timestamp();
        
        let mut stmt = self.conn.prepare(
            "SELECT 
                ((strftime('%s', timestamp) - ?1) / ?2) * ?2 + ?1 as bucket_ts,
                SUM(up_bytes),
                SUM(down_bytes)
             FROM traffic_log
             WHERE pid = ?3 AND timestamp >= ?4
             GROUP BY bucket_ts
             ORDER BY bucket_ts ASC"
        )?;

        let history_iter = stmt.query_map(params![start_ts, bucket_size, pid, start_time], |row| {
            let ts_secs: i64 = row.get(0)?;
            let up: u64 = row.get(1)?;
            let down: u64 = row.get(2)?;
            
            let dt = DateTime::from_timestamp(ts_secs, 0).unwrap_or_else(|| Utc::now());
            Ok((dt, up, down))
        })?;

        let mut results = Vec::new();
        for item in history_iter {
            results.push(item?);
        }
        Ok(results)
    }

    pub fn load_historical_stats(&self) -> Result<HashMap<u32, ProcessRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT pid, name, total_up, total_down FROM processes"
        )?;
        let process_iter = stmt.query_map([], |row| {
            let pid: u32 = row.get(0)?;
            let name: String = row.get(1)?;
            let total_up: u64 = row.get(2)?;
            let total_down: u64 = row.get(3)?;
            
            Ok((pid, ProcessRow {
                pid,
                name,
                up_bytes: 0, // Reset for current session
                down_bytes: 0,
                total_bytes: total_up + total_down,
                last_up_bytes: 0, // Should be updated by eBPF map on first tick
                last_down_bytes: 0,
            }))
        })?;

        let mut stats = HashMap::new();
        for process in process_iter {
            let (pid, row) = process?;
            stats.insert(pid, row);
        }
        Ok(stats)
    }

    pub fn flush_batch(&mut self, data: &[(u32, String, u64, u64)]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let now = Utc::now();
            let mut stmt_proc = tx.prepare(
                "INSERT INTO processes (pid, name, first_seen, last_seen, total_up, total_down)
                 VALUES (?1, ?2, ?3, ?3, ?4, ?5)
                 ON CONFLICT(pid) DO UPDATE SET
                    name = excluded.name,
                    last_seen = excluded.last_seen,
                    total_up = total_up + excluded.total_up,
                    total_down = total_down + excluded.total_down"
            )?;
            
            let mut stmt_log = tx.prepare(
                "INSERT INTO traffic_log (pid, timestamp, up_bytes, down_bytes)
                 VALUES (?1, ?2, ?3, ?4)"
            )?;

            for (pid, name, up_delta, down_delta) in data {
                stmt_proc.execute(params![pid, name, now, up_delta, down_delta])?;
                if *up_delta > 0 || *down_delta > 0 {
                    stmt_log.execute(params![pid, now, up_delta, down_delta])?;
                }
            }
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_initialization() {
        let db = DbManager::new(":memory:").unwrap();
        let stats = db.load_historical_stats().unwrap();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_save_and_load_stats() {
        let mut db = DbManager::new(":memory:").unwrap();
        db.flush_batch(&[(1234, "test-proc".to_string(), 100, 200)]).unwrap();
        
        let stats = db.load_historical_stats().unwrap();
        assert_eq!(stats.len(), 1);
        let row = stats.get(&1234).unwrap();
        assert_eq!(row.name, "test-proc");
        assert_eq!(row.total_bytes, 300);
    }

    #[test]
    fn test_flush_batch() {
        let mut db = DbManager::new(":memory:").unwrap();
        let batch = vec![
            (1, "proc1".to_string(), 100, 50),
            (2, "proc2".to_string(), 200, 100),
        ];
        db.flush_batch(&batch).unwrap();

        let stats = db.load_historical_stats().unwrap();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats.get(&1).unwrap().total_bytes, 150);
        assert_eq!(stats.get(&2).unwrap().total_bytes, 300);
    }
}
