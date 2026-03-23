use crate::core::collector::Collector;
use crate::core::types::Pid;
use anyhow::Result;
use std::collections::HashMap;

pub struct EnforcementService {
    thresholds: HashMap<Pid, u64>, // KB/s
    throttles: HashMap<Pid, u64>,  // KB/s
}

impl EnforcementService {
    pub fn new() -> Self {
        Self {
            thresholds: HashMap::new(),
            throttles: HashMap::new(),
        }
    }

    pub fn set_threshold(&mut self, pid: Pid, kb_s: u64) {
        self.thresholds.insert(pid, kb_s);
    }

    pub fn get_threshold(&self, pid: Pid) -> Option<u64> {
        self.thresholds.get(&pid).copied()
    }

    pub fn remove_threshold(&mut self, pid: Pid) {
        self.thresholds.remove(&pid);
    }

    pub fn set_throttle(
        &mut self,
        collector: &mut dyn Collector,
        pid: Pid,
        kb_s: u64,
    ) -> Result<()> {
        collector.set_throttle(pid.0, kb_s)?;
        self.throttles.insert(pid, kb_s);
        Ok(())
    }

    pub fn clear_throttle(&mut self, collector: &mut dyn Collector, pid: Pid) -> Result<()> {
        collector.clear_throttle(pid.0)?;
        self.throttles.remove(&pid);
        Ok(())
    }

    pub fn get_throttle(&self, pid: Pid) -> Option<u64> {
        self.throttles.get(&pid).copied()
    }
}
