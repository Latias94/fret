#![deny(deprecated)]
//! Plot/chart components (data-to-geometry + interaction policy) built on top of `fret-ui`.
//!
//! This crate must stay portable: no `wgpu`/`winit` and no dependency on `fret-render`.

pub mod cartesian;
pub mod chart;
#[cfg(feature = "imui")]
pub mod imui;
pub mod input_map;
pub mod linking;
pub mod plot;
pub mod retained;
pub mod series;

mod theme_tokens;
