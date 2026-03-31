#![deny(deprecated)]
//! Plot/chart components (data-to-geometry + interaction policy) built on top of `fret-ui`.
//!
//! This crate must stay portable: no `wgpu`/`winit` and no dependency on `fret-render`.

pub mod cartesian;
pub mod chart;
pub mod input_map;
pub mod linking;
pub mod plot;
pub mod retained;
pub mod series;

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
        assert!(public_surface.contains("pub mod retained;"));
        assert!(!CARGO_TOML.contains("\nimui = ["));
        assert!(!CARGO_TOML.contains("fret-authoring"));
    }
}
