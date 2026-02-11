#![deny(deprecated)]
//! 3D plot widgets rendered via viewport surfaces.
//!
//! This crate intentionally does **not** depend on `wgpu`/`winit` and does not emit renderer-specific
//! primitives. Instead, it embeds an engine-owned render target via `SceneOp::ViewportSurface` (or
//! `fret-ui`'s declarative `ViewportSurface` element) and
//! forwards input using `Effect::ViewportInput` (see ADR 0097).

pub mod retained;

pub use retained::{Plot3dCanvas, Plot3dModel, Plot3dStyle, Plot3dViewport};
