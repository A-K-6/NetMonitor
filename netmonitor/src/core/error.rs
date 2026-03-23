use crate::core::types::Pid;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetMonitorError {
    #[error("BPF error: {0}")]
    BpfError(#[from] aya::EbpfError),

    #[error("Map error: {0}")]
    MapError(#[from] aya::maps::MapError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Process not found: {0}")]
    ProcessNotFound(Pid),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, NetMonitorError>;
