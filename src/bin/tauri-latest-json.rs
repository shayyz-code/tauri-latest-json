use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clap::Parser;
use colored::*;
use dialoguer::Input;
use serde_json::{json, Value};
#[cfg(feature = "verify-signature")]
use std::process::Command;
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Generate multi-platform Tauri updater latest.json from built installers
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

    // Handle 'help' and 'version' as positional arguments for backward compatibility
    if let Some(ref first_arg) = args.download_url_base {
        match first_arg.as_str() {
            "help" => {
                use clap::CommandFactory;
                Args::command().print_help().unwrap();
                return;
            }
            "version" => {
                println!("tauri-latest-json {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            _ => {}
        }
    }

    let is_tty = console::Term::stdout().is_term();

    let download_url_base = match args.download_url_base {
        Some(url) => url,
        None => {
            if !is_tty {
                eprintln!(
                    "{} Argument 'download_url_base' missing and not in a terminal.",
                    "error:".red().bold()
                );
                std::process::exit(1);
            }
            println!(
                "{} Argument 'download_url_base' missing. Entering interactive mode...",
                "info:".cyan()
            );
            Input::<String>::new()
                .with_prompt("Enter the download URL base")
                .interact_text()
                .unwrap_or_else(|_| {
                    eprintln!("{} Failed to read input", "error:".red().bold());
                    std::process::exit(1);
                })
        }
    };

    let notes = if args.notes.is_empty() {
        if !is_tty {
            eprintln!(
                "{} Argument 'notes' missing and not in a terminal.",
                "error:".red().bold()
            );
            std::process::exit(1);
        }
        Input::<String>::new()
            .with_prompt("Enter the release notes")
            .interact_text()
            .unwrap_or_else(|_| {
                eprintln!("{} Failed to read input", "error:".red().bold());
                std::process::exit(1);
            })
    } else {
        args.notes.join(" ")
    };

    if let Err(e) = generate_latest_json_auto(&download_url_base, &notes) {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn read_version_from_dir(base: &Path) -> Result<String> {
    let pkg_path = base.join("package.json");
    if pkg_path.exists() {
        let pkg_str = fs::read_to_string(&pkg_path).context("failed to read package.json")?;
        let pkg_json: serde_json::Value =
            serde_json::from_str(&pkg_str).context("failed to parse package.json")?;
        if let Some(ver) = pkg_json["version"].as_str() {
            return Ok(ver.to_string());
        }
    }

    let tauri_conf_path = base.join("tauri.conf.json");
    if tauri_conf_path.exists() {
        let conf_str =
            fs::read_to_string(&tauri_conf_path).context("failed to read tauri.conf.json")?;
        let conf_json: serde_json::Value =
            serde_json::from_str(&conf_str).context("failed to parse tauri.conf.json")?;
        if let Some(ver) = conf_json["package"]["version"].as_str() {
            return Ok(ver.to_string());
        }
        if let Some(ver) = conf_json["version"].as_str() {
            return Ok(ver.to_string());
        }
    }

    let src_tauri_conf_path = base.join("src-tauri").join("tauri.conf.json");
    if src_tauri_conf_path.exists() {
        let conf_str = fs::read_to_string(&src_tauri_conf_path)
            .context("failed to read src-tauri/tauri.conf.json")?;
        let conf_json: serde_json::Value =
            serde_json::from_str(&conf_str).context("failed to parse src-tauri/tauri.conf.json")?;
        if let Some(ver) = conf_json["package"]["version"].as_str() {
            return Ok(ver.to_string());
        }
        if let Some(ver) = conf_json["version"].as_str() {
            return Ok(ver.to_string());
        }
    }

    let cargo_path = base.join("Cargo.toml");
    if cargo_path.exists() {
        let cargo_str = fs::read_to_string(&cargo_path).context("failed to read Cargo.toml")?;
        let value: toml::Value =
            toml::from_str(&cargo_str).context("failed to parse Cargo.toml")?;
        if let Some(pkg) = value.get("package") {
            if let Some(ver) = pkg.get("version").and_then(|v| v.as_str()) {
                if !ver.is_empty() {
                    return Ok(ver.to_string());
                }
            }
        }
    }

    let src_tauri_cargo_path = base.join("src-tauri").join("Cargo.toml");
    if src_tauri_cargo_path.exists() {
        let cargo_str = fs::read_to_string(&src_tauri_cargo_path)
            .context("failed to read src-tauri/Cargo.toml")?;
        let value: toml::Value =
            toml::from_str(&cargo_str).context("failed to parse src-tauri/Cargo.toml")?;
        if let Some(pkg) = value.get("package") {
            if let Some(ver) = pkg.get("version").and_then(|v| v.as_str()) {
                if !ver.is_empty() {
                    return Ok(ver.to_string());
                }
            }
        }
    }

    Err(anyhow!(
        "Could not find version in package.json, Cargo.toml, or tauri.conf.json"
    ))
}

fn generate_latest_json_auto(download_url_base: &str, notes: &str) -> Result<()> {
    let bundle_dir = detect_bundle_dir()?;
    let tauri_conf_path = detect_tauri_conf_path()?;
    let public_key = read_public_key(&tauri_conf_path)?;
    generate_latest_json(&bundle_dir, &public_key, download_url_base, notes)
}

fn generate_latest_json(
    bundle_dir: &Path,
    public_key: &str,
    download_url_base: &str,
    notes: &str,
) -> Result<()> {
    let project_dir = std::env::current_dir().context("failed to get current directory")?;
    generate_latest_json_for_project(
        bundle_dir,
        public_key,
        download_url_base,
        notes,
        &project_dir,
    )
}

fn generate_latest_json_for_project(
    bundle_dir: &Path,
    public_key: &str,
    download_url_base: &str,
    notes: &str,
    project_dir: &Path,
) -> Result<()> {
    let version = read_version_from_dir(project_dir)?;
    println!("{} detected version: {}", "info:".cyan(), version.bold());

    let installers = find_installers_by_platform(bundle_dir)?;
    if installers.is_empty() {
        return Err(anyhow!("No installers found in {}", bundle_dir.display()));
    }

    let signature_paths = find_signatures(bundle_dir)?;
    let mut platforms = HashMap::new();
    for (platform_key, installer) in installers {
        let installer_name = match installer
            .file_name()
            .and_then(|s| s.to_str().map(|s| s.to_string()))
        {
            Some(s) => s,
            None => continue,
        };

        let sig_path = signature_paths.get(platform_key.as_str());

        if sig_path.is_none() {
            if installer_name.ends_with(".dmg") {
                eprintln!("{} No signature found for DMG on platform {}. Tauri doesn't generate .sig files for DMG, so it will be skipped.", "warning:".yellow().bold(), platform_key.bold());
                continue;
            } else {
                return Err(anyhow!("Signature not found for platform {}", platform_key));
            }
        }

        let mut f_sig = std::fs::File::open(sig_path.unwrap()).context(format!(
            "failed to open signature file for {}",
            platform_key
        ))?;
        let mut signature = String::new();
        f_sig
            .read_to_string(&mut signature)
            .context(format!("failed to read signature for {}", platform_key))?;

        #[cfg(feature = "verify-signature")]
        {
            verify_signature(&installer, &signature, public_key)?;
        }
        #[cfg(not(feature = "verify-signature"))]
        {
            let _ = &public_key;
        }

        println!(
            "{} matched platform {}: {}",
            "success:".green(),
            platform_key.bold(),
            installer_name.dimmed()
        );

        platforms.insert(
            platform_key,
            json!({
                "signature": signature.trim(),
                "url": format!("{}/{}", download_url_base, installer_name)
            }),
        );
    }

    if platforms.is_empty() {
        return Err(anyhow!(
            "No platforms with valid signatures found. Cannot generate latest.json."
        ));
    }

    let latest_json = json!({
        "version": version,
        "notes": notes,
        "pub_date": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "platforms": platforms
    });

    let output_path = project_dir.join("latest.json");
    fs::write(&output_path, serde_json::to_string_pretty(&latest_json)?)
        .context("failed to write latest.json")?;
    println!(
        "\n{} generated at {}",
        "✔".green().bold(),
        output_path.display().to_string().bold()
    );
    Ok(())
}

fn read_public_key(conf_path: &Path) -> Result<String> {
    let conf_str = fs::read_to_string(conf_path).context("failed to read tauri.conf.json")?;
    let conf_json: Value =
        serde_json::from_str(&conf_str).context("failed to parse tauri.conf.json")?;

    // Try Tauri 2.0 path: plugins > updater > pubkey
    if let Some(pubkey) = conf_json["plugins"]["updater"]["pubkey"].as_str() {
        return Ok(pubkey.to_string());
    }

    // Try Tauri 1.0 path: tauri > updater > pubkey
    if let Some(pubkey) = conf_json["tauri"]["updater"]["pubkey"].as_str() {
        return Ok(pubkey.to_string());
    }

    Err(anyhow!("No public key found in tauri.conf.json (checked plugins.updater.pubkey and tauri.updater.pubkey)"))
}

fn detect_bundle_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("failed to get current directory")?;
    let candidates = [
        current_dir.join("target").join("release").join("bundle"),
        current_dir
            .join("src-tauri")
            .join("target")
            .join("release")
            .join("bundle"),
        current_dir
            .join("..")
            .join("src-tauri")
            .join("target")
            .join("release")
            .join("bundle"),
        current_dir.join("target").join("debug").join("bundle"),
        current_dir
            .join("src-tauri")
            .join("target")
            .join("debug")
            .join("bundle"),
    ];

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow!(
        "Could not detect bundle dir. Build your Tauri app to produce target/*/bundle."
    ))
}

fn find_installers(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.context("failed to read directory entry")?;
        if entry.file_type().is_file() {
            let fname = entry.file_name().to_string_lossy();
            if fname.ends_with(".msi")
                || fname.ends_with(".exe")
                || fname.ends_with(".dmg")
                || fname.ends_with(".AppImage")
                || fname.ends_with(".deb")
                || fname.ends_with(".rpm")
                || fname.ends_with(".tar.gz")
            {
                results.push(entry.path().to_path_buf());
            }
        }
    }
    Ok(results)
}

fn installer_priority(platform: &str, filename: &str) -> u8 {
    let lower = filename.to_ascii_lowercase();
    match platform {
        "windows-x86_64" => {
            if lower.ends_with(".msi") {
                30
            } else if lower.ends_with(".exe") {
                20
            } else {
                10
            }
        }
        "darwin-aarch64" | "darwin-x86_64" => {
            if lower.ends_with(".app.tar.gz") {
                40
            } else if lower.ends_with(".dmg") {
                10
            } else {
                5
            }
        }
        "linux-aarch64" | "linux-x86_64" => {
            if lower.ends_with(".appimage") {
                40
            } else if lower.ends_with(".deb") {
                30
            } else if lower.ends_with(".rpm") {
                20
            } else if lower.ends_with(".tar.gz") {
                10
            } else {
                5
            }
        }
        _ => 0,
    }
}

fn find_installers_by_platform(dir: &Path) -> Result<HashMap<String, PathBuf>> {
    let installers = find_installers(dir)?;
    let mut selected: HashMap<String, (PathBuf, u8)> = HashMap::new();

    for installer in installers {
        let installer_name = match installer.file_name().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let platform = detect_platform_key(&installer_name).to_string();
        if platform == "unknown" {
            continue;
        }
        let priority = installer_priority(&platform, &installer_name);
        match selected.get(&platform) {
            Some((_, existing_priority)) if *existing_priority >= priority => {}
            _ => {
                selected.insert(platform, (installer, priority));
            }
        }
    }

    Ok(selected.into_iter().map(|(k, (v, _))| (k, v)).collect())
}

fn find_signatures(dir: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut results = HashMap::new();
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.context("failed to read directory entry")?;
        if entry.file_type().is_file() {
            let fname = entry.file_name().to_string_lossy();
            if fname.ends_with(".sig") {
                let platform = detect_platform_key(&fname.replace(".sig", ""));
                results.insert(platform.to_string(), entry.path().to_path_buf());
            }
        }
    }
    Ok(results)
}

fn detect_tauri_conf_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("failed to get current directory")?;
    let candidates = [
        current_dir.join("tauri.conf.json"),
        current_dir.join("src-tauri").join("tauri.conf.json"),
        current_dir
            .join("..")
            .join("src-tauri")
            .join("tauri.conf.json"),
    ];
    for c in candidates {
        if c.exists() {
            return Ok(c);
        }
    }
    Err(anyhow!(
        "Could not find tauri.conf.json. Provide it at project root or src-tauri/."
    ))
}

#[cfg(feature = "verify-signature")]
fn verify_signature(installer: &Path, signature: &str, public_key: &str) -> Result<()> {
    let installer_str = installer
        .to_str()
        .ok_or_else(|| anyhow!("Installer path is not valid UTF-8"))?;
    let output = Command::new("tauri")
        .args([
            "signer",
            "verify",
            "--public-key",
            public_key,
            installer_str,
            signature,
        ])
        .output()
        .context("failed to execute tauri signer verify")?;

    if !output.status.success() {
        return Err(anyhow!(
            "Signature verification failed for {:?}: {}",
            installer,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn detect_platform_key(filename: &str) -> &'static str {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".msi") || lower.ends_with(".exe") {
        "windows-x86_64"
    } else if lower.ends_with(".app.tar.gz") || lower.ends_with(".dmg") {
        if lower.contains("aarch64") || lower.contains("arm64") {
            "darwin-aarch64"
        } else {
            "darwin-x86_64"
        }
    } else if lower.ends_with(".appimage")
        || lower.ends_with(".deb")
        || lower.ends_with(".rpm")
        || lower.ends_with(".tar.gz")
    {
        if lower.contains("aarch64") || lower.contains("arm64") {
            "linux-aarch64"
        } else {
            "linux-x86_64"
        }
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    #[cfg(not(feature = "verify-signature"))]
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(not(feature = "verify-signature"))]
    static CWD_LOCK: Mutex<()> = Mutex::new(());
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[cfg(not(feature = "verify-signature"))]
    struct CurrentDirGuard {
        original: PathBuf,
    }

    #[cfg(not(feature = "verify-signature"))]
    impl CurrentDirGuard {
        fn switch_to(path: &Path) -> Self {
            let original = std::env::current_dir().unwrap();
            std::env::set_current_dir(path).unwrap();
            Self { original }
        }
    }

    #[cfg(not(feature = "verify-signature"))]
    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.original).unwrap();
        }
    }

    fn make_temp_dir() -> PathBuf {
        let mut base = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        base.push(format!(
            "tauri-latest-json-test-{}-{}-{}",
            std::process::id(),
            nanos,
            seq
        ));
        create_dir_all(&base).unwrap();
        base
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn test_args_parsing() {
        // Test valid arguments
        let args = Args::try_parse_from(vec![
            "tauri-latest-json",
            "https://example.com",
            "Release",
            "notes",
        ])
        .unwrap();
        assert_eq!(
            args.download_url_base,
            Some("https://example.com".to_string())
        );
        assert_eq!(args.notes, vec!["Release", "notes"]);

        // Test missing arguments (now allowed for interactive fallback)
        let result = Args::try_parse_from(vec!["tauri-latest-json"]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert!(args.download_url_base.is_none());
        assert!(args.notes.is_empty());
    }

    #[test]
    fn test_detect_platform_key_variants() {
        assert_eq!(
            detect_platform_key("app_0.1.0_x64_en-US.msi"),
            "windows-x86_64"
        );
        assert_eq!(
            detect_platform_key("app_0.1.0_x64_en-US.exe"),
            "windows-x86_64"
        );
        assert_eq!(detect_platform_key("app_0.1.0_x64.dmg"), "darwin-x86_64");
        assert_eq!(detect_platform_key("app_0.1.0_arm64.dmg"), "darwin-aarch64");
        assert_eq!(
            detect_platform_key("app_0.1.0_aarch64.app.tar.gz"),
            "darwin-aarch64"
        );
        assert_eq!(
            detect_platform_key("app_0.1.0_x64.app.tar.gz"),
            "darwin-x86_64"
        );
        assert_eq!(
            detect_platform_key("AppImage-0.1.0-x86_64.AppImage"),
            "linux-x86_64"
        );
        assert_eq!(
            detect_platform_key("AppImage-0.1.0-arm64.AppImage"),
            "linux-aarch64"
        );
        assert_eq!(detect_platform_key("app_0.1.0_amd64.deb"), "linux-x86_64");
        assert_eq!(
            detect_platform_key("app_0.1.0_aarch64.rpm"),
            "linux-aarch64"
        );
        assert_eq!(detect_platform_key("app-0.1.0-x64.tar.gz"), "linux-x86_64");
        assert_eq!(detect_platform_key("unknown.bin"), "unknown");
    }

    #[test]
    fn test_read_version_prefers_package_json() {
        let dir = make_temp_dir();
        {
            let mut f = File::create(dir.join("Cargo.toml")).unwrap();
            writeln!(
                f,
                "[package]\nname = \"dummy\"\nversion = \"0.2.0\"\n\n[dependencies]\n"
            )
            .unwrap();
        }
        {
            let mut f = File::create(dir.join("package.json")).unwrap();
            writeln!(f, "{{\"name\":\"dummy\",\"version\":\"1.2.3\"}}").unwrap();
        }
        let v = read_version_from_dir(&dir).unwrap();
        assert_eq!(v, "1.2.3");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_read_version_from_cargo_toml() {
        let dir = make_temp_dir();
        {
            let mut f = File::create(dir.join("Cargo.toml")).unwrap();
            writeln!(
                f,
                "[package]\nname = \"dummy\"\nversion = \"9.9.9\"\n\n[dependencies]\n"
            )
            .unwrap();
        }
        let v = read_version_from_dir(&dir).unwrap();
        assert_eq!(v, "9.9.9");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_read_version_from_src_tauri_cargo_toml() {
        let dir = make_temp_dir();
        create_dir_all(dir.join("src-tauri")).unwrap();
        {
            let mut f = File::create(dir.join("src-tauri").join("Cargo.toml")).unwrap();
            writeln!(
                f,
                "[package]\nname = \"dummy\"\nversion = \"3.4.5\"\n\n[dependencies]\n"
            )
            .unwrap();
        }
        let v = read_version_from_dir(&dir).unwrap();
        assert_eq!(v, "3.4.5");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(not(feature = "verify-signature"))]
    #[test]
    fn test_generate_latest_json_writes_expected_structure() {
        let dir = make_temp_dir();
        let bundle_dir = dir.join("target").join("release").join("bundle");

        write_file(
            &dir.join("package.json"),
            r#"{"name":"dummy","version":"1.2.3"}"#,
        );
        write_file(
            &bundle_dir.join("app_1.2.3_x64_en-US.msi"),
            "windows installer",
        );
        write_file(
            &bundle_dir.join("app_1.2.3_x64_en-US.msi.sig"),
            "windows-signature",
        );
        write_file(&bundle_dir.join("app_1.2.3_arm64.dmg"), "mac installer");
        write_file(&bundle_dir.join("app_1.2.3_arm64.dmg.sig"), "mac-signature");

        generate_latest_json_for_project(
            &bundle_dir,
            "unused-public-key",
            "https://example.com/downloads",
            "release notes",
            &dir,
        )
        .unwrap();

        let latest_json = std::fs::read_to_string(dir.join("latest.json")).unwrap();
        let latest: Value = serde_json::from_str(&latest_json).unwrap();

        assert_eq!(latest["version"], "1.2.3");
        assert_eq!(latest["notes"], "release notes");
        assert!(latest["pub_date"].as_str().is_some());
        assert_eq!(
            latest["platforms"]["windows-x86_64"]["url"],
            "https://example.com/downloads/app_1.2.3_x64_en-US.msi"
        );
        assert_eq!(
            latest["platforms"]["windows-x86_64"]["signature"],
            "windows-signature"
        );
        assert_eq!(
            latest["platforms"]["darwin-aarch64"]["url"],
            "https://example.com/downloads/app_1.2.3_arm64.dmg"
        );
        assert_eq!(
            latest["platforms"]["darwin-aarch64"]["signature"],
            "mac-signature"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(not(feature = "verify-signature"))]
    #[test]
    fn test_generate_latest_json_prefers_mac_updater_archive_over_dmg() {
        let dir = make_temp_dir();
        let bundle_dir = dir.join("target").join("release").join("bundle");
        write_file(
            &dir.join("package.json"),
            r#"{"name":"dummy","version":"1.2.3"}"#,
        );
        write_file(&bundle_dir.join("app_1.2.3_arm64.dmg"), "mac installer");
        write_file(
            &bundle_dir.join("app_1.2.3_arm64.app.tar.gz"),
            "mac updater archive",
        );
        write_file(
            &bundle_dir.join("app_1.2.3_arm64.app.tar.gz.sig"),
            "mac-signature",
        );

        generate_latest_json_for_project(
            &bundle_dir,
            "unused-public-key",
            "https://example.com/downloads",
            "release notes",
            &dir,
        )
        .unwrap();

        let latest_json = std::fs::read_to_string(dir.join("latest.json")).unwrap();
        let latest: Value = serde_json::from_str(&latest_json).unwrap();
        assert_eq!(
            latest["platforms"]["darwin-aarch64"]["url"],
            "https://example.com/downloads/app_1.2.3_arm64.app.tar.gz"
        );
        assert_eq!(
            latest["platforms"]["darwin-aarch64"]["signature"],
            "mac-signature"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_generate_latest_json_returns_error_when_no_installers() {
        let dir = make_temp_dir();
        let bundle_dir = dir.join("target").join("release").join("bundle");
        create_dir_all(&bundle_dir).unwrap();
        write_file(
            &dir.join("package.json"),
            r#"{"name":"dummy","version":"1.2.3"}"#,
        );

        let err = generate_latest_json_for_project(
            &bundle_dir,
            "unused-public-key",
            "https://example.com/downloads",
            "release notes",
            &dir,
        )
        .unwrap_err();
        assert!(err.to_string().contains("No installers found"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_generate_latest_json_returns_error_when_signature_is_missing() {
        let dir = make_temp_dir();
        let bundle_dir = dir.join("target").join("release").join("bundle");
        write_file(
            &dir.join("package.json"),
            r#"{"name":"dummy","version":"1.2.3"}"#,
        );
        write_file(
            &bundle_dir.join("app_1.2.3_x64_en-US.msi"),
            "windows installer",
        );

        let err = generate_latest_json_for_project(
            &bundle_dir,
            "unused-public-key",
            "https://example.com/downloads",
            "release notes",
            &dir,
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("Signature not found for platform windows-x86_64"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(not(feature = "verify-signature"))]
    #[test]
    fn test_generate_latest_json_public_api_uses_cwd_for_version_and_output() {
        let _cwd_guard = CWD_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let dir = make_temp_dir();
        let bundle_dir = dir.join("target").join("release").join("bundle");
        write_file(
            &dir.join("package.json"),
            r#"{"name":"dummy","version":"7.8.9"}"#,
        );
        write_file(
            &bundle_dir.join("app_7.8.9_x64_en-US.msi"),
            "windows installer",
        );
        write_file(
            &bundle_dir.join("app_7.8.9_x64_en-US.msi.sig"),
            "windows-signature",
        );

        {
            let _project_dir_guard = CurrentDirGuard::switch_to(&dir);
            generate_latest_json(
                &bundle_dir,
                "unused-public-key",
                "https://example.com/downloads",
                "public-api notes",
            )
            .unwrap();
        }

        let latest_json = std::fs::read_to_string(dir.join("latest.json")).unwrap();
        let latest: Value = serde_json::from_str(&latest_json).unwrap();
        assert_eq!(latest["version"], "7.8.9");
        assert_eq!(latest["notes"], "public-api notes");
        assert_eq!(
            latest["platforms"]["windows-x86_64"]["url"],
            "https://example.com/downloads/app_7.8.9_x64_en-US.msi"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(not(feature = "verify-signature"))]
    #[test]
    fn test_generate_latest_json_auto_detects_paths_and_reads_pubkey() {
        let _cwd_guard = CWD_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let dir = make_temp_dir();
        let bundle_dir = dir.join("target").join("release").join("bundle");
        write_file(
            &dir.join("package.json"),
            r#"{"name":"dummy","version":"2.0.0"}"#,
        );
        write_file(
            &dir.join("tauri.conf.json"),
            r#"{"plugins":{"updater":{"pubkey":"test-pubkey"}}}"#,
        );
        write_file(&bundle_dir.join("app_2.0.0_arm64.dmg"), "mac installer");
        write_file(&bundle_dir.join("app_2.0.0_arm64.dmg.sig"), "mac-signature");

        {
            let _project_dir_guard = CurrentDirGuard::switch_to(&dir);
            generate_latest_json_auto("https://example.com/downloads", "auto notes").unwrap();
        }

        let latest_json = std::fs::read_to_string(dir.join("latest.json")).unwrap();
        let latest: Value = serde_json::from_str(&latest_json).unwrap();
        assert_eq!(latest["version"], "2.0.0");
        assert_eq!(
            latest["platforms"]["darwin-aarch64"]["signature"],
            "mac-signature"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
