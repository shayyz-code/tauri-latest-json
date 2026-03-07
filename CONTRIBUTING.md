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

- Library: `src/lib.rs`
- CLI binary: `src/bin/tauri-latest-json.rs`
- Examples: `examples/`
- Tests: unit tests live alongside code

## Development Tips

- Keep changes small and focused.
- Add tests when fixing bugs or adding features.
- Follow existing code style; run `cargo fmt`.

## Pull Requests

- Describe the problem and solution clearly.
- Include reproduction steps if fixing a bug.
- Ensure `cargo test` passes.

## Release Process

- Bump the version in `Cargo.toml`.
- Update `README.md` if behavior or usage changes.
- Create a Git tag matching the version.

## License

By contributing, you agree that your contributions are licensed under the MIT license of this repository.
