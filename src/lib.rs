use chrono::Utc;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn read_version() -> Result<String, Box<dyn std::error::Error>> {
    // Try package.json first
    if Path::new("package.json").exists() {
        let pkg_str = fs::read_to_string("package.json")?;
        let pkg_json: serde_json::Value = serde_json::from_str(&pkg_str)?;
        if let Some(ver) = pkg_json["version"].as_str() {
            return Ok(ver.to_string());
        }
    }

    // Fallback to Cargo.toml
    if Path::new("Cargo.toml").exists() {
        let cargo_str = fs::read_to_string("Cargo.toml")?;
        for line in cargo_str.lines() {
            if let Some(rest) = line.strip_prefix("version") {
                if let Some(eq_pos) = rest.find('=') {
                    let version = rest[eq_pos + 1..].trim().trim_matches('"').to_string();
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
    private_key_path: &Path,
    download_url_base: &str,
    notes: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle_dir = detect_bundle_dir()?;
    let tauri_conf_path = Path::new("src-tauri/tauri.conf.json");

    let public_key = read_public_key(tauri_conf_path)?;
    generate_latest_json(
        &bundle_dir,
        &private_key_path,
        &public_key,
        download_url_base,
        notes,
    )
}

/// Generates `latest.json` from a given bundle dir and paths.
pub fn generate_latest_json(
    bundle_dir: &Path,
    private_key_path: &Path,
    public_key: &str,
    download_url_base: &str,
    notes: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // === 1. Read version from Cargo.toml ===
    let version = read_version()?;

    // === 2. Find installers ===
    let installers = find_installers(bundle_dir)?;
    if installers.is_empty() {
        return Err("No installers found".into());
    }

    // === 3. Build platforms map ===
    let mut platforms = HashMap::new();
    for installer in installers {
        let installer_name = installer.file_name().unwrap().to_str().unwrap();

        // Sign installer
        let signature = sign_installer(&installer, private_key_path)?;

        // Verify signature
        verify_signature(&installer, &signature, public_key)?;

        // Detect platform key
        let platform_key = detect_platform_key(installer_name);

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
    let public_key = conf_json["tauri"]["bundle"]["updater"]["pubkey"]
        .as_str()
        .ok_or("No public key found in tauri.conf.json")?;
    Ok(public_key.to_string())
}

/// Reads `tauri.conf.json` and figures out the `bundle` directory.
fn detect_bundle_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let bundle_dir = Path::new("src-tauri")
        .join("target")
        .join("release")
        .join("bundle");

    if bundle_dir.exists() {
        Ok(bundle_dir)
    } else {
        Err("Could not detect bundle dir. Run `npm run tauri build` first.".into())
    }
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
            {
                results.push(entry.path().to_path_buf());
            }
        }
    }
    Ok(results)
}

fn sign_installer(
    installer: &Path,
    private_key_path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("tauri")
        .args([
            "signer",
            "sign",
            "--private-key",
            &private_key_path.to_string_lossy(),
            installer.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Signing failed for {:?}: {}",
            installer,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let signature = output_str.lines().last().unwrap_or("").trim().to_string();
    if signature.is_empty() {
        return Err(format!("Empty signature for {:?}", installer).into());
    }
    Ok(signature)
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
    if filename.ends_with(".msi") || filename.ends_with(".exe") {
        "windows-x86_64"
    } else if filename.ends_with(".dmg") {
        if filename.contains("aarch64") || filename.contains("arm64") {
            "darwin-aarch64"
        } else {
            "darwin-x86_64"
        }
    } else if filename.ends_with(".AppImage") {
        "linux-x86_64"
    } else {
        "unknown"
    }
}
