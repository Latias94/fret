//! Web/wasm runner glue for Fret.
//!
//! On `wasm32`, this crate currently re-exports `fret-platform-web` services used by
//! `fret-runtime::Effect`s. It intentionally keeps non-wasm builds explicit via a stub module.
//!
//! For module ownership and “where should this go?” guidance, see
//! `crates/fret-runner-web/README.md`.

#[cfg(target_arch = "wasm32")]
pub use fret_platform_web::*;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
