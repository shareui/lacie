# Lacie Developer Documentation

Technical guide for package maintainers, repository authors, and source plugin developers.

---

## Table of Contents

- [1. Overview & Architecture](#1-overview--architecture)
- [2. Manifest Specification (`lacie.toml`)](#2-manifest-specification-lacietoml)
  - [Workspace Structure](#workspace-structure)
  - [Package Definition (`[packages."<id>"]`)](#package-definition-packagesid)
  - [Target Platform Triples](#target-platform-triples)
  - [Binary Resolution (`[packages."<id>".binaries."<target>"]`)](#binary-resolution-packagesidbinariestarget)
  - [Platform Fallback Rules](#platform-fallback-rules)
  - [Mandatory Companion Packages (`required`)](#mandatory-companion-packages-required)
  - [Dependencies (`apps` and `libs`)](#dependencies-apps-and-libs)
  - [Manifest Validation](#manifest-validation)
- [3. Lifecycle Scripts (Hooks)](#3-lifecycle-scripts-hooks)
  - [Available Hooks](#available-hooks)
  - [Execution Context & Constraints](#execution-context--constraints)
  - [Security & Integrity Pinning (`scripts.toml`)](#security--integrity-pinning-scriptstoml)
- [4. Source Plugin Architecture](#4-source-plugin-architecture)
  - [Plugin Directory Layout](#plugin-directory-layout)
  - [Plugin Manifest (`source.toml`)](#plugin-manifest-sourcetoml)
  - [JSON-RPC over stdio Protocol](#json-rpc-over-stdio-protocol)
  - [RPC Methods Reference](#rpc-methods-reference)
  - [Custom Plugin Implementation Example](#custom-plugin-implementation-example)

---

## 1. Overview & Architecture

Lacie treats packages as declarative, immutable assets. Key developer concepts:
- **Workspaces:** A single `lacie.toml` manifest can define multiple distinct packages sharing common metadata, versioning, or binaries.
- **Zero-trust verification:** Binaries must define SHA256 checksums. Any lifecycle script is pinned by its hash in `scripts.toml` upon user approval.
- **Pluggable registry sources:** Sources (like GitHub, GitLab, custom HTTP/S3 endpoints) are implemented as standalone plugins communicating with Lacie via a standardized JSON-RPC stdio protocol.

---

## 2. Manifest Specification (`lacie.toml`)

`lacie.toml` defines packages, binaries per target, dependencies, and lifecycle scripts.

### Workspace Structure

A manifest contains a global `[workspace]` table and one or more `[packages."<id>"]` tables.

```toml
[workspace]
author = "shareui"
version = "1.0.0"
version_number = 1
source_code = "https://github.com/shareui/lacie"
links = ["https://example.com"]

[packages."com.example.app"]
name = "app"

[packages."com.example.app".binaries."linux-gnu-x86_64"]
file = "app-linux-x86_64"
sha256 = "3f2a9c0e41b7d2a85c6f0e19b4d7a3c28e5f1096ab7d4c3e2f81b0a9d6c57e43"
```

#### Inheritance Rules
- Keys defined in `[workspace]` act as defaults for all packages.
- A package table overrides workspace defaults.
- Overriding replaces values entirely; arrays are not merged.
- `name` and `binaries` cannot be defined in `[workspace]`; they are strictly package-specific.

---

### Package Definition (`[packages."<id>"]`)

Package IDs use reverse-DNS notation (`com.domain.tool`). IDs must contain at least one dot, use lowercase alphanumeric characters, hyphens, or underscores, and be between 3 and 128 characters long.

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | **Yes** | Display name of the package and default executable name. |
| `version` | string | **Yes** | Semantic version string (e.g. `1.2.0`). |
| `version_number` | integer | No | Monotonically increasing build/release counter. |
| `author` | string | No | Package author or maintaining organization. |
| `source_code` | string | No | URL to source repository. |
| `links` | array of strings | No | Relevant project URLs or documentation links. |
| `required` / `requires` | array of strings | No | Companion package references automatically pulled in during installation. |
| `apps` | array of tables | No | Tool/application dependencies. |
| `libs` | array of tables | No | Library/resource dependencies. |
| `check_deps` | string | No | Relative path to dependency verification script. |
| `prebuild` | string | No | Relative path to pre-installation script. |
| `postbuild` | string | No | Relative path to post-installation script. |
| `preremove` | string | No | Relative path to pre-removal script. |
| `postremove` | string | No | Relative path to post-removal script. |

---

### Target Platform Triples

Binaries are defined per target triple using the `{os}-{abi}-{arch}` format (or `{os}-{arch}` when the OS has a single standard ABI):

| Target Triple | Platform |
|---|---|
| `linux-gnu-x86_64`, `linux-gnu-aarch64` | Linux with glibc |
| `linux-musl-x86_64`, `linux-musl-aarch64` | Linux with musl libc |
| `windows-msvc-x86_64`, `windows-msvc-aarch64` | Windows 11 (MSVC runtime) |
| `windows-mingw-x86_64` | Windows 11 (MinGW runtime) |
| `macos-aarch64`, `macos-x86_64` | macOS (Apple Silicon / Intel) |
| `freebsd-x86_64`, `openbsd-x86_64` | FreeBSD / OpenBSD |

---

### Binary Resolution (`[packages."<id>".binaries."<target>"]`)

Each target table **must define exactly one** of `file`, `links`, or `source`. Specifying more than one or none causes validation error `E028`.

```toml
# 1. Release asset filename or relative path
[packages."com.example.tool".binaries."linux-gnu-x86_64"]
file = "tool-linux-x86_64.tar.gz"
sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

# 2. Direct download URLs with mirror fallback
[packages."com.example.tool".binaries."windows-msvc-x86_64"]
links = [
  "https://cdn.example.com/tool-windows.zip",
  "https://backup.example.com/tool-windows.zip"
]
sha256 = "b81d02c7aa90e4f35d6218cb07a9e3f4152d8c60b7e9a14f03d5c28e6b79a1f0"

# 3. Delegation to another package repository
[packages."com.example.tool".binaries."macos-aarch64"]
source = "github:shareui/tool-macos#com.example.tool"
```

#### Binary Fields Reference
- `file`: Asset filename in the current release, or relative file path for local/archive manifests.
- `links`: Array of direct download URLs tried in order.
- `source`: Reference to an external repository in standard package reference format (`source:owner/repo#pkg`).
- `sha256`: Hexadecimal SHA256 checksum of the archive or binary.
- `size`: Expected file size in bytes (optional, queries upstream API if omitted).
- `static`: Boolean. Set `true` if binary is statically linked and independent of libc.

---

### Platform Fallback Rules

Lacie applies deterministic ABI fallback resolution when the primary target is absent:

1. **Linux (glibc to musl):**
   - On glibc systems (`linux-gnu-*`), if no GNU binary exists, Lacie falls back to `linux-musl-*` **only if** `static = true` or `ld-musl-{arch}.so.1` exists in `/lib`.
   - On musl systems (e.g. Alpine), glibc binaries cannot run; no reverse fallback is offered.

2. **Windows 11 (MSVC vs MinGW):**
   - Controlled by user setting `windows_priority` (default `msvc`).
   - If the preferred build is missing, Lacie offers the alternative build with warning `W003`.

---

### Mandatory Companion Packages (`required`)

Packages can specify companions that must always be installed alongside them:

```toml
[packages."com.example.editor"]
name = "editor"
version = "2.0.0"
required = ["github:shareui/treesitter-parser", "com.example.linter"]
```

When a user runs `lacie add editor`, Lacie transitively resolves and queues all packages listed in `required` (or `requires`).

---

### Dependencies (`apps` and `libs`)

- `apps`: External command-line tools required at runtime:
  ```toml
  apps = [
    { id = "org.git.git", from = "github:git/git", min_number = 10 }
  ]
  ```
- `libs`: Shared libraries or runtime assets placed in the package directory:
  ```toml
  libs = [
    { id = "org.openssl.libssl", from = "github:openssl/openssl", dest = "lib" }
  ]
  ```

---

### Manifest Validation

Maintainers can validate `lacie.toml` locally before publishing:

```bash
lacie validate ./lacie.toml
```

This verifies schema consistency, target syntax, and checks SHA256 hashes against local assets when present.

---

## 3. Lifecycle Scripts (Hooks)

Lacie allows repository authors to attach Ruby lifecycle scripts for custom setup and teardown.

### Available Hooks

| Hook | Invocation Point | Purpose |
|---|---|---|
| `check_deps` | Before installation | Verifies system prerequisites, reporting missing libraries or tools. |
| `prebuild` | After download, before activation | Compiles, prepares, or unpacks extra assets. |
| `postbuild` | After binary activation | Generates default config, completions, or caches. |
| `preremove` | Before removal/deactivation | Stops running daemons, backs up state. |
| `postremove` | After binary deletion | Cleans up caches, temporary directories. |

### Execution Context & Constraints

Hooks are executed with Ruby in an isolated working directory inside `var/`.
- Working directory contains the package files.
- The environment provides package metadata and execution flags.
- Scripts must exit with status code `0` on success. Non-zero exit aborts the operation.

### Security & Integrity Pinning (`scripts.toml`)

To prevent supply-chain attacks:
1. When a package with `preremove` or `postremove` is installed, Lacie calculates the SHA256 hash of the scripts and saves them in `scripts.toml`.
2. When removing the package (`lacie remove`), Lacie re-hashes the script.
3. If the hash does not match `scripts.toml`, execution is blocked with error `E035`.
4. Users can skip script execution entirely using the `--no-scripts` flag.

---

## 4. Source Plugin Architecture

A source plugin connects Lacie to external registries, package indexes, or storage providers.

### Plugin Directory Layout

Source plugins reside in `plugins/{name}/`:
```
plugins/
└── custom_source/
    ├── source.toml       # Plugin manifest
    └── custom_source.rb  # Executable RPC entrypoint
```

---

### Plugin Manifest (`source.toml`)

```toml
name = "custom"
id = "org.example.custom"
author = "Your Name"
version = "0.1.0"
version_number = 1
api = 1
entry = "custom.rb"
capabilities = ["search"]
env = ["CUSTOM_API_KEY"]

[[files]]
path = "custom.rb"
executable = true
```

#### Fields Reference
- `name`: Identifier used in CLI package references (`custom:owner/repo`). Must be 2–32 characters, lowercase alphanumeric/hyphen.
- `id`: Unique reverse-DNS package identifier.
- `api`: Protocol version. Currently must be `1`.
- `entry`: Relative path to the executable script.
- `capabilities`: Supported optional operations (e.g. `["search"]`).
- `env`: List of environment variables forwarded to the plugin subprocess.
- `files`: File list for plugin distribution and validation.

---

### JSON-RPC over stdio Protocol

Lacie starts the plugin as a background subprocess and communicates via standard JSON-RPC 2.0 over standard I/O (`stdin` / `stdout`). Each message is a single line terminated with `\n`.

#### Request Structure
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "releases",
  "params": {
    "storage": "owner/repo"
  }
}
```

#### Response Structure
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": [ ... ]
}
```

#### Error Structure
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "Repository not found"
  }
}
```

---

### RPC Methods Reference

#### 1. `releases`
Fetches available releases for a given repository.
- **Params:** `{"storage": "owner/repo"}`
- **Returns:** Array of release objects:
  ```json
  [
    {
      "tag": "v1.2.0",
      "pre_release": false,
      "published_at": "2026-03-01T12:00:00Z",
      "has_manifest": true
    }
  ]
  ```

#### 2. `manifest`
Retrieves the raw `lacie.toml` content for a release.
- **Params:** `{"storage": "owner/repo", "release": "v1.2.0"}`
- **Returns:** String containing the raw TOML text.

#### 3. `search` (Optional, requires capability `"search"`)
Queries packages matching a search string.
- **Params:** `{"query": "search-term"}`
- **Returns:** Array of search results:
  ```json
  [
    {
      "storage": "owner/repo",
      "stars": 42
    }
  ]
  ```

#### 4. `fetch`
Downloads an asset directly to a destination path.
- **Params:** `{"storage": "owner/repo", "release": "v1.2.0", "file": "asset.tar.gz", "dest": "/absolute/path/dest"}`
- **Returns:** `null` on success.

---

### Custom Plugin Implementation Example

A minimal Ruby source plugin (`custom.rb`):

```ruby
#!/usr/bin/env ruby
require 'json'

STDOUT.sync = true

while (line = STDIN.gets)
  req = JSON.parse(line.strip)
  id = req['id']
  method = req['method']
  params = req['params'] || {}

  case method
  when 'releases'
    storage = params['storage']
    # Query your backend or API
    result = [
      {
        'tag' => '1.0.0',
        'pre_release' => false,
        'has_manifest' => true
      }
    ]
    puts JSON.generate({ 'jsonrpc' => '2.0', 'id' => id, 'result' => result })

  when 'manifest'
    # Return raw lacie.toml
    toml = <<~TOML
      [workspace]
      version = "1.0.0"

      [packages."org.custom.tool"]
      name = "tool"

      [packages."org.custom.tool".binaries."linux-gnu-x86_64"]
      links = ["https://custom.example.com/tool-linux"]
      sha256 = "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b"
    TOML
    puts JSON.generate({ 'jsonrpc' => '2.0', 'id' => id, 'result' => toml })

  when 'search'
    query = params['query']
    result = [
      { 'storage' => "custom/#{query}", 'stars' => 10 }
    ]
    puts JSON.generate({ 'jsonrpc' => '2.0', 'id' => id, 'result' => result })

  else
    err = { 'code' => -32601, 'message' => "Method not found: #{method}" }
    puts JSON.generate({ 'jsonrpc' => '2.0', 'id' => id, 'error' => err })
  end
end
```
