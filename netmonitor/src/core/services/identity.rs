use crate::core::types::Pid;
use crate::process::{ProcessInfo, ProcessResolver};
use std::time::Duration;

/// Trait for resolving PIDs to process information.
pub trait Resolver {
    fn resolve(&mut self, pid: Pid) -> ProcessInfo;
}

pub struct LocalResolver {
    inner: ProcessResolver,
}

impl LocalResolver {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: ProcessResolver::new(ttl),
        }
    }
}

impl Resolver for LocalResolver {
    fn resolve(&mut self, pid: Pid) -> ProcessInfo {
        self.inner.get_process_info(pid.0)
    }
}

pub struct IdentityService<R: Resolver> {
    resolver: R,
}

impl<R: Resolver> IdentityService<R> {
    pub fn new(resolver: R) -> Self {
        Self { resolver }
    }

    pub fn get_info(&mut self, pid: Pid) -> ProcessInfo {
        self.resolver.resolve(pid)
    }
}

#[cfg(test)]
pub struct MockResolver {}

#[cfg(test)]
impl MockResolver {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
impl Resolver for MockResolver {
    fn resolve(&mut self, _pid: Pid) -> ProcessInfo {
        ProcessInfo {
            name: "test-process".to_string(),
            context: crate::process::ProcessContext::User("test.service".to_string()),
        }
    }
}
