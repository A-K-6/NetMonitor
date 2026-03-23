use crate::theme::ThemeType;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub alerts: AlertConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: ThemeType,
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: u64,
    #[serde(default = "default_show_graph")]
    pub show_graph: bool,
    #[serde(default = "default_view")]
    pub default_view: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    #[serde(default = "default_true")]
    pub dns_resolution: bool,
    #[serde(default = "default_true")]
    pub geo_ip_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertConfig {
    #[serde(default = "default_threshold")]
    pub default_threshold: u64,
    #[serde(default)]
    pub processes: HashMap<String, u64>,
}

fn default_theme() -> ThemeType {
    ThemeType::Auto
}
fn default_refresh_rate() -> u64 {
    1000
}
fn default_show_graph() -> bool {
    false
}
fn default_view() -> String {
    "Dashboard".to_string()
}
fn default_true() -> bool {
    true
}
fn default_threshold() -> u64 {
    0
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            refresh_rate: default_refresh_rate(),
            show_graph: default_show_graph(),
            default_view: default_view(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            dns_resolution: default_true(),
            geo_ip_enabled: default_true(),
        }
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            default_threshold: default_threshold(),
            processes: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> (Self, Option<PathBuf>) {
        let config_path = path.unwrap_or_else(|| {
            if let Some(proj_dirs) = ProjectDirs::from("", "", "netmonitor") {
                let config_dir = proj_dirs.config_dir();
                if !config_dir.exists() {
                    let _ = fs::create_dir_all(config_dir);
                }
                config_dir.join("config.toml")
            } else {
                PathBuf::from("config.toml")
            }
        });

        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return (config, Some(config_path));
                }
            }
        } else {
            // Save a default config file if one doesn't exist
            let default_config = Self::default();
            let _ = default_config.save(&config_path);
            return (default_config, Some(config_path));
        }

        (Self::default(), Some(config_path))
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_config_resilience(ref content in ".*") {
            // Config parser should not panic on arbitrary strings
            let _: Result<Config, _> = toml::from_str(content);
        }

        #[test]
        fn test_config_serialization_roundtrip(
            refresh_rate in 100u64..10000,
            show_graph in proptest::bool::ANY,
            dns_res in proptest::bool::ANY,
        ) {
            let mut config = Config::default();
            config.ui.refresh_rate = refresh_rate;
            config.ui.show_graph = show_graph;
            config.network.dns_resolution = dns_res;

            let serialized = toml::to_string(&config).unwrap();
            let deserialized: Config = toml::from_str(&serialized).unwrap();

            prop_assert_eq!(deserialized.ui.refresh_rate, refresh_rate);
            prop_assert_eq!(deserialized.ui.show_graph, show_graph);
            prop_assert_eq!(deserialized.network.dns_resolution, dns_res);
        }
    }
}
