#![deny(deprecated)]
//! MagicUI-inspired component facade.
//!
//! This crate is intended to be a **naming + taxonomy surface** that mirrors common MagicUI-style
//! components while staying within Fret's layering model:
//!
//! - Kernel primitives live in `fret-core` / renderer crates.
//! - Ecosystem crates provide recipes and authoring helpers.
//! - Components are built out of the stable primitives (paint/materials/masks/effects/compositing).

pub mod border_beam;
pub mod dock;
pub mod lens;
pub mod magic_card;
pub mod marquee;

pub use border_beam::{BorderBeamProps, border_beam};
pub use dock::{DockProps, dock};
pub use lens::{LensProps, lens};
pub use magic_card::{MagicCardProps, magic_card};
pub use marquee::{MarqueeDirection, MarqueeProps, marquee};

#[cfg(feature = "app-integration")]
pub mod app_integration;
