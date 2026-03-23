use once_cell::sync::Lazy;
use std::collections::HashMap;

pub struct ProtocolResolver {
    services: HashMap<(u32, u16), &'static str>,
}

impl ProtocolResolver {
    pub fn new() -> Self {
        let mut services = HashMap::new();

        // Well-known TCP services (proto 6)
        services.insert((6, 22), "SSH");
        services.insert((6, 23), "Telnet");
        services.insert((6, 25), "SMTP");
        services.insert((6, 53), "DNS");
        services.insert((6, 80), "HTTP");
        services.insert((6, 110), "POP3");
        services.insert((6, 143), "IMAP");
        services.insert((6, 443), "HTTPS");
        services.insert((6, 587), "SMTP (TLS)");
        services.insert((6, 993), "IMAPS");
        services.insert((6, 995), "POP3S");
        services.insert((6, 3306), "MySQL");
        services.insert((6, 5432), "PostgreSQL");
        services.insert((6, 6379), "Redis");
        services.insert((6, 8080), "HTTP-Proxy");

        // Well-known UDP services (proto 17)
        services.insert((17, 53), "DNS");
        services.insert((17, 67), "DHCP-Srv");
        services.insert((17, 68), "DHCP-Cli");
        services.insert((17, 123), "NTP");
        services.insert((17, 161), "SNMP");
        services.insert((17, 443), "QUIC/HTTP3");
        services.insert((17, 514), "Syslog");
        services.insert((17, 1194), "OpenVPN");
        services.insert((17, 5353), "mDNS");

        Self { services }
    }

    pub fn resolve(&self, proto: u32, port: u16) -> String {
        if let Some(service) = self.services.get(&(proto, port)) {
            service.to_string()
        } else {
            "-".to_string()
        }
    }
}

pub static RESOLVER: Lazy<ProtocolResolver> = Lazy::new(ProtocolResolver::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_known() {
        let resolver = ProtocolResolver::new();
        assert_eq!(resolver.resolve(6, 22), "SSH");
        assert_eq!(resolver.resolve(6, 443), "HTTPS");
        assert_eq!(resolver.resolve(17, 53), "DNS");
    }

    #[test]
    fn test_resolve_unknown() {
        let resolver = ProtocolResolver::new();
        assert_eq!(resolver.resolve(6, 12345), "-");
        assert_eq!(resolver.resolve(17, 12345), "-");
    }
}
