use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

pub struct ProcessResolver {
    cache: HashMap<u32, (String, Instant)>,
    ttl: Duration,
}

impl ProcessResolver {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            ttl,
        }
    }

    pub fn get_process_name(&mut self, pid: u32) -> String {
        let now = Instant::now();

        if let Some((name, last_updated)) = self.cache.get(&pid) {
            if now.duration_since(*last_updated) < self.ttl {
                return name.clone();
            }
        }

        // Cache miss or expired
        let name = self.resolve_pid(pid).unwrap_or_else(|| "unknown".to_string());
        self.cache.insert(pid, (name.clone(), now));
        name
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
                        s.split('/').last().unwrap_or(&s).to_string()
                    })
            })
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
        let name = resolver.get_process_name(pid);
        // The process name should be "netmonitor" or similar during tests
        assert!(!name.is_empty());
        assert_ne!(name, "unknown");
    }
}
