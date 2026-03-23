# Idea: Final Polish, Technical Debt Cleanup & Architectural Validation

**Status:** Proposed
**Phase:** 3 (Consolidation)
**Objective:** Address the remaining technical debt introduced during the Phase 3 refactor. This includes resolving linting warnings (unused imports, dead code) and ensuring the new modular architecture is fully utilized across the codebase.

## 1. Context: Recap of Phase 3 Refactor (Task 029)
The project has undergone a major architectural shift to a **Service-Oriented Architecture**:
- **Kernel Abstraction:** `AyaCollector` now encapsulates all eBPF logic. The TUI interacts only with the `Collector` trait.
- **Service Layer:** Logic is divided into `TrafficService` (stats), `IdentityService` (PID resolution), `EnforcementService` (throttling/thresholds), and `MonitoringService` (orchestration).
- **Domain Model:** Data is passed via `MonitoringSnapshot` and `ProcessSummary` rather than raw eBPF map types.
- **Type Safety:** Introduced `Pid` and `Bytes` Newtypes to prevent primitive obsession.

---

## 2. Technical Debt: Remaining Warnings
The refactor left several "skeletal" components and unused imports that need to be cleaned up or fully integrated.

### Task A: Resolve Unused Imports
- **`netmonitor/src/app.rs`**: Clean up `Pid`, `Bytes`, `EnforcementService`, etc.
- **`netmonitor/src/core/mod.rs`**: Clean up re-exports that are not yet consumed by other modules.
- **`netmonitor/src/main.rs`**: Remove legacy imports of `aya` programs and maps that are now handled by `AyaCollector`.

### Task B: Dead Code & Skeletal Implementation Cleanup
- **Mocking Framework:** `MockCollector` is marked as unused because it is only used in unit tests. Wrap it or its implementation in `#[cfg(test)]` where appropriate, or verify if a dedicated test suite should consume it.
- **Enforcement Service:** The `thresholds` field and methods like `set_threshold` were moved from `App` but are currently shadowed by `App`'s local state. Migrate the remaining TUI logic to use the service methods.
- **Domain Model:** Fields like `total`, `up_rate`, and `down_rate` in `ProcessSummary` are populated with placeholders. Implement the rate calculation logic within `TrafficService`.
- **Storage Abstraction:** The `Storage` trait and `DbStorage` are defined but not yet linked to the `DbManager`. Bridge these to allow for mockable storage.

---

## 3. Roadmap for the Next Agent

1.  **Audit & Lint:** Run `cargo check` and surgically remove all unused imports identified in the Phase 3 logs.
2.  **Service Integration:** Move the remaining business logic from `main.rs` (e.g., the alert logic and threshold checks) into `EnforcementService` and `MonitoringService`.
3.  **Rate Calculation:** Implement a proper `update_rates()` method in `TrafficService` to calculate bytes-per-second using the `history_up/down` buffers.
4.  **Full Test Coverage:** Expand tests in `core/collector.rs` and create new tests for `TrafficService` using the `MockCollector`.
5.  **Final Hardening:** Ensure `#[deny(clippy::all)]` passes across the `core` module.

---

## 4. Verification Criteria
- [ ] Zero `cargo check` warnings (including unused imports and dead code).
- [ ] `TrafficService` provides real-time rate calculations.
- [ ] `main.rs` contains zero direct eBPF program management logic.
- [ ] `cargo test` passes for all modules.
