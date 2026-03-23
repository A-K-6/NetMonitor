use crate::core::collector::Collector;
use crate::core::domain::{ConnectionSummary, MonitoringSnapshot, ProcessSummary};
use crate::core::error::Result;
use crate::core::services::identity::Resolver;
use crate::core::services::{EnforcementService, IdentityService, TrafficService};
use crate::core::types::Bytes;
use crate::dns;
use crate::geoip;
use crate::protocol;
use std::collections::HashMap;

pub struct MonitoringService<C: Collector, R: Resolver> {
    pub collector: C,
    pub identity: IdentityService<R>,
    pub traffic: TrafficService,
    pub enforcement: EnforcementService,
}

impl<C: Collector, R: Resolver> MonitoringService<C, R> {
    pub fn new(collector: C, identity: IdentityService<R>) -> Self {
        Self {
            collector,
            identity,
            traffic: TrafficService::new(),
            enforcement: EnforcementService::new(),
        }
    }

    pub fn snapshot(
        &mut self,
        dns_enabled: bool,
        geo_ip_enabled: bool,
    ) -> Result<MonitoringSnapshot> {
        self.traffic
            .update(&mut self.collector)
            .map_err(|e| crate::core::error::NetMonitorError::InternalError(e.to_string()))?;

        let mut processes = Vec::new();
        for (pid, stats) in self.traffic.get_stats() {
            let info = self.identity.get_info(*pid);
            let (up_rate, down_rate) = self.traffic.get_process_rates(*pid);
            processes.push(ProcessSummary {
                pid: *pid,
                name: info.name,
                context: info.context,
                up: Bytes(stats.bytes_sent),
                down: Bytes(stats.bytes_recv),
                total: Bytes(stats.bytes_sent + stats.bytes_recv),
                up_rate,
                down_rate,
            });
        }

        let mut connections = HashMap::new();
        let mut protocol_stats = HashMap::new();
        let mut country_stats = HashMap::new();

        for (key, stats) in self.traffic.get_connections() {
            use std::net::{IpAddr, Ipv4Addr};
            let dst_addr = Ipv4Addr::from(u32::from_be(key.dst_ip));
            let src_ip = Ipv4Addr::from(u32::from_be(key.src_ip)).to_string();
            let dst_ip_addr = IpAddr::V4(dst_addr);
            let dst_ip = dst_addr.to_string();

            let (country, isp) = if geo_ip_enabled {
                geoip::RESOLVER.resolve(dst_ip_addr)
            } else {
                ("Disabled".to_string(), "Disabled".to_string())
            };
            let service = protocol::RESOLVER.resolve(key.proto, key.dst_port);

            // Aggregate protocol/country stats
            let p_stats = protocol_stats
                .entry(key.proto)
                .or_insert((Bytes(0), Bytes(0)));
            p_stats.0 += Bytes(stats.bytes_sent);
            p_stats.1 += Bytes(stats.bytes_recv);

            let c_stats = country_stats
                .entry(country.clone())
                .or_insert((Bytes(0), Bytes(0)));
            c_stats.0 += Bytes(stats.bytes_sent);
            c_stats.1 += Bytes(stats.bytes_recv);

            let hostname = if dns_enabled {
                match dns::RESOLVER.get_cached(dst_ip_addr) {
                    Some(h) => h,
                    None => {
                        let resolver_addr = dst_ip_addr;
                        tokio::spawn(async move {
                            dns::RESOLVER.resolve(resolver_addr).await;
                        });
                        None
                    }
                }
            } else {
                None
            };

            let conn_summary = ConnectionSummary {
                proto: key.proto,
                src_ip,
                src_port: key.src_port,
                dst_ip,
                dst_port: key.dst_port,
                up: Bytes(stats.bytes_sent),
                down: Bytes(stats.bytes_recv),
                country,
                isp,
                hostname,
                service,
            };
            connections
                .entry(key.pid)
                .or_insert_with(Vec::new)
                .push(conn_summary);
        }

        Ok(MonitoringSnapshot {
            timestamp: chrono::Utc::now(),
            processes,
            connections,
            total_up: self.traffic.total_upload(),
            total_down: self.traffic.total_download(),
            session_up: self.traffic.session_upload(),
            session_down: self.traffic.session_download(),
            protocol_stats,
            country_stats,
        })
    }
}
