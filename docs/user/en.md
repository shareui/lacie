# Lacie User Documentation

A fast, lightweight, zero-dependency package manager for prebuilt binaries with explicit security, version isolation, and declarative manifests.

---

## Table of Contents

- [1. Overview & Concepts](#1-overview--concepts)
- [2. Installation & Quick Start](#2-installation--quick-start)
  - [Initial Setup (`init-lacie`)](#initial-setup-init-lacie)
  - [Configuring PATH](#configuring-path)
- [3. Package References](#3-package-references)
- [4. Command Reference](#4-command-reference)
  - [`add`](#add)
  - [`update`](#update)
  - [`switch`](#switch)
  - [`remove`](#remove)
  - [`freeze` / `unfreeze` / `frozen`](#freeze--unfreeze--frozen)
  - [`rename`](#rename)
  - [`source`](#source)
  - [`list`](#list)
  - [`info`](#info)
  - [`doctor`](#doctor)
  - [`config`](#config)
  - [`export` & `import`](#export--import)
  - [`validate`](#validate)
  - [`gen-toml`](#gen-toml)
  - [`completions`](#completions)
  - [`explain`](#explain)
  - [`suicide`](#suicide)
  - [Version & Help](#version--help)
- [5. Filtering Syntax](#5-filtering-syntax)
- [6. Configuration Reference](#6-configuration-reference)
- [7. Environment Variables](#7-environment-variables)

---

## 1. Overview & Concepts

Lacie installs standalone prebuilt executables without requiring root/administrator privileges.

Key principles:
- **Zero-trust verification:** Releases must provide checksums. Unknown manifests and lifecycle scripts prompt for approval before execution.
- **Version coexistence:** Different versions of the same package are stored side-by-side in `bin/{id}/{version}/`. Only one version is active in PATH at a time, but switching is instant.
- **Non-destructive upgrades:** Updating a package downloads the new version alongside the old one. If an update breaks, you can immediately switch back.
- **Symlink/Active execution:** On Unix systems, Lacie maintains symbolic links in `active/`. On Windows 11, the active binary is copied to `active/`.

---

## 2. Installation & Quick Start

### Initial Setup (`init-lacie`)

Initialize your local environment by running:

```bash
lacie init-lacie
```

This interactive setup guides you through 5 core settings:
1. **Language:** English (`en`), Russian (`ru`), or automatic system detection (`auto`).
2. **Colors:** Terminal palette (`terminal`), fixed 256-color (`always`), or disabled (`never`).
3. **Verify SHA256:** Enforce strict hash verification (`true`/`false`).
4. **Auto Switch:** Automatically activate the newest version after installation (`true`/`false`).
5. **Confirm:** Request confirmation prompts before operations (`true`/`false`).

### Configuring PATH

Lacie places active executables inside its `active/` directory:
- **Unix:** `~/lacie/active`
- **Windows:** `%LOCALAPPDATA%\lacie\active`

Add this directory to your system `PATH`:

#### Bash / Zsh (`~/.bashrc` or `~/.zshrc`):
```bash
export PATH="$HOME/lacie/active:$PATH"
```

#### Fish (`~/.config/fish/config.fish`):
```fish
fish_add_path ~/lacie/active
```

#### Windows (PowerShell):
```powershell
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:LOCALAPPDATA\lacie\active", "User")
```

---

## 3. Package References

Lacie supports several ways to identify a package:

| Format | Meaning | Example |
|---|---|---|
| `owner/repo` | Repository via `default_source` (default: `github`) | `lacie add shareui/nyano` |
| `source:owner/repo` | Explicit source plugin | `lacie add github:shareui/nyano` |
| `source:owner/repo#pkg` | Specific package from a multi-package workspace | `lacie add github:shareui/lacie#tool` |
| `source:owner/repo 1.2.0` | Specific version or tag | `lacie add github:shareui/nyano 1.0.0` |
| `https://.../archive.tar.gz` | Direct archive download via HTTP/HTTPS | `lacie add https://example.com/bin.tar.gz` |
| `/path/to/archive.tar.gz` | Local archive file (`.tar.gz`, `.tar.xz`, `.tar.zst`, `.zip`) | `lacie add ./pkg-v1.tar.gz` |
| `/path/to/lacie.toml` | Local manifest file | `lacie add ./lacie.toml` |

---

## 4. Command Reference

### `add`

Installs a package into the local environment.

```bash
lacie add <package-ref> [options]
```

**Options:**
- `-y`, `--yes`: Skip confirmation prompt and proceed with installation.
- `--show`: Open the interactive terminal file viewer to inspect the manifest and scripts before installation.
- `--list`: List available releases for the repository without downloading or installing.
- `--info`: View repository metadata and package details without installing.
- `--force`: Reinstall the package even if the target version already exists.
- `--set-hash <sha256>`: Provide an expected SHA256 checksum when the manifest lacks one.
- `--ignore-hash`: Skip SHA256 integrity verification (not recommended).
- `--max <n>`: Limit the number of search results when querying packages.

**Interactive File Viewer Controls (`--show`):**
- `n`: View next file.
- `p`: View previous file.
- `a`: Approve changes and continue installation.
- `d` / `q`: Deny and cancel installation.

---

### `update`

Checks for updates and upgrades installed packages.

```bash
lacie update [packages...] [options]
```

If no packages are specified, Lacie checks all installed packages.

**Options:**
- `-a`, `--all`: Explicitly update all installed packages.
- `-y`, `--yes`: Automatically install available updates without asking for confirmation.
- `-d`, `--dry-run`: Check for updates and display available versions without downloading.
- `-i`, `--ignore <package>`: Exclude a specific package from being updated.

---

### `switch`

Switches the active version of an installed package without redownloading.

```bash
lacie switch <package> <version>
```

Example:
```bash
lacie switch ripgrep 13.0.0
```

---

### `remove`

Uninstalls packages or cleans up inactive versions.

```bash
lacie remove <packages...> [options]
```

**Options:**
- `-y`, `--yes`: Skip confirmation prompt.
- `--unused`: Remove only cached, inactive versions; keeps the current active version.
- `--symlink-only`: Remove the command symlink from `active/` without deleting the binary files from disk.
- `--active-only`: Remove only the currently active version.
- `--no-scripts`: Skip running `preremove.rb` and `postremove.rb` lifecycle hooks.

---

### `freeze` / `unfreeze` / `frozen`

Prevents packages from being modified by `lacie update`.

```bash
# Freeze package at current version
lacie freeze <package>
lacie freeze ALL

# Unfreeze package to re-enable updates
lacie unfreeze <package>
lacie unfreeze ALL

# List all frozen packages
lacie frozen
```

---

### `rename`

Changes the executable alias in `active/` (PATH).

```bash
# Rename active executable command
lacie rename <package> <new-command-name>

# Reset command name back to original package name using DEFAULT
lacie rename <package> DEFAULT
```

Example:
```bash
lacie rename neovim nvim
lacie rename neovim DEFAULT
```

---

### `source`

Manages package sources and registry plugins.

```bash
# List all registered sources
lacie source list

# Register a new source plugin
lacie source add <name> <ref>

# Remove a third-party source plugin
lacie source remove <name>

# Update source plugins
lacie source update [name]
```

---

### `list`

Lists all installed packages, their active versions, and frozen status.

```bash
lacie list [filters...] [options]
```

**Options:**
- `--json`: Output packages as a structured JSON array.
- `[filters]`: Filter packages using filter criteria (see [Filtering Syntax](#5-filtering-syntax)).

---

### `info`

Displays metadata and version history for an installed package.

```bash
lacie info <package>
```

Shows: package identifier, active version, all installed versions, disk size, command name, author, links, and installation dates.

---

### `doctor`

Audits your Lacie setup and reports configuration issues.

```bash
# Check environment health
lacie doctor

# Automatically repair missing directories, plugin files, and PATH configuration
lacie doctor --fix
```

Verifies:
- Existence and permissions of the Lacie home directory.
- `PATH` variable integration.
- Integrity of symlinks and active binaries.
- Consistency of state databases (`packages.toml`, `sources.toml`, `seen.toml`, `scripts.toml`).
- Health of registered source plugins.

---

### `config`

Reads and modifies user configuration settings.

```bash
# Display all configuration settings and current values
lacie config list

# Get current value of a setting
lacie config get <key>

# Update a setting
lacie config set <key> <value>

# Reset a setting to its default value using the DEFAULT constant
lacie config set <key> DEFAULT
```

Example:
```bash
lacie config set language en
lacie config set confirm false
lacie config set colors DEFAULT
```

---

### `export` & `import`

Backs up and restores your installed packages.

```bash
# Export installed state to a file (default: lacie-lock.toml)
lacie export [file] [options]

# Export to stdout
lacie export -

# Restore state from a file
lacie import [file]

# Restore state from stdin
lacie import -
```

**Export Options:**
- `--all-versions`: Include all downloaded versions, not just active ones.
- `--with-config`: Include user configuration settings in the export file.

---

### `validate`

Checks a `lacie.toml` manifest file for syntax and structural correctness.

```bash
lacie validate [path/to/lacie.toml]
```

Verifies:
- Package IDs and table naming.
- Target triple validity and uniqueness.
- Presence of required metadata (`name`, `version`, binary targets).
- Local binary checksums when files exist on disk.

---

### `gen-toml`

Generates a starter `lacie.toml` template in the current working directory.

```bash
lacie gen-toml
```

---

### `completions`

Generates command autocompletion scripts for your shell.

```bash
lacie completions <shell>
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`.

Example:
```bash
# Fish shell
lacie completions fish > ~/.config/fish/completions/lacie.fish

# Bash shell
lacie completions bash >> ~/.bash_completion
```

---

### `explain`

Displays troubleshooting information and fixes for error and warning codes.

```bash
lacie explain <CODE>
```

Example:
```bash
lacie explain E001
lacie explain W003
```

---

### `suicide`

Completely uninstalls Lacie and cleans up all related files.

```bash
lacie suicide [options]
```

**Options:**
- `-y`, `--yes`: Confirm uninstallation without interactive prompts.

This operation:
1. Deletes all downloaded binaries, symlinks, and state databases.
2. Removes the `active/` directory from shell profile configuration files.
3. Removes the Lacie home directory.

---

### Version & Help

- `lacie`: Prints version, license, and repository URL.
- `lacie -v`, `lacie --version`: Prints version and help hint.
- `lacie --help`: Prints command line help.

---

## 5. Filtering Syntax

Commands like `lacie list` support expressive queries to filter packages:

### Syntax
- `key:value`: Field must match value.
- `not-key:value`: Inverted match (field must not equal value).
- `key:val1,val2`: Match any of the specified values (OR).
- `key:min..max`: Numeric or version range (inclusive).
- `key:min..`: Values greater than or equal to `min`.
- `key:..max`: Values less than or equal to `max`.

### Supported Filter Keys
- `name`: Package display name.
- `command`: Active binary executable name.
- `id`: Unique domain package ID (e.g. `com.example.tool`).
- `author`: Package author.
- `link`: Web link or repository URL.
- `source`: Source provider name (e.g. `github`).
- `storage`: Upstream storage path (`owner/repo`).
- `version`: Version string.
- `number`: Integer release version counter.
- `installed`: Installation date.
- `released`: Upstream release date.
- `hash`: SHA256 checksum of the binary.
- `active`: Filter by active state (`true`/`false`).
- `frozen`: Filter by frozen state (`true`/`false`).
- `unused`: Filter packages with inactive cached versions.

### Filter Examples
```bash
# Find packages by author
lacie list author:shareui

# Exclude packages from GitHub
lacie list not-source:github

# Find frozen packages
lacie list frozen:true

# Filter by version range
lacie list version:1.0.0..2.0.0
```

---

## 6. Configuration Reference

Configuration is stored in `config.toml` inside the Lacie home directory.

| Key | Type | Allowed Values | Default | Description |
|---|---|---|---|---|
| `language` | string | `auto`, `en`, `ru` | `auto` | UI language |
| `colors` | string | `terminal`, `always`, `never` | `terminal` | Terminal color mode |
| `verify_sha256` | boolean | `true`, `false` | `true` | Enforce binary SHA256 checksums |
| `auto_switch` | boolean | `true`, `false` | `true` | Automatically activate newly installed versions |
| `confirm` | boolean | `true`, `false` | `true` | Prompt for confirmation before changes |
| `always_show` | boolean | `true`, `false` | `false` | Always display file inspection before install |
| `cache_ttl` | integer | Minutes ($\ge 0$) | `10` | Cache TTL for search queries and release lists |
| `default_source`| string | Any registered source | `github` | Default source when not specified in package ref |
| `windows_priority` | string | `msvc`, `mingw` | `msvc` | Preferred Windows ABI (Windows 11 only) |

---

## 7. Environment Variables

| Variable | Description |
|---|---|
| `LACIE_HOME` | Overrides the root directory where Lacie stores data and binaries. |
| `LACIE_GH_TOKEN` | GitHub Personal Access Token (`ghp_...`). Used by the built-in GitHub source plugin to bypass anonymous rate limits (60 requests/hr) and access private repositories. |
| `NO_COLOR` | When set to any value, disables ANSI color output (in accordance with `no-color.org`). |
