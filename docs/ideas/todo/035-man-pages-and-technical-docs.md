# Idea: Professional Man Pages & Technical Documentation

**Status:** Proposed
**Phase:** 4 (Stability & Testing)
**Objective:** Provide standard Linux `man` pages for the `netmonitor` utility and generate a comprehensive technical specification for the kernel/userspace interaction.

## 1. Context
As we approach the `v0.1.0` release, `NetMonitor` needs to follow standard Linux conventions:
1.  **Standard Help:** Users expect to run `man netmonitor` to find detailed information about flags, hotkeys, and configuration.
2.  **Kernel/eBPF Internals:** For developers and auditors, we need to document the specific eBPF hooks, maps, and memory structures used.
3.  **Deployment Guide:** A formal specification of the systemd unit and its requirements (capabilities, paths).

## 2. Proposed Tasks

### Task A: Man Page Generation (roff/gzip)
- **Content:** Detail all CLI flags (`--daemon`, `--headless`, etc.), TUI hotkeys (Kill, Filter, Sort, Graph), and the `config.toml` structure.
- **Tools:** Use `pandoc` or a dedicated Rust crate like `roff` to generate the `netmonitor.1` file from Markdown.
- **Integration:** Update `cargo-deb` assets to include the man page in `/usr/share/man/man1/netmonitor.1.gz`.

### Task B: API & Kernel Technical Specs
- **Content:** Document the eBPF map schemas (TRAFFIC_STATS, CONNECTIONS, THROTTLE_CONFIG) and their update frequencies.
- **Kernel Hooks:** Detail the `kprobes`, `kretprobes`, and `cgroup_skb` hooks used.
- **Output:** Generate a clean Markdown or HTML technical guide in `docs/kernel/INTERNAL_SPEC.md`.

### Task C: Interactive TUI Help Expansion
- **Content:** Ensure the `?` help overlay is synchronized with the new documentation.
- **Feature:** Add a "See man page for details" hint to the footer or help screen.

## 3. Implementation Plan
1.  **Draft Man Page:** Create a comprehensive Markdown source for the man page.
2.  **Convert to Roff:** Use `pandoc` (if available) or a Rust-based tool to generate the `.1` file.
3.  **Update Packaging:** Add the generated man page to `netmonitor/Cargo.toml`'s assets for `cargo-deb`.
4.  **Export Technical Specs:** Consolidate existing technical notes into a formal `INTERNAL_SPEC.md`.

## 4. Verification Criteria
- [ ] `man ./target/man/netmonitor.1` displays the correctly formatted manual.
- [ ] The `.deb` package installs the man page in the standard location.
- [ ] The technical specification accurately reflects the final eBPF and userspace architecture.
