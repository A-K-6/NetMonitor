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
