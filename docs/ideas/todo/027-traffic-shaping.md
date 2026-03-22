# Idea: Active Traffic Shaping (Bandwidth Throttling)

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Provide system administrators with the ability to "throttle" the bandwidth of specific processes directly from the TUI. This ensures that background processes (e.g., updates, backups) don't starve critical interactive applications (e.g., SSH, Video Calls).

## 1. Research & Strategy
- **Mechanism:** Use **`cgroup_skb`** eBPF programs attached to the root cgroup (or specific slices).
- **Control:** Implement a **Token Bucket Filter (TBF)** algorithm in eBPF.
    - **Bucket Map:** An eBPF map (`BPF_MAP_TYPE_HASH`) stores the current "tokens" (bytes) available for each PID.
    - **Refill Logic:** Userspace periodic task or eBPF timer (if supported by kernel/Aya) refills tokens based on the configured rate.
    - **Enforcement:** The `cgroup_skb` program checks the bucket:
        - If `tokens >= packet_len`: Subtract tokens and return `SK_PASS`.
        - If `tokens < packet_len`: Return `SK_DROP`.
- **UI Integration:**
    - Add a new hotkey `s` (for "Shape" or "Limit") in the `ProcessTable`.
    - A dialog allows entering a limit in KB/s.
    - Display an indicator (e.g., `[L]` or a colored rate) for throttled processes.

## 2. Technical Specification
### eBPF Map (Proposed)
```rust
#[repr(C)]
pub struct ThrottleConfig {
    pub rate_bytes_per_sec: u64,
    pub bucket_size: u64,
    pub last_refill_ts: u64,
    pub tokens: u64,
}

// Map: PID -> ThrottleConfig
```

## 3. Implementation Plan
### Phase A: eBPF Core
- [ ] Define `THROTTLE_CONFIG` map in `netmonitor-ebpf/src/main.rs`.
- [ ] Implement `cgroup_skb/egress` and `cgroup_skb/ingress` programs.
- [ ] Implement Token Bucket logic (pass/drop based on map state).

### Phase B: Userspace Controller
- [ ] Add `ThrottleManager` to handle map updates and token refills.
- [ ] Update `App` state to track active limits.

### Phase C: TUI & UX
- [ ] Implement the `s` hotkey dialog for setting limits.
- [ ] Update `ProcessTable` to show active limits and "dropped" statistics.
- [ ] Add a "Throttled" view/tab to see all restricted processes.

## 4. Verification & Testing
- **Validation:** Use `wget` or `iperf3` to verify that a process restricted to 100KB/s does not exceed that rate.
- **Overhead:** Monitor CPU usage to ensure the per-packet check in `cgroup_skb` is efficient.
- **Safety:** Ensure that "Unknown" or critical system processes aren't accidentally throttled unless explicitly requested.
