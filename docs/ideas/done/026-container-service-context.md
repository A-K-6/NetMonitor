# Idea: Container & Service Context Resolution

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Enhance PID metadata by resolving the underlying context (systemd service, Docker container, Kubernetes pod) using cgroup analysis. This provides system administrators with infrastructure-level visibility into which services or containers are consuming bandwidth.

## 1. Research & Strategy
- **Cgroup Parsing:** Most modern Linux distributions use **cgroup v2** (unified hierarchy). The file `/proc/[pid]/cgroup` contains the cgroup path.
- **Service Mapping:** 
    - **Systemd:** Paths like `/system.slice/nginx.service` map directly to service names.
    - **Docker:** Paths usually contain `/docker/[container_id]`. Container names can be fetched via `docker.sock` or `/var/lib/docker/containers` (advanced).
    - **Kubernetes:** Paths follow `/kubepods/.../pod[pod_id]/[container_id]`.
- **Implementation Strategy:**
    - Create a `ContextResolver` that caches cgroup-to-metadata mappings.
    - Integrate with the existing `ProcessResolver` to enrich `ProcessRow`.
    - Update the TUI to display the "Context" column (perhaps replacing/toggling with "Name" or as a detail).

## 2. Technical Specification
### Cgroup Parsing Logic (Proposed)
```rust
pub enum ProcessContext {
    Systemd(String),    // e.g., "nginx.service"
    Docker(String),     // e.g., "db-container" or ID
    Kubernetes(String), // e.g., "auth-pod"
    User(String),       // e.g., "user@1000.service"
    Unknown,
}
```

## 3. Implementation Plan
### Phase A: Core Infrastructure
- [ ] Implement cgroup v2 path parser in `src/process.rs`.
- [ ] Define `ProcessContext` enum and logic to map cgroup strings to types.

### Phase B: Integration
- [ ] Update `ProcessRow` struct to include `context: ProcessContext`.
- [ ] Update `ProcessResolver` to populate the context during the `/proc` crawl.

### Phase C: UI Enhancements
- [ ] Add a new column "Context" to the `ProcessTable`.
- [ ] Add context information to the "Details" (Enter) view.
- [ ] Implement a toggle (e.g., `c`) to switch the "Name" column between "Binary Name" and "Service Context".

## 4. Verification & Testing
- [ ] **Systemd:** Verify that `sshd` or `nginx` shows their respective `.service` names.
- [ ] **Docker:** Run a container (e.g., `docker run -d --name test nginx`) and verify its traffic is tagged with the container ID/name.
- [ ] **Performance:** Ensure parsing `/proc/[pid]/cgroup` for every new PID doesn't introduce significant latency.
