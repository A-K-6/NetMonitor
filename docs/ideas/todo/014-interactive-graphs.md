# Task 014: Interactive Graphs & Historical View

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Leverage the persisted SQLite data to provide full-screen historical bandwidth graphs for specific processes, allowing users to analyze traffic spikes over time.

## 1. Research & Strategy
- [ ] **Data Retrieval:** Implement a method in `DbManager` to query `traffic_log` for a specific PID over a time range (e.g., last 10 minutes, 1 hour, or 24 hours).
- [ ] **UI Component:** Use `ratatui::widgets::Chart` or `Sparkline` (for a more detailed historical view) to render the time-series data.
- [ ] **Navigation:** Add a hotkey (e.g., `g`) to enter the "Graph View" for the selected process.

## 2. Implementation (Userspace)
- [ ] **Database Query:** 
    - Add `get_traffic_history(pid, duration)` to `DbManager`.
    - Ensure results are bucketed (e.g., sum bytes per 10-second or 1-minute window for larger timeframes).
- [ ] **TUI State:** 
    - Add `show_graph_view: bool` and `selected_process_history: Vec<(f64, f64)>` to the `App` struct.
- [ ] **Rendering:**
    - Create `ui::render_graph_view` to draw a full-screen chart with X-axis (time) and Y-axis (bandwidth).
    - Support toggling between "Upload" and "Download" or showing both as separate lines.
- [ ] **Interactions:**
    - `Esc`: Return to the main process table.
    - `Tab`: Cycle through time ranges (10m, 1h, 24h).

## 3. Verification
- [ ] **Functional Test:** Select a process, generate bursty traffic, wait for a flush, and verify the graph correctly reflects the spike in "Graph View".
- [ ] **Performance Test:** Ensure that large historical queries don't lock the UI; use `tokio::spawn` or async DB calls if necessary.

## 4. Documentation
- [ ] Update `TUI/TECHNICAL_SPEC.md` with the new Graph View layout.
- [ ] Log completion in `devlog.md`.
