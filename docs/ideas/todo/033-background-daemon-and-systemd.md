# Idea: Background Daemon (netmonitord) & Systemd Integration

**Status:** Proposed
**Phase:** 4 (Stability & Testing)
**Objective:** Transform NetMonitor into a persistent system service that logs network activity continuously, even when the TUI is not active.

## 1. Context
Currently, NetMonitor only tracks and logs traffic when it is actively running in the foreground (TUI) or manually in headless mode. For comprehensive observability, users need:
1.  **Continuous Monitoring:** Capture spikes and usage patterns 24/7.
2.  **Systemd Integration:** Standard way to start/stop the monitor as a system service.
3.  **Privilege Management:** Run with minimal capabilities (`CAP_BPF`, `CAP_NET_ADMIN`) as a non-root user.
4.  **Auto-Logging:** Automatically save snapshots to `/var/log/netmonitor/` or a database.

## 2. Proposed Tasks

### Task A: Systemd Service Unit
- **`netmonitor.service`:** Create a standard systemd unit file.
- **Capabilities:** Use `AmbientCapabilities=CAP_NET_ADMIN CAP_BPF` to avoid running as full `root`.
- **User/Group:** Configure the service to run under a dedicated `netmonitor` user if possible, or `nobody`.

### Task B: Daemon Mode Flag
- **`--daemon`:** A flag that performs double-fork (or uses a crate like `daemonize`) to detach from the terminal.
- **PID File:** Support `--pid-file` for process tracking.

### Task C: Configurable Logging Path
- **Default Path:** Set default logging to `/var/log/netmonitor/traffic.json` or similar.
- **Permissions:** Ensure the daemon has write access to the logging directory.

### Task D: Database Auto-Flush
- Ensure the background service continuously flushes data to the SQLite database (`netmonitor.db`) for the TUI to read later (Historical View).

## 3. Implementation Plan

1.  **Refactor Main:** Extract the "Headless Loop" logic into a reusable component that both the CLI and the Daemon can use.
2.  **Add `--daemon` Flag:** Use the `daemonize` crate or manual fork to background the process.
3.  **Create Systemd Template:** Add `resources/netmonitor.service` to the repository.
4.  **Update Xtask:** Add a `cargo xtask install` command to help users set up the service, user, and permissions.

## 4. Verification Criteria
- [ ] Running `systemctl start netmonitor` successfully initiates background tracking.
- [ ] `journalctl -u netmonitor` shows startup logs and capability verification.
- [ ] The TUI's "Historical View" shows data captured while the TUI was closed.
- [ ] The service survives a system reboot.
- [ ] Memory usage remains stable over 24 hours of background operation.
