use anyhow::{Result, bail, Context};
use colored::*;

const GITHUB_REPO: &str = "amjiddader/ustore";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run() -> Result<()> {
    // Block running as root
    if std::env::var("USER").unwrap_or_default() == "root"
        && std::env::var("SUDO_USER").ok().filter(|s| !s.is_empty()).is_none()
    {
        bail!(
            "{}",
            "Please use ustore as non-root user to install and update apps.".red().bold()
        );
    }

    println!("{} Checking for uStore updates...", "→".cyan().bold());

    // Fetch latest release from GitHub API
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("ustore")
        .build()
        .context("Failed to create HTTP client")?;

    let resp = client
        .get(&url)
        .send()
        .context("Failed to reach GitHub API")?;

    if !resp.status().is_success() {
        bail!("GitHub API returned status: {}", resp.status());
    }

    let body: serde_json::Value = resp.json().context("Failed to parse GitHub response")?;

    let latest_tag = body["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v');

    if latest_tag.is_empty() {
        bail!("Could not determine latest version from GitHub.");
    }

    println!(
        "  {} Current: v{}  |  Latest: v{}",
        "ℹ".blue().bold(),
        CURRENT_VERSION,
        latest_tag
    );

    if latest_tag == CURRENT_VERSION {
        println!(
            "{} uStore is already up to date (v{}).",
            "✓".green().bold(),
            CURRENT_VERSION
        );
        return Ok(());
    }

    // Find the .deb asset in the release
    let assets = body["assets"].as_array();
    let deb_asset = assets
        .and_then(|a: &Vec<serde_json::Value>| {
            a.iter().find(|asset| {
                asset["name"]
                    .as_str()
                    .is_some_and(|n: &str| n.ends_with("_amd64.deb") || n.ends_with(".deb"))
            })
        });

    let download_url = match deb_asset {
        Some(asset) => asset["browser_download_url"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        None => bail!("No .deb asset found in latest release."),
    };

    if download_url.is_empty() {
        bail!("Empty download URL for .deb asset.");
    }

    println!(
        "{} Downloading uStore v{}...",
        "→".cyan().bold(),
        latest_tag
    );

    // Download to temp
    let tmp_dir = std::env::temp_dir().join("ustore_update");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).context("Failed to create temp dir")?;

    let deb_path = tmp_dir.join(format!("ustore_{}_amd64.deb", latest_tag));

    let status = std::process::Command::new("aria2c")
        .args([
            "--max-connection-per-server=16",
            "--split=16",
            "--allow-overwrite=true",
            "-d", tmp_dir.to_str().unwrap(),
            "-o", deb_path.file_name().unwrap().to_str().unwrap(),
            &download_url,
        ])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to run aria2c")?;

    if !status.success() {
        bail!("Download failed.");
    }

    println!("{} Installing update...", "→".cyan().bold());

    let install = std::process::Command::new("sudo")
        .args(["dpkg", "-i", deb_path.to_str().unwrap()])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to run dpkg")?;

    if !install.success() {
        // Try fixing deps
        let _ = std::process::Command::new("sudo")
            .args(["apt-get", "install", "-f", "-y"])
            .status();
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp_dir);

    println!(
        "{} uStore updated to v{}!",
        "✓".green().bold(),
        latest_tag
    );

    Ok(())
}
