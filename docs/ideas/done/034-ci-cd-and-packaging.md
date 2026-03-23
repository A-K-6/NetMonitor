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
- **Caching:** Use `swatinem/rust-cache` to speed up builds across workflow runs.
- **Steps:**
    - `cargo fmt --check`
    - `cargo clippy --all-targets -- -D warnings`
    - `cargo test --workspace`

### Task B: Build Workflow
- **`build.yml`:** Ensure the eBPF and userspace code builds for `x86_64-unknown-linux-gnu`.
- **Environment Setup:** Install `llvm`, `clang`, `libcap-dev`, `libelf-dev`, and `bpf-linker` in the GitHub Actions runner.
- **Xtask Integration:** Use `cargo xtask build-ebpf` to ensure the eBPF skeleton is generated correctly before the userspace build.

### Task C: Automated Packaging (cargo-deb)
- **`release.yml`:** A workflow triggered by git tags (e.g., `v*`).
- **`Cargo.toml` Metadata:** Configure `[package.metadata.deb]` with:
    - **Maintainer:** Project author.
    - **Depends:** `$auto, libcap2-bin` (ensures `setcap` is available for post-install).
    - **Assets:** Map the binary to `/usr/bin/netmonitor` and the service to `/lib/systemd/system/netmonitor.service`.
- **Artifacts:** Upload the `netmonitor` binary and `.deb` file as release assets with descriptive names.

### Task D: Debian Lifecycle Scripts
- **Location:** Create `netmonitor/debian/` for management scripts.
- **`postinst`:** 
    - Create `netmonitor` system user/group.
    - Provision `/var/lib/netmonitor` and `/var/log/netmonitor` with correct ownership.
    - Apply `setcap cap_net_admin,cap_bpf=ep /usr/bin/netmonitor`.
- **`prerm`:** Stop and disable the `netmonitor.service` before removal.
- **`postrm`:** Clean up systemd units and configurations.

## 3. Implementation Plan
1.  **Update `Cargo.toml`:** Add `[package.metadata.deb]` section and asset mappings.
2.  **Create Debian Scripts:** Implement `postinst`, `prerm`, and `postrm` in `netmonitor/debian/`.
3.  **Create GitHub Workflows:** Add `.github/workflows/ci.yml` and `.github/workflows/release.yml` with `rust-cache` support.
4.  **Test Packaging Locally:** Run `cargo deb` locally and inspect the `.deb` structure using `dpkg -c`.
5.  **Refine Xtask:** Ensure `cargo xtask build-ebpf` works seamlessly in a non-interactive CI environment.

## 4. Verification Criteria
- [x] PRs show green checkmarks for fmt, clippy, and tests.
- [x] Tagging a commit with `v0.x.x` triggers a build that produces a downloadable `.deb`.
- [x] The generated `.deb` package successfully installs on a clean Ubuntu/Debian system and starts the service.
- [x] The binary inside the `.deb` has the correct `CAP_NET_ADMIN` and `CAP_BPF` capabilities.
