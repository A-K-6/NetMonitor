# Task 001: Build Engine & Toolchain Setup

**Status:** Proposed
**Phase:** 1 (Foundation)
**Objective:** Establish a unified, automated build and execution pipeline for the Rust/eBPF workspace.

## 1. Toolchain Configuration (`rust-toolchain.toml`)
- [ ] Create `rust-toolchain.toml` in the project root.
- [ ] Set channel to `nightly`.
- [ ] Add components: `rust-src`, `llvm-tools-preview`.
- [ ] Add targets: `x86_64-unknown-linux-gnu`, `bpfel-unknown-none`.

## 2. Scaffold `netmonitor-xtask` Crate
- [ ] Initialize `netmonitor-xtask` binary crate.
- [ ] Add `netmonitor-xtask` to workspace members in root `Cargo.toml`.
- [ ] Add dependencies: `anyhow`, `clap`, `cargo_metadata`.

## 3. Implement eBPF Build Logic (`build-ebpf`)
- [ ] Implement `build_ebpf()` function in `xtask/src/main.rs`.
- [ ] Command: `cargo build --package netmonitor-ebpf --target bpfel-unknown-none -Z build-std=core --release`.
- [ ] Add logic to verify `bpf-linker` is installed.

## 4. Implement Runner Logic (`run`)
- [ ] Implement `run()` function to orchestrate the full build.
- [ ] Logic: `build_ebpf()` -> `cargo build --package netmonitor` -> `sudo cargo run --package netmonitor`.
- [ ] Pass the path of the compiled eBPF `.o` file to the userspace binary via environment variable or CLI argument.

## 5. Initial "Hello World" Probe
- [ ] Implement a minimal `kprobe/tcp_sendmsg` in `netmonitor-ebpf/src/main.rs`.
- [ ] Use `aya-log` to emit a "Hello from Kernel" message.
- [ ] Implement the loading logic in `netmonitor/src/main.rs`.

## Validation Criteria
- [ ] `cargo xtask build-ebpf` succeeds and creates a `.o` file.
- [ ] `cargo xtask run` builds and attempts to load the probe.
- [ ] Debug logs from the kernel appear in the userspace terminal.
