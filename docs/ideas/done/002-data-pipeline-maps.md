# Task 002: Data Pipeline & BPF Maps

**Status:** Proposed
**Phase:** 1 (Foundation)
**Objective:** Implement BPF Hash Maps to track real-time bandwidth statistics (bytes sent) indexed by Process ID (PID).

## 1. Define Shared Data Structures (`netmonitor-common`)
- [ ] Create a `TrafficStats` struct in `lib.rs`.
- [ ] Include fields: `bytes_sent: u64`, `packets_sent: u64`.
- [ ] Ensure `#[repr(C)]` and `derive(Copy, Clone)` for BPF compatibility.
- [ ] Add `serde` support (optional/gated) for userspace display.

## 2. Declare BPF Maps (`netmonitor-ebpf`)
- [ ] Declare a `HashMap<u32, TrafficStats>` (PID -> Stats) in `main.rs`.
- [ ] Use `aya_ebpf::macros::map`.

## 3. Implement Kernel logic (`netmonitor-ebpf`)
- [ ] Update `tcp_sendmsg` kprobe to:
    - [ ] Extract the current PID using `aya_ebpf::helpers::bpf_get_current_pid_tgid`.
    - [ ] Extract the message size from the `tcp_sendmsg` arguments (likely `size_t len`).
    - [ ] Perform a map lookup for the PID.
    - [ ] Update the `TrafficStats` (create if missing, increment `bytes_sent`).

## 4. Implement Userspace Polling (`netmonitor`)
- [ ] Add a tokio task/loop to periodically (e.g., every 1 second) read from the BPF map.
- [ ] Iterate over all entries in the map.
- [ ] Log the bandwidth per PID to the console: `PID: 1234 -> Sent: 500 KB`.

## 5. Verification
- [ ] Run `cargo xtask run`.
- [ ] Generate traffic (e.g., `curl https://google.com`).
- [ ] Verify that the console output correctly attributes bytes to the corresponding PID.
