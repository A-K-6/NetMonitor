# Idea: Phase 3 Final Review & Stability Refactor

**Status:** Proposed
**Phase:** 3 (Advanced/Transition)
**Objective:** Conduct a comprehensive, project-wide code review and refactor session to consolidate Phase 3 features (Throttling, Geo-IP, DNS, Persistence) and ensure architectural stability before moving into Phase 4 (Stability & Testing).

## 1. Research & Strategy
- **Consolidation:** Phase 3 added significant complexity (SQLite, eBPF Cgroup hooks, Async DNS). We need to ensure these components are cleanly decoupled and share a consistent error-handling strategy.
- **Performance Audit:** 
    - Profile the user-space main loop to ensure 1Hz+ refresh doesn't spike CPU on systems with 1000+ active connections.
    - Review eBPF map access patterns (LRU for connections, Hash for stats).
- **Security & Least Privilege:** 
    - Finalize the capability-only execution path (`setcap`).
    - Audit `unsafe` blocks in Rust (especially where interacting with eBPF maps and libc).
- **Testing Infrastructure:** Define the base traits for mocking eBPF data and database interactions to enable Phase 4 unit testing.

## 2. Technical Specification
- **Modularization:** Move eBPF map management into a dedicated `BpfManager` or `KernelCollector` struct to isolate `aya` dependencies.
- **Error Handling:** Replace remaining `unwrap()` calls with proper `anyhow` or `thiserror` patterns.
- **TUI State:** Refactor `App` into sub-components (e.g., `Navigation`, `MonitoringState`, `AlertManager`) to reduce the "God Object" anti-pattern in `app.rs`.

## 3. Implementation Plan
- [ ] **Architecture Audit:**
    - [ ] Refactor `main.rs` to move eBPF loading/attaching into a separate module.
    - [ ] Clean up `app.rs` by moving dialog and navigation logic into smaller, testable units.
- [ ] **Performance & Resource Management:**
    - [ ] Optimize the SQLite flush frequency and batching logic.
    - [ ] Ensure all background tasks (DNS, Geo-IP) are properly handled via `tokio` join handles or similar.
- [ ] **Dependency & Safety Review:**
    - [ ] Audit all `unsafe` blocks for memory safety and documented invariants.
    - [ ] Update all dependencies in `Cargo.toml` to their latest stable versions.
- [ ] **Preparation for Testing:**
    - [ ] Define traits for the `DbManager` and `ProcessResolver` to allow for easy mocking.
    - [ ] Create a "Dummy eBPF" mode for testing the TUI without requiring root/eBPF capabilities.

## 4. Verification & Testing
- **Resource Monitoring:** Use `top`/`htop` and `valgrind` (if applicable) to monitor memory and CPU usage under heavy network load.
- **Binary Size:** Check the impact of embedded bytecode and bundled resources (GeoIP DB) on the final binary size.
- **Binary Portability:** Test the release binary on different kernel versions (CO-RE verification).
