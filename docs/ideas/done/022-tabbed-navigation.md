# Task 022: Tabbed Navigation (UI Management)

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Replace the simple view toggling logic with a robust `Tabs` widget to manage multiple screens (Dashboard, Process Table, Alerts Log) more effectively.

## 1. Research & Strategy
- [ ] **Tab Structure:**
    - Define the core tabs: `Dashboard (F1)`, `Processes (F2)`, `Alerts (F3)`.
    - Plan for additional tabs like `History (F4)` or `Settings (F5)` if needed.
- [ ] **Navigation Logic:**
    - Use `Tab` key to cycle forward and `Shift+Tab` to cycle backward.
    - Support function keys (`F1`-`F3`) for direct access.
    - Support mouse clicks on the tab headers.
- [ ] **State Management:**
    - Refactor `App::view_mode` to be an index or a more formal `Tab` enum if not already done.
    - Ensure each tab maintains its own scroll/selection state where applicable.

## 2. Implementation (Userspace)
- [ ] **App State Update:**
    - Ensure `ViewMode` enum covers all desired tabs.
    - Add `tab_index: usize` or use `view_mode` directly.
- [ ] **Tab Renderer:**
    - Implement `render_tabs` in `netmonitor/src/ui.rs` using Ratatui `Tabs` widget.
    - Place the tabs at the top of the main content area (below the header).
- [ ] **Input Handling:**
    - Update `main.rs` to handle `F1`-`F3` keys.
    - Implement `Shift+Tab` detection if possible, or stick to `Tab` cycling.
    - Update footer to reflect the new navigation paradigm.

## 3. Verification
- [ ] **Consistency:** Ensure switching tabs doesn't reset the state of other tabs (e.g., sorting in the process table should persist).
- [ ] **Visual Clarity:** Confirm that the active tab is clearly highlighted.
- [ ] **Accessibility:** Verify that all views are accessible via both keyboard and mouse.

## 4. Documentation
- [ ] Update `devlog.md` with progress.
- [ ] Update `ROADMAP.md` (already updated).
