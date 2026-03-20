# Technical Specification: Kernel & eBPF Core

This document defines the eBPF-based data collection layer for NetMonitor.

## 1. Architecture Overview
NetMonitor uses **Aya**, a library that allows writing both the userspace and the kernel-space code in **Rust**.
- **Kernel-space:** Collects raw packet/socket statistics and stores them in BPF Maps.
- **Userspace:** Polls the BPF Maps, performs PID-to-Process resolution via `/proc`, and renders the UI.

## 2. eBPF Program Hooks
To track per-process bandwidth, we hook into the following kernel functions:

### 2.1. Transmission (Egress)
- **TCP:** `kprobe:tcp_sendmsg`
- **UDP:** `kprobe:udp_sendmsg`
- **RAW/ICMP:** `kprobe:raw_sendmsg`
- **Data Collected:** 
    - `pid`: Process ID (via `bpf_get_current_pid_tgid()`)
    - `bytes`: Number of bytes being sent.
- **Logic:** Increments the total "sent" count for the given PID in a BPF Map.

### 2.2. Reception (Ingress)
- **TCP:** `kprobe:tcp_cleanup_rbuf`
- **UDP:** `kretprobe:udp_recvmsg`
- **RAW/ICMP:** `kretprobe:raw_recvmsg`
- **Data Collected:**
    - `pid`: Process ID.
    - `bytes`: Number of bytes received/cleared from the buffer.
- **Logic:** Increments the total "received" count for the given PID.

## 3. BPF Maps
Maps are used for kernel-to-user communication.

### 3.1. `TRAFFIC_STATS` (HashMap)
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
- **Type:** `BPF_MAP_TYPE_HASH` (with max entries 1024).

### 3.2. `CONNECTIONS` (LruHashMap)
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
- **Type:** `BPF_MAP_TYPE_LRU_HASH` (with max entries 10000).
- **Purpose:** Tracks per-connection bandwidth for detailed view.

## 4. CO-RE (Compile Once – Run Everywhere)
- **Mechanism:** Aya uses BTF (BPF Type Format) to automatically adjust field offsets based on the target kernel version.
- **Requirement:** The target system must have `/sys/kernel/btf/vmlinux` or a provided BTF file.

## 5. Security & Capabilities
The application will use **Systemd Ambient Capabilities** to avoid running as `root`:
- `CAP_BPF`: For loading and managing BPF programs/maps.
- `CAP_NET_ADMIN`: For attaching to network-related hooks.
- `CAP_PERFMON`: For performance monitoring hooks (if required for certain tracepoints).
