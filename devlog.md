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