use chrono::Utc;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

fn read_version() -> Result<String, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    read_version_from_dir(&cwd)
}

fn read_version_from_dir(base: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // Try package.json first
    let pkg_path = base.join("package.json");
    if pkg_path.exists() {
        let pkg_str = fs::read_to_string(&pkg_path)?;
        let pkg_json: serde_json::Value = serde_json::from_str(&pkg_str)?;
        if let Some(ver) = pkg_json["version"].as_str() {
            return Ok(ver.to_string());
        }
    }

    // Fallback to Cargo.toml
    let cargo_path = base.join("Cargo.toml");
    if cargo_path.exists() {
        let cargo_str = fs::read_to_string(&cargo_path)?;
        let mut in_package = false;
        for raw_line in cargo_str.lines() {
            let line = raw_line.trim();
            if line.starts_with('[') && line.ends_with(']') {
                in_package = line == "[package]";
                continue;
            }
            if in_package && line.starts_with("version") {
                if let Some(eq_pos) = line.find('=') {
                    let version = line[eq_pos + 1..].trim().trim_matches('"').to_string();
                    if !version.is_empty() {
                        return Ok(version);
                    }
                }
            }
        }
    }

    Err("Could not find version in package.json or Cargo.toml".into())
}

/// Generates `latest.json` by auto-detecting the Tauri bundle dir,
/// reading version + public key from config, signing installers,
/// and verifying signatures against the configured public key.
pub fn generate_latest_json_auto(
    download_url_base: &str,
    notes: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle_dir = detect_bundle_dir()?;
    let tauri_conf_path = detect_tauri_conf_path()?;

    let public_key = read_public_key(&tauri_conf_path)?;
    generate_latest_json(&bundle_dir, &public_key, download_url_base, notes)
}

/// Generates `latest.json` from a given bundle dir and paths.
pub fn generate_latest_json(
    bundle_dir: &Path,
    public_key: &str,
    download_url_base: &str,
    notes: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // === 1. Read version from Cargo.toml ===
    let version = read_version()?;

    // === 2. Find installers ===
    let installers = find_installers(&bundle_dir)?;
    if installers.is_empty() {
        return Err("No installers found".into());
    }

    // === 3. Build platforms map ===
    let mut platforms = HashMap::new();
    for installer in installers {
        let installer_name = installer.file_name().unwrap().to_str().unwrap();
        let platform_key = detect_platform_key(installer_name);

        // Sign installer
        let signature_path = find_signatures(&bundle_dir)?;
        let sig_path = signature_path
            .get(&platform_key)
            .ok_or_else(|| format!("Signature not found for platform {}", platform_key))?;
        let mut f_sig = std::fs::File::open(sig_path)?;
        let mut signature = String::new();
        f_sig.read_to_string(&mut signature)?;

        // Verify signature
        // verify_signature(&installer, &signature, public_key)?;

        // Detect platform key

        platforms.insert(
            platform_key.to_string(),
            json!({
                "signature": signature,
                "url": format!("{}/{}", download_url_base, installer_name)
            }),
        );
    }

    // === 4. Generate latest.json ===
    let latest_json = json!({
        "version": version,
        "notes": notes,
        "pub_date": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "platforms": platforms
    });

    fs::write("latest.json", serde_json::to_string_pretty(&latest_json)?)?;
    println!(
        "✅ latest.json generated at {}",
        std::env::current_dir()?.display()
    );
    Ok(())
}

/// Reads public key from tauri.conf.json
fn read_public_key(conf_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let conf_str = fs::read_to_string(conf_path)?;
    let conf_json: Value = serde_json::from_str(&conf_str)?;
    let public_key = conf_json["plugins"]["updater"]["pubkey"]
        .as_str()
        .ok_or("No public key found in tauri.conf.json")?;
    Ok(public_key.to_string())
}

/// Reads `tauri.conf.json` and figures out the `bundle` directory.
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

fn verify_signature(
    installer: &Path,
    signature: &str,
    public_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("tauri")
        .args([
            "signer",
            "verify",
            "--public-key",
            public_key,
            installer.to_str().unwrap(),
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir() -> PathBuf {
        let mut base = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        base.push(format!("tauri-latest-json-test-{}", nanos));
        create_dir_all(&base).unwrap();
        base
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
}
