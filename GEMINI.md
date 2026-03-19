# NetMonitor (Advanced Linux Edition) - Developer Context

## Overview
**NetMonitor** is a high-performance network monitoring tool for Linux that leverages **eBPF** for deep packet inspection and **Rust** for a modern, secure Terminal User Interface (TUI). It targets system administrators and power users who need real-time, per-process bandwidth visibility with zero external dependencies.
- **Phase:** MVP Development.
- **Core Tech:** Rust, eBPF (aya/libbpf), Ratatui (TUI), cargo-deb.
- **Key Feature:** Real-time per-process tracking, eBPF CO-RE, integrated "Kill-Switch", and Geo-IP mapping.

## Documentation Structure
- **Roadmap:** `docs/workflow/ROADMAP.md`
- **Kernel/eBPF Spec:** `docs/kernel/TECHNICAL_SPEC.md`
- **TUI/Frontend Spec:** `docs/tui/TECHNICAL_SPEC.md`
- **Design Guide:** `docs/design/STYLE_GUIDE.md`
- **Ideas (Todo):** `docs/ideas/todo/`
- **Ideas (Done):** `docs/ideas/done/`
- **Product Requirements:** `docs/prd/PRD.md`

---

## 🚀 Workflows

### 1. Idea Generation Workflow
Use this when exploring new monitoring hooks (e.g., new `kprobes`) or UI components.
1.  **Request:** User asks for a new monitoring capability or UI enhancement.
2.  **Research:** Analyze kernel headers/BTF for relevant eBPF hooks or Ratatui widgets for UI.
3.  **Refinement:** Propose a technical plan (e.g., which `kprobe` or `tracepoint` to use). Refine with user feedback.
4.  **Documentation:** Create a detailed plan in `docs/ideas/todo/XXX-featurename.md`.
5.  **Log:** Update `devlog.md`.

### 2. Implementation Workflow
Use this for executing changes or fixing issues.

#### Case A: Implementing an Idea
1.  **Review:** Read the idea file from `docs/ideas/todo/` carefully.
2.  **Clarify:** Ensure the eBPF hook strategy and UI layout are clear.
3.  **Execute:** Implement Rust/eBPF code. Verify using `sudo` or appropriate capabilities (`CAP_BPF`).
4.  **Finalize:** Move the idea file from `todo/` to `docs/ideas/done/`.
5.  **Document & Log:** Update `TECHNICAL_SPEC.md` and `devlog.md`.

#### Case B: Simple Task / Bug Fix
1.  **Analyze:** Use `bpftrace` or application logs to reproduce issues.
2.  **Execute:** Apply fixes to Rust or eBPF bytecode.
3.  **Document & Log:** Update related documentation and `devlog.md`.

---

## Engineering Mandates
- **Single Source of Truth:** `docs/prd/PRD.md`.
- **README Synchronization:** Any major change to the `PRD.md` or `ROADMAP.md` MUST be reflected in the root `README.md` to ensure external documentation is always accurate.
- **Security-First (Least Privilege):** 
    - Never run as full `root` if `CAP_BPF` and `CAP_NET_ADMIN` suffice.
    - Validate all BPF bytecode for safety before deployment.
- **Performance Excellence:** 
    - Keep CPU overhead <1% during peak traffic.
    - Use efficient BPF maps (e.g., `hash` or `lru_hash`) for kernel-to-user communication.
- **Unified Binary:** Maintain a zero-dependency build where eBPF bytecode is embedded via `include_bytes!`.
- **CO-RE (Compile Once – Run Everywhere):** All eBPF code MUST be CO-RE compliant using BTF.
- **DevLog Protocol:** 
    - **READ** `devlog.md` (limit to first 20 lines) before starting any task.
    - **UPDATE** `devlog.md` after every major workflow step or implementation session.
    - **ORDER:** Newest entries **MUST** be appended to the **TOP** of the file (Reverse Chronological).
    - **FORMAT:** Use `YYYY-MM-DD HH:MM - Brief Description of the Changes` as the header.
    - **ARCHIVE:** If `devlog.md` exceeds 500 lines, move content to `docs/archive/devlogs/devlog-YYYY-MM-DD.md`.
