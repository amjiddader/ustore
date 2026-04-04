use anyhow::{Result, bail};
use colored::*;

use crate::config;
use crate::registry::fetch;
use crate::registry::search::find_package;
use crate::installer;
use crate::store;

pub fn run(id: &str) -> Result<()> {
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
            "Usage: sudo ustore remove <package>".yellow()
        );
    }

    if !store::is_tracked(id)? {
        bail!("Package '{}' is not installed via ustore.", id);
    }

    let cfg = config::load_config()?;
    let registry = fetch::get_registry(&cfg)?;

    let package = find_package(&registry, id);
    let pkg_type = package
        .and_then(|p| p.variants.first())
        .map(|v| v.pkg_type.as_str())
        .unwrap_or("deb");

    println!("{} {} {}...", "→".cyan().bold(), "Removing".bold(), id);

    match pkg_type {
        "appimage" | "tar.gz" | "tar.xz" => {
            let binary_name = package
                .and_then(|p| p.binary_name.as_deref())
                .unwrap_or(id);
            installer::remove_tarball(id, binary_name)?;
        }
        "run" => {
            // .run apps install system-wide, not in /opt/ustore/
            // Just remove tracking — user may need to uninstall manually
            println!(
                "  {} .run packages may need manual uninstall. Removing uStore tracking.",
                "⚠".yellow().bold()
            );
        }
        _ => {
            let dpkg_name = package
                .and_then(|p| p.dpkg_name.as_deref())
                .unwrap_or(id);
            installer::remove_deb(dpkg_name)?;
        }
    }

    store::record_remove(id)?;

    println!(
        "{} {} removed successfully.",
        "✓".green().bold(),
        id.green().bold()
    );

    Ok(())
}
