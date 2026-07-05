//! Fret chart UI adapter for the headless `delinea` engine.
//!
//! This crate focuses on:
//! - translating `delinea` marks into Fret draw ops (`SceneOp`)
//! - mapping UI input into `delinea` actions/patches

mod binding;
pub mod declarative;
#[cfg(feature = "echarts")]
pub mod echarts;
pub mod input_map;
pub mod linking;
pub mod output;
pub mod style;
pub mod tooltip;

mod a11y;
mod legend_logic;
mod slider_logic;
mod tooltip_layout;
mod visual_map_logic;

pub use binding::ChartCanvasPanelBinding;
pub use declarative::*;
pub use input_map::*;
pub use linking::*;
pub use output::*;
pub use style::*;
pub use tooltip::*;

#[cfg(test)]
mod public_surface_policy {
    fn compact(source: &str) -> String {
        source.split_whitespace().collect::<String>()
    }

    fn public_surface() -> &'static str {
        include_str!("lib.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or(include_str!("lib.rs"))
    }

    #[test]
    fn declarative_shared_contracts_do_not_depend_on_retained_namespace() {
        for (name, source) in [
            ("declarative/panel.rs", include_str!("declarative/panel.rs")),
            (
                "declarative/legend_overlay.rs",
                include_str!("declarative/legend_overlay.rs"),
            ),
            (
                "declarative/tooltip_overlay.rs",
                include_str!("declarative/tooltip_overlay.rs"),
            ),
            ("output.rs", include_str!("output.rs")),
        ] {
            for marker in [
                "crate::retained::ChartStyle",
                "crate::retained::TooltipFormatter",
                "crate::retained::DefaultTooltipFormatter",
                "crate::retained::TooltipTextLine",
                "crate::retained::tooltip",
                "crate::retained::style",
            ] {
                assert!(
                    !source.contains(marker),
                    "{name} should import shared chart contracts from top-level modules, not retained namespace marker `{marker}`"
                );
            }
        }
    }

    #[test]
    fn chart_linking_does_not_depend_on_retained_output_namespace() {
        let source = include_str!("linking.rs");

        for marker in [
            "crate::retained::ChartCanvasOutput",
            "retained::ChartCanvasOutput",
        ] {
            assert!(
                !source.contains(marker),
                "chart linking should import shared output contracts from top-level modules, not retained namespace marker `{marker}`"
            );
        }
    }

    #[test]
    fn retained_widgets_are_not_glob_reexported_from_crate_root() {
        let root = compact(public_surface());
        let marker = ["pubuse", "retained::*;"].concat();

        assert!(
            !root.contains(&marker),
            "retained chart widgets should require explicit fret_chart::retained imports"
        );
    }

    #[test]
    fn retained_chart_compat_feature_is_noop_and_module_is_quarantined() {
        let root = compact(public_surface());
        let cargo_toml = include_str!("../Cargo.toml");

        assert!(
            !root.contains("modretained;"),
            "retained chart source should stay deleted instead of compiling behind a bridge feature"
        );
        assert!(
            !cargo_toml.contains("fret-ui/unstable-retained-bridge"),
            "fret-chart should not map any feature to fret-ui/unstable-retained-bridge"
        );
        assert!(
            cargo_toml.contains("compat-retained-canvas = []"),
            "fret-chart should keep compat-retained-canvas only as a no-op transition alias"
        );
        assert!(
            !cargo_toml.contains(
                "fret-ui = { version = \"0.1.0\", path = \"../../crates/fret-ui\", features = [\"unstable-retained-bridge\"] }"
            ),
            "fret-chart should not enable unstable-retained-bridge from the default fret-ui dependency"
        );
    }

    #[test]
    fn default_chart_dependency_does_not_enable_unstable_retained_bridge() {
        let cargo_toml = include_str!("../Cargo.toml");
        let root = compact(public_surface());

        assert!(
            cargo_toml.contains("compat-retained-canvas = []"),
            "fret-chart should keep compat-retained-canvas only as a no-op transition alias"
        );
        assert!(
            !cargo_toml.contains("fret-ui/unstable-retained-bridge"),
            "fret-chart should not map any feature to fret-ui/unstable-retained-bridge"
        );
        assert!(
            !cargo_toml.contains(
                "fret-ui = { version = \"0.1.0\", path = \"../../crates/fret-ui\", features = [\"unstable-retained-bridge\"] }"
            ),
            "fret-chart should not enable unstable-retained-bridge from the default fret-ui dependency"
        );
        assert!(
            !root.contains("#[cfg(feature=\"compat-retained-canvas\")]modretained;")
                && !root.contains("#[cfg(feature=\"compat-retained-canvas\")]pubmodretained;"),
            "fret-chart compat-retained-canvas should not compile a retained chart module"
        );
    }
}
