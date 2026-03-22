# Idea: Persistent Configuration System

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Decouple application settings from the binary by implementing a robust, user-editable configuration file using the TOML format. This ensures that user preferences (themes, thresholds, UI state) persist across sessions.

## 1. Research & Strategy
- **Format Selection:** **TOML** is the preferred format for Rust-based CLI tools (familiar to users via `Cargo.toml`).
- **Location Standards:** Adhere to the **XDG Base Directory Specification**. On Linux, the config should reside in `${XDG_CONFIG_HOME:-~/.config}/netmonitor/config.toml`.
- **Crate Selection:**
    - `serde` / `serde_derive`: For high-performance serialization/deserialization.
    - `toml`: For parsing the configuration file.
    - `directories`: To handle cross-platform path resolution safely.

## 2. Technical Specification
### Configuration Schema (Proposed)
```toml
[ui]
theme = "Auto"          # Default theme (Auto, Terminal, Dracula, etc.)
refresh_rate = 1000     # Refresh interval in milliseconds
show_graph = false      # Whether to open the graph by default
default_view = "Dashboard" # Initial view mode

[network]
dns_resolution = true   # Enable/disable reverse DNS
geo_ip_enabled = true   # Enable/disable Geo-IP lookups

[alerts]
# Global or per-process bandwidth thresholds
default_threshold = 1024 # Default threshold in KB/s
[alerts.processes]
"discord" = 5120
"steam" = 20480
```

## 3. Implementation Plan
### Phase A: Core Infrastructure
- [ ] Create `src/config.rs` to house the `Config` struct and its sub-structs.
- [ ] Implement `Default` trait for `Config` to provide a fallback if no file exists.
- [ ] Implement `Config::load()` and `Config::save()` methods.

### Phase B: Integration
- [ ] Update `App::new()` to load configuration at startup.
- [ ] Refactor `theme.rs` and `app.rs` to consume settings from the `Config` object instead of hardcoded constants.
- [ ] Ensure `App` state changes (like theme selection) can be persisted back to the file.

### Phase C: UX & CLI
- [ ] Add a `--config <PATH>` CLI argument via `clap` to allow overriding the default location.
- [ ] Add a status message in the UI when the config is successfully loaded or if parsing fails.

## 4. Verification & Testing
- [ ] **Validation:** Verify that modifying the `config.toml` manually reflects in the app on restart.
- [ ] **Resilience:** Ensure the app does not crash if the config file is empty or contains invalid keys (use `#[serde(default)]`).
- [ ] **Permissions:** Ensure the app handles cases where the config directory is read-only.
