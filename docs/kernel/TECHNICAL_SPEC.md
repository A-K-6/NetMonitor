# Technical Specification: Kernel & eBPF Core

This document defines the eBPF-based data collection layer for NetMonitor.

## 1. Architecture Overview
NetMonitor uses **Aya**, a library that allows writing both the userspace and the kernel-space code in **Rust**.
- **Kernel-space:** Collects raw packet/socket statistics and stores them in BPF Maps.
- **Userspace:** Polls the BPF Maps, performs PID-to-Process resolution via `/proc`, and renders the UI.

## 2. eBPF Program Hooks
To track per-process bandwidth, we hook into the following kernel functions:

### 2.1. Transmission (Egress)
- **Hook:** `kprobe:tcp_sendmsg` and `kprobe:udp_sendmsg`
- **Data Collected:** 
    - `pid`: Process ID (via `bpf_get_current_pid_tgid()`)
    - `bytes`: Number of bytes being sent.
- **Logic:** Increments the total "sent" count for the given PID in a BPF Map.

### 2.2. Reception (Ingress)
- **Hook:** `fentry:tcp_cleanup_rbuf` (or `kprobe:tcp_recvmsg` as fallback)
- **Data Collected:**
    - `pid`: Process ID.
    - `bytes`: Number of bytes received/cleared from the buffer.
- **Logic:** Increments the total "received" count for the given PID.

## 3. BPF Maps
Maps are used for kernel-to-user communication.

### 3.1. `PROCESS_STATS` (HashMap)
- **Key:** `u32` (PID)
- **Value:** `struct BandwidthStats`
    ```rust
    #[repr(C)]
    struct BandwidthStats {
        bytes_sent: u64,
        bytes_recv: u64,
        last_updated: u64,
    }
    ```
- **Type:** `BPF_MAP_TYPE_LRU_HASH` (to prevent memory exhaustion from short-lived PIDs).

## 4. CO-RE (Compile Once – Run Everywhere)
- **Mechanism:** Aya uses BTF (BPF Type Format) to automatically adjust field offsets based on the target kernel version.
- **Requirement:** The target system must have `/sys/kernel/btf/vmlinux` or a provided BTF file.

## 5. Security & Capabilities
The application will use **Systemd Ambient Capabilities** to avoid running as `root`:
- `CAP_BPF`: For loading and managing BPF programs/maps.
- `CAP_NET_ADMIN`: For attaching to network-related hooks.
- `CAP_PERFMON`: For performance monitoring hooks (if required for certain tracepoints).
