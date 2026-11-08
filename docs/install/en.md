# Lacie Installation Guide

This guide covers installing and setting up the **Lacie** package manager on Linux, macOS, Windows 11, FreeBSD, and OpenBSD.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation Methods Overview](#installation-methods-overview)
- [Linux](#linux)
  - [Method 1: Ruby Installer (Recommended)](#linux---method-1-ruby-installer-recommended)
  - [Method 2: Nix](#linux---method-2-nix)
  - [Method 3: Manual Binary Download](#linux---method-3-manual-binary-download)
- [macOS](#macos)
  - [Method 1: Ruby Installer (Recommended)](#macos---method-1-ruby-installer-recommended)
  - [Method 2: Nix](#macos---method-2-nix)
  - [Method 3: Manual Binary Download](#macos---method-3-manual-binary-download)
- [Windows 11](#windows-11)
  - [Method 1: Ruby Installer](#windows-11---method-1-ruby-installer)
  - [Method 2: Manual Binary Download](#windows-11---method-2-manual-binary-download)
- [FreeBSD](#freebsd)
  - [Method 1: Ruby Installer](#freebsd---method-1-ruby-installer)
  - [Method 2: Manual Binary Download](#freebsd---method-2-manual-binary-download)
- [OpenBSD](#openbsd)
  - [Method 1: Ruby Installer](#openbsd---method-1-ruby-installer)
  - [Method 2: Manual Binary Download](#openbsd---method-2-manual-binary-download)
- [Post-Installation Setup](#post-installation-setup)
  - [Configuring PATH](#configuring-path)
  - [Initializing Environment (`init-lacie`)](#initializing-environment-init-lacie)
  - [Verifying Health (`doctor`)](#verifying-health-doctor)
- [Upgrading Lacie](#upgrading-lacie)
- [Uninstalling Lacie](#uninstalling-lacie)

---

## Prerequisites

- **Network:** Internet access to fetch releases from GitHub (`github.com/shareui/lacie`).
- **Ruby:** For the interactive installer (`install.rb`), Ruby 3.0+ is recommended (available by default or via package manager on most Unix systems).
- **Permissions:** Root / administrator access is **not required**. Lacie runs entirely within user space (`~/.local/bin` or user-chosen directory, storing packages in `$HOME/lacie` or `%LOCALAPPDATA%\lacie`).

---

## Installation Methods Overview

| Platform | Ruby Installer (`install.rb`) | Nix (`flake.nix`) | Manual Binary |
| :--- | :---: | :---: | :---: |
| **Linux (GNU / musl, x86_64 / aarch64)** | Supported (Recommended) | Supported | Supported |
| **macOS (Intel / Apple Silicon)** | Supported (Recommended) | Supported | Supported |
| **Windows 11 (x86_64)** | Supported | N/A | Supported |
| **FreeBSD (x86_64)** | Supported | N/A | Supported |
| **OpenBSD (x86_64)** | Supported | N/A | Supported |

---

## Linux

### Linux - Method 1: Ruby Installer (Recommended)

Run the interactive installer using `curl` and `ruby`:

```bash
curl -fsSL https://raw.githubusercontent.com/shareui/lacie/main/install.rb | ruby
```

Or download the script and execute it locally:

```bash
curl -fsSL -O https://raw.githubusercontent.com/shareui/lacie/main/install.rb
ruby install.rb
```

**What the installer does:**
1. Detects your OS (`Linux`), C library (`GNU` or `musl`), and CPU architecture (`x86_64` or `aarch64`).
2. Fetches release versions from `shareui/lacie` and lets you select the version to install.
3. Scans your `$PATH` for user-writable directories (such as `~/.local/bin` or `~/bin`) and lets you choose the destination.
4. Downloads the matching uncompressed binary with direct stream progress.
5. Sets executable permissions (`chmod 0755`) and runs `lacie init-lacie` automatically.

### Linux - Method 2: Nix

If you use Nix with flakes enabled:

**Run directly without installing:**
```bash
nix run github:shareui/lacie -- --help
```

**Install into user profile:**
```bash
nix profile install github:shareui/lacie
```

**Include in `flake.nix` (NixOS or Home Manager):**
```nix
{
  inputs = {
    lacie.url = "github:shareui/lacie";
  };

  outputs = { self, nixpkgs, lacie }: {
    # Add lacie.packages.${system}.default to environment.systemPackages or home.packages
  };
}
```

### Linux - Method 3: Manual Binary Download

Download the appropriate binary for your architecture from [GitHub Releases](https://github.com/shareui/lacie/releases):

- `lacie-linux-gnu-x86_64` (glibc x86_64)
- `lacie-linux-gnu-aarch64` (glibc ARM64)
- `lacie-linux-musl-x86_64` (musl x86_64, Alpine Linux / Void Linux)
- `lacie-linux-musl-aarch64` (musl ARM64)

```bash
# Example for Linux x86_64 GNU
curl -fsSL -o ~/.local/bin/lacie https://github.com/shareui/lacie/releases/latest/download/lacie-linux-gnu-x86_64
chmod +x ~/.local/bin/lacie
lacie init-lacie
```

---

## macOS

### macOS - Method 1: Ruby Installer (Recommended)

macOS includes Ruby out of the box or via Homebrew (`brew install ruby`). Run:

```bash
curl -fsSL https://raw.githubusercontent.com/shareui/lacie/main/install.rb | ruby
```

The installer detects whether you are running on Apple Silicon (`aarch64` / M1/M2/M3/M4) or Intel (`x86_64`), offers a choice of user installation paths (e.g. `~/.local/bin`), and finishes the setup.

### macOS - Method 2: Nix

On macOS with Nix installed:

```bash
nix profile install github:shareui/lacie
```

Or run interactively:

```bash
nix run github:shareui/lacie -- --help
```

### macOS - Method 3: Manual Binary Download

Download from [GitHub Releases](https://github.com/shareui/lacie/releases):
- `lacie-macos-aarch64` (Apple Silicon)
- `lacie-macos-x86_64` (Intel)

```bash
# Example for Apple Silicon
mkdir -p ~/.local/bin
curl -fsSL -o ~/.local/bin/lacie https://github.com/shareui/lacie/releases/latest/download/lacie-macos-aarch64
chmod +x ~/.local/bin/lacie
lacie init-lacie
```

---

## Windows 11

### Windows 11 - Method 1: Ruby Installer

If you have Ruby installed (e.g. via RubyInstaller or `winget install RubyInstallerTeam.Ruby.3.3`):

1. Open PowerShell or Windows Terminal.
2. Download and run the installer:
   ```powershell
   Invoke-WebRequest -Uri "https://raw.githubusercontent.com/shareui/lacie/main/install.rb" -OutFile "install.rb"
   ruby install.rb
   ```
3. Follow the interactive prompts to install into `%LOCALAPPDATA%\Programs\bin` or your user binary directory.

### Windows 11 - Method 2: Manual Binary Download

1. Download `lacie-windows-msvc-x86_64.exe` from [GitHub Releases](https://github.com/shareui/lacie/releases).
2. Rename the file to `lacie.exe`.
3. Place it in a folder in your user `PATH` (such as `C:\Users\<Username>\AppData\Local\Microsoft\WindowsApps` or custom user bin folder).
4. Run:
   ```cmd
   lacie init-lacie
   ```

---

## FreeBSD

### FreeBSD - Method 1: Ruby Installer

Ensure Ruby is installed (`pkg install ruby`):

```sh
fetch -o - https://raw.githubusercontent.com/shareui/lacie/main/install.rb | ruby
```

Or using `curl`:
```sh
curl -fsSL https://raw.githubusercontent.com/shareui/lacie/main/install.rb | ruby
```

### FreeBSD - Method 2: Manual Binary Download

Download `lacie-freebsd-x86_64` from [GitHub Releases](https://github.com/shareui/lacie/releases):

```sh
mkdir -p ~/.local/bin
fetch -o ~/.local/bin/lacie https://github.com/shareui/lacie/releases/latest/download/lacie-freebsd-x86_64
chmod 0755 ~/.local/bin/lacie
lacie init-lacie
```

---

## OpenBSD

### OpenBSD - Method 1: Ruby Installer

Ensure Ruby and curl are installed (`pkg_add ruby curl`):

```sh
curl -fsSL https://raw.githubusercontent.com/shareui/lacie/main/install.rb | ruby
```

### OpenBSD - Method 2: Manual Binary Download

Download `lacie-openbsd-x86_64` from [GitHub Releases](https://github.com/shareui/lacie/releases):

```sh
mkdir -p ~/.local/bin
ftp -o ~/.local/bin/lacie https://github.com/shareui/lacie/releases/latest/download/lacie-openbsd-x86_64
chmod 0755 ~/.local/bin/lacie
lacie init-lacie
```

---

## Post-Installation Setup

### Configuring PATH

Lacie manages active binaries in `$HOME/lacie/active` (Unix) or `%LOCALAPPDATA%\lacie\active` (Windows).

To ensure packages installed via `lacie add` are accessible from your terminal:

**Bash / Zsh:**
Add to your `~/.bashrc` or `~/.zshrc`:
```bash
export PATH="$HOME/lacie/active:$PATH"
```

**Fish:**
Add to `~/.config/fish/config.fish`:
```fish
fish_add_path -g $HOME/lacie/active
```

**PowerShell (Windows):**
```powershell
[Environment]::SetEnvironmentVariable("PATH", "$env:PATH;$env:LOCALAPPDATA\lacie\active", "User")
```

### Initializing Environment (`init-lacie`)

The installer runs this automatically. If you installed manually, initialize directories and default configuration:

```bash
lacie init-lacie
```

This creates the required directory tree:
- Unix: `$HOME/lacie/` (`config.toml`, `packages.toml`, `sources.toml`, `active/`, `bin/`, `scripts/`, `plugins/`, `var/`)
- Windows: `%LOCALAPPDATA%\lacie\`

### Verifying Health (`doctor`)

Check that directories, configuration, shell path, and plugins are properly initialized:

```bash
lacie doctor
```

If any issues are reported, automatic remediation is available:

```bash
lacie doctor --fix
```

---

## Upgrading Lacie

To upgrade Lacie to the newest release:

1. **Via Ruby Installer:**
   Run `install.rb` and select option `2` (`Update / Reinstall Lacie`). The installer locates your existing `lacie` binary via `PATH` and atomically replaces it:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/shareui/lacie/main/install.rb | ruby
   ```

2. **Via Nix:**
   ```bash
   nix profile upgrade github:shareui/lacie
   ```

3. **Manual:**
   Replace the existing binary with the newly downloaded binary from GitHub Releases.

---

## Uninstalling Lacie

Lacie provides a built-in clean removal command:

```bash
lacie suicide
```

This prompts for confirmation, removes all downloaded packages, symlinks, caches, and configuration directories (`$HOME/lacie`). Finally, remove the `lacie` binary itself from your binary directory.
