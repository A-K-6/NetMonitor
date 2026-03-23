# Idea: Automated Testing Suite & CI/CD Pipeline

**Status:** Proposed
**Phase:** 4 (Stability & Testing)
**Objective:** Establish a robust testing foundation and automate verification to ensure long-term stability and prevent regressions.

## 1. Context
The project has reached a high level of feature completeness (Phase 3 finished). However, testing is currently limited to basic unit tests for some components. To move towards a professional release, we need comprehensive coverage including the eBPF layer and automated checks.

## 2. Proposed Tasks

### Task A: Userspace Unit Testing (Expansion)
- **Service Layer:** Expand tests for `TrafficService`, `EnforcementService`, and `IdentityService` using the `MockCollector`.
- **Config & Persistence:** Add tests for `Config` loading/saving and `DbManager` edge cases (e.g., database corruption, migrations).
- **TUI State Logic:** Unit test `App` state transitions (e.g., sorting, filtering, view switching) without requiring a terminal.

### Task B: eBPF Integration Testing
- **Aya-test Integration:** Explore using `aya-test` to verify eBPF bytecode logic in a controlled environment.
- **Mocking Kernel Inputs:** Create tests that simulate network events (e.g., `tcp_sendmsg` calls) and verify the BPF map updates.

### Task C: CI/CD Pipeline (GitHub Actions)
- **Linting & Formatting:** Automate `cargo fmt --check` and `cargo clippy -- -D warnings`.
- **Automated Tests:** Run `cargo test` on every push and PR.
- **Build Verification:** Ensure both userspace and eBPF bytecode build successfully in a clean environment.

### Task D: Performance & Safety Audit
- **CPU/Memory Profile:** Use `valgrind` or `cargo-flamegraph` to identify hotspots.
- **Memory Safety:** Ensure no `unsafe` blocks are misused (primarily in TUI/libc interactions).

## 3. Roadmap for the Next Agent

1.  **Setup CI:** Create `.github/workflows/ci.yml` with basic lint/test/build steps.
2.  **Expand Unit Tests:** Increase coverage in `netmonitor/src/core/services/` to >80%.
3.  **Config Testing:** Add a dedicated test suite for `config.rs`.
4.  **Integration Scaffolding:** Set up the initial structure for eBPF integration tests.

## 4. Verification Criteria
- [ ] GitHub Actions green for the repository.
- [ ] `cargo test` covers all major business logic in the service layer.
- [ ] Zero `unsafe` blocks without documented rationale.
- [ ] Benchmark report showing idle CPU usage < 1%.
