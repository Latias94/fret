//! Shared launcher/runner API surface for Fret.
//!
//! This crate exists to keep the public app-facing launcher surface stable while allowing
//! platform-specific implementations (desktop/web/mobile) to evolve independently.
//!
//! Hard rule: this crate must not depend on platform SDK crates such as `objc*`, `windows*`,
//! or browser-only crates such as `web-sys`.

pub mod common;
mod error;

pub use error::RunnerError;
