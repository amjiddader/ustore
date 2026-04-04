use anyhow::Result;
use colored::*;

use crate::config;
use crate::registry::fetch;

pub fn run() -> Result<()> {
    let cfg = config::load_config()?;

    let registry = fetch::get_registry(&cfg)?;

    if registry.categories.is_empty() {
        println!("{} No categories found.", "✗".yellow().bold());
        return Ok(());
    }

    println!(
        "{} {} categories:\n",
        "→".cyan().bold(),
        registry.categories.len()
    );

    for cat in &registry.categories {
        println!(
            "  {} {:<20} {}",
            cat.icon,
            cat.name.bold(),
            cat.description.dimmed()
        );
    }

    Ok(())
}
