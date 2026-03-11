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
pub mod patterns;
pub mod sparkles_text;

pub use border_beam::{BorderBeamProps, border_beam};
pub use dock::{DockProps, dock};
pub use lens::{LensProps, lens};
pub use magic_card::{MagicCardProps, magic_card};
pub use marquee::{MarqueeDirection, MarqueeProps, marquee};
pub use patterns::{
    DotPatternProps, GridPatternProps, PatternMotionProps, StripePatternProps, dot_pattern,
    grid_pattern, stripe_pattern,
};
pub use sparkles_text::{SparklesTextProps, sparkles_text};

#[cfg(feature = "app-integration")]
pub mod advanced;

#[cfg(all(test, feature = "app-integration"))]
mod surface_policy_tests {
    const LIB_RS: &str = include_str!("lib.rs");
    const ADVANCED_RS: &str = include_str!("advanced.rs");

    fn public_surface() -> &'static str {
        LIB_RS
            .split("#[cfg(all(test, feature = \"app-integration\"))]")
            .next()
            .unwrap_or(LIB_RS)
    }

    #[test]
    fn material_service_helper_stays_on_explicit_advanced_surface() {
        let public_surface = public_surface();
        assert!(public_surface.contains("pub mod advanced;"));
        assert!(!public_surface.contains("pub mod app_integration;"));
        assert!(ADVANCED_RS.contains("pub fn ensure_materials("));
        assert!(ADVANCED_RS.contains("MaterialService"));
    }
}
