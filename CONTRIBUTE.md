# 🤝 Contributing to uStore

Want to add your favorite app to uStore? Follow this guide.

---

## Adding a New App

### 1. Fork & Clone

```bash
git clone https://github.com/YOUR_USERNAME/ustore.git
cd ustore
```

### 2. Add Your App to `source.json`

Add a new entry inside the `"packages"` array. Use this template:

```json
{
  "id": "app-name",
  "name": "App Name",
  "description": "Short description of what the app does.",
  "category": "CATEGORY_ID",
  "icon": "https://example.com/icon.svg",
  "website": "https://example.com",
  "publisher": "Publisher Name",
  "license": "mit | gpl-3.0 | proprietary",
  "tags": ["keyword1", "keyword2", "keyword3"],
  "alt_names": ["alias1", "shortname"],
  "verified": false,
  "added_date": "YYYY-MM-DD",
  "updated_date": "YYYY-MM-DD",
  "popularity": 3,
  "variants": [
    {
      "version": "1.0.0",
      "arch": "amd64",
      "type": "deb",
      "url": "https://example.com/download/app_amd64.deb",
      "sha256": "",
      "size_mb": 50,
      "min_ubuntu": "20.04"
    }
  ],
  "dependencies": [],
  "dpkg_name": "app-name",
  "binary_name": "app-name",
  "file_name": "app-name_1.0.0_amd64.deb",
  "desktop_entry": "app-name.desktop",
  "install_args": "",
  "post_script": "",
  "post_install": [],
  "pre_remove": [],
  "auto_update": false,
  "notes": "Any additional notes."
}
```

### 3. Field Reference

| Field | Required | Description |
|-------|----------|-------------|
| `id` | ✅ | Unique lowercase ID with hyphens (e.g. `google-chrome`) |
| `name` | ✅ | Display name |
| `description` | ✅ | One-line description |
| `category` | ✅ | One of: `browsers`, `development`, `media`, `utilities`, `communication`, `gaming`, `security`, `office` |
| `icon` | ✅ | URL to app icon (SVG or PNG) |
| `website` | ✅ | Official website |
| `publisher` | ✅ | Developer/company name |
| `license` | ✅ | SPDX identifier or `proprietary` |
| `tags` | ✅ | Array of search keywords |
| `verified` | ✅ | Set to `false` (maintainers verify) |
| `popularity` | ✅ | 1-5 (set 3 for new apps) |
| `variants` | ✅ | At least one variant (see below) |
| `dpkg_name` | ⬜ | The `dpkg` package name (for `.deb` type) |
| `binary_name` | ⬜ | The executable name |
| `file_name` | ✅ | Download filename for caching (e.g. `app-name_1.0.0_amd64.deb`) |
| `alt_names` | ⬜ | Search aliases (e.g. `["chrome", "gc"]`) |
| `install_args` | ⬜ | Arguments for `.run` installer (e.g. `"-i"` for silent) |
| `post_script` | ⬜ | URL to post-install bash script |
| `notes` | ⬜ | Additional info |

### Variant Fields

| Field | Required | Values |
|-------|----------|--------|
| `version` | ✅ | Real version number (NOT `latest`) |
| `arch` | ✅ | `amd64`, `arm64`, `armhf`, `i386`, `all` |
| `type` | ✅ | `deb`, `tar.gz`, `tar.xz`, `binary`, `appimage`, `run` |
| `url` | ✅ | Direct download URL (HTTPS only) |
| `sha256` | ⬜ | SHA256 checksum (leave `""` if unknown) |
| `size_mb` | ✅ | Approximate size in MB |
| `min_ubuntu` | ⬜ | Minimum Ubuntu version (e.g. `"20.04"`) |

### 4. For `.tar.gz` Apps (No Installer)

If your app is a `.tar.gz` and doesn't create a desktop entry, also add:

- `config/<app-id>.desktop` — Desktop entry file
- `config/<app-id>.png` — App icon (128x128)

### 5. For `.run` Apps

For `.run` installers (e.g. DaVinci Resolve):
- Set `"type": "run"` in variants
- Set `"install_args": "-i"` if the installer supports silent mode
- Add system dependencies in the `"dependencies"` array
- Optionally set `"post_script"` to a URL of a bash script to run after install

### 6. Update `APPS.md`

Add your app to the numbered list in `APPS.md`.

### 7. Validate

Make sure `source.json` is valid JSON:

```bash
python3 -m json.tool source.json > /dev/null
```

### 8. Submit PR

```bash
git checkout -b add-app-name
git add source.json APPS.md
git commit -m "feat: add app-name"
git push origin add-app-name
```

Then open a Pull Request with:
- **Title:** `Add <App Name>`
- **Description:** What the app does, why it should be in uStore, and the download source

---

## Rules

1. **HTTPS only** — All download URLs must use HTTPS
2. **Official sources** — Link to official download pages, not mirrors
3. **No malware** — All apps are reviewed before merging
4. **Real versions** — Use actual version numbers, not `latest`
5. **One app per PR** — Makes review easier
6. **Validate JSON** — Broken JSON = instant reject

## Questions?

Open an [issue](https://github.com/amjiddader/ustore/issues) if you need help.