# Internal Technical Specification: NetMonitor Kernel/Userspace Interaction

This document provides a deep dive into the eBPF architecture, map structures, and the communication protocol between the Linux kernel and the NetMonitor userspace application.

## 1. eBPF Program Architecture
NetMonitor uses a suite of eBPF programs attached to various kernel hooks to capture network events without traditional packet capture overhead.

### 1.1. Hooks & Probes
| Hook Type | Kernel Function | Layer | Purpose |
|-----------|-----------------|-------|---------|
| `kprobe` | `tcp_sendmsg` | Transport | Capture TCP transmission start and byte count. |
| `kprobe` | `udp_sendmsg` | Transport | Capture UDP transmission start and byte count. |
| `kprobe` | `raw_sendmsg` | Network | Capture RAW/ICMP transmission. |
| `kprobe` | `tcp_cleanup_rbuf` | Transport | Capture TCP reception (when data is cleared from socket buffer). |
| `kretprobe`| `udp_recvmsg` | Transport | Capture UDP reception after successful return. |
| `kretprobe`| `raw_recvmsg` | Network | Capture RAW/ICMP reception. |

### 1.2. Context Collection
Each probe uses `bpf_get_current_pid_tgid()` to identify the process responsible for the network event. This allows NetMonitor to provide per-PID granularity.

## 2. BPF Map Schemas
Maps are the primary mechanism for state persistence and kernel-to-user communication.

### 2.1. `TRAFFIC_STATS` (Global Process Stats)
- **Type:** `BPF_MAP_TYPE_HASH`
- **Key:** `u32` (PID)
- **Value:** `struct TrafficStats`
```rust
#[repr(C)]
pub struct TrafficStats {
    pub bytes_sent: u64,
    pub packets_sent: u64,
    pub bytes_recv: u64,
    pub packets_recv: u64,
}
```
- **Update Frequency:** Updated on every `sendmsg`/`recvmsg` event.
- **Polling:** Userspace polls this map at the configured refresh rate (default 1Hz).

### 2.2. `CONNECTIONS` (Flow-Specific Stats)
- **Type:** `BPF_MAP_TYPE_LRU_HASH` (Max entries: 10,000)
- **Key:** `struct ConnectionKey`
```rust
#[repr(C)]
pub struct ConnectionKey {
    pub pid: u32,
    pub proto: u32,
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
}
```
- **Value:** `struct TrafficStats`
- **Purpose:** Used for the "Connection Deep-Dive" view. The LRU (Least Recently Used) property ensures that the map doesn't grow indefinitely, automatically evicting old flows.

## 3. Userspace Data Pipeline
The userspace application (Rust) orchestrates the following pipeline:

1.  **Collection:** Read `TRAFFIC_STATS` and `CONNECTIONS` maps using Aya.
2.  **Resolution:**
    - **Process Identity:** Resolve PIDs to binary names via `/proc/[pid]/comm`.
    - **Service Context:** Parse `/proc/[pid]/cgroup` to identify systemd units or container IDs.
    - **Network Identity:** Resolve destination IPs to hostnames (Reverse DNS) and physical locations (Geo-IP).
3.  **Aggregation:** Accumulate session-wide and total bandwidth usage.
4.  **Persistence:** Periodically flush deltas to the SQLite database (`netmonitor.db`) for historical analysis.
5.  **Visualization:** Render data via the Ratatui-based TUI or export to JSON/CSV in headless mode.

## 4. Security & Capabilities
NetMonitor is designed to run with minimal privileges using Linux capabilities:
- `CAP_BPF`: Required for `bpf()` system calls (loading programs, creating maps).
- `CAP_NET_ADMIN`: Required for attaching probes to network-related kernel functions.
- `CAP_PERFMON`: Required for accessing tracepoints and kprobes on newer kernels.

The provided `netmonitor.service` and Debian package use `AmbientCapabilities` to ensure these are granted without requiring a full `root` shell.

## 5. Performance Considerations
- **Kernel-side:** All eBPF programs are verified for safety and must terminate within a strict instruction limit. We use efficient hash maps to minimize lookups.
- **Userspace-side:** DNS resolution and Geo-IP lookups are performed asynchronously or cached to prevent blocking the TUI refresh.
- **Overhead:** Under typical loads, NetMonitor consumes <1% CPU and <50MB RAM.
