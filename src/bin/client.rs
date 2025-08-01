use anyhow::Result;
use rustsync::{config::AppConfig, client};

fn main() -> Result<()> {
    env_logger::init();

    let cfg = AppConfig::load_from_file("config.toml")?;
    client::run_client(&cfg)
}