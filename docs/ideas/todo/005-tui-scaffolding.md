# Task 005: TUI Scaffolding (Ratatui MVP)

**Status:** Proposed
**Phase:** 2 (MVP)
**Objective:** Replace the simple `info!` logging with a functional Terminal UI using `Ratatui` to display real-time bandwidth statistics.

## 1. Research & Strategy
- [ ] Review `Ratatui`'s `Table` and `Layout` widgets for a multi-column process view.
- [ ] Determine how to handle terminal events (key presses, resizing) alongside the 1Hz eBPF map polling. **Constraint:** Keep it single-threaded or use simple `tokio` tasks to avoid complex state synchronization.

## 2. Implementation (`netmonitor`)
- [ ] Define the `App` struct to hold process statistics and UI state (table selection, etc.).
- [ ] Implement a basic layout with three areas:
    - **Header:** Title and system summary (total bytes sent).
    - **Main:** A table showing `[Name] (PID)`, `Bytes Sent`, and `Packets Sent`.
    - **Footer:** Helpful shortcuts (e.g., `q` to quit).
- [ ] Integrate `Crossterm` for terminal raw mode and event handling.
- [ ] Refactor the main loop in `main.rs` to render the UI on every interval tick and handle terminal events.

## 3. Integration
- [ ] Use `ProcessResolver` to populate the names in the TUI table.
- [ ] Ensure the table updates correctly as new PIDs appear in the eBPF maps.

## 4. Verification
- [ ] Run `cargo xtask run`.
- [ ] Verify that the TUI renders correctly in the terminal.
- [ ] Verify that pressing `q` exits the application gracefully (restoring terminal state).
- [ ] Ensure bandwidth stats are still updating in real-time within the UI.
