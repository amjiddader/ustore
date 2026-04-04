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
| `ustore update` | Fetch latest package registry |
| `ustore search <query>` | Search for apps |
| `ustore install <package>` | Install an app |
| `ustore remove <package>` | Remove an app |
| `ustore list` | List installed apps with upgrade status |
| `ustore info <package>` | Show detailed app info |
| `ustore upgrade [package]` | Upgrade one or all apps |
| `ustore categories` | List app categories |

## Troubleshooting

### `aria2c: command not found`
```bash
sudo apt install aria2
```

### `Permission denied`
uStore needs `sudo` for install/remove (dpkg requires it).

### `No cached registry found`
Run `ustore update` first to fetch the registry.

## Architecture

_Detailed architecture docs coming soon._

## File Locations

| Path | Purpose |
|------|---------|
| `/usr/bin/ustore` | CLI binary |
| `~/.config/ustore/config.toml` | User config |
| `~/.cache/ustore/source.json` | Cached registry |
| `~/.local/share/ustore/ustore.db` | Installed apps database |
| `/opt/ustore/` | tar.gz app installs |