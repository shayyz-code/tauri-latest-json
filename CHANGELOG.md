# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed
- Documentation updates and release prep notes.

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
