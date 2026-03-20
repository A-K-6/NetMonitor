# Task 003: Capability Management & Embedded Bytecode

**Status:** Proposed
**Phase:** 1 (Foundation)
**Objective:** Finalize Phase 1 by embedding the eBPF object into the binary and ensuring the application has the necessary Linux capabilities to run.

## 1. Embedded Bytecode (`netmonitor`)
- [ ] Use `include_bytes!` in `main.rs` to embed the eBPF object file.
- [ ] Update `Ebpf::load` to use the embedded bytes instead of reading from a file path.
- [ ] Remove the requirement for `EBPF_PATH` environment variable.

## 2. Capability Management (`netmonitor`)
- [ ] Implement a check for `CAP_BPF` and `CAP_NET_ADMIN`.
- [ ] Provide clear error messages if capabilities are missing.
- [ ] (Optional) Add a "Sudo-less" mode guide to the README (using `setcap`).

## 3. Refactor `xtask` (`netmonitor-xtask`)
- [ ] Ensure `cargo xtask run` still works by building the eBPF object before building the userspace binary.

## 4. Verification
- [ ] Run `cargo build`.
- [ ] Run the resulting binary (e.g., `sudo ./target/debug/netmonitor`) without `EBPF_PATH`.
- [ ] Verify it still collects stats correctly.
