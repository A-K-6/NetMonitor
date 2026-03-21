# Task 019: Extended Mouse Support

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Enhance mouse interaction by making table headers clickable for sorting and adding click support for dialog buttons (Kill, Theme, etc.).

## 1. Research & Strategy
- [ ] **Header Hit-Testing:** 
    - Determine the X-coordinates of each column in the process table.
    - Map a click on the header row to the specific column to trigger `toggle_sort(column)`.
- [ ] **Dialog Interaction:**
    - Map clicks on "y/n" text areas in the Kill dialog to their respective actions.
    - Map clicks on footer keybinding hints to trigger those actions (e.g., clicking "Quit" exits the app).
- [ ] **Coordinate Precision:** 
    - Ensure hit-testing remains accurate even when the terminal is resized (handled by dynamic layout calls).

## 2. Implementation (Userspace)
- [ ] **Sorting Integration:** 
    - Update `netmonitor/src/main.rs` to detect clicks in the header area.
    - Calculate which column was clicked based on current widths.
- [ ] **Kill Dialog Click:**
    - Update event loop to handle clicks when `show_kill_dialog` is active.
- [ ] **Footer Interaction:**
    - (Optional) Make common footer actions clickable.

## 3. Verification
- [ ] **Sorting Test:** Click "UP (KB/s)" header and verify it toggles sort order/column.
- [ ] **Kill Dialog Test:** Click "(y)es" in the kill dialog and verify the process is killed.
- [ ] **Regression Test:** Ensure existing row selection and scrolling still work.

## 4. Documentation
- [ ] Update `devlog.md` with the proposal.
- [ ] Update `ROADMAP.md` (already updated).
