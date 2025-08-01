use anyhow::Result;
use rustsync::{server, config::AppConfig};

fn main() -> Result<()> {
    env_logger::init();

    let cfg = AppConfig::load_from_file("config.toml")?;
    server::run_server(&cfg)
}
