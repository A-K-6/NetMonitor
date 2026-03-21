# Task 018: Mouse Support (Click & Scroll)

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Add basic mouse interaction to the TUI to allow selecting processes by clicking and scrolling through the table.

## 1. Research & Strategy
- [ ] **Event Capture:** Ensure `crossterm` is configured to capture mouse events in `netmonitor/src/tui.rs`.
- [ ] **Coordinate Mapping:** 
    - Determine how to map a mouse click (x, y) to a specific row in the `Table` widget.
    - Ratatui doesn't have built-in "hit testing" for table rows, so we'll need to calculate offsets based on the table's position and current scroll state.
- [ ] **Scrolling:** Map `MouseScrollUp` and `MouseScrollDown` to the existing `previous()` and `next()` logic in `App`.

## 2. Implementation (Userspace)
- [ ] **TUI Initialization:** Update `netmonitor/src/tui.rs` to enable mouse capture via `EnableMouseCapture`.
- [ ] **Event Handling:** 
    - Update the event loop in `netmonitor/src/main.rs` to handle `Event::Mouse`.
    - Implement `MouseScrollUp` and `MouseScrollDown` to scroll the process list.
    - (Optional) Implement `MouseButton::Left` to select a row under the cursor.
- [ ] **TUI Cleanup:** Ensure `DisableMouseCapture` is called on exit.

## 3. Verification
- [ ] **Scrolling Test:** Verify that using the mouse wheel/trackpad correctly scrolls the process table.
- [ ] **Selection Test:** (If implemented) Verify that clicking a row selects it.
- [ ] **Regression Test:** Ensure keyboard navigation still works perfectly alongside mouse support.

## 4. Documentation
- [ ] Update `devlog.md` with the proposal.
- [ ] Update `ROADMAP.md` (if not already listed).
