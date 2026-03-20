# Task 009: Socket Connection View

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Provide a "deep-dive" into individual network connections (IP/Port) for a selected process in the TUI.

## 1. Research & Strategy
- [ ] **eBPF Connection Tracking:** Update the eBPF programs to capture the source/destination IP and port (the "tuple") from the `struct sock`.
- [ ] **BPF Map Strategy:**
    - [ ] Create a new `CONNECTIONS` map keyed by a `ConnectionKey` (PID + Protocol + SrcIP + SrcPort + DstIP + DstPort).
    - [ ] Store `ConnectionStats` (bytes_sent, bytes_recv) as the value.
    - [ ] Use `LruHashMap` to automatically evict old connections and prevent memory exhaustion.
- [ ] **TUI Integration:** 
    - [ ] Add a new "Connections" tab or secondary popup in the Process Detail view.
    - [ ] Display a list of active/recent connections with their rates and remote addresses.

## 2. Implementation (eBPF)
- [ ] **Common Logic:**
    - [ ] Define `ConnectionKey` and `ConnectionStats` in `netmonitor-common`.
    - [ ] Add `CONNECTIONS` map to `netmonitor-ebpf`.
- [ ] **Hook Updates:**
    - [ ] Update `tcp_sendmsg`, `tcp_cleanup_rbuf`, `udp_sendmsg`, etc., to extract the tuple from the `sock` pointer.
    - [ ] Use `bpf_probe_read_kernel` to safely extract `sk_daddr`, `sk_rcv_saddr`, `sk_dport`, etc.

## 3. Implementation (Userspace)
- [ ] **Data Model:** Update `App` to store connection data per process.
- [ ] **UI Rendering:**
    - [ ] Implement a sub-table in the Detail popup to list connections.
    - [ ] (Optional) Add logic to "freeze" the view to inspect specific connections without them disappearing.

## 4. Verification
- [ ] **Connection Test:** Open a few tabs in a browser and verify that specific destination IPs (e.g., Google, GitHub) appear under the browser's PID in NetMonitor.
- [ ] **Overhead Check:** Ensure that tracking thousands of connections simultaneously doesn't impact performance.

## 5. Documentation
- [ ] Update `TECHNICAL_SPEC.md` with the new map and tuple structure.
- [ ] Log completion in `devlog.md`.
