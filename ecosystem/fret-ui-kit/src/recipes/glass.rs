use fret_core::{Color, EffectChain, EffectStep, Px};
use fret_ui::Theme;

use crate::ChromeRefinement;
use crate::recipes::effect_recipe::{alpha_mul, alpha_set, clamp_u32_from_metric};
use crate::recipes::resolve::{DegradationReason, ResolvedWithFallback};
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

#[derive(Debug, Clone, Default)]
pub struct GlassEffectRefinement {
    pub blur_radius_px: Option<Px>,
    pub blur_downsample: Option<u32>,
    pub saturation: Option<f32>,
    pub brightness: Option<f32>,
    pub contrast: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct GlassEffectTokenKeys {
    pub blur_radius_px: Option<&'static str>,
    pub blur_downsample: Option<&'static str>,
    pub saturation: Option<&'static str>,
    pub brightness: Option<&'static str>,
    pub contrast: Option<&'static str>,
}

impl GlassEffectTokenKeys {
    pub const fn none() -> Self {
        Self {
            blur_radius_px: None,
            blur_downsample: None,
            saturation: None,
            brightness: None,
            contrast: None,
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
        .unwrap_or_else(|| alpha_set(theme.color_required("card"), 0.6));

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

#[derive(Debug, Clone, Copy)]
pub struct ResolvedGlassEffect {
    pub blur_radius_px: Px,
    pub blur_downsample: u32,
    pub saturation: f32,
    pub brightness: f32,
    pub contrast: f32,
}

pub fn resolve_glass_effect(
    theme: &Theme,
    refinement: &GlassEffectRefinement,
    keys: GlassEffectTokenKeys,
) -> ResolvedGlassEffect {
    let blur_radius_px = refinement.blur_radius_px.unwrap_or_else(|| {
        keys.blur_radius_px
            .and_then(|k| theme.metric_by_key(k))
            .or_else(|| theme.metric_by_key("component.glass.blur_radius_px"))
            .unwrap_or(Px(12.0))
    });

    let blur_downsample = refinement.blur_downsample.unwrap_or_else(|| {
        let metric = keys
            .blur_downsample
            .and_then(|k| theme.metric_by_key(k))
            .or_else(|| theme.metric_by_key("component.glass.blur_downsample"))
            .unwrap_or(Px(2.0));
        clamp_u32_from_metric(metric, 1, 8, 2)
    });

    let saturation = refinement.saturation.unwrap_or_else(|| {
        keys.saturation
            .and_then(|k| theme.metric_by_key(k))
            .or_else(|| theme.metric_by_key("component.glass.saturation"))
            .map(|v| v.0)
            .unwrap_or(1.05)
    });
    let brightness = refinement.brightness.unwrap_or_else(|| {
        keys.brightness
            .and_then(|k| theme.metric_by_key(k))
            .or_else(|| theme.metric_by_key("component.glass.brightness"))
            .map(|v| v.0)
            .unwrap_or(1.0)
    });
    let contrast = refinement.contrast.unwrap_or_else(|| {
        keys.contrast
            .and_then(|k| theme.metric_by_key(k))
            .or_else(|| theme.metric_by_key("component.glass.contrast"))
            .map(|v| v.0)
            .unwrap_or(1.0)
    });

    ResolvedGlassEffect {
        blur_radius_px: Px(blur_radius_px.0.clamp(0.0, 256.0)),
        blur_downsample: blur_downsample.clamp(1, 16),
        saturation: saturation.clamp(0.0, 3.0),
        brightness: brightness.clamp(0.0, 3.0),
        contrast: contrast.clamp(0.0, 3.0),
    }
}

pub fn glass_effect_chain(effect: ResolvedGlassEffect) -> EffectChain {
    let mut steps: Vec<EffectStep> = Vec::new();

    if effect.blur_radius_px.0 > 0.0 {
        steps.push(EffectStep::GaussianBlur {
            radius_px: effect.blur_radius_px,
            downsample: effect.blur_downsample,
        });
    }

    let needs_color_adjust = (effect.saturation - 1.0).abs() > 1e-6
        || (effect.brightness - 1.0).abs() > 1e-6
        || (effect.contrast - 1.0).abs() > 1e-6;
    if needs_color_adjust {
        steps.push(EffectStep::ColorAdjust {
            saturation: effect.saturation,
            brightness: effect.brightness,
            contrast: effect.contrast,
        });
    }

    EffectChain::from_steps(&steps)
}

/// Returns the glass effect chain, respecting reduced-transparency preferences (ADR 0246).
///
/// When reduced transparency is preferred, this returns an empty chain (no blur or color-adjust).
pub fn glass_effect_chain_for_environment(
    effect: ResolvedGlassEffect,
    prefers_reduced_transparency: bool,
) -> EffectChain {
    if prefers_reduced_transparency {
        EffectChain::EMPTY
    } else {
        glass_effect_chain(effect)
    }
}

pub fn resolve_glass_effect_chain_for_environment(
    effect: ResolvedGlassEffect,
    prefers_reduced_transparency: bool,
) -> ResolvedWithFallback<EffectChain> {
    if prefers_reduced_transparency {
        ResolvedWithFallback::degraded(
            EffectChain::EMPTY,
            "glass.effect_chain",
            DegradationReason::ReducedTransparency,
        )
    } else {
        ResolvedWithFallback::ok(glass_effect_chain(effect))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Px;

    #[test]
    fn glass_effect_chain_disables_effects_when_reduced_transparency_is_true() {
        let effect = ResolvedGlassEffect {
            blur_radius_px: Px(12.0),
            blur_downsample: 2,
            saturation: 1.1,
            brightness: 1.0,
            contrast: 1.0,
        };

        let disabled = glass_effect_chain_for_environment(effect, true);
        assert!(disabled.is_empty());

        let enabled = glass_effect_chain_for_environment(effect, false);
        assert!(!enabled.is_empty());
    }
}
