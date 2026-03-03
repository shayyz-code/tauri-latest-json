# tauri-latest-json

[![Crates.io](https://img.shields.io/crates/v/tauri-latest-json.svg)](https://crates.io/crates/tauri-latest-json)
[![docs.rs](https://docs.rs/tauri-latest-json/badge.svg)](https://docs.rs/tauri-latest-json)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/d/tauri-latest-json.svg)](https://crates.io/crates/tauri-latest-json)

Generate a `latest.json` file for [Tauri](https://v2.tauri.app/) auto-updates, supporting multi-platform builds (Windows, macOS Intel/ARM, Linux).

This crate scans your Tauri `bundle` directory for installers and outputs a valid `latest.json` for the [Tauri Updater](https://v2.tauri.app/plugin/updater/).

## Features

- Detects installers: `.msi`, `.exe`, `.dmg` (Intel/ARM), `.AppImage`, `.deb`, `.rpm`, `.tar.gz`
- Auto-detects platform keys from filenames
- Reads version from `package.json` or `Cargo.toml`
- Generates a single multi-platform `latest.json`
- Works as both a library and a CLI

## Install

Library:

```toml
[dependencies]
tauri-latest-json = "0.2.2"
```

CLI:

```bash
cargo install tauri-latest-json
```

## CLI Usage

```bash
tauri-latest-json <download_url_base> <notes>
```

Example:

```bash
tauri-latest-json https://example.com/downloads "Initial release"
```

`latest.json` is written to the current working directory.

## Library Usage

```rust
use tauri_latest_json::generate_latest_json_auto;

fn main() {
    let download_url = "https://example.com/downloads";
    let notes = "Initial release";
    generate_latest_json_auto(download_url, notes).unwrap();
}
```

## Example

```bash
cargo run --example basic
```

If the paths are correct, you’ll see:

```
✅ latest.json generated successfully
```

## Requirements

- Valid Tauri updater configuration (see the [Tauri Updater docs](https://v2.tauri.app/plugin/updater/))
- A Tauri signing key

```bash
pnpm tauri signer generate -w ~/.tauri/myapp.key
```

## Platform Detection

| File Extension                               | Platform Key     |
| -------------------------------------------- | ---------------- |
| `.msi`, `.exe`                               | `windows-x86_64` |
| `.dmg` (Intel)                               | `darwin-x86_64`  |
| `.dmg` (ARM)                                 | `darwin-aarch64` |
| `.AppImage`, `.deb`, `.rpm`, `.tar.gz` (x64) | `linux-x86_64`   |
| `.AppImage`, `.deb`, `.rpm`, `.tar.gz` (ARM) | `linux-aarch64`  |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Please also read our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT — see [LICENSE](LICENSE).
