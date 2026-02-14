use fret_core::Px;
use fret_core::scene::{EffectChain, EffectStep};

use crate::recipes::resolve::ResolvedWithFallback;

#[derive(Debug, Clone, Copy)]
pub struct BloomEffect {
    /// Luminance cutoff in `[0, 1]` for the threshold stage.
    pub cutoff: f32,
    /// Soft edge width (best-effort), in `[0, 1]`.
    pub soft: f32,
    /// Blur radius in logical pixels.
    pub blur_radius_px: Px,
    /// Blur downsample factor (1 = full-res).
    pub blur_downsample: u32,
    /// Alpha multiplier applied after blur; higher values intensify the glow.
    pub strength: f32,
}

impl Default for BloomEffect {
    fn default() -> Self {
        Self {
            cutoff: 0.65,
            soft: 0.12,
            blur_radius_px: Px(14.0),
            blur_downsample: 1,
            strength: 1.35,
        }
    }
}

fn color_matrix_luma_to_alpha() -> [f32; 20] {
    // Assumes the renderer's color-matrix step operates on unpremultiplied rgb + alpha and
    // returns premultiplied output.
    //
    // Preserve rgb; set alpha = luma(rgb).
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.2126, 0.7152, 0.0722, 0.0, //
        0.0, 0.0, 0.0, 0.0, //
    ]
}

fn color_matrix_alpha_mul(strength: f32) -> [f32; 20] {
    let s = if strength.is_finite() { strength } else { 1.0 };
    [
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, s, //
        0.0, 0.0, 0.0, 0.0, //
    ]
}

pub fn bloom_effect_chain(effect: BloomEffect) -> ResolvedWithFallback<EffectChain> {
    let cutoff = if effect.cutoff.is_finite() {
        effect.cutoff.clamp(0.0, 1.0)
    } else {
        0.65
    };
    let soft = if effect.soft.is_finite() {
        effect.soft.clamp(0.0, 1.0)
    } else {
        0.12
    };

    let blur_radius_px = if effect.blur_radius_px.0.is_finite() {
        Px(effect.blur_radius_px.0.max(0.0))
    } else {
        Px(0.0)
    };
    let blur_downsample = effect.blur_downsample.clamp(1, 4);

    let chain = EffectChain::from_steps(&[
        EffectStep::ColorMatrix {
            m: color_matrix_luma_to_alpha(),
        },
        EffectStep::AlphaThreshold { cutoff, soft },
        EffectStep::GaussianBlur {
            radius_px: blur_radius_px,
            downsample: blur_downsample,
        },
        EffectStep::ColorMatrix {
            m: color_matrix_alpha_mul(effect.strength),
        },
    ]);

    ResolvedWithFallback::ok(chain)
}
