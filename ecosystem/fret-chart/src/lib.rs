//! Fret chart UI adapter for the headless `delinea` engine.
//!
//! This crate focuses on:
//! - translating `delinea` marks into Fret draw ops (`SceneOp`)
//! - mapping UI input into `delinea` actions/patches

pub mod declarative;
#[cfg(feature = "echarts")]
pub mod echarts;
pub mod input_map;
pub mod linking;
pub mod output;
pub mod retained;
pub mod style;
pub mod tooltip;

mod a11y;
mod legend_logic;
mod tooltip_layout;

pub use declarative::*;
pub use input_map::*;
pub use linking::*;
pub use output::*;
pub use retained::*;
pub use style::*;
pub use tooltip::*;

#[cfg(test)]
mod public_surface_policy {
    fn compact(source: &str) -> String {
        source.split_whitespace().collect::<String>()
    }

    #[test]
    fn retained_multi_grid_helpers_are_removed_from_public_surface() {
        let retained_mod = compact(include_str!("retained/mod.rs"));
        let retained_canvas = compact(include_str!("retained/canvas.rs"));

        assert!(
            !retained_mod.contains("modmulti_grid;")
                && !retained_mod.contains("pubusemulti_grid::*;"),
            "retained chart should not keep the legacy multi-grid helper module public"
        );

        for marker in [
            "pubfnnew_grid_view(",
            "pubfnnew_overlay(",
            "ChartCanvasMode::GridView",
            "ChartCanvasMode::Overlay",
        ] {
            assert!(
                !retained_canvas.contains(marker),
                "retained ChartCanvas should not keep legacy multi-grid surface marker `{marker}`"
            );
        }
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
}
