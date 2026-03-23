use crate::config::Config;
use crate::core::collector::Collector;
use crate::core::services::identity::Resolver;
use crate::core::services::MonitoringService;
use crate::db::DbManager;
use crate::export::Formatter;
use log::{error, info};
use std::collections::HashMap;
use std::io::Write;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

pub struct MonitoringLoop<C: Collector, R: Resolver> {
    pub monitoring: MonitoringService<C, R>,
    pub db: DbManager,
    pub config: Config,
    pub tick_rate: Duration,
    pub db_flush_rate: Duration,
}

impl<C: Collector, R: Resolver> MonitoringLoop<C, R> {
    pub fn new(
        monitoring: MonitoringService<C, R>,
        db: DbManager,
        config: Config,
        tick_rate: Duration,
    ) -> Self {
        Self {
            monitoring,
            db,
            config,
            tick_rate,
            db_flush_rate: Duration::from_secs(60),
        }
    }

    pub async fn run(
        &mut self,
        mut shutdown: broadcast::Receiver<()>,
        mut formatter: Option<(Box<dyn Formatter>, Box<dyn Write>)>,
        max_count: Option<usize>,
    ) -> Result<(), anyhow::Error> {
        let mut count = 0;
        let mut db_deltas: HashMap<u32, (u64, u64)> = HashMap::new();
        let mut last_db_flush = Instant::now();

        info!("Starting monitoring loop...");

        loop {
            if let Some(max) = max_count {
                if count >= max {
                    break;
                }
            }

            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Shutdown signal received, exiting loop.");
                    break;
                }
                _ = tokio::time::sleep(self.tick_rate) => {
                    match self.monitoring.snapshot(
                        self.config.network.dns_resolution,
                        self.config.network.geo_ip_enabled,
                    ) {
                        Ok(snapshot) => {
                            if let Some((ref f, ref mut w)) = formatter {
                                f.format(&snapshot, w)?;
                            }
                            count += 1;

                            // Update deltas for DB
                            for proc in &snapshot.processes {
                                let entry = db_deltas.entry(proc.pid.0).or_insert((0, 0));
                                entry.0 += proc.up_rate.0;
                                entry.1 += proc.down_rate.0;
                            }

                            // Periodic DB Flush
                            if last_db_flush.elapsed() >= self.db_flush_rate {
                                self.flush_to_db(&mut db_deltas)?;
                                last_db_flush = Instant::now();
                            }
                        }
                        Err(e) => error!("Snapshot failed: {}", e),
                    }
                }
            }
        }

        // Final DB Flush
        self.flush_to_db(&mut db_deltas)?;
        info!("Monitoring loop finished.");
        Ok(())
    }

    fn flush_to_db(&mut self, db_deltas: &mut HashMap<u32, (u64, u64)>) -> Result<(), anyhow::Error> {
        let mut batch = Vec::new();
        for (pid, (up, down)) in db_deltas.drain() {
            if up > 0 || down > 0 {
                let info = self.monitoring.identity.get_info(crate::core::Pid(pid));
                batch.push((pid, info.name, up, down));
            }
        }
        if !batch.is_empty() {
            self.db.flush_batch(&batch)?;
        }
        Ok(())
    }
}
