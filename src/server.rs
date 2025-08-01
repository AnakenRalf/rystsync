use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use log::{error, info};
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    sync::mpsc::channel,
    time::{Duration},
};

use crate::config::{AppConfig, TargetConfig};

use crossbeam_channel::{unbounded, tick, select};

fn is_ignored(path: &Path, ignore_set: &globset::GlobSet) -> bool {
    ignore_set.is_match(path)
}

fn send_file(path: &Path, target: &TargetConfig, watch_dir: &Path) -> Result<()> {
    let addr = format!("{}:{}", target.host, target.port.unwrap_or(7878));
    let mut stream = TcpStream::connect(addr)?;
    let relative_path = path.strip_prefix(watch_dir).unwrap_or(path);

    let metadata = serde_json::to_vec(&serde_json::json!({
    "path": relative_path.to_string_lossy()
    }))?;
    let meta_len = metadata.len() as u32;

    stream.write_all(&meta_len.to_be_bytes())?;
    stream.write_all(&metadata)?;

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    stream.write_all(&buffer)?;

    info!("[SERVER] Sent file {:?} to {}", relative_path, target.host);
    Ok(())
}

fn initial_sync(cfg: &AppConfig, ignore_set: &globset::GlobSet) -> Result<()> {
    let base_path = PathBuf::from(&cfg.general.watch_dir);

    for entry in walkdir::WalkDir::new(&base_path)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() && !is_ignored(path, ignore_set) {
            for target in &cfg.targets {
                if let Err(e) = send_file(path, target, &base_path) {
                    error!(
                        "[SERVER] ❌ Failed to send {:?} to {}: {}",
                        path, target.host, e
                    );
                }
            }
        }
    }

    Ok(())
}

pub fn run_server(cfg: &AppConfig) -> Result<()> {
    println!("[SERVER] Watching path: {}", cfg.general.watch_dir);
    let watch_path = PathBuf::from(&cfg.general.watch_dir);

    let (tx_raw, rx_raw) = channel();
    let (tx_files, rx_files) = unbounded();

    let mut watcher: RecommendedWatcher = Watcher::new(tx_raw, NotifyConfig::default())?;
    watcher.watch(&watch_path, RecursiveMode::Recursive)?;

    // Build ignore matcher
    let mut builder = GlobSetBuilder::new();
    for pattern in &cfg.ignore.ignore_patterns {
        builder.add(Glob::new(pattern)?);
    }
    let ignore_set = builder.build()?;

    // Initial sync
    initial_sync(cfg, &ignore_set)?;

    // Timer ticker for debouncing
    let tick_rx = tick(Duration::from_millis(500));
    let mut changed_files = HashSet::new();

    // Thread to collect paths
    std::thread::spawn(move || {
        while let Ok(event) = rx_raw.recv() {
            if let Ok(ev) = event {
                match ev.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        for path in ev.paths {
                            if path.is_file() {
                                let _ = tx_files.send(path);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    // Main loop with debounce
    loop {
        select! {
            recv(rx_files) -> msg => {
                if let Ok(path) = msg {
                    if !is_ignored(&path, &ignore_set) {
                        changed_files.insert(path);
                    }
                }
            }
            recv(tick_rx) -> _ => {
                if !changed_files.is_empty() {
                    for path in changed_files.drain() {
                        for target in &cfg.targets {
                            if let Err(e) = send_file(&path, target, &watch_path) {
                                error!(
                                    "[SERVER] Failed to send {:?} to {}: {}",
                                    path, target.host, e
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}