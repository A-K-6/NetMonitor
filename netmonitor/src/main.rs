mod process;

use aya::maps::HashMap;
use aya::programs::KProbe;
use aya::Ebpf;
use aya_log::EbpfLogger;
use caps::{CapSet, Capability, has_cap};
use netmonitor_common::TrafficStats;
use std::env;
use std::time::Duration;
use tokio::signal;
use tokio::time;
use log::{info, warn, error};
use process::ProcessResolver;

fn check_caps() -> Result<(), anyhow::Error> {
    let required = [Capability::CAP_BPF, Capability::CAP_NET_ADMIN];
    for &cap in &required {
        if !has_cap(None, CapSet::Effective, cap).unwrap_or(false) {
            error!("Missing capability: {:?}. Try 'sudo setcap cap_net_admin,cap_bpf=ep {}'", cap, env::current_exe()?.display());
            return Err(anyhow::anyhow!("Insufficient permissions"));
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Check capabilities before loading
    if let Err(e) = check_caps() {
        return Err(e);
    }

    // Bump RLIMIT_MEMLOCK to allow BPF programs to load
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        warn!("Failed to increase rlimit RLIMIT_MEMLOCK: {}", ret);
    }

    // Load the BPF program (embedded)
    let bpf_bytes = include_bytes!("../../target/bpfel-unknown-none/release/netmonitor-ebpf");

    let mut bpf = Ebpf::load(bpf_bytes)?;

    if let Err(e) = EbpfLogger::init(&mut bpf) {
        warn!("failed to initialize eBPF logger: {}", e);
    }

    let program: &mut KProbe = bpf.program_mut("tcp_sendmsg").unwrap().try_into()?;
    program.load()?;
    program.attach("tcp_sendmsg", 0)?;

    let stats_map: HashMap<_, u32, TrafficStats> = HashMap::try_from(bpf.map_mut("TRAFFIC_STATS").unwrap())?;

    let mut interval = time::interval(Duration::from_secs(1));
    let mut resolver = ProcessResolver::new(Duration::from_secs(10));

    info!("Waiting for Ctrl-C...");

    loop {
        tokio::select! {
            _ = interval.tick() => {
                info!("--- Bandwidth Stats (per PID) ---");
                for result in stats_map.iter() {
                    if let Ok((pid, stats)) = result {
                        let name = resolver.get_process_name(pid);
                        info!("[{}] ({}) -> Sent: {} bytes ({} packets)", name, pid, stats.bytes_sent, stats.packets_sent);
                    }
                }
            }
            _ = signal::ctrl_c() => {
                info!("Exiting...");
                break;
            }
        }
    }

    Ok(())
}
