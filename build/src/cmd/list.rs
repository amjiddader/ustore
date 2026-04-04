use anyhow::Result;
use colored::*;

use crate::config;
use crate::registry::fetch;
use crate::registry::search::find_package;
use crate::store;

pub fn run() -> Result<()> {
    let installed = store::list_installed()?;

    if installed.is_empty() {
        println!("{} No packages installed via ustore.", "✗".yellow().bold());
        return Ok(());
    }

    let cfg = config::load_config()?;
    let registry = fetch::load_cached_registry()?
        .or_else(|| fetch::fetch_registry(&cfg).ok());

    println!(
        "{} {} installed package(s):\n",
        "→".cyan().bold(),
        installed.len()
    );

    println!(
        "  {:<20} {:<25} {:<20} {:<20} {:<10}",
        "ID".bold().underline(),
        "Name".bold().underline(),
        "Installed".bold().underline(),
        "Available".bold().underline(),
        "Status".bold().underline(),
    );

    for pkg in &installed {
        let (available, status) = match &registry {
            Some(reg) => match find_package(reg, &pkg.id) {
                Some(p) => {
                    let reg_ver = p.variants.first().map(|v| v.version.as_str()).unwrap_or("?");
                    if reg_ver != pkg.version {
                        (reg_ver.to_string(), "⬆ update".yellow().to_string())
                    } else {
                        (reg_ver.to_string(), "✓".green().to_string())
                    }
                }
                None => ("—".to_string(), "✓".green().to_string()),
            },
            None => ("—".to_string(), "?".dimmed().to_string()),
        };

        println!(
            "  {:<20} {:<25} {:<20} {:<20} {}",
            pkg.id.cyan(), pkg.name, pkg.version, available, status
        );
    }

    Ok(())
}
