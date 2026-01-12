#![deny(deprecated)]
//! Headless chart engine foundation for Fret.
//!
//! `delinea` is renderer-agnostic and portable:
//! - no `wgpu`/`winit`
//! - no dependency on `fret-render`
//!
//! The Fret UI adapter lives in a separate crate (planned: `fret-chart`).

pub mod action;
pub mod coord;
pub mod data;
pub mod engine;
pub mod format;
pub mod ids;
pub mod link;
pub mod marks;
pub mod paint;
pub mod scale;
pub mod scheduler;
pub mod selection;
pub mod spec;
pub mod stats;
pub mod text;
pub mod time_axis;
pub mod tooltip;
pub mod transform;
pub mod view;
pub mod visual_map;

pub use action::*;
pub use engine::*;
pub use ids::*;
pub use link::*;
pub use marks::*;
pub use paint::*;
pub use scale::*;
pub use scheduler::*;
pub use selection::*;
pub use spec::*;
pub use stats::*;
pub use text::*;
pub use tooltip::*;
pub use transform::*;
pub use view::*;
pub use visual_map::*;
