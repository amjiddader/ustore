use anyhow::{Result, bail};
use colored::*;

use crate::config;
use crate::registry::fetch;
use crate::registry::search::find_package;
use crate::downloader;
use crate::installer;
use crate::store;

pub fn run(id: &str) -> Result<()> {
    let cfg = config::load_config()?;

    let registry = fetch::get_registry(&cfg)?;

    let package = match find_package(&registry, id) {
        Some(p) => p,
        None => bail!("Package '{}' not found in registry.", id),
    };

    let dpkg_name = package.dpkg_name.as_deref().unwrap_or(id);

    if store::is_tracked(id)? && installer::is_installed(dpkg_name) {
        println!("{} {} is already installed.", "✓".green().bold(), package.name.bold());
        return Ok(());
    }

    let arch = std::process::Command::new("dpkg")
        .arg("--print-architecture")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "amd64".to_string());

    let variant = package
        .variants
        .iter()
        .find(|v| v.arch == arch || v.arch == "all")
        .or_else(|| package.variants.first());

    let variant = match variant {
        Some(v) => v,
        None => bail!("No compatible variant found for package '{}'.", id),
    };

    println!(
        "{} {} v{} ({})...",
        "→".cyan().bold(),
        "Downloading".bold(),
        variant.version,
        variant.arch
    );

    let filename = format!("{}_{}.deb", id, variant.version);
    let deb_path = downloader::download_to_cache(&variant.url, &filename)?;

    println!("{}  {}...", "→".cyan().bold(), "Installing".bold());
    installer::install_deb(&deb_path, &package.name, &variant.version)?;

    let real_version = installer::get_installed_version(dpkg_name)?
        .unwrap_or_else(|| variant.version.clone());

    store::record_install(
        id,
        &package.name,
        &real_version,
        &variant.pkg_type,
        package.dpkg_name.as_deref(),
        package.binary_name.as_deref(),
    )?;

    println!("{} Cleaning up downloads...", "→".cyan().bold());
    let _ = downloader::cleanup_cache();

    println!(
        "{} {} v{} installed successfully!",
        "✓".green().bold(),
        package.name.green().bold(),
        real_version
    );

    Ok(())
}
