# Task 004: Process Resolver (/proc crawler)

**Status:** Proposed
**Phase:** 2 (MVP)
**Objective:** Implement a mechanism to map Process IDs (PIDs) collected from the eBPF maps to human-readable process names (e.g., `1234` -> `firefox`).

## 1. Research & Strategy
- [ ] Investigate efficient manual parsing of `/proc/[pid]/comm` (short name) vs `/proc/[pid]/cmdline` (full path). **Constraint:** Prefer manual parsing over large dependencies like `sysinfo` to keep the binary lean.
- [ ] Caching Strategy: Use a `HashMap<u32, String>` with a 10-second TTL or a "refresh on map miss" logic to handle PID recycling (where a new process takes the ID of an old one).

## 2. Implementation (`netmonitor`)
- [ ] Create a `process` module to handle PID-to-Name resolution.
- [ ] Implement a function `get_process_name(pid: u32) -> Option<String>`.
- [ ] Integrate a simple `HashMap<u32, String>` cache in the userspace loop to store resolved names.

## 3. Integration
- [ ] Update the main statistics loop to use the resolver.
- [ ] Output should change from `PID: 1234 -> Sent: ...` to `[firefox] (1234) -> Sent: ...`.

## 4. Verification
- [ ] Run `cargo xtask run`.
- [ ] Verify that common processes (curl, wget, browser) are correctly identified by name in the console output.
