use fret_core::{EffectChain, EffectStep, Px};
use fret_ui::Theme;

use crate::recipes::effect_recipe::clamp_u32_from_metric;

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
