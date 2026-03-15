#![deny(deprecated)]
//! MagicUI-inspired component facade.
//!
//! This crate is intended to be a **naming + taxonomy surface** that mirrors common MagicUI-style
//! components while staying within Fret's layering model:
//!
//! - Kernel primitives live in `fret-core` / renderer crates.
//! - Ecosystem crates provide recipes and authoring helpers.
//! - Components are built out of the stable primitives (paint/materials/masks/effects/compositing).

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::IntoUiElement;

pub mod border_beam;
pub mod dock;
pub mod lens;
pub mod magic_card;
pub mod marquee;
pub mod patterns;
pub mod sparkles_text;

pub use border_beam::{border_beam, BorderBeamProps};
pub use dock::{dock, DockProps};
pub use lens::{lens, LensProps};
pub use magic_card::{magic_card, MagicCardProps};
pub use marquee::{marquee, MarqueeDirection, MarqueeProps};
pub use patterns::{
    dot_pattern, grid_pattern, stripe_pattern, DotPatternProps, GridPatternProps,
    PatternMotionProps, StripePatternProps,
};
pub use sparkles_text::{sparkles_text, SparklesTextProps};

pub(crate) fn collect_children<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    children: I,
) -> Vec<AnyElement>
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    children
        .into_iter()
        .map(|child| child.into_element(cx))
        .collect()
}

#[cfg(test)]
mod conversion_surface_tests {
    const LIB_RS: &str = include_str!("lib.rs");
    const BORDER_BEAM_RS: &str = include_str!("border_beam.rs");
    const DOCK_RS: &str = include_str!("dock.rs");
    const LENS_RS: &str = include_str!("lens.rs");
    const MAGIC_CARD_RS: &str = include_str!("magic_card.rs");
    const MARQUEE_RS: &str = include_str!("marquee.rs");
    const PATTERNS_RS: &str = include_str!("patterns.rs");
    const SPARKLES_TEXT_RS: &str = include_str!("sparkles_text.rs");

    #[test]
    fn public_magic_helpers_prefer_typed_child_inputs() {
        let lib_normalized = LIB_RS.split_whitespace().collect::<String>();
        assert!(lib_normalized.contains("pub(crate)fncollect_children<H:UiHost,I,T>("));
        assert!(lib_normalized.contains("T:IntoUiElement<H>"));

        for (label, src) in [
            ("border_beam", BORDER_BEAM_RS),
            ("dock", DOCK_RS),
            ("lens", LENS_RS),
            ("magic_card", MAGIC_CARD_RS),
            ("marquee", MARQUEE_RS),
            ("patterns", PATTERNS_RS),
            ("sparkles_text", SPARKLES_TEXT_RS),
        ] {
            let normalized = src.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains("IntoIterator<Item=AnyElement>"),
                "{label} should not publish raw AnyElement child iterators"
            );
            assert!(
                normalized.contains("IntoUiElement"),
                "{label} should mention the typed child contract"
            );
        }
    }
}

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
