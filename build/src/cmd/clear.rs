use anyhow::Result;
use colored::*;
use std::io::Write;

pub fn run() -> Result<()> {
    let downloads_dir = crate::config::cache_dir().join("downloads");

    if !downloads_dir.exists() {
        println!("{} No cached downloads found.", "ℹ".blue().bold());
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(&downloads_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    if entries.is_empty() {
        println!("{} No cached downloads found.", "ℹ".blue().bold());
        return Ok(());
    }

    println!(
        "  {} Found {} cached file(s):",
        "ℹ".blue().bold(),
        entries.len()
    );
    for entry in &entries {
        let meta = entry.metadata().ok();
        let size = meta
            .map(|m| format!("{:.1} MB", m.len() as f64 / 1_048_576.0))
            .unwrap_or_default();
        println!(
            "    {} {} ({})",
            "•".white(),
            entry.file_name().to_string_lossy().bold(),
            size.yellow()
        );
    }

    println!();
    print!(
        "  {} Are you sure you want to delete all leftover files? [y/N] ",
        "?".yellow().bold()
    );
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        std::fs::remove_dir_all(&downloads_dir)?;
        println!("  {} All cached downloads deleted.", "✓".green().bold());
    } else {
        println!("  {} Cancelled.", "ℹ".blue().bold());
    }

    Ok(())
}
