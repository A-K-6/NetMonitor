# Idea: CI/CD Pipeline & Automated Packaging

**Status:** Proposed
**Phase:** 4 (Stability & Testing) / 5 (Release & Distribution)
**Objective:** Establish a robust GitHub Actions workflow to automate linting, building, and generating release artifacts (like `.deb` packages).

## 1. Context
As the project nears Phase 5 (Release), we need to ensure that:
1.  **Code Quality:** Every Pull Request is automatically checked for formatting (`fmt`) and common mistakes (`clippy`).
2.  **Build Verification:** Userspace and eBPF code must compile successfully on clean environments.
3.  **Automated Releases:** When we tag a version (e.g., `v0.1.0`), GitHub should automatically build and attach the binary and `.deb` package to the release.
4.  **Cross-Kernel Check:** If possible, verify the eBPF skeleton against different BTF (Compile Once – Run Everywhere) environments.

## 2. Proposed Tasks

### Task A: Linting & Testing Workflow
- **`ci.yml`:** Create a workflow that runs on every push/PR.
- **Steps:**
    - `cargo fmt --check`
    - `cargo clippy --all-targets -- -D warnings`
    - `cargo test --workspace`

### Task B: Build Workflow
- **`build.yml`:** Ensure the eBPF and userspace code builds for `x86_64-unknown-linux-gnu`.
- **Dependencies:** Install `llvm`, `clang`, and `bpf-linker` in the CI environment.

### Task C: Automated Packaging (cargo-deb)
- **`release.yml`:** A workflow triggered by git tags.
- **`cargo-deb` Integration:** Configure `Cargo.toml` with metadata for Debian packaging (maintainer, description, systemd service inclusion).
- **Artifacts:** Upload the `netmonitor` binary and `.deb` file as release assets.

### Task D: Systemd Service Inclusion
- Ensure the `cargo-deb` configuration includes:
    - The `netmonitor` binary in `/usr/bin/`.
    - The `netmonitor.service` in `/lib/systemd/system/`.
    - Post-install scripts to create the `netmonitor` user and set file capabilities (`setcap`).

## 3. Implementation Plan
1.  **Update `Cargo.toml`:** Add `[package.metadata.deb]` section with dependencies (`libc6`, `libssl3`) and asset mappings.
2.  **Create GitHub Workflows:** Add `.github/workflows/ci.yml` and `.github/workflows/release.yml`.
3.  **Test Packaging Locally:** Run `cargo deb` locally to verify the generated package structure.
4.  **Refine Xtask:** Ensure `cargo xtask build-ebpf` works seamlessly in a non-interactive CI environment.

## 4. Verification Criteria
- [ ] PRs show green checkmarks for fmt, clippy, and tests.
- [ ] Tagging a commit with `v0.x.x` triggers a build that produces a downloadable `.deb`.
- [ ] The generated `.deb` package successfully installs on a clean Ubuntu/Debian system and starts the service.
- [ ] The binary inside the `.deb` has the correct `CAP_NET_ADMIN` and `CAP_BPF` capabilities.
