# tauri-latest-json

Generate a `latest.json` file for [Tauri](https://v2.tauri.app/) auto-updates, supporting **multi-platform** builds (Windows, macOS Intel/ARM, Linux).

This crate scans your Tauri `bundle` directory for installers, signs each one with your Tauri private key, and outputs a valid `latest.json` for the [Tauri Updater](https://v2.tauri.app/plugin/updater/).

## ✨ Features

- Supports `.msi`, `.exe`, `.dmg` (Intel & ARM), `tar.gz` (ARM) and `.AppImage`
- Automatically detects platform from installer filename
- Reads version from `package.json` or `Cargo.toml`
- Outputs a fully valid `latest.json` with multiple platforms
- Easy to integrate into CI/CD pipelines

## 📦 Installation

Add to `Cargo.toml`:

```toml
[dependencies]
tauri-latest-json = "0.2.1"
```

or

```bash
cargo add tauri-latest-json
```

## 🚀 Usage

```rust
// src/bin/latest_json.rs
use tauri_latest_json::generate_latest_json_auto;

fn main() {
    let download_url = "https://example.com/downloads";
    let release_notes = "Initial release";

    match generate_latest_json_auto(download_url, release_notes) {
        Ok(_) => println!("✅ latest.json generated successfully"),
        Err(e) => eprintln!("❌ Failed to generate latest.json: {e}"),
    }
}

```

```json
// package.json

  "scripts": {
    ...
    "tauri:build": "pnpm tauri build && cd src-tauri && cargo run --bin latest_json"
  },

```

After running `pnpm tauri:build`, you'll get:

```json
{
  "version": "1.0.0",
  "notes": "Initial release",
  "pub_date": "2025-08-19T19:44:22Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "base64-signature-here",
      "url": "https://example.com/downloads/app_1.0.0_x64_en-US.msi"
    },
    "darwin-x86_64": {
      "signature": "...",
      "url": "..."
    },
    "linux-x86_64": {
      "signature": "...",
      "url": "..."
    }
  }
}
```

## 🔑 Requirements

- A valid Tauri updater setup:
  See on [Tauri Updater](https://v2.tauri.app/plugin/updater/)

- A valid Tauri private key:

  ```bash
  pnpm tauri signer generate -w ~/.tauri/myapp.key
  ```

## 📦 Run the example

```bash
cargo run --example basic
```

If the paths are correct, you’ll get:

```
✅ latest.json generated successfully
```

And a `latest.json` file in your project root.

## 🛠 Platform detection

| File Extension  | Platform Key     |
| --------------- | ---------------- |
| `.msi`, `.exe`  | `windows-x86_64` |
| `.dmg` (Intel)  | `darwin-x86_64`  |
| `.dmg` (ARM)    | `darwin-aarch64` |
| `.tar.gz` (ARM) | `darwin-aarch64` |
| `.AppImage`     | `linux-x86_64`   |

## 📄 License

Licensed under the MIT License — see [LICENSE](LICENSE) for details.

🦀 Contributing Rust Community by [Shayy](https://www.codewithshayy.online/me)
