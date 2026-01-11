use fret_core::{EffectChain, EffectStep, Px};
use fret_ui::Theme;

use crate::ChromeRefinement;
use crate::recipes::effect_recipe::clamp_u32_from_metric;
use crate::style::{ColorFallback, ColorRef, MetricFallback, MetricRef};

#[derive(Debug, Clone, Copy)]
pub struct PixelateTokenKeys {
    pub padding_x: Option<&'static str>,
    pub padding_y: Option<&'static str>,
    pub radius: Option<&'static str>,
    pub border_width: Option<&'static str>,
    pub bg: Option<&'static str>,
    pub border: Option<&'static str>,
}

impl PixelateTokenKeys {
    pub const fn none() -> Self {
        Self {
            padding_x: None,
            padding_y: None,
            radius: None,
            border_width: None,
            bg: None,
            border: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedPixelateChrome {
    pub padding_x: Px,
    pub padding_y: Px,
    pub radius: Px,
    pub border_width: Px,
    pub background: fret_core::Color,
    pub border_color: fret_core::Color,
}

pub fn resolve_pixelate_chrome(
    theme: &Theme,
    style: &ChromeRefinement,
    keys: PixelateTokenKeys,
) -> ResolvedPixelateChrome {
    let default_padding_x = MetricRef::Token {
        key: "component.pixelate.padding_x",
        fallback: MetricFallback::ThemePaddingSm,
    };
    let default_padding_y = MetricRef::Token {
        key: "component.pixelate.padding_y",
        fallback: MetricFallback::ThemePaddingSm,
    };
    let default_radius = MetricRef::Token {
        key: "component.pixelate.radius",
        fallback: MetricFallback::ThemeRadiusSm,
    };

    let padding_x = style
        .padding
        .as_ref()
        .and_then(|p| p.left.as_ref().or(p.right.as_ref()))
        .map(|m| m.resolve(theme))
        .or_else(|| keys.padding_x.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.pixelate.padding_x"))
        .unwrap_or_else(|| default_padding_x.resolve(theme));
    let padding_y = style
        .padding
        .as_ref()
        .and_then(|p| p.top.as_ref().or(p.bottom.as_ref()))
        .map(|m| m.resolve(theme))
        .or_else(|| keys.padding_y.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.pixelate.padding_y"))
        .unwrap_or_else(|| default_padding_y.resolve(theme));
    let radius = style
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.radius.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.pixelate.radius"))
        .unwrap_or_else(|| default_radius.resolve(theme));
    let border_width = style
        .border_width
        .as_ref()
        .map(|m| m.resolve(theme))
        .or_else(|| keys.border_width.and_then(|k| theme.metric_by_key(k)))
        .or_else(|| theme.metric_by_key("component.pixelate.border_width"))
        .unwrap_or(Px(1.0));

    let default_bg = ColorRef::Token {
        key: "component.pixelate.bg",
        fallback: ColorFallback::ThemePanelBackground,
    };
    let default_border = ColorRef::Token {
        key: "component.pixelate.border",
        fallback: ColorFallback::ThemePanelBorder,
    };

    let background = style
        .background
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.bg.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.pixelate.bg"))
        .unwrap_or_else(|| default_bg.resolve(theme));
    let border_color = style
        .border_color
        .as_ref()
        .map(|c| c.resolve(theme))
        .or_else(|| keys.border.and_then(|k| theme.color_by_key(k)))
        .or_else(|| theme.color_by_key("component.pixelate.border"))
        .unwrap_or_else(|| default_border.resolve(theme));

    ResolvedPixelateChrome {
        padding_x: Px(padding_x.0.max(0.0)),
        padding_y: Px(padding_y.0.max(0.0)),
        radius: Px(radius.0.max(0.0)),
        border_width: Px(border_width.0.max(0.0)),
        background,
        border_color,
    }
}

#[derive(Debug, Clone, Default)]
pub struct PixelateEffectRefinement {
    pub scale: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct PixelateEffectTokenKeys {
    pub scale: Option<&'static str>,
}

impl PixelateEffectTokenKeys {
    pub const fn none() -> Self {
        Self { scale: None }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedPixelateEffect {
    pub scale: u32,
}

pub fn resolve_pixelate_effect(
    theme: &Theme,
    refinement: &PixelateEffectRefinement,
    keys: PixelateEffectTokenKeys,
) -> ResolvedPixelateEffect {
    let scale = refinement.scale.unwrap_or_else(|| {
        let metric = keys
            .scale
            .and_then(|k| theme.metric_by_key(k))
            .or_else(|| theme.metric_by_key("component.pixelate.scale"))
            .unwrap_or(Px(8.0));
        clamp_u32_from_metric(metric, 1, 64, 8)
    });

    ResolvedPixelateEffect {
        scale: scale.clamp(1, 256),
    }
}

pub fn pixelate_effect_chain(effect: ResolvedPixelateEffect) -> EffectChain {
    EffectChain::from_steps(&[EffectStep::Pixelate {
        scale: effect.scale,
    }])
}
