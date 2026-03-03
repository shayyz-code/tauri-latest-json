use std::env;
use tauri_latest_json::generate_latest_json_auto;

fn main() {
    let mut args = env::args().skip(1);
    let download_url = match args.next() {
        Some(v) => v,
        None => {
            eprintln!("Usage: tauri-latest-json <download_url_base> <notes>");
            std::process::exit(1);
        }
    };
    let notes = match args.next() {
        Some(v) => v,
        None => {
            eprintln!("Usage: tauri-latest-json <download_url_base> <notes>");
            std::process::exit(1);
        }
    };

    match generate_latest_json_auto(&download_url, &notes) {
        Ok(()) => println!("latest.json generated successfully"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
