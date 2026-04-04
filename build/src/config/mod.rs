use anyhow::{Context, Result, bail};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Read real UID from /proc/self/status (first field of Uid: line).
/// Real UID is the original user's UID — stays non-zero under sudo, is 0 for root login.
fn read_real_uid() -> u32 {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u32>().ok())
        })
        .unwrap_or(0)
}

/// Require that the process is running via sudo from a non-root user.
/// Blocks: direct root login, non-sudo normal user.
/// Allows: normal user running `sudo ustore ...`.
pub fn require_sudo(usage_hint: &str) -> Result<()> {
    let real_uid = read_real_uid();
    if real_uid == 0 {
        bail!(
            "{}\n  {}",
            "Root user cannot use this command.".red().bold(),
            format!("Login as a normal user and run: sudo {}", usage_hint).yellow()
        );
    }
    let euid = {
        use std::os::unix::fs::MetadataExt;
        fs::metadata("/proc/self").map(|m| m.uid()).unwrap_or(1)
    };
    if euid != 0 {
        bail!(
            "{}\n  {}",
            "This command requires sudo.".red().bold(),
            format!("Usage: sudo {}", usage_hint).yellow()
        );
    }
    Ok(())
}

/// Get the real user's home directory, even when running under sudo.
fn real_home() -> PathBuf {
    // If running under sudo, use SUDO_USER's home instead of /root/
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if sudo_user != "root" {
            return PathBuf::from(format!("/home/{}", sudo_user));
        }
    }
    dirs::home_dir().expect("could not determine home directory")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub registry_url: String,
    pub config_base_url: String,
    pub cache_ttl_hours: u64,
    pub install_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registry_url: "https://raw.githubusercontent.com/amjiddader/ustore/main/source.json"
                .to_string(),
            config_base_url:
                "https://raw.githubusercontent.com/amjiddader/ustore/main/config".to_string(),
            cache_ttl_hours: 24,
            install_dir: "/opt/ustore".to_string(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    let dir = real_home().join(".config").join("ustore");
    fs::create_dir_all(&dir).expect("failed to create config directory");
    dir
}

pub fn cache_dir() -> PathBuf {
    let dir = real_home().join(".cache").join("ustore");
    fs::create_dir_all(&dir).expect("failed to create cache directory");
    dir
}

pub fn db_path() -> PathBuf {
    let path = real_home()
        .join(".local")
        .join("share")
        .join("ustore")
        .join("ustore.db");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create database directory");
    }
    path
}

pub fn load_config() -> Result<Config> {
    let path = config_dir().join("config.toml");
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config from {}", path.display()))?;
    let config: Config =
        toml::from_str(&content).with_context(|| "failed to parse config.toml")?;
    Ok(config)
}

#[allow(dead_code)]
pub fn save_config(config: &Config) -> Result<()> {
    let path = config_dir().join("config.toml");
    let content = toml::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(&path, content)
        .with_context(|| format!("failed to write config to {}", path.display()))?;
    Ok(())
}
