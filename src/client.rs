use anyhow::Result;
use std::{
    fs::{create_dir_all, File},
    io::{Read},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
};

use crate::config::AppConfig;
use serde::Deserialize;
use log::{info, error};

#[derive(Debug, Deserialize)]
struct FileMetadata {
    path: String,
}

fn handle_client(mut stream: TcpStream, base_dir: &Path) -> Result<()> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let meta_len = u32::from_be_bytes(len_buf) as usize;

    let mut meta_buf = vec![0u8; meta_len];
    stream.read_exact(&mut meta_buf)?;
    let metadata: FileMetadata = serde_json::from_slice(&meta_buf)?;

    let rel_path = PathBuf::from(metadata.path);
    let full_path = base_dir.join(&rel_path);

    if let Some(parent) = full_path.parent() {
        create_dir_all(parent)?;
    }

    let mut file = File::create(&full_path)?;
    std::io::copy(&mut stream, &mut file)?;

    info!("✅ Synced: {:?}", full_path);
    Ok(())
}

pub fn run_client(cfg: &AppConfig) -> Result<()> {
    let first_target = cfg.targets.first().expect("No targets configured");
    let base_path = PathBuf::from(&first_target.path);
    let port = first_target.port.unwrap_or(7878);

    let listener = TcpListener::bind(("0.0.0.0", port))?;
    println!("📡 Client listening on 0.0.0.0:{}, saving to {:?}", port, base_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let base_path = base_path.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, &base_path) {
                        eprintln!("❌ Error handling client: {}", e);
                    }
                });
            }
            Err(e) => error!("❌ Connection failed: {}", e),
        }
    }

    Ok(())
}
