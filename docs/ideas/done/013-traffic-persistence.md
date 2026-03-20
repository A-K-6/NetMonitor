# Task 013: Traffic Persistence & SQLite Integration

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Persist bandwidth statistics and connection history to a local database (SQLite) to allow for historical analysis and session reloads.

## 1. Research & Strategy
- [ ] **Storage Engine:** Use `rusqlite` for a lightweight, zero-configuration local database.
- [ ] **Schema Design:**
    - `processes`: `pid`, `name`, `first_seen`, `last_seen`, `total_up`, `total_down`.
    - `traffic_log`: `pid`, `timestamp`, `up_bytes`, `down_bytes` (for time-series data).
- [ ] **Flush Strategy:** Periodically flush in-memory stats to the database (e.g., every 60 seconds or on exit).

## 2. Implementation (Userspace)
- [ ] **Database Module:** Add `rusqlite` to `Cargo.toml` and create `netmonitor/src/db.rs`.
- [ ] **Startup Hook:** Load historical data on startup to populate initial process stats.
- [ ] **Background Worker:** Implement a task that flushes cumulative stats from the `App` state to the DB.
- [ ] **Data Migration:** Ensure the DB schema is initialized correctly on first run.

## 3. Verification
- [ ] **Functional Test:** Run the app, generate traffic, close it, and verify that "TOTAL" stats are preserved on the next start.
- [ ] **Performance Test:** Ensure that database writes don't block the TUI rendering loop.

## 4. Documentation
- [ ] Update `TECHNICAL_SPEC.md` with the new storage architecture.
- [ ] Log completion in `devlog.md`.
