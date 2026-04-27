use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn help_flag_prints_usage() {
    Command::cargo_bin("tauri-latest-json")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage"));
}

#[test]
fn help_positional_prints_usage() {
    Command::cargo_bin("tauri-latest-json")
        .unwrap()
        .arg("help")
        .assert()
        .success()
        .stdout(contains("Usage"));
}

#[test]
fn version_flag_prints_version() {
    Command::cargo_bin("tauri-latest-json")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn version_positional_prints_version() {
    Command::cargo_bin("tauri-latest-json")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn no_args_fails_without_tty_prompt() {
    Command::cargo_bin("tauri-latest-json")
        .unwrap()
        .assert()
        .failure()
        .stderr(contains("download_url_base").and(contains("not in a terminal")));
}

#[test]
fn missing_notes_fails_without_tty_prompt() {
    Command::cargo_bin("tauri-latest-json")
        .unwrap()
        .arg("https://example.com/downloads")
        .assert()
        .failure()
        .stderr(contains("notes").and(contains("not in a terminal")));
}
