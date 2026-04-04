mod cmd;
mod config;
mod downloader;
mod installer;
mod registry;
mod store;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ustore", version, about = "🏪 uStore — A modern app store for Ubuntu. No Snap. No bloat.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Self-update uStore to the latest version from GitHub
    Update,
    /// Fetch latest package registry from GitHub
    Refresh,
    /// Search for packages by name or keyword
    Search { query: String },
    /// Install a package
    Install { package: String },
    /// Remove an installed package
    Remove { package: String },
    /// List all installed packages
    List,
    /// Show detailed info about a package
    Info { package: String },
    /// List available categories
    Categories,
    /// Upgrade installed packages to latest version
    Upgrade {
        /// Package to upgrade (upgrades all if omitted)
        package: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Update) => cmd::selfupdate::run()?,
        Some(Commands::Refresh) => cmd::update::run()?,
        Some(Commands::Search { query }) => cmd::search::run(&query)?,
        Some(Commands::Install { package }) => cmd::install::run(&package)?,
        Some(Commands::Remove { package }) => cmd::remove::run(&package)?,
        Some(Commands::List) => cmd::list::run()?,
        Some(Commands::Info { package }) => cmd::info::run(&package)?,
        Some(Commands::Categories) => cmd::categories::run()?,
        Some(Commands::Upgrade { package }) => cmd::upgrade::run(package.as_deref())?,
        None => {
            println!("🏪 uStore — A modern app store for Ubuntu.");
            println!("Run `ustore --help` for usage information.");
        }
    }

    Ok(())
}
