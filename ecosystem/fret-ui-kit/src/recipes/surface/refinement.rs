use fret_core::Px;

use crate::style::PaddingRefinement;
use crate::style::{ColorFallback, MetricFallback};
use crate::{ChromeRefinement, ColorRef, MetricRef};

pub fn surface_base_refinement() -> ChromeRefinement {
    ChromeRefinement {
        padding: Some(PaddingRefinement {
            top: Some(MetricRef::Token {
                key: "component.surface.padding_y",
                fallback: MetricFallback::ThemePaddingSm,
            }),
            right: Some(MetricRef::Token {
                key: "component.surface.padding_x",
                fallback: MetricFallback::ThemePaddingSm,
            }),
            bottom: Some(MetricRef::Token {
                key: "component.surface.padding_y",
                fallback: MetricFallback::ThemePaddingSm,
            }),
            left: Some(MetricRef::Token {
                key: "component.surface.padding_x",
                fallback: MetricFallback::ThemePaddingSm,
            }),
        }),
        radius: Some(MetricRef::Token {
            key: "component.surface.radius",
            fallback: MetricFallback::ThemeRadiusSm,
        }),
        border_width: Some(MetricRef::Token {
            key: "component.surface.border_width",
            fallback: MetricFallback::Px(Px(1.0)),
        }),
        background: Some(ColorRef::Token {
            key: "component.surface.bg",
            fallback: ColorFallback::ThemePanelBackground,
        }),
        border_color: Some(ColorRef::Token {
            key: "component.surface.border",
            fallback: ColorFallback::ThemePanelBorder,
        }),
        ..ChromeRefinement::default()
    }
}
