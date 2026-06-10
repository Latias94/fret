//! Visual recipe catalogs for renderer-owned resources.
//!
//! The catalog stays app-owned so material lifetimes remain explicit and
//! portable descriptors do not leak backend handles into components.

mod material;
mod visual;

pub use material::MaterialCatalog;
pub use visual::VisualCatalog;
