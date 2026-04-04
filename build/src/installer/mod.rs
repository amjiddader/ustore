use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::{Command, Output};

fn run_cmd(cmd: &str, args: &[&str]) -> Result<Output> {
    let output = Command::new(cmd).args(args).output()?;
    Ok(output)
}

pub fn install_dependencies(deps: &[String]) -> Result<()> {
    if deps.is_empty() {
        return Ok(());
    }

    println!(
        "  {} Installing system dependencies...",
        "→".cyan().bold()
    );

    // Update package list first
    let _ = run_cmd("sudo", &["apt-get", "update", "-qq"]);

    // Install with --ignore-missing so unavailable packages are skipped
    let mut args = vec![
        "apt-get", "install", "-y", "--ignore-missing",
    ];
    let dep_refs: Vec<&str> = deps.iter().map(|s| s.as_str()).collect();
    args.extend(&dep_refs);

    let output = Command::new("sudo")
        .args(&args)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    if output.success() {
        println!(
            "  {} Dependencies installed.",
            "✓".green().bold()
        );
    } else {
        println!(
            "  {} Some dependencies may not be available, continuing anyway...",
            "⚠".yellow().bold()
        );
    }

    Ok(())
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

pub fn install_appimage(
    appimage_path: &Path,
    app_id: &str,
    binary_name: &str,
    name: &str,
    version: &str,
) -> Result<()> {
    let install_dir = std::path::PathBuf::from("/opt/ustore").join(app_id);
    let target = install_dir.join(binary_name);
    let symlink = std::path::PathBuf::from("/usr/local/bin").join(binary_name);

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

    run_cmd("sudo", &["mkdir", "-p", install_dir.to_str().unwrap()])?;
    run_cmd("sudo", &[
        "cp",
        appimage_path.to_str().unwrap(),
        target.to_str().unwrap(),
    ])?;
    run_cmd("sudo", &["chmod", "+x", target.to_str().unwrap()])?;

    // Remove old symlink if exists, then create new one
    let _ = run_cmd("sudo", &["rm", "-f", symlink.to_str().unwrap()]);
    run_cmd("sudo", &[
        "ln", "-s",
        target.to_str().unwrap(),
        symlink.to_str().unwrap(),
    ])?;

    // Install desktop entry from config/ repo
    let config_base = crate::config::load_config()
        .map(|c| c.config_base_url)
        .unwrap_or_default();
    if !config_base.is_empty() {
        let _ = install_desktop_entry(app_id, &config_base);
    }

    println!("{}", "Installation complete.".green().bold());
    Ok(())
}

pub fn remove_appimage(app_id: &str, binary_name: &str) -> Result<()> {
    let install_dir = std::path::PathBuf::from("/opt/ustore").join(app_id);
    let symlink = std::path::PathBuf::from("/usr/local/bin").join(binary_name);

    println!("{} {}", "Removing".red().bold(), app_id);

    let _ = run_cmd("sudo", &["rm", "-f", symlink.to_str().unwrap()]);
    let output = run_cmd("sudo", &["rm", "-rf", install_dir.to_str().unwrap()])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{}", format!("Removal failed: {}", stderr).red());
    }

    // Remove desktop entry
    let _ = remove_desktop_entry(app_id);

    println!("{}", "Removal complete.".green().bold());
    Ok(())
}

pub fn is_appimage_installed(app_id: &str) -> bool {
    let install_dir = std::path::PathBuf::from("/opt/ustore").join(app_id);
    install_dir.exists()
}

pub fn install_tarball(
    tarball_path: &Path,
    app_id: &str,
    binary_name: &str,
    name: &str,
    version: &str,
) -> Result<()> {
    let install_dir = std::path::PathBuf::from("/opt/ustore").join(app_id);
    let symlink = std::path::PathBuf::from("/usr/local/bin").join(binary_name);

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

    // Extract to a temp directory first
    let tmp_extract = crate::config::cache_dir().join("extract_tmp");
    let _ = std::fs::remove_dir_all(&tmp_extract);
    std::fs::create_dir_all(&tmp_extract).context("Failed to create extract temp dir")?;

    println!("  {} Extracting archive...", "→".cyan().bold());
    let output = run_cmd(
        "tar",
        &["-xf", tarball_path.to_str().unwrap(), "-C", tmp_extract.to_str().unwrap()],
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Extract failed: {}", stderr);
    }

    // Find the extracted directory (usually the only folder inside)
    let entries: Vec<_> = std::fs::read_dir(&tmp_extract)?
        .filter_map(|e| e.ok())
        .collect();

    let source_dir = if entries.len() == 1 && entries[0].path().is_dir() {
        entries[0].path()
    } else {
        tmp_extract.clone()
    };

    // Move to /opt/ustore/<id>/
    let _ = run_cmd("sudo", &["rm", "-rf", install_dir.to_str().unwrap()]);
    run_cmd("sudo", &["mkdir", "-p", "/opt/ustore"])?;
    run_cmd("sudo", &[
        "mv",
        source_dir.to_str().unwrap(),
        install_dir.to_str().unwrap(),
    ])?;

    // Find and symlink the binary — check root, then bin/ subfolder
    let binary_path = install_dir.join(binary_name);
    let binary_bin_path = install_dir.join("bin").join(binary_name);
    let found_binary = if binary_path.exists() {
        Some(binary_path)
    } else if binary_bin_path.exists() {
        Some(binary_bin_path)
    } else {
        None
    };

    if let Some(bin) = found_binary {
        run_cmd("sudo", &["chmod", "+x", bin.to_str().unwrap()])?;
        let _ = run_cmd("sudo", &["rm", "-f", symlink.to_str().unwrap()]);
        run_cmd("sudo", &[
            "ln", "-s",
            bin.to_str().unwrap(),
            symlink.to_str().unwrap(),
        ])?;
    }

    // Install desktop entry from config/ repo
    let config_base = crate::config::load_config()
        .map(|c| c.config_base_url)
        .unwrap_or_default();
    if !config_base.is_empty() {
        let _ = install_desktop_entry(app_id, &config_base);
    }

    // Cleanup extract temp
    let _ = std::fs::remove_dir_all(&tmp_extract);

    println!("{}", "Installation complete.".green().bold());
    Ok(())
}

pub fn remove_tarball(app_id: &str, binary_name: &str) -> Result<()> {
    remove_appimage(app_id, binary_name)
}

pub fn install_run(
    run_path: &Path,
    name: &str,
    version: &str,
    install_args: Option<&str>,
) -> Result<()> {
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

    let path_str = run_path.to_str().unwrap();

    // Make executable
    run_cmd("chmod", &["+x", path_str])?;

    if let Some(args) = install_args {
        // Auto-install with provided args (e.g. "-i" for DaVinci)
        println!("  {} Running installer with args: {}", "→".cyan().bold(), args);

        let shell_cmd = format!("sudo {} {}", path_str, args);
        let status = Command::new("bash")
            .args(["-c", &shell_cmd])
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()?;

        if !status.success() {
            bail!("Installer exited with error.");
        }
    } else {
        // Interactive mode — ask user to run manually
        println!("  {}", "┌─────────────────────────────────────────────────┐".yellow());
        println!("  {}  {} {}", "│".yellow(), "⚠".yellow().bold(), "Installer needs your attention!".yellow().bold());
        println!("  {}", "│".yellow());
        println!("  {}  Run this in another terminal:", "│".yellow());
        println!("  {}  {}", "│".yellow(), format!("sudo {}", path_str).white().bold());
        println!("  {}", "│".yellow());
        println!("  {}  Press {} when installation is complete.", "│".yellow(), "ENTER".green().bold());
        println!("  {}", "└─────────────────────────────────────────────────┘".yellow());
        println!();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    }

    println!("{}", "Installation complete.".green().bold());
    Ok(())
}

pub fn run_post_script(post_script_url: &str) -> Result<()> {
    if post_script_url.is_empty() {
        return Ok(());
    }

    println!("  {} Running post-install script...", "→".cyan().bold());

    let tmp_dir = crate::config::cache_dir().join("post_script_tmp");
    std::fs::create_dir_all(&tmp_dir).context("Failed to create temp dir")?;

    let script_path = tmp_dir.join("post_install.sh");
    download_raw(post_script_url, &script_path)?;

    run_cmd("chmod", &["+x", script_path.to_str().unwrap()])?;

    let status = Command::new("sudo")
        .arg("bash")
        .arg(script_path.to_str().unwrap())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp_dir);

    if status.success() {
        println!("  {} Post-install script completed.", "✓".green().bold());
    } else {
        println!(
            "  {} Post-install script exited with warnings.",
            "⚠".yellow().bold()
        );
    }

    Ok(())
}

pub fn install_desktop_entry(app_id: &str, config_base_url: &str) -> Result<()> {
    // Remove any existing desktop entry first to prevent duplicates
    let _ = remove_desktop_entry(app_id);

    let desktop_url = format!("{}/{}.desktop", config_base_url, app_id);
    let icon_url = format!("{}/{}.png", config_base_url, app_id);

    let tmp_dir = crate::config::cache_dir().join("desktop_tmp");
    std::fs::create_dir_all(&tmp_dir).context("Failed to create temp dir")?;

    let desktop_tmp = tmp_dir.join(format!("{}.desktop", app_id));
    let icon_tmp = tmp_dir.join(format!("{}.png", app_id));

    let desktop_dest = format!("/usr/share/applications/ustore-{}.desktop", app_id);
    let icon_dest = format!(
        "/usr/share/icons/hicolor/128x128/apps/ustore-{}.png",
        app_id
    );

    // Download .desktop file
    println!("  {} Downloading desktop entry...", "→".cyan().bold());
    let desktop_ok = download_raw(&desktop_url, &desktop_tmp);

    if let Ok(()) = desktop_ok {
        run_cmd("sudo", &["mkdir", "-p", "/usr/share/applications"])?;
        run_cmd("sudo", &["cp", desktop_tmp.to_str().unwrap(), &desktop_dest])?;
    } else {
        println!(
            "  {} Desktop entry not found, skipping.",
            "⚠".yellow().bold()
        );
    }

    // Download icon
    println!("  {} Downloading icon...", "→".cyan().bold());
    let icon_ok = download_raw(&icon_url, &icon_tmp);

    if let Ok(()) = icon_ok {
        run_cmd(
            "sudo",
            &["mkdir", "-p", "/usr/share/icons/hicolor/128x128/apps"],
        )?;
        run_cmd("sudo", &["cp", icon_tmp.to_str().unwrap(), &icon_dest])?;
    } else {
        println!("  {} Icon not found, skipping.", "⚠".yellow().bold());
    }

    // Refresh system menus
    let _ = run_cmd("sudo", &["update-desktop-database", "/usr/share/applications"]);
    let _ = run_cmd("sudo", &["gtk-update-icon-cache", "/usr/share/icons/hicolor"]);

    // Cleanup temp
    let _ = std::fs::remove_dir_all(&tmp_dir);

    println!("  {} Desktop entry installed.", "✓".green().bold());
    Ok(())
}

pub fn remove_desktop_entry(app_id: &str) -> Result<()> {
    let desktop_path = format!("/usr/share/applications/ustore-{}.desktop", app_id);
    let icon_path = format!(
        "/usr/share/icons/hicolor/128x128/apps/ustore-{}.png",
        app_id
    );

    let _ = run_cmd("sudo", &["rm", "-f", &desktop_path]);
    let _ = run_cmd("sudo", &["rm", "-f", &icon_path]);
    let _ = run_cmd("sudo", &["update-desktop-database", "/usr/share/applications"]);

    Ok(())
}

fn download_raw(url: &str, dest: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["-sL", "-o", dest.to_str().unwrap(), "--fail", url])
        .status()
        .context("Failed to run curl")?;

    if !status.success() {
        bail!("Download failed: {}", url);
    }
    Ok(())
}
