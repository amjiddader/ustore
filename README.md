<div align="center">

# 🏪 uStore

**A modern app store for Ubuntu — No Snap. No bloat.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/amjiddader/ustore?include_prereleases)](https://github.com/amjiddader/ustore/releases)
[![Platform](https://img.shields.io/badge/platform-Ubuntu%2020.04+-orange)](https://ubuntu.com)

Install apps on Ubuntu the way it should be — fast, clean, and simple.

</div>

---

## 🤔 Why uStore?

Snap is slow, bloated, and creates disk waste. uStore is the alternative.

| | 🏪 uStore | 📦 Snap |
|---|---|---|
| **Install speed** | ⚡ Fast (aria2c, 16 connections) | 🐌 Slow (single connection) |
| **Disk usage** | 💾 Minimal (native .deb) | 💽 Heavy (sandboxed copies) |
| **Startup time** | 🚀 Instant (native binary) | 🐢 Delayed (snap mount) |
| **Auto-mount loops** | ❌ None | ♾️ Creates `/dev/loop` spam |
| **App size** | 📦 Actual package size | 📦 2-3x larger (bundled deps) |
| **Background daemon** | ❌ None | ✅ `snapd` always running |
| **CLI binary** | 1.7 MB | ~200 MB (snapd + deps) |
| **Updates** | 🎯 You choose when | 🔄 Forced auto-updates |
| **Registry** | 📋 Open source JSON | 🔒 Canonical-controlled |

## 🚀 Install

One command:

```bash
curl -sL https://raw.githubusercontent.com/amjiddader/ustore/main/install.sh | bash
```

Or manually:

```bash
wget https://github.com/amjiddader/ustore/releases/download/beta/ustore_1.0.0_amd64.deb
sudo dpkg -i ustore_1.0.0_amd64.deb
sudo apt-get install -f -y
```

## 📖 Usage

```bash
# Fetch the latest app registry
ustore update

# Search for apps
ustore search browser
ustore search discord

# Install an app
ustore install google-chrome
ustore install discord
ustore install brave-browser

# See what's installed
ustore list

# Get detailed info
ustore info google-chrome

# Upgrade all apps
ustore upgrade

# Upgrade a specific app
ustore upgrade discord

# Remove an app
ustore remove discord

# Browse categories
ustore categories
```

## 📦 Available Apps

| App | Category | Type |
|-----|----------|------|
| Google Chrome | 🌐 Browsers | .deb |
| Brave Browser | 🌐 Browsers | .deb |
| Discord | 💬 Communication | .deb |

> More apps coming soon! [Add your favorite app →](CONTRIBUTE.md)

## 🏗️ How It Works

```
┌──────────────────────────────────────────┐
│  GitHub Repo (source.json)               │
│  Curated registry of apps + URLs         │
└──────────────┬───────────────────────────┘
               │ ustore update
               ▼
┌──────────────────────────────────────────┐
│  Local Cache (~/.cache/ustore/)          │
│  Cached registry + smart TTL (24h)       │
└──────────────┬───────────────────────────┘
               │ ustore install <app>
               ▼
┌──────────────────────────────────────────┐
│  aria2c Download (16 connections)        │
│  Fast parallel download from source URL  │
└──────────────┬───────────────────────────┘
               │ dpkg -i / extract
               ▼
┌──────────────────────────────────────────┐
│  Installed & Tracked (SQLite DB)         │
│  Real version detection via dpkg         │
└──────────────────────────────────────────┘
```

## 🔄 Auto Version Updates

A GitHub Action runs every 12 hours to:
1. Download each `.deb` package
2. Extract the real version via `dpkg-deb`
3. Update `source.json` with current versions
4. Push changes automatically

This means `ustore list` always knows when upgrades are available.

## 🤝 Contributing

Want to add an app to uStore? See [CONTRIBUTE.md](CONTRIBUTE.md) for the full guide.

**Quick version:** Edit `source.json`, add your app entry, submit a PR.

## 📄 License

MIT — do whatever you want with it.

---

<div align="center">

**Built with 🦀 Rust • Downloads with aria2c • No Snap Required**

</div>
