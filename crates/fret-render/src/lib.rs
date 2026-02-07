//! Compatibility facade for Fret's default renderer backend.
//!
//! Today the default renderer is wgpu-based (`fret-render-wgpu`). This crate exists to keep the
//! historical `fret-render` crate name stable while we split backend implementations into
//! explicit crates.

pub use fret_render_wgpu::*;
