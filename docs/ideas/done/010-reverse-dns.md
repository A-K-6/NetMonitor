# Task 010: Reverse DNS Resolution

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Resolve destination IP addresses to human-readable hostnames (e.g., `142.250.190.46` -> `google.com`) in the Connection View.

## 1. Research & Strategy
- [ ] **Async Resolver:** Use a non-blocking DNS resolver (like `trust-dns-resolver` or `tokio::net::lookup_addr`) to prevent UI freezes.
- [ ] **Caching Layer:** Implement a TTL-respecting cache to avoid redundant DNS queries and reduce latency.
- [ ] **Batch Processing:** Resolve IPs in the background as they appear in the `CONNECTIONS` map.

## 2. Implementation (Userspace)
- [ ] **Resolver Integration:** Add `trust-dns-resolver` (or similar) to `Cargo.toml`.
- [ ] **Background Worker:** Create a task that monitors new IPs and triggers reverse DNS lookups.
- [ ] **Data Model:** Update `ConnectionInfo` to include an optional `hostname` field.
- [ ] **UI Update:** Display the hostname instead of (or alongside) the IP in the "Active Connections" table when available.

## 3. Verification
- [ ] **Functional Test:** Run `curl https://github.com` and verify that `github.com` appears in the Connection View for the process.
- [ ] **Performance Test:** Ensure that hundreds of concurrent lookups don't cause UI lag or excessive memory usage.

## 4. Documentation
- [ ] Update `TECHNICAL_SPEC.md` if the data flow changes significantly.
- [ ] Log completion in `devlog.md`.
