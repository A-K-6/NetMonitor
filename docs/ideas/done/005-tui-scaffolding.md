# Task 005: TUI Scaffolding (Ratatui MVP)

**Status:** Proposed
**Phase:** 2 (MVP)
**Objective:** Replace the simple `info!` logging with a functional Terminal UI using `Ratatui` to display real-time bandwidth statistics, adhering to the project's styling and layout guidelines.

## 1. Research & Strategy
- [ ] Review `Ratatui`'s `Table` and `Layout` widgets for a multi-column process view.
- [ ] Determine how to handle terminal events (key presses, resizing) alongside the 1Hz eBPF map polling. **Constraint:** Keep it single-threaded or use simple `tokio` tasks to avoid complex state synchronization.
- [ ] Review `STYLE_GUIDE.md` for color palettes (Green for Upload, Yellow/Amber for Download, Blue/Cyan for Accent).
- [ ] Ensure smooth refreshing using double-buffering.

## 2. Implementation (`netmonitor`)
- [ ] Define the `App` struct to hold process statistics and UI state:
  - Core Data: `process_data: Vec<ProcessRow>`, `total_upload`, `total_download`.
  - UI State: `table_state: TableState`, `sort_column: Column`, `is_running: bool`.
- [ ] Implement layout with three areas (using `symbols::border::PLAIN` or `THICK` with 1-char padding):
    - **Header:** System-wide upload/download totals, kernel version, and active capabilities.
    - **Main:** A sortable table showing `PID` (left-aligned), `NAME` (left-aligned), `UP (KB/s)` (right-aligned), `DOWN (KB/s)` (right-aligned), and `TOTAL`. Highlight the active row.
    - **Footer:** Helpful shortcuts (`q` to quit, `k` to kill process, `s` to sort, `Enter` for deep-dive).
- [ ] Integrate `Crossterm` for terminal raw mode and event handling.
- [ ] Refactor the main loop in `main.rs` to render the UI on every interval tick and handle terminal events (Timer, Key, Resize).
- [ ] Implement dynamic resizing (hide secondary columns if terminal is too small).

## 3. Integration
- [ ] Use `ProcessResolver` to populate the names in the TUI table.
- [ ] Ensure the table updates correctly as new PIDs appear in the eBPF maps.
- [ ] Implement logic for the active sorted column (e.g., prefix header with `↓ UP`).
- [ ] Implement the structural foundation for confirmation dialogs (e.g., for the `k` kill-switch action).

## 4. Verification
- [ ] Run `cargo xtask run`.
- [ ] Verify that the TUI renders correctly in the terminal without flickering.
- [ ] Verify that resizing the terminal adapts the widget layout appropriately.
- [ ] Verify that pressing `q` exits the application gracefully (restoring terminal state).
- [ ] Ensure bandwidth stats (upload/download) update in real-time within the UI.
- [ ] Confirm ANSI colors match the styling guidelines (Green/Yellow/Blue).
