use std::path::Path;
use tauri_latest_json::generate_latest_json_auto;

fn main() {
    // This example assumes you have:
    // 1. Built your Tauri app so `bundle` directory exists
    // 2. A valid Tauri private.pem key
    // 3. Either package.json or Cargo.toml with a version field

    let download_url = "https://example.com/downloads";
    let release_notes = "Initial release";

    match generate_latest_json_auto(download_url, release_notes) {
        Ok(_) => println!("✅ latest.json generated successfully"),
        Err(e) => eprintln!("❌ Failed to generate latest.json: {e}"),
    }
}
