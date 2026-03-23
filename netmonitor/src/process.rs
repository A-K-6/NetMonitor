use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProcessContext {
    Systemd(String),    // e.g., "nginx.service"
    Docker(String),     // e.g., "db-container" or ID
    Kubernetes(String), // e.g., "auth-pod"
    User(String),       // e.g., "user@1000.service"
    Unknown,
}

impl ProcessContext {
    pub fn label(&self) -> String {
        match self {
            ProcessContext::Systemd(s) => format!("svc:{}", s),
            ProcessContext::Docker(s) => format!("docker:{}", &s[..std::cmp::min(s.len(), 12)]),
            ProcessContext::Kubernetes(s) => format!("k8s:{}", s),
            ProcessContext::User(s) => format!("user:{}", s),
            ProcessContext::Unknown => "-".to_string(),
        }
    }
}

pub struct ProcessInfo {
    pub name: String,
    pub context: ProcessContext,
}

pub struct ProcessResolver {
    cache: HashMap<u32, (ProcessInfo, Instant)>,
    ttl: Duration,
}

impl ProcessResolver {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            ttl,
        }
    }

    pub fn get_process_info(&mut self, pid: u32) -> ProcessInfo {
        let now = Instant::now();

        if let Some((info, last_updated)) = self.cache.get(&pid) {
            if now.duration_since(*last_updated) < self.ttl {
                return ProcessInfo {
                    name: info.name.clone(),
                    context: info.context.clone(),
                };
            }
        }

        // Cache miss or expired
        let name = self
            .resolve_pid(pid)
            .unwrap_or_else(|| "unknown".to_string());
        let context = self.resolve_context(pid);

        let info = ProcessInfo {
            name: name.clone(),
            context: context.clone(),
        };

        self.cache.insert(pid, (info, now));
        ProcessInfo { name, context }
    }

    fn resolve_pid(&self, pid: u32) -> Option<String> {
        let comm_path = format!("/proc/{}/comm", pid);
        fs::read_to_string(comm_path)
            .map(|s| s.trim().to_string())
            .ok()
            .or_else(|| {
                // Fallback to cmdline if comm fails or is empty
                let cmdline_path = format!("/proc/{}/cmdline", pid);
                fs::read(cmdline_path)
                    .ok()
                    .and_then(|bytes| {
                        if bytes.is_empty() {
                            return None;
                        }
                        // cmdline is null-terminated, we want the first part (the executable)
                        let first_part = bytes.split(|&b| b == 0).next()?;
                        String::from_utf8(first_part.to_vec()).ok()
                    })
                    .map(|s| {
                        // Extract just the file name from the path
                        s.split('/').next_back().unwrap_or(&s).to_string()
                    })
            })
    }

    fn resolve_context(&self, pid: u32) -> ProcessContext {
        let cgroup_path = format!("/proc/{}/cgroup", pid);
        if let Ok(content) = fs::read_to_string(cgroup_path) {
            for line in content.lines() {
                // cgroup v2: 0::/path
                // cgroup v1: hierarchy:controller:/path
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    let path = parts[2];
                    if path == "/" {
                        continue;
                    }

                    // Systemd services
                    if path.contains(".service") {
                        if let Some(svc) = path.split('/').find(|s| s.ends_with(".service")) {
                            if svc.starts_with("user@") {
                                return ProcessContext::User(svc.to_string());
                            }
                            return ProcessContext::Systemd(svc.to_string());
                        }
                    }

                    // Docker containers
                    if path.contains("/docker/") {
                        if let Some(id) = path.split('/').next_back() {
                            return ProcessContext::Docker(id.to_string());
                        }
                    }

                    // Kubernetes pods
                    if path.contains("/kubepods") {
                        // Example: /kubepods.slice/kubepods-burstable.slice/kubepods-burstable-pod[pod_id].slice/docker-[container_id].scope
                        if let Some(pod_part) = path.split('/').find(|s| s.contains("pod")) {
                            return ProcessContext::Kubernetes(pod_part.to_string());
                        }
                    }
                }
            }
        }
        ProcessContext::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn test_resolve_self() {
        let mut resolver = ProcessResolver::new(Duration::from_secs(1));
        let pid = process::id();
        let info = resolver.get_process_info(pid);
        // The process name should be "netmonitor" or similar during tests
        assert!(!info.name.is_empty());
        assert_ne!(info.name, "unknown");
    }
}
