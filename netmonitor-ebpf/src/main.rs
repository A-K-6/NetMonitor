#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::{bpf_get_current_pid_tgid, bpf_ktime_get_ns, bpf_probe_read_kernel},
    macros::{cgroup_skb, kprobe, kretprobe, map},
    maps::{HashMap, LruHashMap},
    programs::{ProbeContext, RetProbeContext, SkBuffContext},
};
use netmonitor_common::{ConnectionKey, ThrottleConfig, TrafficStats};

#[map]
static TRAFFIC_STATS: HashMap<u32, TrafficStats> = HashMap::with_max_entries(1024, 0);

#[map]
static CONNECTIONS: LruHashMap<ConnectionKey, TrafficStats> =
    LruHashMap::with_max_entries(10000, 0);

#[map]
static THROTTLE_CONFIG: HashMap<u32, ThrottleConfig> = HashMap::with_max_entries(1024, 0);

#[no_mangle]
static _license: [u8; 4] = *b"GPL\0";

#[cgroup_skb(egress)]
pub fn throttle_egress(ctx: SkBuffContext) -> i32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    if pid == 0 {
        return 1; // PASS
    }

    let len = ctx.len() as u64;

    if let Some(config) = THROTTLE_CONFIG.get_ptr_mut(&pid) {
        let now = unsafe { bpf_ktime_get_ns() };
        unsafe {
            let elapsed_ns = now.saturating_sub((*config).last_refill_ts);

            // rate is bytes/sec. elapsed_ns is nanoseconds.
            // tokens_to_add = (rate * elapsed_ns) / 1_000_000_000
            let rate = (*config).rate_bytes_per_sec;
            let tokens_to_add = (rate * elapsed_ns) / 1_000_000_000;

            if tokens_to_add > 0 {
                (*config).tokens =
                    core::cmp::min((*config).bucket_size, (*config).tokens + tokens_to_add);
                (*config).last_refill_ts = now;
            }

            if (*config).tokens >= len {
                (*config).tokens -= len;
                1 // PASS
            } else {
                0 // DROP
            }
        }
    } else {
        1 // PASS
    }
}

#[cgroup_skb(ingress)]
pub fn throttle_ingress(_ctx: SkBuffContext) -> i32 {
    1 // PASS
}

// Minimal struct sock_common to get the fields we need.
#[repr(C)]
#[derive(Copy, Clone)]
struct sock_common {
    skc_daddr: u32,
    skc_rcv_saddr: u32,
    _padding1: u32, // skip skc_addrpair
    skc_dport: u16,
    skc_num: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct sock {
    sk_common: sock_common,
}

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

/// Helper to update or insert traffic stats for a specific connection.
#[inline(always)]
fn update_connection_stats(key: &ConnectionKey, sent: u64, recv: u64) {
    if let Some(stats) = CONNECTIONS.get_ptr_mut(key) {
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
        let _ = CONNECTIONS.insert(key, &stats, 0);
    }
}

#[inline(always)]
fn get_connection_key(sk: *mut sock, proto: u32, pid: u32) -> Option<ConnectionKey> {
    unsafe {
        let daddr: u32 = bpf_probe_read_kernel(&(*sk).sk_common.skc_daddr).ok()?;
        let saddr: u32 = bpf_probe_read_kernel(&(*sk).sk_common.skc_rcv_saddr).ok()?;
        let dport: u16 = bpf_probe_read_kernel(&(*sk).sk_common.skc_dport).ok()?;
        let sport: u16 = bpf_probe_read_kernel(&(*sk).sk_common.skc_num).ok()?;

        Some(ConnectionKey {
            pid,
            proto,
            src_ip: saddr,
            dst_ip: daddr,
            src_port: sport,
            dst_port: u16::from_be(dport), // dport is in network byte order in kernel
        })
    }
}

// --- TCP ---

#[kprobe]
pub fn tcp_sendmsg(ctx: ProbeContext) -> u32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    let sk: *mut sock = ctx.arg(0).unwrap_or(core::ptr::null_mut());
    let size: usize = ctx.arg(2).unwrap_or(0);
    if size > 0 {
        update_stats(pid, size as u64, 0);
        if let Some(key) = get_connection_key(sk, 6, pid) {
            update_connection_stats(&key, size as u64, 0);
        }
    }
    0
}

#[kprobe]
pub fn tcp_cleanup_rbuf(ctx: ProbeContext) -> u32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    let sk: *mut sock = ctx.arg(0).unwrap_or(core::ptr::null_mut());
    let copied: i32 = ctx.arg(1).unwrap_or(0);
    if copied > 0 {
        update_stats(pid, 0, copied as u64);
        if let Some(key) = get_connection_key(sk, 6, pid) {
            update_connection_stats(&key, 0, copied as u64);
        }
    }
    0
}

// --- UDP ---

#[kprobe]
pub fn udp_sendmsg(ctx: ProbeContext) -> u32 {
    let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
    let sk: *mut sock = ctx.arg(0).unwrap_or(core::ptr::null_mut());
    let len: usize = ctx.arg(2).unwrap_or(0);
    if len > 0 {
        update_stats(pid, len as u64, 0);
        if let Some(key) = get_connection_key(sk, 17, pid) {
            update_connection_stats(&key, len as u64, 0);
        }
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
