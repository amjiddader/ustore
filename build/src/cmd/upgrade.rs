use anyhow::{Result, bail};
use colored::*;

use crate::config;
use crate::downloader;
use crate::installer;
use crate::registry::fetch;
use crate::registry::search::find_package;
use crate::store;

pub fn run(id: Option<&str>) -> Result<()> {
    // Only allow: non-root user running with sudo
    let euid = {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata("/proc/self").map(|m| m.uid()).unwrap_or(0)
    };
    let sudo_user = std::env::var("SUDO_USER").ok()
        .filter(|s| !s.is_empty() && s != "root");

    if euid != 0 || sudo_user.is_none() {
        bail!(
            "{}\n  {}",
            "This command requires sudo from a non-root user.".red().bold(),
            "Usage: sudo ustore upgrade [package]".yellow()
        );
    }

    let cfg = config::load_config()?;

    let registry = fetch::get_registry(&cfg)?;

    let installed = store::list_installed()?;

    if installed.is_empty() {
        println!("{} No packages installed.", "ℹ".blue().bold());
        return Ok(());
    }

    let targets: Vec<_> = match id {
        Some(pkg_id) => {
            let found: Vec<_> = installed.iter().filter(|p| p.id == pkg_id).collect();
            if found.is_empty() {
                bail!("Package '{}' is not installed.", pkg_id);
            }
            found
        }
        None => installed.iter().collect(),
    };

    let mut upgraded = 0u32;
    let mut up_to_date = 0u32;

    for pkg in &targets {
        let registry_pkg = match find_package(&registry, &pkg.id) {
            Some(p) => p,
            None => {
                println!(
                    "{} {} not found in registry, skipping.",
                    "⚠".yellow().bold(),
                    pkg.name.bold()
                );
                continue;
            }
        };

        let registry_version = match registry_pkg.variants.first() {
            Some(v) => &v.version,
            None => {
                println!(
                    "{} {} has no variants in registry, skipping.",
                    "⚠".yellow().bold(),
                    pkg.name.bold()
                );
                continue;
            }
        };

        if registry_version == &pkg.version {
            println!(
                "{} {} v{} is already up to date.",
                "✓".green().bold(),
                pkg.name.bold(),
                pkg.version
            );
            up_to_date += 1;
            continue;
        }

        println!(
            "{} Upgrading {} from v{} to v{}...",
            "→".cyan().bold(),
            pkg.name.bold(),
            pkg.version,
            registry_version
        );

        let variant = &registry_pkg.variants[0];
        let filename = format!("{}_{}.deb", pkg.id, variant.version);
        let deb_path = downloader::download_to_cache(&variant.url, &filename)?;

        installer::install_deb(&deb_path, &registry_pkg.name, &variant.version)?;

        let dpkg_name = registry_pkg
            .dpkg_name
            .as_deref()
            .unwrap_or(&pkg.id);

        let real_version = installer::get_installed_version(dpkg_name)?
            .unwrap_or_else(|| variant.version.clone());

        store::record_install(
            &pkg.id,
            &registry_pkg.name,
            &real_version,
            &variant.pkg_type,
            registry_pkg.dpkg_name.as_deref(),
            registry_pkg.binary_name.as_deref(),
        )?;

        downloader::cleanup_file(&deb_path);

        println!(
            "{} {} upgraded to v{}.",
            "✓".green().bold(),
            registry_pkg.name.green().bold(),
            real_version
        );

        upgraded += 1;
    }

    println!();
    println!(
        "{} {} upgraded, {} already up to date.",
        "✓".green().bold(),
        upgraded.to_string().bold(),
        up_to_date.to_string().bold()
    );

    Ok(())
}
