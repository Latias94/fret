use fret_core::{EffectChain, EffectStep, Px};
use fret_ui::Theme;

use crate::recipes::effect_recipe::clamp_u32_from_metric;
use crate::recipes::resolve::{DegradationReason, ResolvedWithFallback};

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
