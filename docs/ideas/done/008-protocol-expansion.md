# Task 008: Protocol Expansion (UDP & ICMP)

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Expand the eBPF monitoring capabilities beyond TCP to include UDP and ICMP traffic for a more comprehensive network overview.

## 1. Research & Strategy
- [ ] **UDP Outbound:** Hook `udp_sendmsg` to track outgoing UDP packets (DNS, QUIC, etc.).
- [ ] **UDP Inbound:** Hook `udp_recvmsg` or a similar kernel function to track incoming UDP traffic.
- [ ] **ICMP/Raw:** Investigate `ip_local_out` or `raw_sendmsg` to capture ICMP (ping) traffic if possible.
- [ ] **Map Optimization:** Ensure the `TRAFFIC_STATS` map in eBPF correctly aggregates these different protocols under the same PID.

## 2. Implementation (eBPF)
- [ ] **UDP Hooks:**
    - [ ] Implement `kprobe/udp_sendmsg` to extract size and increment `bytes_sent`.
    - [ ] Implement `kprobe/udp_recvmsg` (or appropriate hook) to track `bytes_recv`.
- [ ] **ICMP Hooks (Optional/Experimental):**
    - [ ] Explore hooking `icmp_reply` or general IP-level hooks to attribute pings to PIDs.

## 3. Implementation (Userspace)
- [ ] **Loader Updates:** Update `main.rs` to load and attach the new BPF programs.
- [ ] **TUI Legend:** (Optional) Add a small legend or indicator in the detail view to show which protocols are contributing to the bandwidth.

## 4. Verification
- [ ] **UDP Test:** Run `dig @8.8.8.8 google.com` or `curl --http3` and verify the bandwidth is captured in NetMonitor.
- [ ] **ICMP Test:** Run `ping -c 4 1.1.1.1` and see if the activity is recorded.
- [ ] **Performance:** Ensure the additional hooks do not noticeably increase CPU overhead.

## 5. Documentation
- [ ] Update `TECHNICAL_SPEC.md` with the new hooks.
- [ ] Log completion in `devlog.md`.
