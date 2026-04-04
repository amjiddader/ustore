use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use colored::Colorize;

pub fn download_file(url: &str, dest: &Path) -> Result<PathBuf> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).context("Failed to create destination directory")?;
    }

    let dest_str = dest.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
    let dir = dest.parent().unwrap();
    let filename = dest.file_name().unwrap().to_str().unwrap();

    println!("  {} Downloading.....", "↓".cyan().bold());

    let output = Command::new("aria2c")
        .args([
            "--max-connection-per-server=16",
            "--split=16",
            "--min-split-size=1M",
            "--max-concurrent-downloads=1",
            "--file-allocation=none",
            "--follow-torrent=false",
            "--continue=true",
            "--allow-overwrite=true",
            "--auto-file-renaming=false",
            "--console-log-level=error",
            "--summary-interval=0",
            "--download-result=hide",
            "--human-readable=true",
            &format!("--dir={}", dir.display()),
            &format!("--out={}", filename),
            url,
        ])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to run aria2c. Is it installed? (sudo apt install aria2)")?;

    if !output.success() {
        let _ = fs::remove_file(dest);
        bail!("aria2c download failed for: {}", url);
    }

    if !dest.exists() {
        bail!("Download completed but file not found at {}", dest_str);
    }

    Ok(dest.to_path_buf())
}

pub fn download_to_cache(url: &str, filename: &str) -> Result<PathBuf> {
    let cache = crate::config::cache_dir().join("downloads").join(filename);
    download_file(url, &cache)
}

pub fn cleanup_file(path: &std::path::Path) {
    let _ = fs::remove_file(path);
    let aria2 = std::path::PathBuf::from(format!("{}.aria2", path.display()));
    let _ = fs::remove_file(aria2);
}
