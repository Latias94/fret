//! Shared launcher/runner API surface for Fret.
//!
//! This crate exists to keep the public app-facing launcher surface stable while allowing
//! platform-specific implementations (desktop/web/mobile) to evolve independently.
//!
//! Hard rule: this crate must not depend on platform SDK crates such as `objc*`, `windows*`,
//! or browser-only crates such as `web-sys`.

pub mod common;
mod error;

#[doc(hidden)]
pub mod font_catalog;
pub mod imported_viewport_target;
pub mod native_external_import;
#[doc(hidden)]
pub mod streaming_upload;
pub mod viewport_overlay_immediate_3d;
pub mod viewport_target;
#[doc(hidden)]
pub mod yuv;
#[doc(hidden)]
pub mod yuv_gpu;

pub use error::RunnerError;
