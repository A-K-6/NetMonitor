# Idea: Local Automated Verification & Stability Suite

**Status:** Proposed
**Phase:** 4 (Stability & Testing)
**Objective:** Establish a robust local testing foundation to ensure eBPF accuracy, TUI stability, and PRD-compliant performance without external CI dependencies.

## 1. Context
NetMonitor requires high precision and kernel compatibility. Since we are focusing on **Local-First Development**, we need a "Pre-Flight" verification system that a developer can run locally to certify the binary's accuracy and safety before release.

## 2. Proposed Tasks

### Task A: Userspace & Service Layer Logic
- **State Machine Testing:** Unit test `App<C, R>` transitions (e.g., verifying that hotkey 'k' correctly sets the kill dialog state).
- **Service Layer Mocks:** Full coverage for `TrafficService` and `EnforcementService` using the `MockCollector`.
- **Config Property Testing:** Use `proptest` to ensure the `Config` parser is resilient against malformed `config.toml` files.

### Task B: eBPF Unit & Logic (Local Kernel)
- **aya-test Logic Tests:** Mock kernel contexts to verify the Token Bucket Filter (TBF) math in `netmonitor-ebpf` at nanosecond precision.
- **Resource Leak Audit:** Verify that BPF map file descriptors are correctly closed when the application exits to prevent system-wide resource exhaustion.

### Task C: Isolated Accuracy & Benchmarking
- **Isolated Accuracy Suite:** Use **Linux Network Namespaces (`ip netns`)** to create a "clean-room" environment. Generate controlled traffic (using `iperf3` or `curl`) to verify < 2% error margins without host noise.
- **Performance Guardrails:** Measure CPU overhead during a simulated 1Gbps burst; fail the local verification if overhead exceeds 1%.

### Task D: TUI Snapshot & UX Verification
- **TestBackend Snapshotting:** Use `ratatui`'s `TestBackend` to render the UI to a virtual buffer and verify layouts for "Kill-Switch", "Details", and "Historical" views across various terminal sizes.

### Task E: Local "Pre-Flight" Certification (The `xtask` approach)
- **Unified Verification Command:** Implement `cargo xtask verify` to automate the local sequence:
    1. `cargo fmt --check` & `cargo clippy`.
    2. Unit tests (Userspace + eBPF Logic).
    3. Accuracy tests in an isolated namespace (requires `sudo` or caps).
    4. Binary size and Capability (`getcap`) audit.
- **Local Multi-Kernel Check:** (Optional) Use `vmtest` to run the test suite against different local kernel images to ensure CO-RE compatibility.

## 3. Roadmap for the Next Agent

1.  **Mock Extension:** Ensure `MockCollector` covers 100% of the `Collector` trait methods.
2.  **App State Tests:** Add the first TUI-specific unit tests in `netmonitor/src/app.rs`.
3.  **Isolated Accuracy Script:** Create a bash script or `xtask` that sets up a `veth` pair in a network namespace for clean-room traffic testing.
4.  **Resource Audit:** Add a basic leak check using `valgrind` or `LSan` for the local build.

## 4. Verification Criteria
- [ ] `cargo xtask verify` passes locally with zero warnings.
- [ ] Accuracy report proves < 2% margin of error in an isolated namespace.
- [ ] TUI snapshot tests cover all major interactive dialogs.
- [ ] Peak CPU usage verified < 1% on the local development machine.
- [ ] Binary successfully requests `CAP_BPF` and `CAP_NET_ADMIN` without full root.
