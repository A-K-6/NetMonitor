#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::bpf_get_current_pid_tgid,
    macros::{kprobe, map},
    maps::HashMap,
    programs::ProbeContext,
};
use netmonitor_common::TrafficStats;

#[map]
static TRAFFIC_STATS: HashMap<u32, TrafficStats> = HashMap::with_max_entries(1024, 0);

#[no_mangle]
static _license: [u8; 4] = *b"GPL\0";

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

    if let Some(stats) = TRAFFIC_STATS.get_ptr_mut(&pid) {
        unsafe {
            (*stats).bytes_sent += size as u64;
            (*stats).packets_sent += 1;
        }
    } else {
        let stats = TrafficStats {
            bytes_sent: size as u64,
            packets_sent: 1,
            bytes_recv: 0,
            packets_recv: 0,
        };
        let _ = TRAFFIC_STATS.insert(&pid, &stats, 0);
    }

    Ok(0)
}

#[kprobe]
pub fn tcp_cleanup_rbuf(ctx: ProbeContext) -> u32 {
    match try_tcp_cleanup_rbuf(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_tcp_cleanup_rbuf(ctx: ProbeContext) -> Result<u32, u32> {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    // void tcp_cleanup_rbuf(struct sock *sk, int copied)
    // copied is the 2nd argument (index 1)
    let copied: i32 = ctx.arg(1).ok_or(1u32)?;

    if copied <= 0 {
        return Ok(0);
    }

    if let Some(stats) = TRAFFIC_STATS.get_ptr_mut(&pid) {
        unsafe {
            (*stats).bytes_recv += copied as u64;
            (*stats).packets_recv += 1;
        }
    } else {
        let stats = TrafficStats {
            bytes_sent: 0,
            packets_sent: 0,
            bytes_recv: copied as u64,
            packets_recv: 1,
        };
        let _ = TRAFFIC_STATS.insert(&pid, &stats, 0);
    }

    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
