#![no_std]

#[derive(Copy, Clone)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrafficStats {
    pub bytes_sent: u64,
    pub packets_sent: u64,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for TrafficStats {}
