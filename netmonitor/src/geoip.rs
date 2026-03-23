use maxminddb::Reader;
use once_cell::sync::Lazy;
use std::net::IpAddr;

// This will be a small dummy file if the real one isn't downloaded.
// In a real build, we'd include the real MaxMind or DB-IP mmdb.
static GEOIP_DB: &[u8] = include_bytes!("../resources/geoip-dummy.mmdb");

pub struct GeoResolver {
    reader: Option<Reader<&'static [u8]>>,
}

impl GeoResolver {
    pub fn new() -> Self {
        let reader = Reader::from_source(GEOIP_DB).ok();
        Self { reader }
    }

    pub fn resolve(&self, ip: IpAddr) -> (String, String) {
        if let Some(ref reader) = self.reader {
            let country: Option<maxminddb::geoip2::Country> = reader.lookup(ip).ok();
            let country_name = country
                .and_then(|c| c.country)
                .and_then(|c| c.names)
                .and_then(|n| n.get("en").map(|&s| s.to_string()))
                .unwrap_or_else(|| "Unknown".to_string());

            // For ASN/ISP, we'd usually use a separate ASN database.
            // For now, we'll just mock it or if we use a combined DB it might be there.
            let isp = "Unknown".to_string(); // Placeholder for ISP/ASN

            (country_name, isp)
        } else {
            // Mock logic for demonstration if database is missing/invalid
            if ip.is_loopback() {
                ("Localhost".to_string(), "Internal".to_string())
            } else if ip.is_multicast() {
                ("Multicast".to_string(), "Network".to_string())
            } else {
                ("Unknown".to_string(), "Unknown".to_string())
            }
        }
    }
}

pub static RESOLVER: Lazy<GeoResolver> = Lazy::new(GeoResolver::new);

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_fallback_localhost() {
        let resolver = GeoResolver::new();
        let (country, isp) = resolver.resolve(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(country, "Localhost");
        assert_eq!(isp, "Internal");
    }

    #[test]
    fn test_fallback_unknown() {
        let resolver = GeoResolver::new();
        let (country, isp) = resolver.resolve(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert_eq!(country, "Unknown");
        assert_eq!(isp, "Unknown");
    }
}
