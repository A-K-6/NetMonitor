# Technical Specification: TUI & Application State

This document defines the userspace architecture, state management, and user interaction logic for the NetMonitor TUI.

## 1. TUI Framework
NetMonitor uses **Ratatui** (the successor to `tui-rs`) as its primary UI framework.
- **Backend:** `Crossterm` for terminal interaction.
- **Rendering:** Immediate-mode rendering (60 FPS or on-event/on-timer).

## 2. Application State Management
The state is managed in a central `App` struct that coordinates data between the eBPF maps and the UI.

### 2.1. `App` Struct
```rust
struct App {
    // Core Data
    process_data: Vec<ProcessRow>,
    total_upload: u64,
    total_download: u64,

    // UI State
    table_state: TableState,
    sort_column: Column,
    is_running: bool,
    
    // Feature State
    geo_ip_resolver: GeoIpResolver,
    dns_resolver: DnsResolver,
    protocol_resolver: ProtocolResolver,
}
```

### 2.2. Data Processing Loop
The userspace application runs a main loop with three primary responsibilities:
1.  **Poll BPF Maps:** Fetch latest bandwidth stats and active connections from the kernel.
2.  **Resolve PIDs:** Query `/proc/[pid]/comm` (or `/proc/[pid]/cmdline`) to get the application name.
3.  **Resolve Metadata:**
    - Perform asynchronous **Reverse DNS** lookups for destination IPs.
    - Apply **Protocol Heuristics** based on destination ports (e.g., 443 -> HTTPS).
    - Map IPs to geographical locations using the **Geo-IP** database.
4.  **Update UI:** Refresh the table and recalculate aggregate totals.

## 3. Terminal UI Layout
The UI is divided into three functional areas:

### 3.1. Header (System Summary)
- Displays global bandwidth totals (System-wide upload/download).
- Shows kernel version and active capabilities (`CAP_BPF`, etc.).

### 3.2. Main Content (Process Table)
A sortable table with columns for:
- **PID:** Process identifier.
- **NAME:** Human-readable application name.
- **UP (KB/s):** Real-time upload speed.
- **DOWN (KB/s):** Real-time download speed.
- **TOTAL:** Cumulative data transferred.

### 3.3. Footer (Hotkeys)
- `q`: Quit.
- `k`: Kill selected process (requires user confirmation).
- `s`: Change sorting column.
- `Enter`: Toggle "Connection Deep-Dive" detail view for selected process.

## 4. Input & Events
NetMonitor uses an event-driven model:
- **Timer Events:** Trigger UI refreshes and BPF map polling (default 1Hz).
- **Key Events:** Handle user interactions (sorting, navigation, killing).
- **Resize Events:** Recalculate UI layout for the new terminal dimensions.

## 5. Data Persistence & SQLite
To provide historical analysis and maintain statistics across application restarts, NetMonitor integrates a local **SQLite** database.

### 5.1. Database Schema
- **`processes` Table:** Stores cumulative statistics for each process identified by PID.
    - `pid`: INTEGER (Primary Key)
    - `name`: TEXT
    - `first_seen`: DATETIME
    - `last_seen`: DATETIME
    - `total_up`: INTEGER
    - `total_down`: INTEGER
- **`traffic_log` Table:** Stores time-series data for bandwidth usage.
    - `pid`: INTEGER (Foreign Key)
    - `timestamp`: DATETIME
    - `up_bytes`: INTEGER
    - `down_bytes`: INTEGER

### 5.2. Persistence Strategy
- **Startup:** Load cumulative `total_up` and `total_down` from the `processes` table to pre-populate the `App` state.
- **Periodic Flush:** Every 60 seconds, the userspace application flushes accumulated deltas to the database in a single transaction.
- **Shutdown:** A final flush is performed when the application receives a termination signal (`q` or `Ctrl+C`) to ensure no data loss.
- **Storage:** The database is stored in a local file (`netmonitor.db`) in the application's working directory.
