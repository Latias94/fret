use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::style::PaddingRefinement;
use crate::style::{ColorFallback, MetricFallback};
use crate::{ChromeRefinement, ColorRef, MetricRef};

#[derive(Debug, Clone, Copy)]
pub struct SurfaceTokenKeys {
    pub padding_x: Option<&'static str>,
    pub padding_y: Option<&'static str>,
    pub radius: Option<&'static str>,
    pub border_width: Option<&'static str>,
    pub bg: Option<&'static str>,
    pub border: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedSurfaceChrome {
    pub padding_x: Px,
    pub padding_y: Px,
    pub radius: Px,
    pub border_width: Px,
    pub background: Color,
    pub border_color: Color,
}

pub fn resolve_surface_chrome(
    theme: &Theme,
    style: &ChromeRefinement,
    keys: SurfaceTokenKeys,
) -> ResolvedSurfaceChrome {
    let default_padding_x = MetricRef::Token {
        key: "component.surface.padding_x",
        fallback: MetricFallback::ThemePaddingSm,
    };
    let default_padding_y = MetricRef::Token {
        key: "component.surface.padding_y",
        fallback: MetricFallback::ThemePaddingSm,
    };
    let default_radius = MetricRef::Token {
        key: "component.surface.radius",
        fallback: MetricFallback::ThemeRadiusSm,
    };
    let default_bg = ColorRef::Token {
        key: "component.surface.bg",
        fallback: ColorFallback::ThemePanelBackground,
    };
    let default_border = ColorRef::Token {
        key: "component.surface.border",
        fallback: ColorFallback::ThemePanelBorder,
    };

    // Note: surface chrome is interpreted as symmetric `padding_x/padding_y`. To avoid Tailwind
    // per-edge no-ops, accept either edge as a shorthand for the axis (e.g. `pr-*` behaves like
    // setting `px-*` for surfaces).
    let padding_x = style
        .padding
        .as_ref()
        .and_then(|p| p.left.as_ref().or(p.right.as_ref()))
        .map(|m| m.resolve(theme))
        .or_else(|| keys.padding_x.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.surface.padding_x"))
        .unwrap_or_else(|| default_padding_x.resolve(theme));
    let padding_y = style
        .padding
        .as_ref()
        .and_then(|p| p.top.as_ref().or(p.bottom.as_ref()))
        .map(|m| m.resolve(theme))
        .or_else(|| keys.padding_y.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.surface.padding_y"))
        .unwrap_or_else(|| default_padding_y.resolve(theme));
    let radius = style
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.radius.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.surface.radius"))
        .unwrap_or_else(|| default_radius.resolve(theme));
    let border_width = style
        .border_width
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.border_width.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.surface.border_width"))
        .unwrap_or(Px(1.0));

    let background = style
        .background
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.bg.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.surface.bg"))
        .unwrap_or_else(|| default_bg.resolve(theme));
    let border_color = style
        .border_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.border.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.surface.border"))
        .unwrap_or_else(|| default_border.resolve(theme));

    ResolvedSurfaceChrome {
        padding_x: Px(padding_x.0.max(0.0)),
        padding_y: Px(padding_y.0.max(0.0)),
        radius: Px(radius.0.max(0.0)),
        border_width: Px(border_width.0.max(0.0)),
        background,
        border_color,
    }
}

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
