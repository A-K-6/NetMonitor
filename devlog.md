2026-03-23 11:30 - Completed Task 030: Final Polish, Technical Debt Cleanup & Architectural Validation.
- Resolved all unused imports and dead code warnings left from the Phase 3 refactor.
- Integrated `EnforcementService` into the TUI to handle real-time thresholds and throttles.
- Implemented `update_rates()` logic in `TrafficService` and bound it to `ProcessSummary`.
- Handled mock tests inside `#[cfg(test)]` modules and increased overall coverage for services.
- Achieved strict `#[deny(clippy::all)]` compliance with zero warnings.
- Moved 030 idea document to `done/`.

2026-03-23 10:15 - Proposed Task 030: Final Polish, Technical Debt Cleanup & Architectural Validation.
- Identified remaining linting warnings (unused imports, dead code) from the Phase 3 refactor.
- Created `docs/ideas/todo/030-final-polish-and-cleanup.md` with a detailed roadmap for the next agent.
- Documented the architectural recap and technical debt resolution strategy.

2026-03-23 10:00 - Completed Task 029: Phase 3 Professional Review & Architectural Refactor.
- Implemented `Collector` trait to abstract eBPF interaction from the TUI.
- Created `AyaCollector` for real kernel monitoring and `MockCollector` for zero-dependency testing.
- Introduced a modular Service Layer (`TrafficService`, `IdentityService`, `EnforcementService`, `MonitoringService`).
- Refactored `App` to be generic and decoupled from the specific eBPF backend.
- Standardized error handling with `thiserror` and established a `core` domain model.
- Verified architecture with a new test suite and project-wide `cargo fmt` compliance.

2026-03-22 18:45 - Proposed Task 029: Phase 3 Final Review & Stability Refactor.
- Defined a new transitional task to consolidate Phase 3 features and refactor for Phase 4.
- Added `docs/ideas/todo/029-stability-and-refactor.md`.

2026-03-22 18:30 - Completed Task 028: Comprehensive README & Documentation.
- Created a professional `README.md` featuring a custom ASCII header and clear project mission.
- Documented all core and advanced features, including the new Traffic Shaping (Throttling) capability.
- Detailed the full suite of TUI hotkeys (Navigation, Sorting, Killing, Throttling, etc.).
- Provided a technical architecture overview of the eBPF hooks (TCP/UDP/RAW/Cgroup) and Rust components.
- Added a "Quick Start" guide with installation steps and capability management instructions.
- Documented the `config.toml` structure for user personalization.
- Updated `ROADMAP.md` and archived the idea file as completed.

2026-03-22 17:45 - Completed Task 027: Active Traffic Shaping (Bandwidth Throttling).
- Implemented `cgroup_skb` eBPF programs for egress and ingress to enforce bandwidth limits.
- Developed a Token Bucket Filter (TBF) algorithm in eBPF with nanosecond precision using `bpf_ktime_get_ns()`.
- Added `ThrottleConfig` map to `netmonitor-ebpf` for per-PID bandwidth management.
- Integrated `ThrottleManager` logic into the TUI with a new hotkey 'b' to set limits in KB/s.
- Enhanced the Process Table with a `[L]` (Limited) indicator for throttled processes.
- Verified compilation and linking for both eBPF and userspace components.

2026-03-22 15:30 - Strategic Roadmap Expansion for v0.1.0 and Production Hardening.
- Expanded `ROADMAP.md` with detailed Phase 4 (Stability & Testing) and Phase 5 (Release v0.1.0).
- Introduced **Phase 6: Hardening & Ecosystem** to address security audits, least privilege, and long-term maintainability.
- Integrated automated testing (unit/integration) and CI/CD pipelines into the core development flow.
- Deferred `CONTRIBUTING.md` and community growth tasks to Phase 6 to prioritize immediate release goals.
- Verified alignment between `PRD.md` and the updated multi-phase roadmap.

2026-03-22 14:15 - Completed Task 026: Container & Service Context Resolution.
- Implemented cgroup v2 path parser in `process.rs` to resolve systemd services, Docker containers, and Kubernetes pods.
- Added `ProcessContext` enum and updated `ProcessResolver` to cache metadata.
- Enhanced TUI with a "Context" view toggle (key 'c') in the process table.
- Updated sorting and filtering to handle context-aware metadata.
- Verified resolution for system services and added unit tests for `ProcessResolver`.

2026-03-22 12:30 - Completed Task 025: Persistent Configuration System.
- Implemented a robust TOML-based configuration system using `serde` and `toml` crates.
- Adhered to XDG Base Directory Specification for config file location via `directories` crate.
- Added CLI support for custom config paths using `clap`.
- Decoupled UI settings (theme, refresh rate, default view) and network settings (DNS, Geo-IP) into the configuration file.
- Integrated name-based bandwidth thresholds into the alert system.
- Verified compilation and added status messages for config loading in the TUI.

2026-03-22 11:10 - Corrected Roadmap and Restored Feature Goals.
- Restored "Container & Service Context", "Active Traffic Shaping", and "Headless Data Exporting" to the Roadmap as high-level goals.
- Removed all Task IDs/numbers from the Roadmap to maintain a clean, goal-oriented view.
- Polished the Persistent Configuration idea file as the primary next step.

2026-03-22 11:00 - Refined Roadmap and Polished Persistent Config Idea.
- Cleaned up `ROADMAP.md`: Removed explicit task numbering and premature feature entries for Container Context, Traffic Shaping, and Headless Exporting.
- Polished `docs/ideas/todo/025-persistent-config.md`: Added a detailed TOML schema, implementation phases, and technical specifications.
- Removed premature idea files (026, 027, 028) to focus on the current priority.

2026-03-22 10:30 - Completed Task 024: Adaptive Theming (System Integration).
- Integrated `dark-light` crate for system-wide light/dark mode detection on startup.
- Implemented `ThemeType::Terminal` using standard 16-color ANSI palettes and `Color::Reset`.
- Enhanced the theme engine to support a `bg` color, allowing for background transparency.
- Updated `ui.rs` to ensure all widgets respect the theme's background color.
- Refined existing themes (Dracula, Solarized, Monokai) with explicit background colors for consistency.
- Verified compilation and updated theme selection dialog to include "Auto" and "Terminal" options.

2026-03-21 16:30 - Completed Task 023: Graph UI Overhaul (Visual Analytics).
- Implemented multi-process overlay support in the traffic history graph.
- Added a legend to distinguish between different processes and traffic directions (Up/Down).
- Implemented a toggle for Logarithmic vs. Linear scaling for better visualization of small spikes.
- Updated the process table to show selection status using `Space` key.
- Refactored `App` state to use `GraphSeries` and `HashSet<u32>` for selected PIDs.
- Verified compilation and updated help overlay with new keybindings (`Space`, `l`).

2026-03-21 16:15 - Updated Roadmap and Proposed Task 023.
- Marked Task 022 (Tabbed Navigation) as complete in `ROADMAP.md`.
- Proposed Task 023: Graph UI Overhaul (Visual Analytics) in `docs/ideas/todo/023-graph-ui-overhaul.md`.
- Planned multi-process overlay, dynamic scaling, and legends for the graph view.

2026-03-21 16:00 - Completed Task 022: Tabbed Navigation (UI Management).
- Implemented a robust `Tabs` widget to manage Dashboard, Processes, and Alerts views.
- Updated `ViewMode` enum and `App` state to reflect the new navigation paradigm.
- Added keybindings: `Tab`/`Shift+Tab` for cycling, `F1`-`F3` for direct access.
- Implemented mouse support for clicking on tab headers.
- Refactored Alerts from an overlay to a full-page view mode.
- Updated layout and chunking logic in `ui.rs` to accommodate the tab bar.
- Verified userspace compilation.

2026-03-21 15:30 - Proposed Task 022: Tabbed Navigation (UI Management).
- Created `docs/ideas/todo/022-tabbed-navigation.md`.
- Planned implementation of a robust `Tabs` widget to manage multiple screens (Dashboard, Processes, Alerts).
- Planned navigation via `Tab`/`Shift+Tab` and `F1`-`F3` keys.
- Planned UI update to include a tab bar below the header.

2026-03-21 15:15 - Completed Task 021: Global Dashboard (Overall View).
- Implemented `ViewMode` enum and updated `App` state to support multiple view modes.
- Implemented `render_dashboard` in `ui.rs` providing a high-level system summary.
- Added real-time aggregation for protocol and country-based traffic statistics.
- Implemented `Top Talkers` and `Top Destinations` widgets in the dashboard.
- Added `Tab` keybinding to toggle between Process Table and Global Dashboard.
- Updated mouse handling to support view switching and differentiated footer actions.
- Verified compilation and layout consistency.

2026-03-21 14:30 - Proposed Task 021: Global Dashboard (Overall View).
- Created `docs/ideas/todo/021-global-dashboard.md`.
- Planned implementation of a high-level summary view (Dashboard) for system-wide network health.
- Planned features: Big-number throughput, Protocol distribution (TCP/UDP), Top processes, and Geo-IP summary.
- Planned navigation via `Tab` key to switch between Dashboard and Process Monitor.

2026-03-21 14:15 - Completed Task 020: Time-Travel/Historical Analysis.
- Implemented `get_aggregated_stats` in `DbManager` to fetch bandwidth data for specific time windows.
- Added indexes to `traffic_log` table for performant historical queries.
- Added `HistoricalRange` selection dialog (accessible via 'H') to select Last 1h, 4h, or 24h.
- Implemented `historical_view_mode` in `App` to toggle between live and historical process data.
- Updated TUI to reflect historical state in the header and process table.
- Added unit tests for historical data aggregation in `db.rs`.
- Verified compilation and test pass.

2026-03-21 13:25 - Updated ROADMAP.md and Proposed Task 020.
- Updated Phase 3 of the roadmap to mark Task 019 as completed.
- Added Task 020: Time-Travel/Historical Analysis to the roadmap.
- Added "Phase 3 Final Review" to the roadmap for project-wide code review.
- Proposed new feature: Time frame selection for historical analysis (Task 020).

2026-03-21 13:15 - Completed Task 019: Extended Mouse Support.
- Implemented clickable table headers for sorting by PID, Name, Up/Down rate, or Total.
- Added mouse click support for "(y)es" and "(n)o" buttons in the Kill confirmation dialog.
- Added mouse click support for footer keybinding hints (Quit, Kill, Sort, Filter, etc.).
- Refactored `netmonitor/src/ui.rs` to expose `get_footer_rect` and `get_column_widths` for userspace hit-testing.
- Verified compilation with `cargo check`.

2026-03-21 12:45 - Proposed Task 019: Extended Mouse Support.
- Created `docs/ideas/todo/019-extended-mouse-support.md`.
- Planned implementation of clickable table headers to toggle sorting by PID, Name, Up/Down rate, or Total.
- Planned adding click support for interactive dialog buttons (e.g., "y/n" in Kill dialog).
- Planned research into coordinate mapping for dynamic column widths.

2026-03-21 12:15 - Completed Task 018: Mouse Support (Click & Scroll).
- Implemented `Event::Mouse` handling in `netmonitor/src/main.rs`.
- Added `MouseEventKind::ScrollUp` and `ScrollDown` for process table and theme selector navigation.
- Implemented click-to-select for the main process table and theme selector dialog using coordinate hit-testing.
- Exposed `get_table_rect` and `centered_rect` in `netmonitor/src/ui.rs` for userspace layout calculations.
- Exposed `Tui::size()` in `netmonitor/src/tui.rs`.
- Verified changes with `cargo check`.

2026-03-21 11:00 - Proposed Task 018: Mouse Support (Click & Scroll).
- Created `docs/ideas/todo/018-mouse-support.md`.
- Planned implementation of mouse event capture and basic scrolling for the process table.
- Planned research into coordinate mapping for click-to-select.

2026-03-21 10:30 - Completed Task 017: Custom Themes & Theme Selector.
- Implemented `Theme` engine in `netmonitor/src/theme.rs` with presets: Default, Dracula, Solarized, Monokai, and High Contrast.
- Decoupled hardcoded colors from `netmonitor/src/ui.rs` by referencing `app.current_theme`.
- Added `current_theme` and `theme_state` to `App` struct in `netmonitor/src/app.rs`.
- Implemented interactive Theme Selector dialog (hotkey `t`) with Up/Down navigation and Enter to apply.
- Updated `ROADMAP.md` and `TECHNICAL_SPEC.md` to reflect theme engine integration.
- Verified changes with `cargo check`.

2026-03-20 14:10 - Proposed Task 017: Custom Themes & Theme Selector.
- Created `docs/ideas/todo/017-custom-themes.md`.
- Planned implementation of a theme engine with preset color schemes (Default, Dracula, Solarized).
- Planned UI enhancement for a theme selection dialog (hotkey `t`).
- Planned decoupling of hardcoded colors from the rendering pipeline.

2026-03-20 14:00 - Completed Task 016: Smart Bandwidth Alerts & Thresholds.
- Added `Alert` struct and `thresholds`, `alerts` to `App` state.
- Implemented PID-based bandwidth thresholding in the 1Hz update loop.
- Added interactive "Set Threshold" dialog (hotkey `a`) for per-process limits.
- Implemented "Alerts Log" overlay (hotkey `A`) to view recent threshold violations.
- Updated main process table to highlight rows exceeding their thresholds in Red.
- Integrated alert notifications into the header and status bar.
- Verified logic via `cargo check`.

2026-03-20 13:10 - Proposed Task 016: Smart Bandwidth Alerts & Thresholds.
- Created `docs/ideas/todo/016-smart-alerts.md`.
- Planned implementation of PID-based bandwidth thresholds (KB/s).
- Planned UI enhancements for alert indicators and a threshold configuration dialog (hotkey `a`).

2026-03-20 13:00 - Completed Task 015: Help Overlay & Keybinding Documentation.
- Added `show_help: bool` to the `App` state in `netmonitor/src/app.rs`.
- Implemented `render_help_overlay` in `netmonitor/src/ui.rs` using a centered Magenta-themed table.
- Updated `netmonitor/src/main.rs` to handle `?` and `h` for toggling the help screen.
- Ensured `Esc`, `q`, and `?` close the help overlay, and that other keys are suppressed while open.
- Updated footer to include `?: Help` documentation.
- Verified build and interaction logic.

2026-03-20 12:00 - Completed Task 014: Interactive Graphs & Historical View.
- Implemented `get_traffic_history` in `DbManager` with time-based bucketing (10s, 1m, 15m) for different time ranges (10m, 1h, 24h).
- Added `show_graph`, `graph_time_range`, and historical traffic buffers to the `App` state.
- Integrated keyboard handling for "Graph View" (toggle with `g`, cycle range with `Tab`, close with `Esc`).
- Implemented full-screen interactive bandwidth chart using `ratatui::widgets::Chart`.
- Added color-coded lines for "Upload" (Green) and "Download" (Yellow) with dynamic Y-axis scaling.
- Verified time-series data retrieval and rendering through `cargo check` and UI review.

2026-03-20 11:00 - Completed Task 013: Traffic Persistence & SQLite Integration.
- Integrated `rusqlite` for local database storage.
- Designed and implemented schema for `processes` (cumulative) and `traffic_log` (time-series) tables.
- Implemented `DbManager` in `netmonitor/src/db.rs` with batch flushing capabilities.
- Added startup hook to load historical statistics into the `App` state.
- Integrated a periodic 60-second flush and a final shutdown flush in the main loop.
- Added unit tests for database schema initialization, batch inserts, and historical data loading.
- Updated `TECHNICAL_SPEC.md` with the new data persistence architecture.
- Verified build and unit test integrity.

2026-03-20 10:00 - Proposed Task 013: Traffic Persistence & SQLite Integration.
- Created `docs/ideas/todo/013-traffic-persistence.md`.
- Objective: Persist bandwidth statistics and connection history to a local database for historical analysis and session reloads.

2026-03-20 09:30 - Completed Tasks 010 (Reverse DNS) and 012 (Protocol Heuristics).
- Implemented `ProtocolResolver` in `netmonitor/src/protocol.rs` for common port-to-service mapping.
- Implemented `DnsResolver` in `netmonitor/src/dns.rs` using `trust-dns-resolver` with a `DashMap` cache.
- Updated `ConnectionInfo` to store `hostname` and `service` fields.
- Integrated resolvers into the main userspace loop with asynchronous DNS lookups triggered on-demand.
- Enhanced the TUI "Active Connections" table with "SERVICE" and "REMOTE HOST/IP" columns.
- Verified `ProtocolResolver` with unit tests and confirmed build integrity.
- Moved Task 010 and 012 idea files to `docs/ideas/done/`.
- Updated `TECHNICAL_SPEC.md` to reflect the new metadata resolution steps.

2026-03-20 08:30 - Completed Task 011: Geo-IP & ASN Mapping.
- Integrated `maxminddb` for destination IP address to country/organization mapping.
- Implemented `GeoResolver` with a fallback mechanism and `include_bytes!` embedding.
- Updated `ConnectionInfo` to include `country` and `isp` fields.
- Enhanced the "Connection Table" in the TUI Detail View with new "GEO" and "ISP" columns.
- Added unit tests to verify Geo-IP fallback logic and build integrity.
- Updated `PRD.md` with database update instructions and status markers.

2026-03-20 07:00 - Completed Task 009: Socket Connection View.
- Implemented per-connection traffic tracking in eBPF via a new `CONNECTIONS` LruHashMap.
- Defined `ConnectionKey` and `ConnectionInfo` to store PID, Protocol, IP, and Port tuples.
- Updated eBPF hooks (`tcp_sendmsg`, `tcp_cleanup_rbuf`, `udp_sendmsg`) to extract connection details from `struct sock`.
- Enhanced userspace `App` to store and organize connection statistics by process ID.
- Upgraded the "Process Detail" view in the TUI to include a sub-table of active network connections.
- Successfully verified build for both eBPF and userspace components.
- Updated `TECHNICAL_SPEC.md` to reflect new connection tracking maps and structures.

2026-03-20 06:15 - Completed Task 008: Protocol Expansion (UDP & ICMP).
- Expanded eBPF monitoring to include UDP and RAW/ICMP traffic for broader visibility.
- Implemented `kprobe` for `udp_sendmsg` and `raw_sendmsg` to capture outbound data.
- Implemented `kretprobe` for `udp_recvmsg` and `raw_recvmsg` to capture inbound data.
- Refactored eBPF code with an `update_stats` helper for consistent data aggregation.
- Updated userspace loader in `main.rs` to attach all new probes at startup.
- Verified successful build of both eBPF bytecode and userspace application.
- Updated `TECHNICAL_SPEC.md` and `ROADMAP.md` to reflect new protocol coverage.

2026-03-20 05:45 - Completed Task 007: TUI Polish & The "Kill-Switch".
- Implemented `libc::kill` with `SIGKILL` for selected processes in the TUI.
- Added a confirmation dialog for the kill-switch and status messages for success/failure.
- Enhanced TUI header with real-time global traffic Sparklines for Upload and Download.
- Refined process table with right-aligned numeric columns for better readability.
- Implemented a "Process Detail" popup (Enter key) showing extended stats.
- Integrated `status_message` into the footer for user feedback.
- Verified build for userspace application (`cargo build -p netmonitor`).

2026-03-20 05:20 - Fixed compiler warnings in eBPF and userspace.
- Changed eBPF `TRAFFIC_STATS` from `static mut` to `static` to align with Aya standards and resolve Edition 2024 warnings.
- Cleaned up unused imports and variables in `main.rs` and `netmonitor-ebpf/src/main.rs`.
- All components now build without warnings.

2026-03-20 05:15 - Completed Task 006: Filtering & Cumulative Stats Tracking.
- Implemented process filtering by name using keyboard input ('/' or 'f' to activate).
- Added `filter_text` and `is_filtering` state to `App`.
- Integrated a search bar in the UI that appears when filtering is active.
- Refactored data aggregation to calculate per-tick rates (KB/s) and maintain cumulative totals.
- Added `process_history` to `App` to preserve stats for processes even when they become inactive.
- Updated UI table columns to reflect real-time rates (UP/DOWN) and absolute totals.
- Verified build with `cargo xtask build-ebpf` and `cargo build`.

2026-03-20 05:05 - Created Task 006: Filtering & Cumulative Stats Tracking.
- Proposed plan to implement process filtering by name using keyboard input in Ratatui.
- Outlined strategy to maintain cumulative traffic stats across application lifecycle, preventing resets on process idle/death.


2026-03-20 05:00 - Completed Task 005: TUI Scaffolding (Ratatui MVP).
- Implemented TUI using `ratatui` and `crossterm`.
- Created `App` struct to hold process statistics and UI state.
- Implemented layout with Header, Main (sortable table), and Footer.
- Integrated Crossterm for terminal raw mode and event handling.
- Refactored `main.rs` to render UI on interval and handle terminal events.
- Used `ProcessResolver` to populate names in the TUI table.
- Added kill confirmation dialog logic.
- Moved `005-tui-scaffolding.md` to `done/`.

2026-03-20 04:48 - Enhanced Task 005: TUI Scaffolding (Ratatui MVP).
- Integrated layout details, color palette, and event constraints based on TUI and Style Guide PRDs.
- Specified UI states, table columns, dynamic resizing, and double-buffering requirements.

2026-03-20 23:30 - Created Task 005: TUI Scaffolding (Ratatui MVP).
- Proposed a plan to replace basic `info!` logging with a functional `Ratatui` terminal interface.
- Outlined strategy for `App` state management and 1Hz UI refresh cycles.
- Defined a multi-area layout (Header, Process Table, Footer) as per technical specifications.

2026-03-20 23:10 - Completed Task 004: Process Resolver (/proc crawler).
- Implemented `ProcessResolver` in `netmonitor` to map PIDs to human-readable names.
- Manual parsing of `/proc/[pid]/comm` and `/proc/[pid]/cmdline` for lean binary footprint.
- Implemented `HashMap` cache with 10-second TTL to handle PID recycling.
- Updated userspace statistics loop to display `[process_name] (PID)` for better visibility.
- Verified `ProcessResolver` with a new unit test for self-resolution.

2026-03-20 22:20 - Completed Task 003: Capability Management & Embedded Bytecode.
- Embedded eBPF bytecode into the userspace binary using `include_bytes!`.
- Implemented `check_caps` function to verify `CAP_BPF` and `CAP_NET_ADMIN`.
- Updated `netmonitor-xtask` to remove the `EBPF_PATH` environment variable.
- Restored `aya-log` in kernel-space for better observability.
- Verified successful execution with `cargo xtask run`.

2026-03-20 21:50 - Completed Task 002: Data Pipeline & BPF Maps.
- Defined `TrafficStats` struct in `netmonitor-common` with `Pod` support and `serde` integration.
- Implemented `HashMap` in `netmonitor-ebpf` to track bytes and packets per PID.
- Updated `tcp_sendmsg` kprobe to extract PID and size, and update the BPF map.
- Implemented userspace polling in `netmonitor` using `tokio::select!` for real-time stats display.
- Verified PID-to-bandwidth mapping via `cargo xtask run`.

2026-03-19 16:15 - Completed Task 001: Build Engine & Toolchain Setup.
- Configured `rust-toolchain.toml` with nightly and `rust-src`.
- Scaffolded `netmonitor-xtask` crate for automated build/run management.
- Implemented `cargo xtask build-ebpf` and `cargo xtask run` commands.
- Unified `aya` version to 0.13 across the workspace.
- Successfully implemented a "Hello World" `kprobe/tcp_sendmsg` and verified kernel loading.
- Optimized eBPF build to always use release mode for verifier compatibility.

2026-03-19 14:30 - Initial project setup and environment initialization.
- Created `docs/` structure with `ROADMAP.md`, `TECHNICAL_SPEC.md` (Kernel/TUI), and `STYLE_GUIDE.md`.
- Drafted `GEMINI.md` for project context and engineering mandates.
- Configured Apache-2.0 `LICENSE` and `.gitignore`.
- Initialized Cargo workspace with `netmonitor`, `netmonitor-ebpf`, and `netmonitor-common`.
- Pushed initial project setup to GitHub: `17f8acf`.