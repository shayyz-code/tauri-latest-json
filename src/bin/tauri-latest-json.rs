use chrono::Utc;
use serde_json::{json, Value};
#[cfg(feature = "verify-signature")]
use std::process::Command;
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const HELP_TEXT: &str = "\
tauri-latest-json

Usage:
  tauri-latest-json <download_url_base> <notes...>
  tauri-latest-json help
  tauri-latest-json version

Options:
  -h, --help       Show help
  -V, --version    Show version
";

enum CliAction {
    Help,
    Version,
    Generate { download_url: String, notes: String },
}

fn parse_args<I, S>(args: I) -> Result<CliAction, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let parts: Vec<String> = args.into_iter().map(Into::into).collect();
    if parts.is_empty() {
        return Err("missing arguments".to_string());
    }

    match parts[0].as_str() {
        "-h" | "--help" | "help" => Ok(CliAction::Help),
        "-V" | "--version" | "version" => Ok(CliAction::Version),
        _ => {
            if parts.len() < 2 {
                return Err("missing notes".to_string());
            }
            let download_url = parts[0].clone();
            let notes = parts[1..].join(" ");
            Ok(CliAction::Generate {
                download_url,
                notes,
            })
        }
    }
}

fn main() {
    match parse_args(std::env::args().skip(1)) {
        Ok(CliAction::Help) => {
            print!("{HELP_TEXT}");
        }
        Ok(CliAction::Version) => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
        Ok(CliAction::Generate {
            download_url,
            notes,
        }) => match generate_latest_json_auto(&download_url, &notes) {
            Ok(()) => println!("latest.json generated successfully"),
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        Err(_) => {
            eprintln!("{HELP_TEXT}");
            std::process::exit(1);
        }
    }
}

fn read_version_from_dir(base: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let pkg_path = base.join("package.json");
    if pkg_path.exists() {
        let pkg_str = fs::read_to_string(&pkg_path)?;
        let pkg_json: serde_json::Value = serde_json::from_str(&pkg_str)?;
        if let Some(ver) = pkg_json["version"].as_str() {
            return Ok(ver.to_string());
        }
    }

    let cargo_path = base.join("Cargo.toml");
    if cargo_path.exists() {
        let cargo_str = fs::read_to_string(&cargo_path)?;
        let value: toml::Value = toml::from_str(&cargo_str)?;
        if let Some(pkg) = value.get("package") {
            if let Some(ver) = pkg.get("version").and_then(|v| v.as_str()) {
                if !ver.is_empty() {
                    return Ok(ver.to_string());
                }
            }
        }
    }

    Err("Could not find version in package.json or Cargo.toml".into())
}

fn generate_latest_json_auto(
    download_url_base: &str,
    notes: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::env::current_dir()?;
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
) -> Result<(), Box<dyn std::error::Error>> {
    let version = read_version_from_dir(project_dir)?;

    let installers = find_installers(bundle_dir)?;
    if installers.is_empty() {
        return Err("No installers found".into());
    }

    let signature_paths = find_signatures(bundle_dir)?;
    let mut platforms = HashMap::new();
    for installer in installers {
        let installer_name = match installer
            .file_name()
            .and_then(|s| s.to_str().map(|s| s.to_string()))
        {
            Some(s) => s,
            None => continue,
        };
        let platform_key = detect_platform_key(installer_name.as_str());

        let sig_path = signature_paths
            .get(platform_key)
            .ok_or_else(|| format!("Signature not found for platform {}", platform_key))?;
        let mut f_sig = std::fs::File::open(sig_path)?;
        let mut signature = String::new();
        f_sig.read_to_string(&mut signature)?;

        #[cfg(feature = "verify-signature")]
        {
            verify_signature(&installer, &signature, public_key)?;
        }
        #[cfg(not(feature = "verify-signature"))]
        {
            let _ = &public_key;
        }

        platforms.insert(
            platform_key.to_string(),
            json!({
                "signature": signature,
                "url": format!("{}/{}", download_url_base, installer_name)
            }),
        );
    }

    let latest_json = json!({
        "version": version,
        "notes": notes,
        "pub_date": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "platforms": platforms
    });

    let output_path = project_dir.join("latest.json");
    fs::write(&output_path, serde_json::to_string_pretty(&latest_json)?)?;
    println!("latest.json generated at {}", output_path.display());
    Ok(())
}

fn read_public_key(conf_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let conf_str = fs::read_to_string(conf_path)?;
    let conf_json: Value = serde_json::from_str(&conf_str)?;
    let public_key = conf_json["plugins"]["updater"]["pubkey"]
        .as_str()
        .ok_or("No public key found in tauri.conf.json")?;
    Ok(public_key.to_string())
}

fn detect_bundle_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
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

    Err("Could not detect bundle dir. Build your Tauri app to produce target/*/bundle.".into())
}

fn find_installers(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
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

fn find_signatures(
    dir: &Path,
) -> Result<HashMap<&'static str, PathBuf>, Box<dyn std::error::Error>> {
    let mut results = HashMap::new();
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let fname = entry.file_name().to_string_lossy();
            if fname.ends_with(".sig") {
                let platform = detect_platform_key(&fname.replace(".sig", ""));
                results.insert(platform, entry.path().to_path_buf());
            }
        }
    }
    Ok(results)
}

fn detect_tauri_conf_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
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
    Err("Could not find tauri.conf.json. Provide it at project root or src-tauri/.".into())
}

#[cfg(feature = "verify-signature")]
fn verify_signature(
    installer: &Path,
    signature: &str,
    public_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let installer_str = installer
        .to_str()
        .ok_or("Installer path is not valid UTF-8")?;
    let output = Command::new("tauri")
        .args([
            "signer",
            "verify",
            "--public-key",
            public_key,
            installer_str,
            signature,
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Signature verification failed for {:?}: {}",
            installer,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

fn detect_platform_key(filename: &str) -> &'static str {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".msi") || lower.ends_with(".exe") {
        "windows-x86_64"
    } else if lower.ends_with(".dmg") {
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
    fn test_parse_args_help_and_version() {
        assert!(matches!(parse_args(vec!["help"]), Ok(CliAction::Help)));
        assert!(matches!(parse_args(vec!["--help"]), Ok(CliAction::Help)));
        assert!(matches!(parse_args(vec!["-h"]), Ok(CliAction::Help)));
        assert!(matches!(
            parse_args(vec!["version"]),
            Ok(CliAction::Version)
        ));
        assert!(matches!(
            parse_args(vec!["--version"]),
            Ok(CliAction::Version)
        ));
        assert!(matches!(parse_args(vec!["-V"]), Ok(CliAction::Version)));
    }

    #[test]
    fn test_parse_args_generate() {
        let parsed = parse_args(vec!["https://example.com", "Initial", "release"]).unwrap();
        match parsed {
            CliAction::Generate {
                download_url,
                notes,
            } => {
                assert_eq!(download_url, "https://example.com");
                assert_eq!(notes, "Initial release");
            }
            _ => panic!("expected generate action"),
        }
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
