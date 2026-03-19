# NetMonitor Project Roadmap

This roadmap outlines the path from initial scaffolding to a production-ready Linux network monitoring utility.

## Phase 1: Foundation (Infrastructure & eBPF Core)
*Goal: Establish the communication bridge between the Linux Kernel and Rust.*
- [x] **Project Scaffolding:** Initialize Cargo workspace, documentation structure, and Git repository.
- [ ] **Toolchain Configuration:** Setup `rust-toolchain.toml` for nightly/stable selection and install `bpf-linker`.
- [ ] **eBPF Build System:** Implement an `xtask` or build script to automate eBPF compilation to `.o` files.
- [ ] **Kernel Scaffolding:** Create the initial eBPF program with `kprobes` for `tcp_sendmsg` and `udp_sendmsg`.
- [ ] **Data Pipeline:** Implement BPF Maps (Hash Maps) to store bandwidth stats per PID.
- [ ] **Capability Management:** Implement the logic to verify `CAP_BPF` and `CAP_NET_ADMIN` at startup.
- [ ] **Embedded Bytecode:** Configure `include_bytes!` to bundle the eBPF object into the Rust binary.

## Phase 2: MVP (Core Features & TUI)
*Goal: A functional Terminal UI displaying real-time process bandwidth.*
- [ ] **Process Resolver:** Build a `/proc` crawler to map PIDs to human-readable application names.
- [ ] **TUI Scaffolding:** Initialize `Ratatui` with a basic layout (Header, Process Table, Footer).
- [ ] **Real-time Refresh:** Implement a 1Hz (or higher) refresh loop that pulls data from BPF maps.
- [ ] **Sorting & Totals:** Add ability to sort the table by "Upload" or "Download" speed and show system-wide totals.
- [ ] **Cumulative Stats:** Track total data consumed since the application started.

## Phase 3: Advanced Features (Power-User Tools)
*Goal: Add intelligence and control to the monitoring experience.*
- [ ] **Geo-IP Integration:** Embed a MaxMind Lite database to map destination IPs to countries and ASNs.
- [ ] **The "Kill-Switch":** Implement the `k` hotkey to send `SIGKILL` to a selected bandwidth-hogging process.
- [ ] **Connection Deep-Dive:** Add a "Detail View" to see individual socket connections (IP:Port) for a specific process.
- [ ] **Smart Alerts:** Implement a notification system for when a process exceeds a defined bandwidth threshold.

## Phase 4: Stability & Distribution
*Goal: Professional-grade packaging and performance optimization.*
- [ ] **Performance Audit:** Verify CPU usage remains <1% under heavy network load (e.g., during a stress test).
- [ ] **CO-RE Validation:** Test the binary on multiple kernel versions (Ubuntu 20.04, 22.04, and Arch).
- [ ] **Packaging:** 
    - [ ] Create `.deb` package via `cargo-deb`.
    - [ ] Create AUR (Arch User Repository) PKGBUILD.
- [ ] **Background Mode:** Implement `netmonitord` (Systemd service) for continuous logging without the TUI.
- [ ] **Documentation:** Finalize user manuals and API/Kernel specifications.
