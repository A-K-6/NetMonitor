# NETMONITOR(1)
## NAME
netmonitor - A high-performance network monitoring tool for Linux using eBPF.

## SYNOPSIS
**netmonitor** [*OPTIONS*]

## DESCRIPTION
**NetMonitor** is a real-time network traffic analyzer that provides per-process bandwidth visibility. It leverages eBPF (Extended Berkeley Packet Filter) for deep packet inspection with minimal overhead and provides a modern Terminal User Interface (TUI).

## OPTIONS
- **-c, --config** *PATH*
    Path to the configuration file (default: `~/.config/netmonitor/config.toml` or `./config.toml`).
- **--headless**
    Run in headless mode without the TUI. Output is printed to stdout or a log file.
- **-d, --daemon**
    Run as a background daemon. Requires `CAP_NET_ADMIN` and `CAP_BPF` capabilities.
- **--pid-file** *PATH*
    Path to the PID file when running in daemon mode (default: `/run/netmonitor/netmonitor.pid`).
- **--db-path** *PATH*
    Path to the SQLite database for traffic persistence (default: `netmonitor.db` or `/var/lib/netmonitor/netmonitor.db`).
- **-o, --output** *FORMAT*
    Output format for headless mode. Supported formats: `plain`, `json`, `csv` (default: `plain`).
- **-i, --interval** *SECONDS*
    Interval between snapshots in seconds for headless and daemon modes (default: 1).
- **-n, --count** *NUMBER*
    Number of snapshots to take before exiting in headless mode.
- **--log-file** *PATH*
    Path to a file to log snapshots in headless mode.
- **--verify-accuracy**
    Run a short traffic accuracy verification in a temporary network namespace.

## TUI HOTKEYS
- **q / Esc**
    Quit the application or go back from a dialog.
- **k**
    Kill the selected process (requires user confirmation).
- **Space**
    Toggle multi-process selection for the graph view.
- **a**
    Set a bandwidth threshold alert for the selected process.
- **A**
    View the recent alerts log.
- **s**
    Cycle through sorting columns (PID, Name, Context, Up, Down, Total).
- **c**
    Toggle the "Name" column between binary name and service context (systemd/docker/k8s).
- **/**
    Search and filter processes by name or context.
- **Enter**
    Deep-dive into process connections (shows destination IPs, ports, and protocols).
- **g**
    Toggle the traffic history graph for the selected process(es).
- **l**
    Toggle between logarithmic and linear scales in the graph view.
- **t**
    Open the theme selector to change the TUI color scheme.
- **?**
    Toggle the help overlay.
- **Up / Down**
    Navigate the process table or menus.
- **Tab**
    Cycle through graph time ranges (10m, 1h, 24h) when the graph view is active.
- **F1 / F2 / F3**
    Switch between Dashboard, Process Table, and Alerts views.

## CONFIGURATION
NetMonitor uses a TOML configuration file. The structure includes sections for network monitoring, TUI settings, and alert thresholds.

## SECURITY
NetMonitor requires specific Linux capabilities to function:
- **CAP_NET_ADMIN**: To attach eBPF programs to network hooks.
- **CAP_BPF**: To load and manage eBPF programs and maps.
It is recommended to use `setcap` to grant these capabilities to the binary rather than running it as full root.

## FILES
- **/usr/bin/netmonitor**: The main binary.
- **/etc/netmonitor/config.toml**: System-wide configuration.
- **/var/lib/netmonitor/netmonitor.db**: Default database path for the daemon.

## BUGS
Report bugs at <https://github.com/aeen/netmonitor/issues>.

## AUTHOR
Aeen <your-email@example.com>
