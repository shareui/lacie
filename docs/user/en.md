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
  - [Command: init-lacie](#command-init-lacie)
  - [Command: add](#command-add)
  - [Command: update](#command-update)
  - [Command: switch](#command-switch)
  - [Command: remove](#command-remove)
  - [Command: freeze](#command-freeze)
  - [Command: unfreeze](#command-unfreeze)
  - [Command: frozen](#command-frozen)
  - [Command: rename](#command-rename)
  - [Command: source](#command-source)
  - [Command: list](#command-list)
  - [Command: info](#command-info)
  - [Command: doctor](#command-doctor)
  - [Command: config](#command-config)
  - [Command: export](#command-export)
  - [Command: import](#command-import)
  - [Command: validate](#command-validate)
  - [Command: gen-toml](#command-gen-toml)
  - [Command: completions](#command-completions)
  - [Command: explain](#command-explain)
  - [Command: suicide](#command-suicide)
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
- **Local installation optimization:** When installing from local archives or manifests without internet access, Lacie skips simulated download progress bars and directly unpacks and copies binaries.

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

### Command: init-lacie

Initializes or reconfigures the Lacie working environment, shell integration, and global settings.

```bash
lacie init-lacie [QUESTION_ID] [options]
```

**Parameters:**
- `[QUESTION_ID]`: Optional identifier to jump directly to a specific question:
  - `1` or `lang`: Language selection (`auto`, `en`, `ru`).
  - `2` or `color`: Color styling mode (`terminal`, `always`, `never`).
  - `3` or `verify`: Enforce strict SHA256 verification (`true`/`false`).
  - `4` or `path`: Add `active/` directory to shell `PATH`.
  - `5` or `switch`: Automatically switch active version upon installation (`true`/`false`).

**Options & Flags:**
- `--recfg`: Re-prompts questions even if they have already been configured and recorded.

**Examples:**
```bash
# Run the complete interactive setup
lacie init-lacie

# Reconfigure only the color scheme
lacie init-lacie color --recfg

# Reconfigure PATH integration
lacie init-lacie path --recfg
```

---

### Command: add

Installs one or more packages from remote sources, direct URLs, or local files.

```bash
lacie add <package-ref> [version] [options]
```

**Parameters:**
- `<package-ref>`: Package reference identifier. Can be an `owner/repo` shorthand, `source:owner/repo`, a direct URL, a local archive path (`.tar.gz`, `.tar.xz`, `.tar.zst`, `.zip`), or a local manifest file (`.toml`).
- `[version]`: Optional release tag or version number (defaults to `latest`).

**Options & Flags:**
- `-y`, `--yes`: Skip confirmation prompts and install automatically.
- `--show`: Open the full-screen interactive terminal viewer to inspect the manifest and scripts before approving.
- `--list`: List all available upstream releases and tags for the target repository without downloading.
- `--info`: Display manifest metadata and architecture targets without installing.
- `--force`: Reinstall the package even if the target version already exists in the database.
- `--set-hash <sha256>`: Explicitly provide an expected SHA256 checksum when the manifest lacks one.
- `--ignore-hash`: Skip SHA256 integrity verification (produces warning `W002`, not recommended).
- `--max <n>`: Limit the maximum number of search results displayed when resolving ambiguous references.
- `-b`, `--no-bar`: Suppress bracketed download progress bars while retaining progress statistics.

**Local vs. Network Installation:**
- When installing local files (manifests or archives) without network downloads, Lacie switches to fast local mode: it displays `Unpacking` or `Copying` without artificial progress bars or fake network transfer rates.
- If a manifest references remote assets, download progress bars `[-----> ... ]` with transfer speeds and byte counts are rendered automatically.

**Interactive Viewer Hotkeys (`--show`):**
- `n`: View next file in review queue.
- `p`: View previous file in review queue.
- `a`: Approve package and proceed with installation.
- `d` / `q`: Deny package and abort installation immediately.

**Examples:**
```bash
# Install package from default source (GitHub)
lacie add shareui/nyano

# Install a specific version
lacie add shareui/nyano 0.2.0

# Install from a local archive without internet
lacie add ./yukino-lsp-0.1.0.tar.gz

# Inspect manifest and scripts before approving
lacie add shareui/nyano --show

# Force reinstall with a custom checksum
lacie add shareui/nyano --force --set-hash e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

---

### Command: update

Checks for newer releases and updates installed packages.

```bash
lacie update [packages...] [options]
```

If no packages are specified, Lacie checks every installed package that is not frozen or locally installed.

**Parameters:**
- `[packages...]`: Optional list of specific package names or IDs to check and update.

**Options & Flags:**
- `-a`, `--all`: Explicitly check and update all installed packages.
- `-y`, `--yes`: Apply all available updates automatically without confirmation prompt.
- `-d`, `--dry-run`: Check for updates and display available versions without downloading or altering files.
- `-i`, `--ignore <package>`: Exclude specific package(s) from update checking (can be specified multiple times).
- `--pre`: Include pre-release versions (alpha, beta, release candidates).
- `--no-cache`: Bypass the local release cache and query upstream registries directly.
- `-b`, `--no-bar`: Suppress download progress bars.

**Behavior:**
- Updates download alongside older versions. If `auto_switch` is enabled, the new version becomes active immediately. Older versions remain intact in `bin/` and can be switched back instantly using `lacie switch`.

**Examples:**
```bash
# Check and update all packages interactively
lacie update

# Non-interactive update of all packages
lacie update -y

# Check updates without downloading
lacie update --dry-run

# Update specific package including pre-releases
lacie update ripgrep --pre

# Update all packages except a pinned tool
lacie update --all --ignore neovim
```

---

### Command: switch

Switches the active version of an installed package without redownloading.

```bash
lacie switch <package> <version> [<package2> <version2> ...]
```

**Parameters:**
- `<package>`: Name or ID of an already installed package.
- `<version>`: Installed version number to activate in `active/` (PATH).
- Multiple package-version pairs can be provided in a single command.

**Examples:**
```bash
# Switch ripgrep to version 13.0.0
lacie switch ripgrep 13.0.0

# Switch multiple packages in one invocation
lacie switch ripgrep 13.0.0 neovim 0.9.5
```

---

### Command: remove

Uninstalls packages or purges inactive versions from disk.

```bash
lacie remove <packages...> [options]
```

**Parameters:**
- `<packages...>`: One or more package names or IDs to remove.

**Options & Flags:**
- `-y`, `--yes`: Skip interactive confirmation prompt.
- `--unused`: Purge only inactive/cached versions, leaving the current active version intact.
- `--symlink-only`: Remove executable links/files from `active/` without deleting binary files from `bin/`.
- `--active-only`: Remove only the currently active version.
- `--no-scripts`: Bypass execution of package lifecycle scripts (`preremove.rb` and `postremove.rb`).

**Security:**
- Prior to running removal scripts, Lacie verifies their SHA256 hash against `scripts.toml`. If a script was modified on disk, execution is refused with error `E035`.

**Examples:**
```bash
# Completely remove a package
lacie remove ripgrep

# Clean up older, inactive versions of all packages
lacie remove ripgrep --unused

# Deactivate command from PATH without deleting files
lacie remove ripgrep --symlink-only
```

---

### Command: freeze

Freezes a package at its currently active version, shielding it from updates.

```bash
lacie freeze <package|ALL>
```

**Parameters:**
- `<package>`: Name or ID of the package to lock.
- `ALL`: Freeze all currently installed packages at their active versions.

**Behavior:**
- Frozen packages are marked as `[frozen]` in `lacie list` and are completely skipped during `lacie update`.

**Examples:**
```bash
# Freeze a single package
lacie freeze neovim

# Freeze all installed packages
lacie freeze ALL
```

---

### Command: unfreeze

Unfreezes a previously frozen package, re-enabling updates.

```bash
lacie unfreeze <package|ALL>
```

**Parameters:**
- `<package>`: Name or ID of the package to unlock.
- `ALL`: Unfreeze all currently frozen packages.

**Examples:**
```bash
# Unfreeze a single package
lacie unfreeze neovim

# Unfreeze all packages
lacie unfreeze ALL
```

---

### Command: frozen

Displays a list of all currently frozen packages.

```bash
lacie frozen
```

**Parameters:**
- None.

**Examples:**
```bash
lacie frozen
```

---

### Command: rename

Changes the executable command name created in `active/` (PATH).

```bash
lacie rename <package> <new-command-name|DEFAULT>
```

**Parameters:**
- `<package>`: Name or ID of the target package.
- `<new-command-name>`: Desired executable name in `active/`.
- `DEFAULT`: Restores the original command name specified in the package manifest.

**Examples:**
```bash
# Rename neovim binary to nvim in PATH
lacie rename neovim nvim

# Reset back to default command name
lacie rename neovim DEFAULT
```

---

### Command: source

Manages package registries and source plugins.

```bash
lacie source <subcommand> [options]
```

**Subcommands:**
- `lacie source list`: Lists all registered sources with ID, version, origin, author, and status (`builtin`, `active`, `disabled`).
- `lacie source add <name> <ref> [options]`: Registers a new source plugin from a Git repository or local directory.
  - `-y`, `--yes`: Skip confirmation prompt.
- `lacie source remove <name> [options]`: Removes an installed third-party source plugin. Built-in plugins (e.g. `github`) cannot be removed.
  - `-y`, `--yes`: Skip confirmation prompt.
- `lacie source update [name] [options]`: Updates source plugins from their origin repositories.
  - `-y`, `--yes`: Skip confirmation prompt.
- `lacie source enable <name>`: Enables a previously disabled source plugin.
- `lacie source disable <name>`: Disables a source plugin without deleting its files.

**Examples:**
```bash
# List all sources
lacie source list

# Add a community registry
lacie source add community shareui/lacie-community-plugin

# Disable a source temporarily
lacie source disable community
```

---

### Command: list

Lists installed packages with version information, active status, and origin.

```bash
lacie list [filters...] [options]
```

**Parameters:**
- `[filters...]`: Optional filter expressions (e.g. `frozen:true`, `author:shareui`, `name:ripgrep*`). See [Filtering Syntax](#5-filtering-syntax).

**Options & Flags:**
- `--json`: Output packages as a structured JSON array for scripting and programmatic integration.

**Examples:**
```bash
# List all packages in table format
lacie list

# Filter packages by author
lacie list author:shareui

# Output as JSON
lacie list --json
```

---

### Command: info

Displays detailed metadata, installed versions, file sizes, and checksums for packages.

```bash
lacie info <packages...>
```

**Parameters:**
- `<packages...>`: One or more package names or IDs.

**Output Details:**
- Package ID, name, command alias, active version, list of all installed versions with byte sizes, target architecture, author, source URL, and SHA256 checksums.
- When inspecting multiple packages, outputs responsive bordered ASCII cards that scale to terminal dimensions.

**Examples:**
```bash
# View details for a single package
lacie info ripgrep

# View multiple packages in bordered cards
lacie info ripgrep neovim fd
```

---

### Command: doctor

Audits the local environment, filesystem permissions, symlink integrity, and state files.

```bash
lacie doctor [options]
```

**Options & Flags:**
- `--fix`: Automatically repairs identified issues (recreates required directories, sets permissions, fixes dangling symlinks, and validates state files).

**Audit Scope:**
- Home directory structure and permissions (`0700` on Unix).
- Shell configuration and `PATH` variable presence.
- Active directory links and binary execution permissions (`0755` on Unix).
- Consistency across `packages.toml`, `sources.toml`, `seen.toml`, and `scripts.toml`.
- Plugin availability and Ruby sandbox runtime health.

**Examples:**
```bash
# Run environment audit
lacie doctor

# Automatically fix detected problems
lacie doctor --fix
```

---

### Command: config

Inspects and updates configuration settings in `config.toml`.

```bash
lacie config <subcommand> [options]
```

**Subcommands:**
- `lacie config list`: Displays a categorized overview (`UI & Environment`, `Behavior & Safety`, `Engine & Storage`) with keys, current values, defaults, modification markers (`*`), and hints.
  - Automatically adapts to the host platform: `windows_priority` appears only on Windows, `linux_priority` appears only on Linux, and neither appears on macOS/BSD.
- `lacie config get <key>`: Prints the current value of a configuration setting.
- `lacie config set <key> <value|DEFAULT>`: Sets a configuration key. Passing `DEFAULT` resets the key to its factory default value.
- `lacie config reset [options]`: Resets all settings back to factory defaults.
  - `-y`, `--yes`: Skip confirmation prompt.

**Examples:**
```bash
# List all configuration options
lacie config list

# View active language
lacie config get language

# Change default source to a custom plugin
lacie config set default_source community

# Reset confirmation prompt to default
lacie config set confirm DEFAULT
```

---

### Command: export

Exports installed package state into a declarative lockfile.

```bash
lacie export [file] [options]
```

**Parameters:**
- `[file]`: Destination file path (defaults to `lacie-lock.toml`), or `-` to write directly to standard output.

**Options & Flags:**
- `--all-versions`: Include all downloaded and cached versions instead of only currently active ones.
- `--with-config`: Export user configuration settings alongside the package list.

**Examples:**
```bash
# Export active packages to default lacie-lock.toml
lacie export

# Export to stdout for piped processing
lacie export - > my-backup.toml

# Export full state including all versions and config
lacie export backup.toml --all-versions --with-config
```

---

### Command: import

Restores package state from an exported lockfile.

```bash
lacie import [file]
```

**Parameters:**
- `[file]`: Source lockfile path (defaults to `lacie-lock.toml`), or `-` to read from standard input.

**Examples:**
```bash
# Restore packages from default lockfile
lacie import

# Restore packages from custom file
lacie import backup.toml

# Restore piped state from stdin
cat backup.toml | lacie import -
```

---

### Command: validate

Validates a `lacie.toml` manifest file for syntax, required fields, and structural validity.

```bash
lacie validate [path/to/lacie.toml]
```

**Parameters:**
- `[path/to/lacie.toml]`: Path to the manifest file to validate (defaults to `./lacie.toml`).

**Checks Performed:**
- Package identifier reverse-DNS format (`com.example.app`).
- Valid target triple naming and architecture compatibility.
- Presence of mandatory fields (`name`, `version`, binary entries).
- Verification of local binary file checksums when present on disk.

**Examples:**
```bash
# Validate local manifest
lacie validate

# Validate custom manifest path
lacie validate path/to/lacie.toml
```

---

### Command: gen-toml

Generates a starter `lacie.toml` template in the current working directory.

```bash
lacie gen-toml [options]
```

**Options & Flags:**
- `-y`, `--yes`: Overwrite existing `lacie.toml` without prompting.

**Examples:**
```bash
# Generate starter template
lacie gen-toml

# Overwrite existing template non-interactively
lacie gen-toml -y
```

---

### Command: completions

Generates command autocompletion scripts for various shells.

```bash
lacie completions <shell>
```

**Parameters:**
- `<shell>`: Target shell name: `bash`, `zsh`, `fish`, `powershell`.

**Examples:**
```bash
# Fish shell
lacie completions fish > ~/.config/fish/completions/lacie.fish

# Bash shell
lacie completions bash > /etc/bash_completion.d/lacie

# Zsh shell
lacie completions zsh > "${fpath[1]}/_lacie"
```

---

### Command: explain

Provides detailed explanations and step-by-step remediation instructions for diagnostic codes.

```bash
lacie explain <CODE>
```

**Parameters:**
- `<CODE>`: Error code (e.g. `E001`–`E036`) or warning code (e.g. `W001`–`W007`).

**Examples:**
```bash
# Explain missing target architecture error
lacie explain E006

# Explain checksum mismatch error
lacie explain E007

# Explain fallback warning
lacie explain W003
```

---

### Command: suicide

Completely and irreversibly uninstalls Lacie from your system.

```bash
lacie suicide [options]
```

**Options & Flags:**
- `-y`, `--yes`: Skip confirmation prompt and perform immediate removal.

**Removal Actions:**
1. Deactivates and deletes all binaries and symlinks from `active/` and `bin/`.
2. Deletes all database files, caches, and plugins.
3. Automatically strips Lacie `PATH` configuration from shell profiles (`.bashrc`, `.zshrc`, `config.fish`).
4. Removes the root Lacie home directory.

**Examples:**
```bash
# Uninstall with confirmation prompt
lacie suicide

# Immediate removal
lacie suicide -y
```

---

### Version & Help

- `lacie`: When invoked without arguments, displays binary version, build metadata, license, and repository link.
- `lacie -v`, `lacie --version`: Prints version number.
- `lacie --help`: Displays summary of all available commands and global options.

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
| `linux_priority` | string | `glibc`, `musl` | runtime detected (`glibc`/`musl`) | Preferred Linux libc ABI (Linux only) |

---

## 7. Environment Variables

| Variable | Description |
|---|---|
| `LACIE_HOME` | Overrides the root directory where Lacie stores data and binaries. |
| `LACIE_GH_TOKEN` | GitHub Personal Access Token (`ghp_...`). Used by the built-in GitHub source plugin to bypass anonymous rate limits (60 requests/hr) and access private repositories. |
| `NO_COLOR` | When set to any value, disables ANSI color output (in accordance with `no-color.org`). |
