# Task 006: Filtering & Cumulative Stats Tracking

**Status:** Proposed
**Phase:** 2 (MVP)
**Objective:** Add process filtering by name, refine sorting mechanics, and implement global cumulative data tracking.

## 1. Research & Strategy
- [ ] Investigate how to cleanly capture keyboard input in `Ratatui` without blocking the main event loop to create a "search/filter bar".
- [ ] We currently track "Total bytes uploaded" across iterations, but since we recreate the `process_data` list every tick, we lose historical totals if a process dies or stops sending traffic. We need a persistent state layer to hold cumulative metrics across the entire application lifetime.

## 2. Implementation (`netmonitor`)
- [ ] Add `filter_text: String` and `is_filtering: bool` state variables to `App`.
- [ ] Update `tui.rs`/`main.rs` to intercept keystrokes and append/pop characters to `filter_text` when `is_filtering` is active.
- [ ] Modify `ui.rs` to render a search bar (perhaps modifying the Header or Footer area dynamically) to display the active filter text.
- [ ] Modify the data aggregation loop in `main.rs` to skip inserting rows into `process_data` if their name doesn't contain `filter_text` (case-insensitive).
- [ ] Create a persistent `process_history: HashMap<u32, ProcessRow>` or similar structure in `App` to maintain a running total of `total_bytes` per PID, even if the eBPF map values reset or the process pauses network activity.

## 3. Integration
- [ ] Assign the hotkey `/` or `f` to toggle `is_filtering` state.
- [ ] Allow `Esc` or `Enter` to confirm/exit the filtering mode.

## 4. Verification
- [ ] Build and run the project (`cargo xtask run`).
- [ ] Press the filter hotkey, type "firefox" or "curl", and verify the table shrinks to show only matching processes.
- [ ] Verify that cumulative `TOTAL` bytes accurately increase over a multi-minute session, rather than resetting.
