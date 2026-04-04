use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{Context, Result};

use crate::config::{self, Config};
use crate::registry::models::SourceRegistry;

fn cache_path() -> PathBuf {
    config::cache_dir().join("source.json")
}

pub fn fetch_registry(config: &Config) -> Result<SourceRegistry> {
    let body = reqwest::blocking::get(&config.registry_url)
        .context("Failed to download registry")?
        .text()
        .context("Failed to read registry response")?;

    let cache_dir = config::cache_dir();
    fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
    fs::write(cache_path(), &body).context("Failed to write registry cache")?;

    let registry: SourceRegistry =
        serde_json::from_str(&body).context("Failed to parse registry JSON")?;

    Ok(registry)
}

pub fn load_cached_registry() -> Result<Option<SourceRegistry>> {
    let path = cache_path();
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path).context("Failed to read cached registry")?;
    let registry: SourceRegistry =
        serde_json::from_str(&data).context("Failed to parse cached registry JSON")?;

    Ok(Some(registry))
}

pub fn is_cache_stale(cache_ttl_hours: u64) -> bool {
    let path = cache_path();
    let Ok(metadata) = fs::metadata(&path) else {
        return true;
    };
    let Ok(modified) = metadata.modified() else {
        return true;
    };
    let Ok(elapsed) = SystemTime::now().duration_since(modified) else {
        return true;
    };

    elapsed.as_secs() > cache_ttl_hours * 3600
}

pub fn get_registry(config: &Config) -> Result<SourceRegistry> {
    if is_cache_stale(config.cache_ttl_hours) {
        fetch_registry(config)
    } else {
        load_cached_registry()?.ok_or_else(|| anyhow::anyhow!("No cached registry found"))
    }
}
