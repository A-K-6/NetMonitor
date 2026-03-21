# Task 021: Global Dashboard (Overall View)

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Implement a "Global Dashboard" view that provides a high-level summary of system-wide network health and activity, aggregating data across all processes and protocols.

## 1. Research & Strategy
- [ ] **Aggregation Logic:**
    - Define global metrics: Total system throughput (Up/Down), Protocol distribution (TCP vs UDP vs ICMP), Top 5 "Talker" processes, and Top 5 Destination Countries.
    - Ensure real-time updates for these aggregates without redundant calculations.
- [ ] **UI Design:**
    - Layout a new dashboard screen using Ratatui `Block`s and `Gauge`s.
    - Plan for a "Big Number" display for current throughput.
    - Integrate small Sparklines for system-wide history.
- [ ] **State Management:**
    - Determine how to switch between the "Process Table" view and the "Global Dashboard" (e.g., using 'Tab' or a specific key).

## 2. Implementation (Userspace)
- [ ] **App State Update:**
    - Add `show_dashboard: bool` to `App`.
    - Add fields to `App` for storing global aggregates (updated during the tick).
- [ ] **Dashboard Renderer:**
    - Create `render_dashboard` in `netmonitor/src/ui.rs`.
    - Implement widgets for:
        - **Traffic Gauges:** Visualizing current load vs. a configurable "Max" or recent peak.
        - **Protocol Breakdown:** A simple list or bar showing % of TCP vs UDP traffic.
        - **Top Talkers:** A mini-table showing the most active processes.
        - **Geo-Summary:** Top countries by volume.
- [ ] **Input Handling:**
    - Add a keybinding (e.g., `Tab` or `D`) to toggle the dashboard.
    - Update footer hints to reflect the new view.

## 3. Verification
- [ ] **Data Consistency:** Verify that "Global Dashboard" totals match the sum of the "Process Table" rows.
- [ ] **UI Responsiveness:** Ensure switching views is instantaneous and does not leak memory.
- [ ] **Visual Clarity:** Confirm that gauges and sparklines provide immediate "at-a-glance" value.

## 4. Documentation
- [ ] Update `devlog.md` with progress.
- [ ] Update `ROADMAP.md` (already updated).
