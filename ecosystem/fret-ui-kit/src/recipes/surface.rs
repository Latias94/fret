//! Surface recipe helpers built on top of shared theme/style resolution.
//!
//! This module owns surface chrome fallback policy and the base refinement used
//! by declarative wrappers. App-facing wrappers compose these helpers instead of
//! duplicating token defaults.

mod chrome;
mod refinement;

pub use chrome::{ResolvedSurfaceChrome, SurfaceTokenKeys, resolve_surface_chrome};
pub use refinement::surface_base_refinement;
