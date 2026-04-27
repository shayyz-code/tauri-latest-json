# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.4.4] - 2026-04-27

### Added

- [Internal] Added a library target and documented public API entrypoints to support docs.rs pages.
- [Testing] Added CLI integration tests for `--help`, `help`, `--version`, `version`, and non-TTY missing-argument failures.

### Changed

- [Behavior] Refactored core generation logic from bin to library while preserving CLI behavior.
- [Docs] Expanded README with CI-safe usage, root/`src-tauri` examples, and troubleshooting guidance.
- [CI] Added `cargo publish --dry-run` to CI checks.
- [Docs.rs] Enabled docs.rs metadata to build with all features.

### Fixed

- [Behavior] CI/smoke test failures by adding TTY detection and handling `help`/`version` positional arguments.
- [Dependency] Added `console` dependency for terminal detection.

## [0.4.0] - 2026-04-27

### Added

- Interactive mode: prompts for missing arguments (`download_url_base`, `notes`) if they are not provided via CLI.
- Enhanced CLI UX with colorized output (success, warnings, errors).
- Structured argument parsing and help messages using `clap`.
- Improved error handling with context using `anyhow`.

### Changed

- Refactored core logic to use modern CLI crates (`clap`, `anyhow`, `colored`).
- Improved automatic detection of version and public key from both project root and `src-tauri` directories.
- Support for both Tauri 1.0 (`tauri.updater.pubkey`) and 2.0 (`plugins.updater.pubkey`) configuration paths.

## [0.3.1] - 2026-04-27

### Changed

- Documentation updates and release prep notes.
- Improved macOS updater handling by preferring `.app.tar.gz` artifacts over `.dmg` and skipping platforms where signatures are missing (e.g. DMG files which Tauri doesn't sign).
- Made CLI root-run friendly for `src-tauri` projects by reading version and public key from both root and `src-tauri` locations, supporting both Tauri 1.0 and 2.0 configurations.

### Added

- Added optional real-app smoke test script: `scripts/smoke-real-tauri-app.sh`.
- Added `make smoke-real-app` target for validating against a local real Tauri app.
- Added `real-tauri-app/` to `.gitignore` for local real app fixtures.
- Added support in real-app smoke testing for `src-tauri/tauri.conf.json` and temporary installer/version bootstrapping.

## [0.3.0] - 2026-04-27

### Added

- Added test scripts:
  - `scripts/test.sh` for full verification checks.
  - `scripts/smoke-cli.sh` for CLI command smoke tests.
  - `scripts/smoke-generate.sh` for end-to-end generation smoke test.
  - `scripts/smoke-generate-current-conf.sh` for smoke testing with the repository `tauri.conf.json`.
- Added `Makefile` with local developer targets (`verify`, `test`, `smoke-*`, `dry-publish`).
- Added CI workflow to run format checks, tests, and smoke scripts on push/pull requests.

### Changed

- Converted project to CLI-only usage (removed library usage path).
- Added explicit `help`/`version` commands and short flags.
- Updated docs and specifications for CLI-only behavior.
- Expanded test coverage for argument parsing and generation flow.

## [0.2.5] - 2026-04-27

### Changed

- Baseline release prior to CLI-only and CI/test-script enhancements.
