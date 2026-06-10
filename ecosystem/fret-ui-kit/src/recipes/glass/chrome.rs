use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::ChromeRefinement;
use crate::recipes::effect_recipe::{alpha_mul, alpha_set};
use crate::style::{ColorFallback, ColorRef, MetricFallback, MetricRef};

#[derive(Debug, Clone, Copy)]
pub struct GlassTokenKeys {
    pub padding_x: Option<&'static str>,
    pub padding_y: Option<&'static str>,
    pub radius: Option<&'static str>,
    pub border_width: Option<&'static str>,
    pub tint: Option<&'static str>,
    pub border: Option<&'static str>,
}

impl GlassTokenKeys {
    pub const fn none() -> Self {
        Self {
            padding_x: None,
            padding_y: None,
            radius: None,
            border_width: None,
            tint: None,
            border: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedGlassChrome {
    pub padding_x: Px,
    pub padding_y: Px,
    pub radius: Px,
    pub border_width: Px,
    pub tint: Color,
    pub border: Color,
}

pub fn resolve_glass_chrome(
    theme: &Theme,
    style: &ChromeRefinement,
    keys: GlassTokenKeys,
) -> ResolvedGlassChrome {
    let default_padding_x = MetricRef::Token {
        key: "component.glass.padding_x",
        fallback: MetricFallback::ThemePaddingSm,
    };
    let default_padding_y = MetricRef::Token {
        key: "component.glass.padding_y",
        fallback: MetricFallback::ThemePaddingSm,
    };
    let default_radius = MetricRef::Token {
        key: "component.glass.radius",
        fallback: MetricFallback::ThemeRadiusLg,
    };

    let padding_x = style
        .padding
        .as_ref()
        .and_then(|p| p.left.as_ref().or(p.right.as_ref()))
        .map(|m| m.resolve(theme))
        .or_else(|| keys.padding_x.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.glass.padding_x"))
        .unwrap_or_else(|| default_padding_x.resolve(theme));
    let padding_y = style
        .padding
        .as_ref()
        .and_then(|p| p.top.as_ref().or(p.bottom.as_ref()))
        .map(|m| m.resolve(theme))
        .or_else(|| keys.padding_y.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.glass.padding_y"))
        .unwrap_or_else(|| default_padding_y.resolve(theme));
    let radius = style
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.radius.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.glass.radius"))
        .unwrap_or_else(|| default_radius.resolve(theme));
    let border_width = style
        .border_width
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.border_width.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.glass.border_width"))
        .unwrap_or(Px(1.0));

    let tint = style
        .background
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.tint.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.glass.tint"))
        .unwrap_or_else(|| alpha_set(theme.color_token("card"), 0.6));

    let border_default = ColorRef::Token {
        key: "component.glass.border",
        fallback: ColorFallback::ThemePanelBorder,
    };
    let border = style
        .border_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.border.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.glass.border"))
        .unwrap_or_else(|| alpha_mul(border_default.resolve(theme), 0.75));

    ResolvedGlassChrome {
        padding_x: Px(padding_x.0.max(0.0)),
        padding_y: Px(padding_y.0.max(0.0)),
        radius: Px(radius.0.max(0.0)),
        border_width: Px(border_width.0.max(0.0)),
        tint,
        border,
    }
}
