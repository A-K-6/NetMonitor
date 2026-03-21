# Task 020: Time-Travel/Historical Analysis

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Implement a mechanism to select specific time windows from the SQLite database to analyze historical network usage and trends.

## 1. Research & Strategy
- [ ] **Database Queries:**
    - Develop SQL queries to aggregate bandwidth data for a user-specified time range.
    - Ensure efficient querying with appropriate indexes on timestamps and PIDs.
- [ ] **UI for Time Selection:**
    - Design a modal dialog to allow the user to input or select start/end times or relative ranges (e.g., "Last 4 hours").
    - Provide a "playback" or "static view" mode for the process table reflecting that period.
- [ ] **Performance:**
    - Ensure large historical data sets do not hang the UI during aggregation.

## 2. Implementation (Userspace)
- [ ] **DbManager Update:**
    - Add methods to `netmonitor/src/db.rs` to fetch aggregated stats for any time window.
- [ ] **App State:**
    - Add `historical_view_mode: bool` and `selected_time_range: Option<TimeWindow>` to `App`.
- [ ] **UI Integration:**
    - Implement the "Time-Travel" dialog.
    - Modify the main table renderer to switch data source when in historical mode.

## 3. Verification
- [ ] **Range Accuracy:** Select a 1-hour range from 2 hours ago and verify stats match expected data.
- [ ] **Performance Test:** Query a 24-hour range and ensure UI responsiveness.
- [ ] **Consistency:** Ensure "Live" mode can be resumed seamlessly.

## 4. Documentation
- [ ] Update `devlog.md` with progress.
- [ ] Update `ROADMAP.md` (already updated).
