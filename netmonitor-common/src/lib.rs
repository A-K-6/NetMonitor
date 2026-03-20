#![no_std]

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrafficStats {
    pub bytes_sent: u64,
    pub packets_sent: u64,
    pub bytes_recv: u64,
    pub packets_recv: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConnectionKey {
    pub pid: u32,
    pub proto: u32,  // Using u32 for protocol to maintain alignment
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for TrafficStats {}
#[cfg(feature = "user")]
unsafe impl aya::Pod for ConnectionKey {}
