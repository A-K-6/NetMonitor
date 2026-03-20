#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::bpf_get_current_pid_tgid,
    macros::{kprobe, kretprobe, map},
    maps::HashMap,
    programs::{ProbeContext, RetProbeContext},
};
use netmonitor_common::TrafficStats;

#[map]
static TRAFFIC_STATS: HashMap<u32, TrafficStats> = HashMap::with_max_entries(1024, 0);

#[no_mangle]
static _license: [u8; 4] = *b"GPL\0";

/// Helper to update or insert traffic stats for a given PID.
#[inline(always)]
fn update_stats(pid: u32, sent: u64, recv: u64) {
    if let Some(stats) = TRAFFIC_STATS.get_ptr_mut(&pid) {
        unsafe {
            if sent > 0 {
                (*stats).bytes_sent += sent;
                (*stats).packets_sent += 1;
            }
            if recv > 0 {
                (*stats).bytes_recv += recv;
                (*stats).packets_recv += 1;
            }
        }
    } else {
        let stats = TrafficStats {
            bytes_sent: sent,
            packets_sent: if sent > 0 { 1 } else { 0 },
            bytes_recv: recv,
            packets_recv: if recv > 0 { 1 } else { 0 },
        };
        let _ = TRAFFIC_STATS.insert(&pid, &stats, 0);
    }
}

// --- TCP ---

#[kprobe]
pub fn tcp_sendmsg(ctx: ProbeContext) -> u32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    // tcp_sendmsg(struct sock *sk, struct msghdr *msg, size_t size)
    let size: usize = ctx.arg(2).unwrap_or(0);
    if size > 0 {
        update_stats(pid, size as u64, 0);
    }
    0
}

#[kprobe]
pub fn tcp_cleanup_rbuf(ctx: ProbeContext) -> u32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    // void tcp_cleanup_rbuf(struct sock *sk, int copied)
    let copied: i32 = ctx.arg(1).unwrap_or(0);
    if copied > 0 {
        update_stats(pid, 0, copied as u64);
    }
    0
}

// --- UDP ---

#[kprobe]
pub fn udp_sendmsg(ctx: ProbeContext) -> u32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    // int udp_sendmsg(struct sock *sk, struct msghdr *msg, size_t len)
    let len: usize = ctx.arg(2).unwrap_or(0);
    if len > 0 {
        update_stats(pid, len as u64, 0);
    }
    0
}

#[kretprobe]
pub fn udp_recvmsg(ctx: RetProbeContext) -> u32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    // int udp_recvmsg returns size
    let ret: i32 = ctx.ret().unwrap_or(0);
    if ret > 0 {
        update_stats(pid, 0, ret as u64);
    }
    0
}

// --- RAW / ICMP ---

#[kprobe]
pub fn raw_sendmsg(ctx: ProbeContext) -> u32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    // int raw_sendmsg(struct sock *sk, struct msghdr *msg, size_t len)
    let len: usize = ctx.arg(2).unwrap_or(0);
    if len > 0 {
        update_stats(pid, len as u64, 0);
    }
    0
}

#[kretprobe]
pub fn raw_recvmsg(ctx: RetProbeContext) -> u32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    // int raw_recvmsg returns size
    let ret: i32 = ctx.ret().unwrap_or(0);
    if ret > 0 {
        update_stats(pid, 0, ret as u64);
    }
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
