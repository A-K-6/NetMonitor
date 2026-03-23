# Idea: Headless Mode & Data Exporting

**Status:** Proposed
**Phase:** 4 (Stability & Testing)
**Objective:** Transform NetMonitor from a TUI-only tool into a versatile observability platform that supports scripting, automation, and remote monitoring.

## 1. Context
While the TUI is excellent for interactive troubleshooting, power users often need to:
1.  **Scripting:** Parse bandwidth data using `jq` or Python.
2.  **Archiving:** Save periodic snapshots of network activity to CSV for long-term audit.
3.  **Observability Integration:** Export metrics to Prometheus or Grafana for dashboarding.
4.  **Low-Resource Monitoring:** Run as a background process without the overhead of rendering a TUI.

## 2. Proposed Tasks

### Task A: CLI Command Expansion (Headless Flags)
- **`--headless`:** Disables the Ratatui TUI and runs in the background or terminal stdout mode.
- **`--output <format>`:** Supports `json`, `csv`, and `plain`.
- **`--interval <seconds>`:** Defines how often data snapshots are printed or logged in headless mode.
- **`--count <n>`:** Exit after `n` iterations (useful for one-off snapshots).

### Task B: JSON & CSV Serializers
- **`ProcessSnapshot` Struct:** Create a serializable representation of the current bandwidth table.
- **JSON Output:** Implement standard JSON array output (e.g., `[{"pid": 123, "app": "firefox", "download": 1024, ...}]`).
- **CSV Output:** Implement RFC 4180 compliant CSV formatting with a header row.

### Task C: Background Logging (File Export)
- **`--log-file <path>`:** Instead of printing to stdout, append snapshots to a specified file.
- **Rotation Support:** (Optional) Basic integration with system log-rotate or a simple file-append strategy.

### Task D: Prometheus Exporter (Experimental)
- **`--prometheus-port <port>`:** Spin up a lightweight HTTP server (e.g., using `tiny_http` or `warp`) to expose a `/metrics` endpoint.
- **Metric Mapping:** Convert `TrafficService` data into Prometheus-compatible Gauges (e.g., `netmonitor_process_download_bytes_total{pid="123", app="nginx"}`).

## 3. Implementation Plan

1.  **Refactor Main Loop:** Decouple the TUI refresh loop from the `TrafficService` update loop to allow running without a terminal.
2.  **Add Clap Flags:** Update `Config` and `main.rs` to parse the new headless and output arguments.
3.  **Output Formatter Trait:** Create an `OutputFormatter` trait to abstract between TUI, JSON, and CSV rendering.
4.  **Signal Handling:** Ensure headless mode respects `SIGTERM` and `SIGINT` for graceful shutdown and resource cleanup.

## 4. Verification Criteria
- [ ] `netmonitor --headless --output json --count 1` returns a valid JSON object.
- [ ] `netmonitor --headless --output csv --interval 1 --log-file bandwidth.csv` correctly appends rows.
- [ ] Memory and CPU usage in headless mode is significantly lower than TUI mode (< 0.5% CPU).
- [ ] Graceful termination closes all eBPF maps and file descriptors.
- [ ] (If implemented) Prometheus endpoint returns valid metrics according to `promtool check metrics`.
