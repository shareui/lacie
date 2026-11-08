# lacie

> Fast, lightweight, zero-dependency package manager for prebuilt binaries with explicit security, version coexistence, and declarative manifests.

> [!WARNING]
> **Active Development Notice**  
> Lacie is in active development. While core package flows and cross-platform targets are implemented, not everything is battle-tested across every edge case and environment. If you encounter bugs, oddities, or unexpected behavior, please [open an issue on GitHub](https://github.com/shareui/lacie/issues).

---

## Quick Install

### Unix (Linux, macOS, FreeBSD, OpenBSD)
```bash
curl -fsSL https://raw.githubusercontent.com/shareui/lacie/main/install.rb | ruby
```

### Nix (Linux & macOS)
```bash
nix profile install github:shareui/lacie
```

For platform-specific instructions, Windows 11 setup, manual binary downloads, and configuration, see the [Installation Guide](docs/install/en.md) ([русская версия](docs/install/ru.md)).

---

## Key Highlights

- **Prebuilt Binaries:** Downloads and manages standalone binaries directly from GitHub and custom registries without requiring build toolchains or root privileges.
- **Side-by-Side Versions:** Keep multiple versions of tools installed simultaneously in isolated paths (`bin/<id>/<version>/`).
- **Instant Version Switching:** Switch active package versions in milliseconds (`lacie switch <pkg> <ver>`).
- **Explicit Security:** SHA256 checksum verification on downloads, hash-pinned manifests, and user prompts before executing lifecycle scripts.
- **Atomic Transactions:** State files and binaries are updated atomically, preventing corrupted states on interruption.
- **Cross-Platform:** First-class support for Linux (GNU & musl), macOS (Intel & Apple Silicon), Windows 11, FreeBSD, and OpenBSD.

---

## Documentation

- **Installation Guide:** [English](docs/install/en.md) | [Русский](docs/install/ru.md)
- **User Documentation:** [English](docs/user/en.md) | [Русский](docs/user/ru.md)
- **Developer Guide:** [English](docs/dev/en.md) | [Русский](docs/dev/ru.md)

---

## Quick Start

```bash
# Initialize user configuration and directories
lacie init-lacie

# Install a package from GitHub
lacie add github:BurntSushi/ripgrep

# Search and install interactively
lacie add ripgrep

# List installed packages and active versions
lacie list

# Inspect release details without installing
lacie add ripgrep --info

# Switch to a previously installed version
lacie switch ripgrep 14.1.0

# Check for updates across all packages
lacie update

# Verify health and fix environment issues
lacie doctor --fix

# Remove a package
lacie remove ripgrep
```

---

## License

[MIT](LICENSE) © 2026 shareui
