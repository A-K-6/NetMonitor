# Task 007: TUI Polish & The "Kill-Switch"

**Status:** Proposed
**Phase:** 2-3 (MVP-Advanced)
**Objective:** Finalize Phase 2 TUI Polish and implement the first Phase 3 advanced feature: the actual Process Kill-Switch.

## 1. Research & Strategy
- [ ] Implement `libc::kill` in `main.rs` to send `SIGKILL` (9) to a selected PID.
- [ ] Investigate `Ratatui`'s `Sparkline` widget for real-time traffic visualization in the header or a dedicated detail view.
- [ ] Refine table column alignment (Right-align numbers) as per `STYLE_GUIDE.md`.

## 2. Implementation (`netmonitor`)
- [ ] **Kill-Switch:** 
    - [ ] Update `main.rs` event loop to call `libc::kill(pid, libc::SIGKILL)` when user confirms in the kill dialog.
    - [ ] Add a visual notification (e.g., a status message) when a process is killed successfully or fails.
- [ ] **TUI Alignment:** 
    - [ ] Update `ui.rs` to right-align "UP", "DOWN", and "TOTAL" columns.
- [ ] **Sparklines:** 
    - [ ] Add `history_up: VecDeque<u64>` and `history_down: VecDeque<u64>` to `App` to store the last ~50-100 global bandwidth data points.
    - [ ] Add a `Sparkline` widget to the Header area to show global traffic trends.
- [ ] **Deep-Dive Detail View (Placeholder):**
    - [ ] Implement `Enter` key to toggle a "Process Detail" popup.
    - [ ] Show `packets_sent` and `packets_recv` in the detail view.

## 3. Integration
- [ ] Ensure `SIGKILL` requires the same capabilities as the rest of the app (or appropriate permissions).
- [ ] Update the footer to reflect the new `Enter` deep-dive functionality.

## 4. Verification
- [ ] Build and run.
- [ ] Select a harmless process (like a `sleep` command) and verify `k` -> `y` actually kills it.
- [ ] Observe global sparklines moving as traffic flows.
- [ ] Verify right-aligned numbers in the table look professional.
