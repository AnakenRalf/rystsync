use anyhow::{bail, Result};
use rustsync::{server, config::AppConfig, client};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rustsync", about = "Sync tool in client or server mode")]
struct Opt {
    /// Mode to run: "server" or "client"
    #[structopt(short, long)]
    mode: String,

    /// Path to config file (default: config.toml)
    #[structopt(short, long, default_value = "config.toml")]
    config: String,
}

fn main() -> Result<()> {
    // env_logger::init();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();


    let opt = Opt::from_args();
    let cfg = AppConfig::load_from_file(&opt.config)?;

    match opt.mode.as_str() {
        "server" => server::run_server(&cfg)?,
        "client" => client::run_client(&cfg)?,
        other => bail!("Unknown mode '{}'. Use 'server' or 'client'", other),
    }

    Ok(())
}
