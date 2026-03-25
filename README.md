# 🛰️ NetMonitor (Advanced Linux Edition)

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![eBPF](https://img.shields.io/badge/technology-eBPF-blue.svg)](https://ebpf.io/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green.svg)](LICENSE)
[![Release](https://img.shields.io/badge/release-v0.1.0--beta.2-blue.svg)](https://github.com/A-K-6/NetMonitor/releases)

**NetMonitor** is a high-performance, real-time network monitoring tool for Linux that leverages the power of **eBPF** for deep packet insights and **Rust** for a modern, secure Terminal User Interface (TUI).

```text
    _   __     __  __  ___             _ __            
   / | / /__  / /_/  |/  /___  ____  (_) /_____  _____
  /  |/ / _ \/ __/ /|_/ / __ \/ __ \/ / __/ __ \/ ___/
 / /|  /  __/ /_/ /  / / /_/ / / / / / /_/ /_/ / /    
/_/ |_/\___/\__/_/  /_/\____/_/ /_/_/\__/\____/_/     
                                                      
   > Real-time Bandwidth. Deep Insights. Zero Overhead.
```

---

## ✨ Key Features

-   🚀 **Real-time Per-Process Tracking:** Monitor upload and download speeds per PID with sub-second precision.
-   🔒 **eBPF Powered:** Deep kernel-level inspection of TCP, UDP, and RAW sockets with minimal CPU overhead (<1%).
-   🛑 **Integrated Kill-Switch:** Terminate bandwidth-hogging processes directly from the UI (`k`).
-   📉 **Active Traffic Shaping:** Throttle specific processes to a defined KB/s limit (`b`) using `cgroup_skb`.
-   🌍 **Geo-IP & ASN Mapping:** Instantly identify where your traffic is going (e.g., "Slack -> AWS Ireland").
-   🕵️ **Connection Deep-Dive:** Toggle detail view to see all active sockets, remote hostnames, and protocols for any process.
-   📦 **Container & Service Context:** Automatically resolve Systemd services, Docker containers, and Kubernetes pods.
-   📊 **Historical Analysis:** Persistent storage (SQLite) allows for time-traveling through past network activity.
-   🔔 **Smart Alerts:** Configure thresholds and get notified when a process exceeds its bandwidth budget.
-   🎨 **Custom Themes:** Support for Dracula, Solarized, Monokai, and terminal-native themes.

---

## 🚀 Quick Start (Beta Release)

NetMonitor is currently in **Beta (v0.1.0-beta.2)**. 

### Recommended: Install via .deb (Debian/Ubuntu)

Download the latest `.deb` package from the [Releases](https://github.com/A-K-6/NetMonitor/releases) page and install it:

```bash
sudo apt install ./netmonitor_0.1.0-beta.2_amd64.deb
sudo systemctl start netmonitor
```

### Build & Install from Source

Requires `bpf-linker`, `rustup` (nightly), and `libcap-dev`.

```bash
# Clone and build
git clone https://github.com/A-K-6/NetMonitor.git
cd NetMonitor

# Use xtask to build and install (automatically handles capabilities and systemd)
cargo xtask install

# Start the service or run manually
sudo systemctl start netmonitor
netmonitor
```

---

## 🎮 Usage & Hotkeys

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit / Close Dialog |
| `Tab` | Switch Tabs (Dashboard / Table / Alerts) |
| `F1` - `F3` | Direct Tab Access |
| `Up` / `Down`| Navigate Process Table |
| `Enter` | Toggle Process Detail View |
| `s` | Cycle Sorting (PID, Name, Up, Down, Total) |
| `k` | **Kill** selected process |
| `b` | **Throttle** (Limit) selected process |
| `g` | Show **Interactive Graph** for selected process |
| `/` | **Filter** processes by name or context |
| `c` | Toggle **Context View** (Systemd/Docker/K8s) |
| `H` | Open **Historical Data** explorer |
| `a` | Set bandwidth **Alert** threshold |
| `t` | Change **Theme** |
| `?` | Show Help Overlay |

---

## 🛠️ Architecture

NetMonitor is split into three core components:

1.  **`netmonitor-ebpf`**: The kernel-space logic. Uses `kprobes` on `tcp_sendmsg`, `udp_sendmsg`, and `raw_sendmsg` to capture traffic. It also uses `cgroup_skb` for granular traffic shaping.
2.  **`netmonitor-common`**: Shared data structures between the kernel and userspace, including the Token Bucket Filter (TBF) logic.
3.  **`netmonitor`**: The userspace Rust application. It manages the eBPF lifecycle using [Aya](https://aya-rs.dev/), resolves process metadata via `/proc`, and renders the UI using [Ratatui](https://ratatui.rs/).

---

## ⚙️ Configuration

NetMonitor automatically creates a configuration file at `~/.config/netmonitor/config.toml`.

```toml
[ui]
theme = "Auto" # Auto, Default, Dracula, Solarized, Monokai, Terminal
refresh_rate = 1000 # refresh interval in ms
show_graph = false
default_view = "Dashboard"

[network]
dns_resolution = true
geo_ip_enabled = true

[alerts]
default_threshold = 1024 # KB/s
[alerts.processes]
"curl" = 500
```

---

## 🤝 Contributing

We are in the **Stability & Testing (Phase 4)** of our roadmap. Check out the [ROADMAP.md](docs/workflow/ROADMAP.md) for upcoming tasks. 

1.  Fork the repo
2.  Create your feature branch (`git checkout -b feature/amazing-feature`)
3.  Commit your changes (`git commit -m 'Add amazing feature'`)
4.  Push to the branch (`git push origin feature/amazing-feature`)
5.  Open a Pull Request

---

## 📜 License

Distributed under the MIT and Apache 2.0 Licenses. See `LICENSE` for more information.
