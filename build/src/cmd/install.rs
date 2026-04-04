use anyhow::{Result, bail};
use colored::*;

use crate::config;
use crate::registry::fetch;
use crate::registry::search::find_package;
use crate::downloader;
use crate::installer;
use crate::store;

pub fn run(id: &str, force: bool) -> Result<()> {
    // Only allow: sudo ustore ... (UID 0 with SUDO_USER set)
    let is_root = std::process::Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
        .unwrap_or(false);
    let has_sudo_user = std::env::var("SUDO_USER").ok().filter(|s| !s.is_empty()).is_some();

    if !is_root || !has_sudo_user {
        bail!(
            "{}\n  {}",
            "This command requires sudo.".red().bold(),
            "Usage: sudo ustore install <package>".yellow()
        );
    }

    let cfg = config::load_config()?;

    let registry = fetch::get_registry(&cfg)?;

    let package = match find_package(&registry, id) {
        Some(p) => p,
        None => bail!("Package '{}' not found in registry.", id),
    };

    let dpkg_name = package.dpkg_name.as_deref().unwrap_or(id);
    let binary_name = package.binary_name.as_deref().unwrap_or(id);

    let arch = std::process::Command::new("dpkg")
        .arg("--print-architecture")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "amd64".to_string());

    let variant = package
        .variants
        .iter()
        .find(|v| v.arch == arch || v.arch == "all")
        .or_else(|| package.variants.first());

    let variant = match variant {
        Some(v) => v,
        None => bail!("No compatible variant found for package '{}'.", id),
    };

    // Check if already installed (deb via dpkg, appimage/tarball via /opt/ustore/)
    let already_installed = match variant.pkg_type.as_str() {
        "appimage" | "tar.gz" | "tar.xz" => store::is_tracked(id)? && installer::is_appimage_installed(id),
        "run" => store::is_tracked(id)?,
        _ => store::is_tracked(id)? && installer::is_installed(dpkg_name),
    };

    if already_installed && !force {
        println!("{} {} is already installed. Use {} to reinstall.", "✓".green().bold(), package.name.bold(), "--force".yellow());
        return Ok(());
    }

    if already_installed && force {
        println!("{} Force reinstalling {}...", "→".cyan().bold(), package.name.bold());
    }

    // Install system dependencies if any
    installer::install_dependencies(&package.dependencies)?;

    let filename = &package.file_name;

    println!("  {} Checking for cached download...", "→".cyan().bold());
    let cache_path = crate::config::cache_dir().join("downloads").join(filename);

    let ext = match variant.pkg_type.as_str() {
        "appimage" => "AppImage",
        "tar.gz" => "tar.gz",
        "tar.xz" => "tar.xz",
        "run" => {
            if variant.url.ends_with(".zip") { "zip" } else { "run" }
        }
        _ => "deb",
    };

    let mut file_path = if cache_path.exists() {
        let aria2_path = std::path::PathBuf::from(format!("{}.aria2", cache_path.display()));
        if aria2_path.exists() {
            // Partial download detected
            println!(
                "  {} Partial download detected for {}",
                "⚠".yellow().bold(),
                filename.bold()
            );
            println!();
            println!("  What would you like to do?");
            println!("    {} Continue download", "[Y]".green().bold());
            println!("    {} Re-download from scratch", "[D]".yellow().bold());
            println!("    {} Try to install as-is", "[T]".red().bold());
            print!("  > ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let choice = input.trim().to_lowercase();

            match choice.as_str() {
                "d" => {
                    let _ = std::fs::remove_file(&cache_path);
                    let _ = std::fs::remove_file(&aria2_path);
                    println!(
                        "  {} Re-downloading {} v{} ({})...",
                        "→".cyan().bold(),
                        "Downloading".bold(),
                        variant.version,
                        variant.arch
                    );
                    downloader::download_to_cache(&variant.url, filename)?
                }
                "t" => {
                    println!(
                        "  {} Attempting install with partial file...",
                        "⚠".yellow().bold()
                    );
                    cache_path
                }
                _ => {
                    // Default: continue download (aria2c resumes automatically)
                    println!(
                        "  {} Resuming download...",
                        "→".cyan().bold()
                    );
                    downloader::download_to_cache(&variant.url, filename)?
                }
            }
        } else {
            println!(
                "  {} {} already downloaded, skipping.",
                "✓".green().bold(),
                filename.bold()
            );
            cache_path
        }
    } else {
        println!(
            "  {} {} v{} ({})...",
            "→".cyan().bold(),
            "Downloading".bold(),
            variant.version,
            variant.arch
        );
        downloader::download_to_cache(&variant.url, filename)?
    };

    // If .run package was downloaded as .zip, extract the .run file
    if variant.pkg_type == "run" && ext == "zip" {
        println!("{} Extracting .run from archive...", "→".cyan().bold());
        let extract_dir = file_path.parent().unwrap().join(format!("{}_extract", id));
        let _ = std::fs::remove_dir_all(&extract_dir);
        std::fs::create_dir_all(&extract_dir)?;

        let status = std::process::Command::new("unzip")
            .args(["-o", file_path.to_str().unwrap(), "-d", extract_dir.to_str().unwrap()])
            .stdout(std::process::Stdio::null())
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to extract zip archive.");
        }

        // Find the .run file inside
        let run_file = std::fs::read_dir(&extract_dir)?
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().is_some_and(|ext| ext == "run"))
            .map(|e| e.path());

        match run_file {
            Some(rf) => file_path = rf,
            None => anyhow::bail!("No .run file found inside zip archive."),
        }
    }

    println!("{}  {}...", "→".cyan().bold(), "Installing".bold());

    let install_result: Result<String> = (|| {
        let ver = match variant.pkg_type.as_str() {
            "appimage" => {
                installer::install_appimage(&file_path, id, binary_name, &package.name, &variant.version)?;
                variant.version.clone()
            }
            "tar.gz" | "tar.xz" => {
                installer::install_tarball(&file_path, id, binary_name, &package.name, &variant.version)?;
                variant.version.clone()
            }
            "run" => {
                installer::install_run(
                    &file_path,
                    &package.name,
                    &variant.version,
                    package.install_args.as_deref(),
                )?;
                variant.version.clone()
            }
            _ => {
                installer::install_deb(&file_path, &package.name, &variant.version)?;
                installer::get_installed_version(dpkg_name)?
                    .unwrap_or_else(|| variant.version.clone())
            }
        };
        Ok(ver)
    })();

    match install_result {
        Ok(real_version) => {
            store::record_install(
                id,
                &package.name,
                &real_version,
                &variant.pkg_type,
                package.dpkg_name.as_deref(),
                package.binary_name.as_deref(),
            )?;

            // Run post-install script if defined
            if let Some(ref script_url) = package.post_script {
                if !script_url.is_empty() {
                    installer::run_post_script(script_url)?;
                }
            }

            println!("{} Cleaning up downloads...", "→".cyan().bold());
            let _ = downloader::cleanup_cache();

            println!(
                "{} {} v{} installed successfully!",
                "✓".green().bold(),
                package.name.green().bold(),
                real_version
            );
        }
        Err(e) => {
            println!();
            println!("  {}", "┌─────────────────────────────────────────────────────┐".red());
            println!("  {}  {} {}", "│".red(), "✗".red().bold(), "Installation Failed".red().bold());
            println!("  {}", "│".red());
            println!("  {}  {} {}", "│".red(), "Reason:".white().bold(), format!("{}", e).yellow());
            println!("  {}", "│".red());
            println!("  {}  {} {}", "│".red(), "Downloaded file:".white().bold(), file_path.display().to_string().cyan());
            println!("  {}  You can try to install this manually.", "│".red());
            println!("  {}", "│".red());
            println!("  {}  Once done, clean up with:", "│".red());
            println!("  {}  {}", "│".red(), format!("rm {}", file_path.display()).white().bold());
            println!("  {}", "└─────────────────────────────────────────────────────┘".red());
            println!();
            return Err(e);
        }
    }

    Ok(())
}
