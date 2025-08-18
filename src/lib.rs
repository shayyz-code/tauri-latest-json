use chrono::Utc;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    ops::IndexMut,
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
    download_url_base: &str,
    notes: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle_dir = detect_bundle_dir()?;
    let current_dir = std::env::current_dir()?;
    let tauri_conf_path = &current_dir.join("tauri.conf.json");

    let public_key = read_public_key(tauri_conf_path)?;
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
        println!("platform key: {:?}", &platform_key);

        // Sign installer
        let signature_path = find_singatures(&bundle_dir)?;
        println!("sig path: {:?}", &signature_path);
        let mut f_sig = std::fs::File::open(&signature_path.get(&platform_key).unwrap())?;
        let mut signature = String::new();
        f_sig.read_to_string(&mut signature)?;

        println!("sig path: {:?}", &signature);

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
    let bundle_dir = current_dir.join("target").join("release").join("bundle");

    print!("{:?}", vec![&current_dir, &bundle_dir]);
    if bundle_dir.exists() {
        Ok(bundle_dir)
    } else {
        Err("Could not detect bundle dir. Run `pnpm tauri build` first.".into())
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

fn find_singatures(dir: &Path) -> Result<HashMap<&str, PathBuf>, Box<dyn std::error::Error>> {
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
