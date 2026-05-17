use super::effects;
use crate::renderer::{DebugPostprocess, EffectMarkerKind, PlanTarget, SceneEncoding};

#[derive(Clone, Copy, Debug)]
pub(super) struct RenderPlanPreflight {
    pub(super) postprocess: DebugPostprocess,
    pub(super) scene_target: PlanTarget,
    pub(super) scissor_sized_intermediates: bool,
}

pub(super) fn plan_render_targets(
    encoding: &SceneEncoding,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    postprocess: DebugPostprocess,
    intermediate_budget_bytes: u64,
) -> RenderPlanPreflight {
    let mut postprocess = postprocess;
    let output_transfer_needed = super::super::output_requires_explicit_srgb_encode(format);
    let scissor_sized_intermediates = scissor_sized_intermediates_enabled(encoding);

    let needs_intermediate =
        backdrop_effect_enabled(encoding, viewport_size, format, intermediate_budget_bytes)
            || matches!(
                postprocess,
                DebugPostprocess::OffscreenBlit { .. }
                    | DebugPostprocess::Pixelate { .. }
                    | DebugPostprocess::Blur { .. }
            );

    if needs_intermediate && matches!(postprocess, DebugPostprocess::None) {
        postprocess = DebugPostprocess::OffscreenBlit {
            src: PlanTarget::Intermediate0,
        };
    }

    let mut scene_target = if needs_intermediate {
        PlanTarget::Intermediate0
    } else {
        PlanTarget::Output
    };

    if scene_target == PlanTarget::Output
        && output_transfer_needed
        && matches!(postprocess, DebugPostprocess::None)
    {
        scene_target = PlanTarget::Intermediate3;
        postprocess = DebugPostprocess::OffscreenBlit { src: scene_target };
    }

    RenderPlanPreflight {
        postprocess,
        scene_target,
        scissor_sized_intermediates,
    }
}

fn scissor_sized_intermediates_enabled(encoding: &SceneEncoding) -> bool {
    !encoding
        .effect_markers
        .iter()
        .any(|marker| match marker.kind {
            EffectMarkerKind::Push { mode, .. } => mode == fret_core::EffectMode::Backdrop,
            _ => false,
        })
}

fn backdrop_effect_enabled(
    encoding: &SceneEncoding,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    intermediate_budget_bytes: u64,
) -> bool {
    encoding.effect_markers.iter().any(|marker| {
        let EffectMarkerKind::Push {
            mode,
            chain,
            quality,
            scissor,
            ..
        } = marker.kind
        else {
            return false;
        };
        if mode != fret_core::EffectMode::Backdrop {
            return false;
        }

        chain.iter().any(|step| match step {
            fret_core::EffectStep::GaussianBlur {
                radius_px,
                downsample,
            } => {
                if !radius_px.0.is_finite() || radius_px.0 <= 0.0 {
                    return false;
                }
                effects::choose_effect_blur_downsample_scale(
                    viewport_size,
                    format,
                    intermediate_budget_bytes,
                    downsample,
                    quality,
                )
                .is_some()
            }
            fret_core::EffectStep::BackdropWarpV1(_w) => {
                effects::backdrop_warp_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::BackdropWarpV2(_w) => {
                effects::backdrop_warp_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::DropShadowV1(_s) => false,
            fret_core::EffectStep::ColorAdjust { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::ColorMatrix { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::AlphaThreshold { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::Pixelate { scale } => effects::pixelate_enabled(
                viewport_size,
                Some(scissor),
                format,
                intermediate_budget_bytes,
                scale,
            ),
            fret_core::EffectStep::Dither { .. } => {
                effects::dither_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::NoiseV1(_n) => {
                effects::noise_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::CustomV1 { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::CustomV2 { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::CustomV3 { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
        })
    })
}
