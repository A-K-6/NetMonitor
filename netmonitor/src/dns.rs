use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

pub struct DnsResolver {
    resolver: TokioAsyncResolver,
    cache: DashMap<IpAddr, (Option<String>, Instant)>,
    ttl: Duration,
}

impl DnsResolver {
    pub fn new() -> Self {
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );

        Self {
            resolver,
            cache: DashMap::new(),
            ttl: Duration::from_secs(3600), // 1 hour TTL
        }
    }

    pub async fn resolve(&self, ip: IpAddr) -> Option<String> {
        // Check cache first
        if let Some(entry) = self.cache.get(&ip) {
            let (hostname, timestamp) = entry.value();
            if timestamp.elapsed() < self.ttl {
                return hostname.clone();
            }
        }

        // Perform reverse DNS lookup
        match self.resolver.reverse_lookup(ip).await {
            Ok(lookup) => {
                let hostname = lookup.iter().next().map(|name| {
                    name.to_utf8().trim_end_matches('.').to_string()
                });
                self.cache.insert(ip, (hostname.clone(), Instant::now()));
                hostname
            }
            Err(_) => {
                // Cache negative results too, but for a shorter period?
                // For now just cache as None for standard TTL
                self.cache.insert(ip, (None, Instant::now()));
                None
            }
        }
    }

    pub fn get_cached(&self, ip: IpAddr) -> Option<Option<String>> {
        self.cache.get(&ip).map(|e| e.value().0.clone())
    }
}

pub static RESOLVER: Lazy<DnsResolver> = Lazy::new(DnsResolver::new);
