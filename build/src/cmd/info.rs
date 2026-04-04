use anyhow::{Result, bail};
use colored::*;

use crate::config;
use crate::registry::fetch;
use crate::registry::search::find_package;
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
    let tracked = store::is_tracked(id)?;
    let installed = installer::is_installed(dpkg_name);

    println!("{}", package.name.bold().cyan());
    println!("{}", "─".repeat(40).dimmed());

    println!("  {:<14} {}", "ID:".bold(), package.id);
    println!("  {:<14} {}", "Description:".bold(), package.description);
    println!("  {:<14} {}", "Publisher:".bold(), package.publisher);
    println!("  {:<14} {}", "License:".bold(), package.license);
    println!("  {:<14} {}", "Category:".bold(), package.category);

    println!("  {:<14} {}", "Website:".bold(), package.website);

    println!(
        "  {:<14} {}",
        "Verified:".bold(),
        if package.verified {
            "Yes".green().to_string()
        } else {
            "No".dimmed().to_string()
        }
    );

    println!(
        "  {:<14} {}",
        "Installed:".bold(),
        if tracked && installed {
            "Yes".green().to_string()
        } else {
            "No".dimmed().to_string()
        }
    );

    if !package.variants.is_empty() {
        println!("\n  {}:", "Variants".bold().underline());
        for v in &package.variants {
            let size = format!("{:.1} MB", v.size_mb);
            println!(
                "    • v{} ({}) — {} [{}]",
                v.version, v.arch, v.pkg_type, size
            );
            if let Some(ref min_ubuntu) = v.min_ubuntu {
                println!("      min Ubuntu: {}", min_ubuntu);
            }
        }
    }

    Ok(())
}
