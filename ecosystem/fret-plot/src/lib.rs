#![deny(deprecated)]
//! Plot/chart components (data-to-geometry + interaction policy) built on top of `fret-ui`.
//!
//! This crate must stay portable: no `wgpu`/`winit` and no dependency on `fret-render`.

pub mod cartesian;
pub mod chart;
pub mod declarative;
#[cfg(feature = "imui")]
pub mod imui;
pub mod input_map;
pub mod linking;
pub mod models;
pub mod plot;
pub mod series;
pub mod state;
pub mod style;

mod theme_tokens;

#[cfg(test)]
mod surface_policy_tests {
    const LIB_RS: &str = include_str!("lib.rs");
    const CARGO_TOML: &str = include_str!("../Cargo.toml");

    fn cargo_toml_without_comments() -> String {
        CARGO_TOML
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn public_surface() -> &'static str {
        LIB_RS.split("#[cfg(test)]").next().unwrap_or(LIB_RS)
    }

    #[test]
    fn imui_adapter_stays_opt_in_and_declarative_only() {
        let public_surface = public_surface();
        let cargo_toml = cargo_toml_without_comments();
        let imui_rs = include_str!("imui.rs");

        assert!(public_surface.contains("#[cfg(feature = \"imui\")]"));
        assert!(public_surface.contains("pub mod imui;"));
        assert!(cargo_toml.contains("default = []"));
        assert!(cargo_toml.contains("imui = [\"ui\", \"dep:fret-authoring\"]"));
        assert!(
            cargo_toml
                .contains("fret-authoring = { path = \"../fret-authoring\", optional = true }")
        );
        assert!(!cargo_toml.contains("default = [\"imui\"]"));
        assert!(imui_rs.contains("use fret_authoring::UiWriter;"));
        assert!(imui_rs.contains("crate::declarative::line_plot_panel(cx, props)"));
        assert!(!imui_rs.contains("LinePlotCanvas"));
        assert!(!imui_rs.contains("retained"));
    }

    #[test]
    fn retained_plot_compat_feature_no_longer_enables_bridge_or_module() {
        let public_surface = public_surface();
        let cargo_toml = cargo_toml_without_comments();
        let retained_bridge_feature = ["fret-ui/", "unstable-retained-bridge"].concat();
        let public_retained_module = ["pub ", "mod ", "retained;"].concat();
        let private_retained_module = ["mod ", "retained;"].concat();

        assert!(
            !public_surface.contains(&public_retained_module),
            "retained plot canvas should not be public through fret_plot::retained"
        );
        assert!(
            !public_surface.contains(&private_retained_module),
            "retained plot source should stay deleted instead of compiling behind a bridge feature"
        );
        assert!(
            cargo_toml.contains("compat-retained-canvas = []"),
            "fret-plot should keep the legacy compat-retained-canvas feature as a no-op transition alias"
        );
        assert!(
            !cargo_toml.contains(&retained_bridge_feature),
            "fret-plot should no longer enable the retained bridge from any package feature"
        );
        assert!(!cargo_toml.contains("fret-ui\", features = [\"unstable-retained-bridge\"]"));
    }

    #[test]
    fn line_chart_builder_stays_model_only_on_default_surface() {
        let line_chart = include_str!("chart/line_chart.rs");
        assert!(line_chart.contains("pub fn install"));
        assert!(line_chart.contains("pub fn into_element"));
        assert!(line_chart.contains("line_plot_panel_in"));
        assert!(!line_chart.contains("into_canvas("));
        assert!(!line_chart.contains("LinePlotCanvas"));
    }
}
