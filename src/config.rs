use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub watch_dir: String,
    pub method: String,
}

#[derive(Debug, Deserialize)]
pub struct TargetConfig {
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct IgnoreConfig {
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub targets: Vec<TargetConfig>,
    pub ignore: IgnoreConfig,
}

impl AppConfig {
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let cfg = toml::from_str(&content)?;
        Ok(cfg)
    }
}
