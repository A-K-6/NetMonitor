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

### Task D: Database Auto-Flush & Concurrency
- Ensure the background service continuously flushes data to the SQLite database (`netmonitor.db`) for the TUI to read later (Historical View).
- **SQLite WAL Mode:** Enable Write-Ahead Logging (`PRAGMA journal_mode=WAL`) to allow the TUI to perform historical queries without blocking the daemon's write operations.

### Task E: Signal Handling & Lifecycle
- **Graceful Shutdown (SIGTERM):** Implement a handler to ensure the final traffic batch is flushed to the database before the service exits.
- **Config Reload (SIGHUP):** Support reloading the configuration file (e.g., to update alert thresholds) without restarting the entire service.

## 3. Implementation Plan

1.  **Refactor Main:** Extract the "Headless Loop" logic into a reusable component that both the CLI and the Daemon can use.
2.  **Path Resolution Logic:** Implement a robust path resolver that defaults to `/var/lib/netmonitor/netmonitor.db` when running as a service, but allows overrides via config or CLI.
3.  **Add `--daemon` Flag:** Use the `daemonize` crate or manual fork to background the process.
4.  **Enable WAL Mode:** Update `DbManager` to initialize SQLite with WAL mode enabled.
5.  **Implement Signal Listeners:** Use `tokio::signal::unix` to handle `SIGTERM` and `SIGHUP`.
6.  **Create Systemd Template:** Add `resources/netmonitor.service` to the repository, utilizing `StandardOutput=journal` for operational logs and `AmbientCapabilities` for security.
7.  **Update Xtask:** Add a `cargo xtask install` command to:
    - Create the `netmonitor` system user/group.
    - Provision `/var/lib/netmonitor/` and `/var/log/netmonitor/` with correct permissions.
    - Install the binary to `/usr/local/bin/`.
    - Deploy and enable the systemd service.

## 4. Verification Criteria
- [ ] Running `systemctl start netmonitor` successfully initiates background tracking.
- [ ] `journalctl -u netmonitor` shows startup logs, capability verification, and no database lock errors.
- [ ] The TUI's "Historical View" shows data captured while the TUI was closed, even if the daemon is still writing.
- [ ] Sending `SIGHUP` to the daemon reloads the configuration successfully.
- [ ] The service survives a system reboot.
- [ ] Memory usage remains stable over 24 hours of background operation.
