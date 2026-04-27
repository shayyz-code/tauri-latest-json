//! Library entrypoint for `tauri-latest-json`.
//!
//! This crate primarily ships a CLI binary, but exposing a small library target
//! ensures docs.rs can build and host crate documentation.
//!
//! See the binary usage in the project README.

/// Current crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
