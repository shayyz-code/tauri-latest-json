<h1 align="center">tauri-latest-json</h1>

<p align="center">
    <a href="https://crates.io/crates/tauri-latest-json">
        <img src="https://img.shields.io/crates/v/tauri-latest-json.svg" alt="Crates.io" />
    </a>
    <a href="https://docs.rs/tauri-latest-json">
        <img src="https://docs.rs/tauri-latest-json/badge.svg" alt="docs.rs" />
    </a>
    <a href="LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License" />
    </a>
    <a href="https://crates.io/crates/tauri-latest-json">
        <img src="https://img.shields.io/crates/d/tauri-latest-json.svg" alt="Crates.io" />
    </a>
</p>

<p align="center">
Generate a <code>latest.json</code> file for <a href="https://v2.tauri.app/">Tauri</a> auto-updates, supporting multi-platform builds (Windows, macOS Intel/ARM, Linux). This CLI scans your Tauri <code>bundle</code> directory for installers and outputs a valid <code>latest.json</code> for the <a href="https://v2.tauri.app/plugin/updater/">Tauri Updater</a>.
</p>

## Features

- **Multi-platform detection**: Automatically finds `.msi`, `.exe`, `.dmg` (Intel/ARM), `.AppImage`, `.deb`, `.rpm`, and `.tar.gz` artifacts.
- **Interactive Mode**: Prompts for missing information (download URL, release notes) if arguments aren't provided.
- **Smart platform mapping**: Maps artifacts to their respective Tauri platform keys (`windows-x86_64`, `darwin-aarch64`, etc.).
- **Flexible Versioning**: Reads version from `package.json`, `Cargo.toml`, or `tauri.conf.json` (supports both Tauri 1.0 and 2.0 structures).
- **Root-run friendly**: Can be run from your project root or `src-tauri` directory.
- **Graceful Signature Handling**: Automatically skips artifacts without `.sig` files (like `.dmg` which Tauri doesn't sign for updates) with a helpful warning.
- **Verification Support**: Optional built-in signature verification against your public key.

## Installation

```bash
cargo install tauri-latest-json
```

## Usage

### 1. Simple Interactive Mode (Recommended)

Just run the command from your Tauri project root. It will prompt you for the download URL and release notes:

```bash
tauri-latest-json
```

### 2. Command Line Arguments

Provide the download URL base and release notes directly:

```bash
tauri-latest-json <download_url_base> <notes...>
```

**Example:**

```bash
tauri-latest-json https://github.com/user/repo/releases/download/v0.4.1 "Fixed security vulnerabilities and improved performance."
```

### 3. What happens next?

The tool will:

1. **Detect Version**: Scans `package.json`, `Cargo.toml`, or `tauri.conf.json`.
2. **Find Artifacts**: Searches `target/release/bundle` for installers.
3. **Verify Signatures**: Matches installers with their `.sig` files (skipping `.dmg` as expected).
4. **Generate Output**: Creates a `latest.json` file in your current directory, ready for upload.

## CLI Commands

```bash
tauri-latest-json help       # Show usage help
tauri-latest-json version    # Show version
```

## Platform Detection Logic

The tool prioritizes updater-compatible artifacts:

| Platform          | Priority Artifact | Extension Fallbacks          |
| ----------------- | ----------------- | ---------------------------- |
| **Windows**       | `.msi`            | `.exe`                       |
| **macOS (Intel)** | `.app.tar.gz`     | `.dmg` (skipped for updates) |
| **macOS (ARM)**   | `.app.tar.gz`     | `.dmg` (skipped for updates) |
| **Linux (x64)**   | `.AppImage`       | `.deb`, `.rpm`, `.tar.gz`    |
| **Linux (ARM)**   | `.AppImage`       | `.deb`, `.rpm`, `.tar.gz`    |

> **Note**: Tauri doesn't generate `.sig` files for `.dmg`. This tool will warn you and skip them for the updater JSON, as they aren't used for auto-updates.

## Requirements

- A valid Tauri project with `tauri.conf.json`.
- A configured `updater` plugin with a `pubkey`.
- Built artifacts in `target/release/bundle` or `src-tauri/target/release/bundle`.

## Development & Testing

### Running Tests

```bash
cargo test
make verify
```

### Smoke Testing

Validate against a local real Tauri app:

```bash
make smoke-real-app
# Or specify a custom directory
REAL_APP_DIR=/path/to/your-app ./scripts/smoke-real-tauri-app.sh
```

## License

MIT — see [LICENSE](LICENSE).
