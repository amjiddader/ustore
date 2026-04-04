use anyhow::Result;
use colored::*;

use crate::config;
use crate::registry::fetch;

pub fn run() -> Result<()> {
    let cfg = config::load_config()?;

    println!("{} Fetching latest package registry...", "→".cyan().bold());

    let registry = fetch::fetch_registry(&cfg)?;

    println!(
        "{} Registry updated — {} packages available.",
        "✓".green().bold(),
        registry.packages.len().to_string().bold()
    );

    Ok(())
}
