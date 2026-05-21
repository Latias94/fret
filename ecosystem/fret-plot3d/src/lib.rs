#![deny(deprecated)]
//! 3D plot panels rendered via declarative viewport surfaces.
//!
//! This crate intentionally does **not** depend on `wgpu`/`winit` and does not emit renderer-specific
//! primitives. Instead, it embeds an engine-owned render target via `fret-ui`'s declarative
//! `ViewportSurface` element and forwards input using `Effect::ViewportInput` (see ADR 0097).

pub mod declarative;

pub use declarative::{
    Plot3dModel, Plot3dPanelProps, Plot3dStyle, Plot3dViewport, plot3d_panel,
    plot3d_panel_with_model,
};
