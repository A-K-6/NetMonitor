use crate::core::types::{Bytes, Pid};
use crate::process::ProcessContext;
use std::collections::HashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProcessSummary {
    pub pid: Pid,
    pub name: String,
    pub context: ProcessContext,
    pub up: Bytes,
    pub down: Bytes,
    #[allow(dead_code)]
    pub total: Bytes,
    pub up_rate: Bytes, // Per second
    pub down_rate: Bytes,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionSummary {
    pub proto: u32,
    pub src_ip: String,
    pub src_port: u16,
    pub dst_ip: String,
    pub dst_port: u16,
    pub up: Bytes,
    pub down: Bytes,
    pub country: String,
    pub isp: String,
    pub hostname: Option<String>,
    pub service: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MonitoringSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub processes: Vec<ProcessSummary>,
    pub connections: HashMap<u32, Vec<ConnectionSummary>>,
    pub total_up: Bytes,
    pub total_down: Bytes,
    pub session_up: Bytes,
    pub session_down: Bytes,
    pub protocol_stats: HashMap<u32, (Bytes, Bytes)>,
    pub country_stats: HashMap<String, (Bytes, Bytes)>,
}
