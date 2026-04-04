use anyhow::{bail, Result};
use colored::Colorize;
use std::path::Path;
use std::process::{Command, Output};

fn run_cmd(cmd: &str, args: &[&str]) -> Result<Output> {
    let output = Command::new(cmd).args(args).output()?;
    Ok(output)
}

pub fn install_deb(deb_path: &Path, name: &str, version: &str) -> Result<()> {
    let path_str = deb_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;

    println!();
    println!("  {}", "┌─────────────────────────────────────────┐".cyan());
    println!(
        "  {} {} {} {} {}",
        "│".cyan(),
        "📦 Installing".green().bold(),
        name.white().bold(),
        format!("v{}", version).yellow(),
        "│".cyan()
    );
    println!("  {}", "└─────────────────────────────────────────┘".cyan());
    println!();

    let output = run_cmd("sudo", &["dpkg", "-i", path_str])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("dependency problems") {
            println!("{}", "Fixing dependencies...".yellow());
            let fix = run_cmd("sudo", &["apt-get", "install", "-f", "-y"])?;
            if !fix.status.success() {
                bail!(
                    "{}",
                    format!(
                        "Failed to fix dependencies: {}",
                        String::from_utf8_lossy(&fix.stderr)
                    )
                    .red()
                );
            }
            println!("{}", "Dependencies fixed.".green());
        } else {
            bail!("{}", format!("Install failed: {}", stderr).red());
        }
    }

    println!("{}", "Installation complete.".green().bold());
    Ok(())
}

pub fn remove_deb(dpkg_name: &str) -> Result<()> {
    println!("{} {}", "Removing".red().bold(), dpkg_name);

    let output = run_cmd("sudo", &["dpkg", "-r", dpkg_name])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{}", format!("Removal failed: {}", stderr).red());
    }

    println!("{}", "Removal complete.".green().bold());
    Ok(())
}

pub fn is_installed(dpkg_name: &str) -> bool {
    run_cmd("dpkg", &["-s", dpkg_name])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_installed_version(dpkg_name: &str) -> Result<Option<String>> {
    let output = run_cmd("dpkg", &["-s", dpkg_name])?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(version) = line.strip_prefix("Version: ") {
            return Ok(Some(version.trim().to_string()));
        }
    }
    Ok(None)
}
