use clap::{CommandFactory, Parser};
use colored::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The base URL where installers are hosted (e.g., https://github.com/user/repo/releases/download/v1.0.0)
    download_url_base: Option<String>,

    /// Release notes for this update
    #[arg(trailing_var_arg = true)]
    notes: Vec<String>,
}

fn main() {
    let args = Args::parse();

    // Keep positional help/version for backward compatibility with existing scripts.
    if let Some(ref first_arg) = args.download_url_base {
        match first_arg.as_str() {
            "help" => {
                Args::command().print_help().unwrap();
                return;
            }
            "version" => {
                println!("tauri-latest-json {}", tauri_latest_json::VERSION);
                return;
            }
            _ => {}
        }
    }

    if let Err(e) = tauri_latest_json::run_with_optional_args(args.download_url_base, args.notes) {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}
