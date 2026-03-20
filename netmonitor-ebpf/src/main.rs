#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::bpf_get_current_pid_tgid,
    macros::{kprobe, map},
    maps::HashMap,
    programs::ProbeContext,
};
use aya_log_ebpf::info;
use netmonitor_common::TrafficStats;

#[map]
static mut TRAFFIC_STATS: HashMap<u32, TrafficStats> = HashMap::with_max_entries(1024, 0);

#[kprobe]
pub fn tcp_sendmsg(ctx: ProbeContext) -> u32 {
    match try_tcp_sendmsg(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_tcp_sendmsg(ctx: ProbeContext) -> Result<u32, u32> {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    // tcp_sendmsg(struct sock *sk, struct msghdr *msg, size_t size)
    // size is the 3rd argument (index 2)
    let size: usize = ctx.arg(2).ok_or(1u32)?;

    unsafe {
        if let Some(stats) = TRAFFIC_STATS.get_ptr_mut(&pid) {
            (*stats).bytes_sent += size as u64;
            (*stats).packets_sent += 1;
        } else {
            let stats = TrafficStats {
                bytes_sent: size as u64,
                packets_sent: 1,
            };
            TRAFFIC_STATS.insert(&pid, &stats, 0).map_err(|_| 1u32)?;
        }
    }

    info!(&ctx, "PID {} sent {} bytes", pid, size);

    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
