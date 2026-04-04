use anyhow::Result;
use colored::*;

use crate::config;
use crate::registry::fetch;
use crate::registry::search::search_packages;

pub fn run(query: &str) -> Result<()> {
    let cfg = config::load_config()?;

    let registry = fetch::get_registry(&cfg)?;

    let results = search_packages(&registry, query);

    if results.is_empty() {
        println!("{} No packages found for '{}'.", "✗".red().bold(), query);
        return Ok(());
    }

    println!(
        "{} Found {} result(s) for '{}':\n",
        "→".cyan().bold(),
        results.len(),
        query
    );

    println!(
        "  {:<20} {:<25} {:<15} {:<10} {:<10} {}",
        "ID".bold().underline(),
        "Name".bold().underline(),
        "Category".bold().underline(),
        "Type".bold().underline(),
        "Size".bold().underline(),
        "Verified".bold().underline(),
    );

    for pkg in &results {
        let pkg_type = pkg
            .variants
            .first()
            .map(|v| v.pkg_type.as_str())
            .unwrap_or("—");

        let size = pkg
            .variants
            .first()
            .map(|v| format!("{:.1} MB", v.size_mb))
            .unwrap_or_else(|| "—".to_string());

        let verified = if pkg.verified {
            "✓".green().to_string()
        } else {
            "—".dimmed().to_string()
        };

        println!(
            "  {:<20} {:<25} {:<15} {:<10} {:<10} {}",
            pkg.id.cyan(), pkg.name, pkg.category, pkg_type, size, verified
        );
    }

    println!();
    println!("  {} Use {} to install.", "💡".dimmed(), "ustore install <ID>".green().bold());

    Ok(())
}
