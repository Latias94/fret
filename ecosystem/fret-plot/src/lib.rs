#![deny(deprecated)]
//! Plot/chart components (data-to-geometry + interaction policy) built on top of `fret-ui`.
//!
//! This crate must stay portable: no `wgpu`/`winit` and no dependency on `fret-render`.

pub mod cartesian;
pub mod chart;
pub mod declarative;
pub mod input_map;
pub mod linking;
pub mod models;
pub mod plot;
#[cfg(feature = "compat-retained-canvas")]
pub mod retained;
pub mod series;
pub mod state;
pub mod style;

#[cfg(feature = "compat-retained-canvas")]
mod theme_tokens;

#[cfg(test)]
mod surface_policy_tests {
    const LIB_RS: &str = include_str!("lib.rs");
    const CARGO_TOML: &str = include_str!("../Cargo.toml");

    fn public_surface() -> &'static str {
        LIB_RS.split("#[cfg(test)]").next().unwrap_or(LIB_RS)
    }

    #[test]
    fn no_public_imui_facade_survives() {
        let public_surface = public_surface();
        assert!(!public_surface.contains("pub mod imui;"));
        assert!(!CARGO_TOML.contains("\nimui = ["));
        assert!(!CARGO_TOML.contains("fret-authoring"));
    }

    #[test]
    fn retained_plot_surface_requires_explicit_compat_feature() {
        let public_surface = public_surface();
        assert!(
            public_surface
                .contains("#[cfg(feature = \"compat-retained-canvas\")]\npub mod retained;")
        );
        assert!(
            CARGO_TOML.contains("compat-retained-canvas = [\"fret-ui/unstable-retained-bridge\"]")
        );
        assert!(!CARGO_TOML.contains("fret-ui\", features = [\"unstable-retained-bridge\"]"));
    }
}
