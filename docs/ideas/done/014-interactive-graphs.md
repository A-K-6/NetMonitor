# Task 014: Interactive Graphs & Historical View

**Status:** Completed
**Phase:** 3 (Advanced)
**Date:** 2026-03-20
**Implementation Details:**
- `DbManager::get_traffic_history` provides time-bucketed data from SQLite.
- `ui::render_graph_view` draws a `ratatui::widgets::Chart`.
- Range cycling (10m, 1h, 24h) via `Tab` in Graph View.
**Objective:** Leverage the persisted SQLite data to provide full-screen historical bandwidth graphs for specific processes, allowing users to analyze traffic spikes over time.

## 1. Research & Strategy
- [x] **Data Retrieval:** Implement a method in `DbManager` to query `traffic_log` for a specific PID over a time range (e.g., last 10 minutes, 1 hour, or 24 hours).
- [x] **UI Component:** Use `ratatui::widgets::Chart` or `Sparkline` (for a more detailed historical view) to render the time-series data.
- [x] **Navigation:** Add a hotkey (e.g., `g`) to enter the "Graph View" for the selected process.

## 2. Implementation (Userspace)
- [x] **Database Query:** 
    - Add `get_traffic_history(pid, duration)` to `DbManager`.
    - Ensure results are bucketed (e.g., sum bytes per 10-second or 1-minute window for larger timeframes).
- [x] **TUI State:** 
    - Add `show_graph_view: bool` and `selected_process_history: Vec<(f64, f64)>` to the `App` struct.
- [x] **Rendering:**
    - Create `ui::render_graph_view` to draw a full-screen chart with X-axis (time) and Y-axis (bandwidth).
    - Support toggling between "Upload" and "Download" or showing both as separate lines.
- [x] **Interactions:**
    - `Esc`: Return to the main process table.
    - `Tab`: Cycle through time ranges (10m, 1h, 24h).

## 3. Verification
- [x] **Functional Test:** Select a process, generate bursty traffic, wait for a flush, and verify the graph correctly reflects the spike in "Graph View".
- [x] **Performance Test:** Ensure that large historical queries don't lock the UI; use `tokio::spawn` or async DB calls if necessary.

## 4. Documentation
- [x] Update `TUI/TECHNICAL_SPEC.md` with the new Graph View layout.
- [x] Log completion in `devlog.md`.
