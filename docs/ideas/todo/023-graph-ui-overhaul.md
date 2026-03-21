# Task 023: Graph UI Overhaul (Visual Analytics)

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Enhance the traffic history visualization with a more professional and informative graph interface, supporting multi-process overlays and better axis management.

## 1. Research & Strategy
- [ ] **Multi-Process Support:**
    - Allow selecting multiple processes (e.g., via spacebar in the table) to overlay on the graph.
    - Assign different colors to different processes based on the current theme.
- [ ] **Axis & Legend Improvement:**
    - Implement dynamic Y-axis scaling that is easier to read (e.g., "1.2 MB/s" instead of "1228.8 KB/s").
    - Add a clear legend indicating which line belongs to which process/direction.
- [ ] **Interactivity:**
    - Allow toggling between "Logarithmic" and "Linear" scales for better visualization of small vs large spikes.
    - Support zooming or panning if feasible within Ratatui constraints.

## 2. Implementation (Userspace)
- [ ] **App State Update:**
    - Add `selected_pids: HashSet<u32>` to `App` for multi-process selection.
    - Add `graph_scale_log: bool` to toggle axis scaling.
- [ ] **Graph Renderer Update:**
    - Refactor `render_graph_view` in `ui.rs` to loop through `selected_pids` and create multiple datasets.
    - Implement a legend widget or section within the graph block.
- [ ] **Input Handling:**
    - Update `main.rs` to handle `Space` for selecting/deselecting processes in the table.
    - Add a keybinding (e.g., `l`) to toggle logarithmic scale within the graph view.

## 3. Verification
- [ ] **Performance:** Ensure that rendering multiple lines doesn't cause lag in the TUI refresh.
- [ ] **Visual Clarity:** Confirm that different processes are easily distinguishable by color and legend.
- [ ] **Accuracy:** Verify that the dynamic scaling correctly represents the data points.

## 4. Documentation
- [ ] Update `devlog.md` with progress.
- [ ] Update `ROADMAP.md` (already updated).
