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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConnectionKey {
    pub pid: u32,
    pub proto: u32, // Using u32 for protocol to maintain alignment
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ThrottleConfig {
    pub rate_bytes_per_sec: u64,
    pub bucket_size: u64,
    pub last_refill_ts: u64,
    pub tokens: u64,
}

#[inline(always)]
pub fn calculate_tokens(rate: u64, elapsed_ns: u64) -> u64 {
    (rate * elapsed_ns) / 1_000_000_000
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for TrafficStats {}
#[cfg(feature = "user")]
unsafe impl aya::Pod for ConnectionKey {}
#[cfg(feature = "user")]
unsafe impl aya::Pod for ThrottleConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_tokens() {
        // 1MB/s = 1_048_576 bytes/s
        let rate = 1_048_576;

        // 1 second elapsed
        let elapsed_ns = 1_000_000_000;
        assert_eq!(calculate_tokens(rate, elapsed_ns), rate);

        // 0.5 second elapsed
        let elapsed_ns = 500_000_000;
        assert_eq!(calculate_tokens(rate, elapsed_ns), rate / 2);

        // 1 nanosecond elapsed (too small for 1MB/s to add a token)
        let elapsed_ns = 1;
        assert_eq!(calculate_tokens(rate, elapsed_ns), 0);
    }
}
