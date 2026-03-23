use anyhow::Result;
use aya::maps::HashMap as BpfHashMap;
use aya::programs::{CgroupAttachMode, CgroupSkb, CgroupSkbAttachType, KProbe};
use aya::Ebpf;
use aya_log::EbpfLogger;
use netmonitor_common::{ConnectionKey, ThrottleConfig, TrafficStats};
use std::collections::HashMap;
use std::fs::File;

/// The Collector trait abstracts the source of network monitoring data.
/// This allows for mocking the kernel layer during tests and decoupling
/// the TUI from specific eBPF libraries.
pub trait Collector {
    /// Collect snapshots of per-PID traffic stats.
    fn collect_stats(&mut self) -> Result<HashMap<u32, TrafficStats>>;

    /// Collect snapshots of per-connection traffic stats.
    fn collect_connections(&mut self) -> Result<HashMap<ConnectionKey, TrafficStats>>;

    /// Set a throttle limit for a specific PID.
    fn set_throttle(&mut self, pid: u32, limit_kb_s: u64) -> Result<()>;

    /// Clear a throttle limit for a specific PID.
    fn clear_throttle(&mut self, pid: u32) -> Result<()>;
}

pub struct AyaCollector {
    _bpf: Ebpf,
    stats_map: BpfHashMap<aya::maps::MapData, u32, TrafficStats>,
    connections_map: BpfHashMap<aya::maps::MapData, ConnectionKey, TrafficStats>,
    throttle_map: BpfHashMap<aya::maps::MapData, u32, ThrottleConfig>,
}

impl AyaCollector {
    pub fn new() -> Result<Self> {
        // Bump RLIMIT_MEMLOCK to allow BPF programs to load
        let rlim = libc::rlimit {
            rlim_cur: libc::RLIM_INFINITY,
            rlim_max: libc::RLIM_INFINITY,
        };
        let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
        if ret != 0 {
            log::warn!("Failed to increase rlimit RLIMIT_MEMLOCK: {}", ret);
        }

        // Load the BPF program (embedded via include_bytes!)
        let ebpf_bytecode = include_bytes!("../../resources/netmonitor-ebpf").to_vec();
        let mut bpf = Ebpf::load(&ebpf_bytecode)?;

        if let Err(e) = EbpfLogger::init(&mut bpf) {
            log::warn!("failed to initialize eBPF logger: {}", e);
        }

        // Initialize KProbes
        Self::load_and_attach_kprobe(&mut bpf, "tcp_sendmsg", "tcp_sendmsg")?;
        Self::load_and_attach_kprobe(&mut bpf, "tcp_cleanup_rbuf", "tcp_cleanup_rbuf")?;
        Self::load_and_attach_kprobe(&mut bpf, "udp_sendmsg", "udp_sendmsg")?;
        Self::load_and_attach_kprobe(&mut bpf, "udp_recvmsg", "udp_recvmsg")?;
        Self::load_and_attach_kprobe(&mut bpf, "raw_sendmsg", "raw_sendmsg")?;
        Self::load_and_attach_kprobe(&mut bpf, "raw_recvmsg", "raw_recvmsg")?;

        // Initialize Traffic Shaping (Cgroup SKB)
        Self::load_and_attach_cgroup_skb(&mut bpf, "throttle_egress", CgroupSkbAttachType::Egress)?;
        Self::load_and_attach_cgroup_skb(
            &mut bpf,
            "throttle_ingress",
            CgroupSkbAttachType::Ingress,
        )?;

        let stats_map: BpfHashMap<_, u32, TrafficStats> =
            BpfHashMap::try_from(bpf.take_map("TRAFFIC_STATS").unwrap())?;
        let connections_map: BpfHashMap<_, ConnectionKey, TrafficStats> =
            BpfHashMap::try_from(bpf.take_map("CONNECTIONS").unwrap())?;
        let throttle_map: BpfHashMap<_, u32, ThrottleConfig> =
            BpfHashMap::try_from(bpf.take_map("THROTTLE_CONFIG").unwrap())?;

        Ok(Self {
            _bpf: bpf,
            stats_map,
            connections_map,
            throttle_map,
        })
    }

    fn load_and_attach_kprobe(bpf: &mut Ebpf, name: &str, function: &str) -> Result<()> {
        let program: &mut KProbe = bpf
            .program_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Program {} not found", name))?
            .try_into()?;
        program.load()?;
        program.attach(function, 0)?;
        Ok(())
    }

    fn load_and_attach_cgroup_skb(
        bpf: &mut Ebpf,
        name: &str,
        attach_type: CgroupSkbAttachType,
    ) -> Result<()> {
        let program: &mut CgroupSkb = bpf
            .program_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Program {} not found", name))?
            .try_into()?;
        program.load()?;
        let cgroup_file = File::open("/sys/fs/cgroup")?;
        program.attach(cgroup_file, attach_type, CgroupAttachMode::default())?;
        Ok(())
    }
}

impl Collector for AyaCollector {
    fn collect_stats(&mut self) -> Result<HashMap<u32, TrafficStats>> {
        let mut stats = HashMap::new();
        for item in self.stats_map.iter() {
            let (pid, traffic) = item?;
            stats.insert(pid, traffic);
        }
        Ok(stats)
    }

    fn collect_connections(&mut self) -> Result<HashMap<ConnectionKey, TrafficStats>> {
        let mut connections = HashMap::new();
        for item in self.connections_map.iter() {
            let (key, traffic) = item?;
            connections.insert(key, traffic);
        }
        Ok(connections)
    }

    fn set_throttle(&mut self, pid: u32, limit_kb_s: u64) -> Result<()> {
        let config = ThrottleConfig {
            rate_bytes_per_sec: limit_kb_s * 1024,
            bucket_size: limit_kb_s * 1024 * 2,
            last_refill_ts: 0,
            tokens: limit_kb_s * 1024 * 2,
        };
        self.throttle_map.insert(pid, config, 0)?;
        Ok(())
    }

    fn clear_throttle(&mut self, pid: u32) -> Result<()> {
        self.throttle_map.remove(&pid)?;
        Ok(())
    }
}

#[cfg(test)]
pub struct MockCollector {
    stats: HashMap<u32, TrafficStats>,
    connections: HashMap<ConnectionKey, TrafficStats>,
    throttles: HashMap<u32, u64>,
}

#[cfg(test)]
impl MockCollector {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
            connections: HashMap::new(),
            throttles: HashMap::new(),
        }
    }

    pub fn set_mock_stats(&mut self, stats: HashMap<u32, TrafficStats>) {
        self.stats = stats;
    }
}

#[cfg(test)]
impl Collector for MockCollector {
    fn collect_stats(&mut self) -> Result<HashMap<u32, TrafficStats>> {
        Ok(self.stats.clone())
    }

    fn collect_connections(&mut self) -> Result<HashMap<ConnectionKey, TrafficStats>> {
        Ok(self.connections.clone())
    }

    fn set_throttle(&mut self, pid: u32, limit_kb_s: u64) -> Result<()> {
        self.throttles.insert(pid, limit_kb_s);
        Ok(())
    }

    fn clear_throttle(&mut self, pid: u32) -> Result<()> {
        self.throttles.remove(&pid);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_collector() {
        let mut collector = MockCollector::new();
        let mut stats = HashMap::new();
        stats.insert(
            1234,
            TrafficStats {
                bytes_sent: 100,
                packets_sent: 1,
                bytes_recv: 200,
                packets_recv: 2,
            },
        );
        collector.set_mock_stats(stats.clone());

        let collected = collector.collect_stats().unwrap();
        assert_eq!(collected.get(&1234).unwrap().bytes_sent, 100);
        assert_eq!(collected.get(&1234).unwrap().bytes_recv, 200);
    }

    #[test]
    fn test_mock_throttle() {
        let mut collector = MockCollector::new();
        collector.set_throttle(1234, 1000).unwrap();
        assert_eq!(collector.throttles.get(&1234).unwrap(), &1000);
        collector.clear_throttle(1234).unwrap();
        assert!(!collector.throttles.contains_key(&1234));
    }
}
