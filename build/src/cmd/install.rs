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
    let binary_name = package.binary_name.as_deref().unwrap_or(id);

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

    // Check if already installed (deb via dpkg, appimage/tarball via /opt/ustore/)
    let already_installed = match variant.pkg_type.as_str() {
        "appimage" | "tar.gz" | "tar.xz" => store::is_tracked(id)? && installer::is_appimage_installed(id),
        _ => store::is_tracked(id)? && installer::is_installed(dpkg_name),
    };

    if already_installed {
        println!("{} {} is already installed.", "✓".green().bold(), package.name.bold());
        return Ok(());
    }

    println!(
        "{} {} v{} ({})...",
        "→".cyan().bold(),
        "Downloading".bold(),
        variant.version,
        variant.arch
    );

    let ext = match variant.pkg_type.as_str() {
        "appimage" => "AppImage",
        "tar.gz" => "tar.gz",
        "tar.xz" => "tar.xz",
        _ => "deb",
    };
    let filename = format!("{}_{}.{}", id, variant.version, ext);
    let file_path = downloader::download_to_cache(&variant.url, &filename)?;

    println!("{}  {}...", "→".cyan().bold(), "Installing".bold());

    let real_version = match variant.pkg_type.as_str() {
        "appimage" => {
            installer::install_appimage(&file_path, id, binary_name, &package.name, &variant.version)?;
            variant.version.clone()
        }
        "tar.gz" | "tar.xz" => {
            installer::install_tarball(&file_path, id, binary_name, &package.name, &variant.version)?;
            variant.version.clone()
        }
        _ => {
            installer::install_deb(&file_path, &package.name, &variant.version)?;
            installer::get_installed_version(dpkg_name)?
                .unwrap_or_else(|| variant.version.clone())
        }
    };

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
