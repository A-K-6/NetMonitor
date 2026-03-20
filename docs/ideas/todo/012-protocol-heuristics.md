# Task 012: Protocol Heuristics

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Identify common traffic types (HTTPS, DNS, SSH, etc.) based on destination ports and pattern analysis.

## 1. Research & Strategy
- [ ] **Port Mapping:** Create a lookup table for well-known ports (e.g., `443` -> `HTTPS`, `53` -> `DNS`).
- [ ] **Data Model:** Update `ConnectionInfo` to include a `service_name` field.
- [ ] **Pattern Analysis (Bonus):** Research if simple BPF helpers can peek at the first few bytes of a packet for deeper identification (e.g., `G-E-T` for HTTP).

## 2. Implementation (Userspace)
- [ ] **Service Resolver:** Build a lookup helper for common ports.
- [ ] **Data Model:** Add `service_name` to `ConnectionInfo`.
- [ ] **UI Update:** Show the service name in the "PROTO" or a new "SERVICE" column in the Connection View.

## 3. Verification
- [ ] **Functional Test:** Verify that common services like HTTPS (443) and SSH (22) are correctly labeled.
- [ ] **Edge Case:** Check behavior for non-standard ports (e.g., SSH on port 2222).

## 4. Documentation
- [ ] Update `TECHNICAL_SPEC.md` if the eBPF peeking logic is implemented.
- [ ] Log completion in `devlog.md`.
