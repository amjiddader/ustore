use anyhow::{Result, bail};
use colored::*;

use crate::config;
use crate::registry::fetch;
use crate::registry::search::find_package;
use crate::installer;
use crate::store;

pub fn run(id: &str) -> Result<()> {
    if !store::is_tracked(id)? {
        bail!("Package '{}' is not installed via ustore.", id);
    }

    let cfg = config::load_config()?;
    let registry = fetch::get_registry(&cfg)?;

    let dpkg_name = find_package(&registry, id)
        .and_then(|p| p.dpkg_name.as_deref())
        .unwrap_or(id);

    println!("{} {} {}...", "→".cyan().bold(), "Removing".bold(), id);
    installer::remove_deb(dpkg_name)?;
    store::record_remove(id)?;

    println!(
        "{} {} removed successfully.",
        "✓".green().bold(),
        id.green().bold()
    );

    Ok(())
}
