# Idea: Phase 3 Professional Review & Architectural Refactor

**Status:** Proposed
**Phase:** 3 (Advanced/Transition)
**Objective:** Transform the current MVP-style codebase into a professional-grade, modular architecture. This multi-step refactor focuses on decoupling the kernel-space interaction, business logic, and TUI, ensuring the project is sustainable, testable, and strictly adheres to Rust best practices.

## 1. Research & Strategy (The Vision)
- **Architectural Boundary:** Establish a "Clean Architecture" approach where the TUI is merely a consumer of a "Monitoring Service."
- **Infrastructure First:** Build a robust abstraction layer for eBPF maps to allow for future backend swaps.
- **Concurrency & Safety:** Move toward a clear ownership model using channels and safe state-update protocols.
- **Idiomatic Rust:** Leverage the type system to make illegal states unrepresentable (Type-Driven Design).

---

## 2. Technical Specification: The Multi-Step Process

### Step 1: Core Decoupling (The Kernel Bridge)
- **Goal:** Isolate `aya` and BPF map management.
- **Action:** Create a `KernelCollector` trait and a default `AyaCollector` implementation.
- **Rust Practice:** Use `async-trait` or standard traits with `Result` types for clean error propagation.

### Step 2: State Management Refactor (Domain Model)
- **Goal:** Solve the "God Object" anti-pattern in `app.rs`.
- **Action:** Split `App` into focused Domain Services (`TrafficService`, `IdentityService`, `EnforcementService`).
- **Rust Practice:** Use the **Newtype pattern** for PIDs and Bytes to prevent logic errors (e.g., `Pid(u32)`).

### Step 3: Observability & Idiomatic Hardening
- **Goal:** "Professional" code quality and resilience.
- **Action:** 
    - Standardize error handling with `thiserror` and `anyhow`.
    - **Linting:** Enforce `deny(clippy::all)` and `deny(clippy::pedantic)` in critical modules.
    - **Documentation:** Ensure all public APIs are documented with `///` for `rustdoc`.
- **Impact:** Code that feels "native" to senior Rust developers and is easy to maintain.

### Step 4: Testing & Mocking Framework
- **Goal:** Zero-dependency automated testing.
- **Action:** 
    - Implement a `MockCollector` for non-root testing.
    - **Formatting:** Ensure project-wide `cargo fmt` compliance.
    - **CI Integration:** Create a `test.sh` script to verify build, clippy, and tests in one pass.

---

## 3. Implementation Plan

- [ ] **Step 1: Kernel Abstraction**
    - [ ] Define `Collector` trait in a new `core` module.
    - [ ] Move map-polling logic into `AyaCollector`.
- [ ] **Step 2: Service Layer Extraction**
    - [ ] Extract `ProcessResolver` and `DbManager` behind traits.
    - [ ] Implement **Type-Driven** updates (e.g., using a `State` enum for UI).
- [ ] **Step 3: Quality & Safety Audit**
    - [ ] Audit and remove all `unwrap()`/`expect()`.
    - [ ] Run `cargo clippy -- -D warnings` and fix all violations.
    - [ ] Add doc-comments to all core modules.
- [ ] **Step 4: Infrastructure Validation**
    - [ ] Verify TUI boots via `MockCollector`.
    - [ ] Enforce `cargo fmt` across the workspace.

## 4. Verification & Testing
- **Refactor Integrity:** No regression in bandwidth accuracy.
- **Code Quality:** Zero `clippy` warnings and 100% `cargo fmt` compliance.
- **Architecture Validation:** TUI is fully decoupled from `aya`.
