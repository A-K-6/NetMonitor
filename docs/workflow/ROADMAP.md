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
- [ ] **Process Resolver:** Build a `/proc` crawler to map PIDs to human-readable application names.
- [ ] **TUI Scaffolding:** Initialize `Ratatui` with a basic layout (Header, Process Table, Footer).
- [ ] **Real-time Refresh:** Implement a 1Hz (or higher) refresh loop that pulls data from BPF maps.
- [ ] **Sorting & Filtering:** Add ability to sort by "Upload"/"Download" and filter by process name.
- [ ] **Cumulative Stats:** Track total data consumed since the application started.

## Phase 3: Advanced Features (Power-User Tools)
*Goal: Add intelligence and control to the monitoring experience.*
- [ ] **Protocol Identification:** Identify traffic by common ports (HTTPS, DNS, SSH, etc.).
- [ ] **Geo-IP Integration:** Embed a MaxMind Lite database to map destination IPs to countries and ASNs.
- [ ] **The "Kill-Switch":** Implement the `k` hotkey to send `SIGKILL` to a selected bandwidth-hogging process.
- [ ] **Connection Deep-Dive:** Add a "Detail View" to see individual socket connections (IP:Port) for a specific process.
- [ ] **Smart Alerts:** Implement a notification system for when a process exceeds a defined bandwidth threshold.

## Phase 4: Stability & Distribution
*Goal: Professional-grade packaging and performance optimization.*
- [ ] **Performance Audit:** Verify CPU usage remains <1% under heavy network load.
- [ ] **CO-RE Validation:** Test the binary on multiple kernel versions (BTF support).
- [ ] **Headless Mode:** Implement a mode for JSON/CSV output or Prometheus exporting.
- [ ] **Packaging:** 
    - [ ] Create `.deb` package via `cargo-deb`.
    - [ ] Create AUR (Arch User Repository) PKGBUILD.
- [ ] **Background Mode:** Implement `netmonitord` (Systemd service) for continuous logging.
- [ ] **Documentation:** Finalize user manuals and API/Kernel specifications.
