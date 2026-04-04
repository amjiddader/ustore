# 📖 uStore Wiki

> Full documentation coming soon.

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Commands](#commands)
- [Troubleshooting](#troubleshooting)
- [Architecture](#architecture)

---

## Installation

```bash
curl -sL https://raw.githubusercontent.com/amjiddader/ustore/main/install.sh | bash
```

## Configuration

Config file: `~/.config/ustore/config.toml`

```toml
registry_url = "https://raw.githubusercontent.com/amjiddader/ustore/main/source.json"
config_base_url = "https://raw.githubusercontent.com/amjiddader/ustore/main/config"
cache_ttl_hours = 24
install_dir = "/opt/ustore"
```

## Commands

| Command | Description |
|---------|-------------|
| `ustore update` | Self-update uStore to latest version from GitHub |
| `ustore refresh` | Fetch latest package registry |
| `ustore search <query>` | Search for apps |
| `ustore install <package>` | Install an app |
| `ustore remove <package>` | Remove an app |
| `ustore list` | List installed apps with upgrade status |
| `ustore info <package>` | Show detailed app info |
| `ustore upgrade [package]` | Upgrade one or all apps |
| `ustore categories` | List app categories |
| `ustore clear` | Delete cached download files |

## Troubleshooting

### `aria2c: command not found`
```bash
sudo apt install aria2
```

### `Permission denied`
uStore needs `sudo` for install/remove (dpkg requires it).

### `No cached registry found`
Run `ustore refresh` first to fetch the registry.

## Architecture

_Detailed architecture docs coming soon._

## File Locations

| Path | Purpose |
|------|---------|
| `/usr/bin/ustore` | CLI binary |
| `~/.config/ustore/config.toml` | User config |
| `~/.cache/ustore/source.json` | Cached registry |
| `~/.cache/ustore/downloads/` | Cached download files |
| `~/.local/share/ustore/ustore.db` | Installed apps database |
| `/opt/ustore/` | tar.gz app installs |

## Package Types

uStore supports multiple package formats:

| Type | Extension | Install Method |
|------|-----------|----------------|
| `deb` | `.deb` | `sudo dpkg -i` with auto dependency fix |
| `appimage` | `.AppImage` | Copy to `/opt/ustore/`, symlink to `/usr/local/bin/` |
| `tar.gz` | `.tar.gz` | Extract to `/opt/ustore/`, symlink binary |
| `tar.xz` | `.tar.xz` | Extract to `/opt/ustore/`, symlink binary |
| `run` | `.run` | Execute with install_args or prompt user |

## New Fields

### `install_args`
Arguments passed to `.run` installers for silent/automated install. Example: `"-i"` for DaVinci Resolve.

### `post_script`
URL to a bash script that runs after installation completes. Useful for firewall rules, system config, etc.

### `dependencies`
System packages installed via `sudo apt-get install -y --ignore-missing` before the main package download.

### `file_name`
Explicit download filename used for caching. This is the name the file is saved as in `~/.cache/ustore/downloads/`. If the file already exists, the download is skipped.

### `alt_names`
Alternative names and aliases for fuzzy search (e.g. `["chrome", "gc"]` for Google Chrome).