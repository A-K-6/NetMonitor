# Task 016: Smart Bandwidth Alerts & Thresholds

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Implement a mechanism to set bandwidth thresholds for specific processes and trigger alerts when these thresholds are exceeded.

## 1. Research & Strategy
- [ ] **Data Model:** 
    - Add `thresholds: HashMap<u32, u64>` to `App` (PID -> Bytes/s).
    - Add `alerts: VecDeque<Alert>` to `App` for historical alerts.
    - Define `Alert` struct: `timestamp`, `pid`, `process_name`, `value`, `threshold`.
- [ ] **UI Component:**
    - Create a dialog to set thresholds (hotkey `a`).
    - Add an "Alerts" indicator in the header if active alerts exist.
    - Potentially a dedicated "Alert Log" view (hotkey `A`).
- [ ] **Logic:**
    - In the 1Hz update loop, check if current `up_bytes + down_bytes` > `threshold`.
    - If exceeded, push to `alerts` and update `status_message`.

## 2. Implementation (Userspace)
- [ ] **App State:** Update `netmonitor/src/app.rs` with `thresholds` and `alerts`.
- [ ] **Keyboard Handling:** 
    - Update `netmonitor/src/main.rs` to handle `a` (Set Threshold).
    - Implement a simple numeric input dialog for the threshold (KB/s).
- [ ] **Rendering:**
    - Create `ui::render_threshold_dialog` in `netmonitor/src/ui.rs`.
    - Update the process table to highlight rows that are exceeding their thresholds (e.g., Red text).
    - Display recent alerts in the status bar or a small overlay.

## 3. Verification
- [ ] **Functional Test:** Set a low threshold (e.g., 10 KB/s) for a known high-traffic process (e.g., a browser or `curl`) and verify the alert triggers.
- [ ] **UI Test:** Ensure the threshold dialog is easy to use and the visual feedback (red rows) is clear.

## 4. Documentation
- [ ] Log completion in `devlog.md`.
- [ ] Update `ROADMAP.md` if necessary.
