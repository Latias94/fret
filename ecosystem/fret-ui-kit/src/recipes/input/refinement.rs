use fret_core::Px;

use crate::style::PaddingRefinement;
use crate::style::{ColorFallback, MetricFallback};
use crate::{ChromeRefinement, ColorRef, MetricRef};

pub fn input_base_refinement() -> ChromeRefinement {
    ChromeRefinement {
        padding: Some(PaddingRefinement {
            top: Some(MetricRef::Token {
                key: "component.input.padding_y",
                fallback: MetricFallback::ThemePaddingSm,
            }),
            right: Some(MetricRef::Token {
                key: "component.input.padding_x",
                fallback: MetricFallback::ThemePaddingSm,
            }),
            bottom: Some(MetricRef::Token {
                key: "component.input.padding_y",
                fallback: MetricFallback::ThemePaddingSm,
            }),
            left: Some(MetricRef::Token {
                key: "component.input.padding_x",
                fallback: MetricFallback::ThemePaddingSm,
            }),
        }),
        border_width: Some(MetricRef::Token {
            key: "component.input.border_width",
            fallback: MetricFallback::Px(Px(1.0)),
        }),
        radius: Some(MetricRef::Token {
            key: "component.input.radius",
            fallback: MetricFallback::ThemeRadiusSm,
        }),
        background: Some(ColorRef::Token {
            key: "component.input.bg",
            fallback: ColorFallback::ThemePanelBackground,
        }),
        border_color: Some(ColorRef::Token {
            key: "component.input.border",
            fallback: ColorFallback::ThemePanelBorder,
        }),
        text_color: Some(ColorRef::Token {
            key: "component.input.fg",
            fallback: ColorFallback::ThemeTextPrimary,
        }),
        ..ChromeRefinement::default()
    }
}
