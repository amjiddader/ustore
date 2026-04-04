use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
    let dir = dirs::config_dir()
        .expect("could not determine config directory")
        .join("ustore");
    fs::create_dir_all(&dir).expect("failed to create config directory");
    dir
}

pub fn cache_dir() -> PathBuf {
    let dir = dirs::cache_dir()
        .expect("could not determine cache directory")
        .join("ustore");
    fs::create_dir_all(&dir).expect("failed to create cache directory");
    dir
}

pub fn db_path() -> PathBuf {
    let path = dirs::data_local_dir()
        .expect("could not determine data directory")
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
