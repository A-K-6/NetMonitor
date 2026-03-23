# NetMonitor Project Roadmap

This roadmap outlines the path from initial scaffolding to a production-ready Linux network monitoring utility.

## Phase 1: Foundation (Infrastructure & eBPF Core)
*Goal: Establish the communication bridge between the Linux Kernel and Rust.*
- [x] **Project Scaffolding:** Initialize Cargo workspace, documentation structure, and Git repository.
- [x] **Toolchain Configuration:** Setup `rust-toolchain.toml` for nightly/stable selection and install `bpf-linker`.
- [x] **eBPF Build System (xtask):** Implement an `xtask` to automate eBPF compilation and userspace execution (Standard `aya` workflow).
- [x] **Kernel Scaffolding:** Create the initial eBPF program with `kprobes` for `tcp_sendmsg` and `udp_sendmsg`.
- [x] **Data Pipeline:** Implement BPF Maps (Hash Maps) to store bandwidth stats per PID.
- [x] **Observability:** Integrate `aya-log` for kernel-space logging and debugging.
- [x] **Capability Management:** Implement logic to verify `CAP_BPF` and `CAP_NET_ADMIN` at startup.
- [x] **Embedded Bytecode:** Configure `include_bytes!` to bundle the eBPF object into the Rust binary.

## Phase 2: MVP (Core Features & TUI)
*Goal: A functional Terminal UI displaying real-time process bandwidth.*
- [x] **Process Resolver:** Build a `/proc` crawler to map PIDs to human-readable application names.
- [x] **TUI Scaffolding:** Initialize `Ratatui` with a basic layout (Header, Process Table, Footer).
- [x] **Real-time Refresh:** Implement a 1Hz (or higher) refresh loop that pulls data from BPF maps.
- [x] **Sorting & Filtering:** Add ability to sort by "Upload"/"Download" and filter by process name.
- [x] **Cumulative Stats:** Track total data consumed since the application started.
- [x] **TUI Polish:** Make the UI more beautiful with advanced Ratatui widgets (Sparklines, Alignment, Dialogs).

## Phase 3: Advanced Power-User Features (Completed)
*Goal: Add intelligence, control, and deep-packet insights.*
- [x] **The "Kill-Switch":** Implement the `k` hotkey to send `SIGKILL` to a selected bandwidth-hogging process.
- [x] **Protocol Expansion:** Implement `udp_sendmsg` and `icmp` tracking to cover non-TCP traffic.
- [x] **Connection Deep-Dive:** Add a "Socket View" to see individual IP:Port connections for a specific PID.
- [x] **Reverse DNS Resolution:** Resolve destination IPs to hostnames (e.g., `172.217.16.14` -> `google.com`).
- [x] **Geo-IP & ASN Mapping:** Map IPs to countries and organizations (e.g., "Dublin, IE - Amazon.com").
- [x] **Protocol Heuristics:** Identify common traffic types (HTTPS, DNS, SSH, Bittorrent) based on port and pattern analysis.
- [x] **Traffic Persistence:** Save and reload session history to track bandwidth across app restarts.
- [x] **Interactive Graphs:** Full-screen historical graph mode for specific processes.
- [x] **Smart Alerts:** System notifications for when a process exceeds a defined bandwidth threshold.
- [x] **Help Overlay:** A dedicated `?` screen explaining all keybindings and features.
- [x] **TUI Refinement:**
    - [x] **Custom Themes:** Support for selectable color schemes (e.g., Dracula, Solarized, Monokai).
    - [x] **Mouse Support:** Basic click-to-select and scroll support for the process table.
    - [x] **Extended Mouse Support:** Clickable table headers for sorting and interactive dialog elements.
    - [x] **Time-Travel/Historical Analysis:** Ability to select specific time frames from the database to analyze past network activity.
    - [x] **Global Dashboard (Overall View):** Comprehensive system-wide analysis with aggregated protocol stats, top-talkers, and network health metrics.
    - [x] **Tabbed Navigation:** Implement a tabbed interface to seamlessly switch between "Global Dashboard", "Process Monitor", and "Alerts".
    - [x] **Graph UI Overhaul:** Enhanced visualization with better axes, legends, and multi-process overlay support with logarithmic scaling.
    - [x] **Adaptive Theming:** Support for terminal default colors (transparency/ANSI) and system-wide theme detection.
- [x] **Persistent Configuration:** Implement `~/.config/netmonitor/config.toml` for saving user preferences and thresholds.
- [x] **Container & Service Context:** Resolve and display systemd service, Docker container, and K8s pod names for PIDs.
- [x] **Active Traffic Shaping:** "Throttling" process bandwidth directly from the TUI using `cgroup_skb` eBPF.
- [x] **Writing the ReadMe:** Comprehensive documentation of new features, configuration, and advanced usage.
- [x] **Phase 3 Final Review:** A comprehensive, project-wide code review and refactor session to ensure stability before Phase 4.

## Phase 4: Stability & Testing (Current)
*Goal: Professional-grade reliability and automated verification.*
- [x] **Automated Testing Suite:** 
    - [x] Implement unit tests for userspace logic (Resolver, Config, TUI state).
    - [x] Implement eBPF integration tests (using `aya-test` or similar).
- [x] **Headless Data Exporting:** Implement `--json` and `--csv` output modes for scripting/automation.
- [x] **Performance Audit:** Verify CPU usage remains <1% under heavy network load.
- [x] **CO-RE Validation:** Test the binary on multiple kernel versions (BTF support).
- [x] **Headless Mode:** Implement a mode for JSON/CSV output or Prometheus exporting.
- [x] **Background Mode:** Implement `netmonitord` (Systemd service) for continuous logging.
- [ ] **CI/CD Pipeline:** Setup GitHub Actions for automated linting (`clippy`), formatting (`fmt`), and cross-platform testing.
- [ ] **Automated Packaging:** Integrate `cargo-deb` to generate `.deb` artifacts in CI.
- [ ] **Documentation Expansion:**
    - [ ] Create initial `man` pages for terminal-based help.
    - [ ] Generate API/Kernel technical specifications.

## Phase 5: Release & Distribution (v0.1.0)
*Goal: Getting NetMonitor into the hands of users with professional packaging.*
- [ ] **Community Feedback Loop:** Establish a Beta/Release Candidate (RC) phase for real-world user feedback.
- [ ] **Packaging:** 
    - [ ] Create `.deb` package via `cargo-deb`.
    - [ ] Create AUR (Arch User Repository) PKGBUILD.
    - [ ] Provide static binaries for Linux via GitHub Actions.
- [ ] **Initial Release (v0.1.0):** Tag the first stable release and publish it to GitHub and `crates.io`.
- [ ] **User Manual:** Finalize a compelling `README.md` with high-quality GIFs and usage examples.

## Phase 6: Hardening & Ecosystem
*Goal: Long-term sustainability, security, and contribution growth.*
- [ ] **Security Audit:** Detailed review of eBPF capability usage and memory safety in userspace.
- [ ] **Graceful Degradation:** Implement fallback logic for legacy kernels or missing system dependencies (e.g., GeoIP DB).
- [ ] **Least Privilege Hardening:** Ensure the application runs with the absolute minimum required Linux capabilities.
- [ ] **Advanced Traffic Control:** Expand "Traffic Shaping" with more granular rules and `cgroup` v1 support if required.
- [ ] **Open Source Readiness:** 
    - [ ] Draft `CONTRIBUTING.md` to guide future community contributors.
    - [ ] Establish a public issue-tracking and feature-request process.
- [ ] **Maintenance Strategy:** Define the release cycle and LTS (Long Term Support) goals for future versions.