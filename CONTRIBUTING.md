# Contributing to tauri-latest-json

Thanks for your interest in contributing!

Please make sure to read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

- Install Rust (stable) and Cargo.
- Clone the repo.
- Build and test:
  - `cargo build`
  - `cargo test`
- Optional:
  - `cargo fmt --all`
  - `cargo clippy -- -D warnings`
  - `cargo run --features verify-signature -- <download_url_base> <notes>`

## Project Layout

- CLI binary: `src/bin/tauri-latest-json.rs`
- Tests: unit tests live alongside code

## Development Tips

- Keep changes small and focused.
- Start from behavior specs in `SPEC.md` and update specs when behavior changes.
- Add tests when fixing bugs or adding features.
- Follow existing code style; run `cargo fmt`.

## Pull Requests

- Describe the problem and solution clearly.
- Include reproduction steps if fixing a bug.
- Ensure the full verification set passes:
  - `make verify`
  - `make clippy`
  - optional real app validation: `make smoke-real-app`
  - or run scripts manually:
  - `./scripts/test.sh`
  - `./scripts/smoke-cli.sh`
  - `./scripts/smoke-generate.sh`
  - `./scripts/smoke-generate-current-conf.sh`
  - `REAL_APP_DIR=/path/to/your-app ./scripts/smoke-real-tauri-app.sh`

## Release Process

- Bump the version in `Cargo.toml`.
- Update `CHANGELOG.md` for user-visible changes.
- Update `README.md` if behavior or usage changes.
- Create a Git tag matching the version.

## License

By contributing, you agree that your contributions are licensed under the MIT license of this repository.
