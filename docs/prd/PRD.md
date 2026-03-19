# PRD: NetMonitor (Advanced Linux Edition)

## 1. Vision & Strategy
NetMonitor aims to be the "Standard Issue" network monitoring tool for Linux, combining the raw power of **eBPF** with a modern **Rust**-driven interface. It is designed to be easily installable (`apt install`), highly secure, and feature-rich for both desktop and server environments.

---

## 2. Technical Excellence (Best Practices)
To ensure high quality and successful packaging as a Linux distribution (e.g., Debian/Ubuntu), we will adhere to the following:

- **Unified Binary:** Embed eBPF bytecode directly into the Rust binary using `include_bytes!`. No external `.o` files needed.
- **CO-RE (Compile Once – Run Everywhere):** Leverage BTF (BPF Type Format) so the same binary runs on various kernel versions (5.4+) without recompilation.
- **Security-First (Least Privilege):** 
    - Use **Systemd Ambient Capabilities** (`CAP_BPF`, `CAP_NET_ADMIN`) to run the monitor as a non-root user.
    - Implement a clean separation between the kernel collector and the user-facing UI.
- **Packaging:** Native support for `cargo-deb` to generate standard `.deb` packages for Ubuntu/Debian.

---

## 3. Core Features (MVP)
- [ ] **Real-time Per-Process Bandwidth:** Hook `tcp_sendmsg`, `udp_sendmsg`, and receive equivalents to track KB/s per PID.
- [ ] **App Identification:** Automatically map PIDs to human-readable names using `/proc`.
- [ ] **Modern TUI (Ratatui):** A high-frequency dashboard with sortable columns and system-wide totals.
- [ ] **Cumulative Tracking:** Monitor total data consumed per process since the app started.

---

## 4. Advanced "Power-User" Features
- [ ] **🌍 Geo-IP & ASN:** Map destination IPs to countries and providers (e.g., "Slack -> AWS Ireland"). Use an embedded MaxMind Lite database.
- [ ] **🛑 Integrated "Kill-Switch":** Terminate a bandwidth-hogging process directly from the UI with a single hotkey (`k`).
- [ ] **📉 Traffic Shaping (Post-MVP):** Use `tc` (Traffic Control) or cgroups to throttle a specific process (e.g., "Limit Chrome to 5Mbps").
- [ ] **🔔 Smart Alerts:** Desktop/System notifications when a process exceeds a configurable threshold (e.g., "Unknown app `backup.sh` is uploading at 50MB/s").
- [ ] **🕵️ Connection Deep-Dive:** Toggle detail view to see all active sockets/IPs for a selected process.

---

## 5. Deployment & Distribution
- **Package Managers:** `.deb` (Debian/Ubuntu), `.rpm` (Fedora), and `Arch` (AUR).
- **Update Frequency:** Support for a background service mode (`netmonitord`) that logs stats even when the TUI isn't open.
- **Telemetry-Free:** No data leaves the local machine; all processing is local.

---

## 6. Success Metrics
- **Performance:** <1% CPU usage during peak traffic.
- **Accuracy:** Match kernel stats within a <2% margin of error.
- **Zero-Dependency:** Binary runs on any compatible kernel with no external library requirements.
