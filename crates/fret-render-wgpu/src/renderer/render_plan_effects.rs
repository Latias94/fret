use super::blur_primitive;
use super::frame_targets::downsampled_size;
use super::intermediate_pool::{
    estimate_clip_mask_bytes, estimate_mipped_texture_bytes, estimate_texture_bytes,
};
use super::{
    AlphaThresholdPass, BackdropWarpPass, BlurAxis, BlurPass, ClipMaskPass, ColorAdjustPass,
    ColorMatrixPass, CustomEffectPass, CustomEffectV2Pass, CustomEffectV3Pass, DitherPass,
    DropShadowPass, FullscreenBlitPass, LocalScissorRect, MaskRef, NoisePass, PlanTarget,
    RenderPlanPass, ScaleMode, ScaleNearestPass, ScissorRect,
};

fn required_bytes_for_full_size_targets(full_target_bytes: u64, target_count: u64) -> u64 {
    // Budget model convention: intermediate budgets are reasoned about as the sum of concurrent
    // full-viewport intermediate targets (plus optional mips / downsampled targets).
    full_target_bytes.saturating_mul(target_count)
}

fn base_required_bytes_for_srcdst_and_single_scratch(full_target_bytes: u64) -> u64 {
    required_bytes_for_full_size_targets(full_target_bytes, 2)
}

fn base_required_bytes_for_srcdst_and_two_scratch(full_target_bytes: u64) -> u64 {
    required_bytes_for_full_size_targets(full_target_bytes, 3)
}

fn base_required_bytes_for_srcdst_and_three_scratch(full_target_bytes: u64) -> u64 {
    required_bytes_for_full_size_targets(full_target_bytes, 4)
}

fn budget_excluding_full_size_targets(
    budget_bytes: u64,
    full_target_bytes: u64,
    excluded_targets: u64,
) -> u64 {
    budget_bytes.saturating_sub(required_bytes_for_full_size_targets(
        full_target_bytes,
        excluded_targets,
    ))
}

#[derive(Clone, Copy, Debug)]
pub(super) struct EffectCompileCtx {
    pub(super) viewport_size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) clear: wgpu::Color,
    pub(super) scale_factor: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CustomEffectChainBudgetEvidence {
    pub(super) effective_budget_bytes: u64,
    pub(super) base_required_bytes: u64,
    pub(super) base_required_full_targets: u32,
    pub(super) optional_mask_bytes: u64,
    pub(super) optional_pyramid_bytes: u64,
}

impl CustomEffectChainBudgetEvidence {
    pub(super) fn optional_required_bytes(&self) -> u64 {
        self.optional_mask_bytes
            .saturating_add(self.optional_pyramid_bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CustomV3PyramidDegradeReason {
    BudgetZero,
    BudgetInsufficient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CustomV3PyramidChoice {
    pub(super) levels: u32,
    pub(super) degraded_to_one: Option<CustomV3PyramidDegradeReason>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BackdropSourceGroupCtx {
    pub(super) raw_target: PlanTarget,
    pub(super) pyramid: Option<CustomV3PyramidChoice>,
    pub(super) scissor: ScissorRect,
    pub(super) pyramid_pad_px: u32,
}

mod blur;
mod builtin;
mod chain;
mod custom;
mod scissor;

use self::blur::{
    compile_drop_shadow_in_place_masked, compile_gaussian_blur_in_place,
    compile_gaussian_blur_in_place_masked,
};
use self::builtin::{
    append_scissored_blur_in_place_single_scratch, append_scissored_blur_in_place_two_scratch,
    apply_alpha_threshold_step, apply_alpha_threshold_step_masked, apply_backdrop_warp_v1_step,
    apply_backdrop_warp_v1_step_masked, apply_backdrop_warp_v2_step,
    apply_backdrop_warp_v2_step_masked, apply_color_adjust_step, apply_color_adjust_step_masked,
    apply_color_matrix_step, apply_color_matrix_step_masked, apply_dither_step,
    apply_dither_step_masked, apply_noise_step, apply_noise_step_masked, apply_pixelate_step,
    apply_pixelate_step_masked, choose_clip_mask_target_capped, effect_blur_desired_downsample,
};
use self::chain::{ChainCoverage, try_apply_padded_chain_in_place};
use self::custom::{
    apply_custom_v1_step, apply_custom_v1_step_masked, apply_custom_v2_step,
    apply_custom_v2_step_masked, apply_custom_v3_step, apply_custom_v3_step_masked,
};

#[cfg(test)]
use self::blur::inflate_scissor_to_viewport;
#[cfg(test)]
use self::custom::plan_custom_v3_sources_and_charge_budget;

pub(super) fn choose_effect_blur_downsample_scale(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    requested_downsample: u32,
    quality: fret_core::EffectQuality,
) -> Option<u32> {
    builtin::choose_effect_blur_downsample_scale(
        viewport_size,
        format,
        budget_bytes,
        requested_downsample,
        quality,
    )
}

pub(super) fn color_adjust_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    builtin::color_adjust_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn dither_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    builtin::dither_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn noise_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    builtin::noise_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn backdrop_warp_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    builtin::backdrop_warp_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn pixelate_enabled(
    viewport_size: (u32, u32),
    scissor: Option<ScissorRect>,
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    scale: u32,
) -> bool {
    builtin::pixelate_enabled(viewport_size, scissor, format, budget_bytes, scale)
}

pub(super) fn choose_custom_v3_pyramid_choice_for_request(
    req: fret_core::scene::CustomEffectPyramidRequestV1,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    base_required_bytes: u64,
) -> CustomV3PyramidChoice {
    custom::choose_custom_v3_pyramid_choice_for_request(
        req,
        size,
        format,
        budget_bytes,
        base_required_bytes,
    )
}

pub(super) fn estimate_custom_v3_pyramid_bytes(
    size: (u32, u32),
    format: wgpu::TextureFormat,
    levels: u32,
) -> u64 {
    custom::estimate_custom_v3_pyramid_bytes(size, format, levels)
}

pub(super) fn map_scissor_to_size(
    scissor: Option<ScissorRect>,
    src_size: (u32, u32),
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    scissor::map_scissor_to_size(scissor, src_size, dst_size)
}

pub(super) fn map_scissor_downsample_nearest(
    scissor: Option<ScissorRect>,
    scale: u32,
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    scissor::map_scissor_downsample_nearest(scissor, scale, dst_size)
}

pub(super) fn available_scratch_targets(
    in_use_targets: &[PlanTarget],
    srcdst: PlanTarget,
) -> Vec<PlanTarget> {
    let mut out: Vec<PlanTarget> = Vec::new();
    for t in [
        PlanTarget::Intermediate0,
        PlanTarget::Intermediate1,
        PlanTarget::Intermediate2,
        PlanTarget::Intermediate3,
    ] {
        if t == srcdst {
            continue;
        }
        if in_use_targets.contains(&t) {
            continue;
        }
        out.push(t);
    }
    out
}

fn is_custom_effect_step(step: &fret_core::EffectStep) -> bool {
    matches!(
        *step,
        fret_core::EffectStep::CustomV1 { .. }
            | fret_core::EffectStep::CustomV2 { .. }
            | fret_core::EffectStep::CustomV3 { .. }
    )
}

fn step_wants_custom_v3_raw(step: &fret_core::EffectStep) -> bool {
    matches!(
        *step,
        fret_core::EffectStep::CustomV3 { sources, .. } if sources.want_raw
    )
}

fn backdrop_source_group_parts(
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
) -> (
    Option<PlanTarget>,
    Option<CustomV3PyramidChoice>,
    Option<(ScissorRect, u32)>,
) {
    let group_raw = backdrop_source_group.map(|g| g.raw_target);
    let group_pyramid = backdrop_source_group.and_then(|g| g.pyramid);
    let group_pyramid_roi =
        backdrop_source_group.and_then(|g| g.pyramid.map(|_| (g.scissor, g.pyramid_pad_px)));
    (group_raw, group_pyramid, group_pyramid_roi)
}

pub(super) fn apply_chain_in_place(
    passes: &mut Vec<RenderPlanPass>,
    in_use_targets: &[PlanTarget],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    chain: fret_core::EffectChain,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    mask_uniform_index: Option<u32>,
    unavailable_mask_targets: &[PlanTarget],
    effect_degradations: &mut super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::BlurQualitySnapshot,
    ctx: EffectCompileCtx,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
) -> Option<CustomEffectChainBudgetEvidence> {
    if srcdst == PlanTarget::Output || scissor.w == 0 || scissor.h == 0 {
        return None;
    }

    let steps: Vec<fret_core::EffectStep> = chain.iter().collect();
    if steps.is_empty() {
        return None;
    }

    let (group_raw, _, _) = backdrop_source_group_parts(backdrop_source_group);

    let mut budget_bytes = ctx.intermediate_budget_bytes;
    let srcdst_bytes = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    let chain_has_custom_effect = steps.iter().any(is_custom_effect_step);
    let full_target_bytes = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    let mut custom_chain_budget =
        chain_has_custom_effect.then_some(CustomEffectChainBudgetEvidence {
            effective_budget_bytes: ctx.intermediate_budget_bytes,
            base_required_bytes: base_required_bytes_for_srcdst_and_single_scratch(
                full_target_bytes,
            ),
            base_required_full_targets: 2,
            optional_mask_bytes: 0,
            optional_pyramid_bytes: 0,
        });

    let scratch_targets = available_scratch_targets(in_use_targets, srcdst);
    let forced_quarter_blur = scratch_targets.len() >= 2
        && steps.iter().any(|step| match *step {
            fret_core::EffectStep::GaussianBlur {
                radius_px,
                downsample,
            } => {
                if !radius_px.0.is_finite() || radius_px.0 <= 0.0 {
                    return false;
                }
                let requested_downsample = if downsample >= 4 { 4 } else { 2 };
                let desired_downsample =
                    effect_blur_desired_downsample(requested_downsample, quality);
                if desired_downsample != 2 {
                    return false;
                }
                let Some(chosen) = choose_effect_blur_downsample_scale(
                    ctx.viewport_size,
                    ctx.format,
                    budget_bytes,
                    requested_downsample,
                    quality,
                ) else {
                    return false;
                };
                chosen == 4
            }
            _ => false,
        });
    let mask_tier_cap = forced_quarter_blur.then_some(PlanTarget::Mask2);

    let mut chosen_mask_bytes: u64 = 0;
    let mask = if let Some(uniform_index) = mask_uniform_index
        && let Some((mask_target, mask_size, mask_bytes)) = choose_clip_mask_target_capped(
            ctx.viewport_size,
            scissor,
            budget_bytes,
            srcdst_bytes,
            quality,
            mask_tier_cap,
            unavailable_mask_targets,
        ) {
        passes.push(RenderPlanPass::ClipMask(ClipMaskPass {
            dst: mask_target,
            dst_size: mask_size,
            dst_scissor: None,
            uniform_index,
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        }));
        chosen_mask_bytes = mask_bytes;
        budget_bytes = budget_bytes.saturating_sub(mask_bytes);
        Some(MaskRef {
            target: mask_target,
            size: mask_size,
            viewport_rect: scissor,
        })
    } else {
        None
    };

    if let Some(e) = custom_chain_budget.as_mut() {
        e.optional_mask_bytes = chosen_mask_bytes;
    }

    // Padded effect chains:
    //
    // Some effects sample their input outside the destination pixel (blur radius, refraction
    // displacement, chromatic offsets). When such an effect appears later in a chain, earlier
    // steps must produce output for an expanded region, otherwise the chain will see edge artifacts
    // (e.g. "dark corners" in blur -> refraction lenses).
    //
    // If the chain declares any non-zero sampling padding and we have enough scratch targets +
    // budget, we evaluate the chain into a work target using a per-step expanded scissor plan,
    // then composite the final result back into `srcdst` while applying clip/mask coverage exactly
    // once.
    if try_apply_padded_chain_in_place(
        passes,
        &steps,
        scratch_targets.as_slice(),
        srcdst,
        mode,
        quality,
        scissor,
        budget_bytes,
        ChainCoverage {
            mask_uniform_index,
            mask,
        },
        effect_degradations,
        effect_blur_quality,
        ctx,
        backdrop_source_group,
        &mut custom_chain_budget,
    ) {
        return custom_chain_budget;
    }

    let chain_wants_raw =
        group_raw.is_none() && steps.len() >= 2 && steps.iter().any(step_wants_custom_v3_raw);

    let full = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    let (chain_raw, scratch_targets): (Option<PlanTarget>, &[PlanTarget]) = if chain_wants_raw
        && scratch_targets.len() >= 2
        && budget_bytes >= base_required_bytes_for_srcdst_and_single_scratch(full)
    {
        let chain_raw = scratch_targets[0];
        passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: srcdst,
            dst: chain_raw,
            src_size: ctx.viewport_size,
            dst_size: ctx.viewport_size,
            dst_scissor: None,
            encode_output_srgb: false,
            load: wgpu::LoadOp::Clear(ctx.clear),
        }));
        budget_bytes = budget_excluding_full_size_targets(budget_bytes, full, 1);
        if let Some(e) = custom_chain_budget.as_mut() {
            e.base_required_full_targets = e.base_required_full_targets.max(3);
            e.base_required_bytes = required_bytes_for_full_size_targets(full, 3);
        }
        (Some(chain_raw), &scratch_targets[1..])
    } else {
        (None, scratch_targets.as_slice())
    };

    // Clip/shape masks are coverage (alpha) multipliers. If we apply them at every effect step in a
    // chain, coverage compounds (e.g. clip^2) and produces visible edge darkening (especially
    // around rounded corners) for common chains like blur -> custom refraction.
    //
    // To avoid this, we apply clip/mask only on the final step of the chain and keep intermediate
    // steps unmasked.
    let chain_mask_uniform_index = mask_uniform_index;
    let chain_mask = mask;
    let step_count = steps.len();

    for (step_index, step) in steps.into_iter().enumerate() {
        let apply_mask = step_index + 1 == step_count;
        let mask_uniform_index = apply_mask.then_some(chain_mask_uniform_index).flatten();
        let mask = apply_mask.then_some(chain_mask).flatten();

        match step {
            fret_core::EffectStep::GaussianBlur {
                radius_px,
                downsample,
            } => {
                let radius_px = if radius_px.0.is_finite() {
                    (radius_px.0 * ctx.scale_factor).max(0.0)
                } else {
                    0.0
                };
                compile_gaussian_blur_in_place_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    quality,
                    scissor,
                    downsample,
                    radius_px,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    effect_blur_quality,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::BackdropWarpV1(w) => {
                apply_backdrop_warp_v1_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    mode,
                    scissor,
                    w,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::BackdropWarpV2(w) => {
                apply_backdrop_warp_v2_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    mode,
                    scissor,
                    w,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::DropShadowV1(s) => {
                compile_drop_shadow_in_place_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    mode,
                    quality,
                    scissor,
                    s,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    effect_blur_quality,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::ColorAdjust {
                saturation,
                brightness,
                contrast,
            } => {
                apply_color_adjust_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    scissor,
                    saturation,
                    brightness,
                    contrast,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::ColorMatrix { m } => {
                apply_color_matrix_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    scissor,
                    m,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::AlphaThreshold { cutoff, soft } => {
                apply_alpha_threshold_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    scissor,
                    cutoff,
                    soft,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::Pixelate { scale } => {
                if scale <= 1 {
                    continue;
                }
                apply_pixelate_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    scissor,
                    scale,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::Dither { mode } => {
                apply_dither_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    scissor,
                    mode,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::NoiseV1(n) => {
                let n = n.sanitize();
                if n.strength <= 0.0 {
                    continue;
                }
                apply_noise_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    scissor,
                    n,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::CustomV1 {
                id,
                params,
                max_sample_offset_px: _,
            } => {
                apply_custom_v1_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    scissor,
                    id,
                    params,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::CustomV2 {
                id,
                params,
                max_sample_offset_px: _,
                input_image,
            } => {
                apply_custom_v2_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    scissor,
                    id,
                    params,
                    input_image,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::CustomV3 {
                id,
                params,
                max_sample_offset_px: _,
                user0,
                user1,
                sources,
            } => {
                apply_custom_v3_step_masked(
                    passes,
                    scratch_targets,
                    srcdst,
                    scissor,
                    id,
                    params,
                    user0,
                    user1,
                    sources,
                    ctx,
                    &mut budget_bytes,
                    effect_degradations,
                    chain_raw,
                    backdrop_source_group,
                    &mut custom_chain_budget,
                    mask_uniform_index,
                    mask,
                );
            }
        }
    }

    custom_chain_budget
}

#[allow(clippy::too_many_arguments)]
fn apply_step_in_place_with_scratch_targets(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    step: fret_core::EffectStep,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::BlurQualitySnapshot,
    ctx: EffectCompileCtx,
    custom_v3_chain_raw: Option<PlanTarget>,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
    custom_chain_budget: &mut Option<CustomEffectChainBudgetEvidence>,
) {
    match step {
        fret_core::EffectStep::GaussianBlur {
            radius_px,
            downsample,
        } => {
            let radius_px = if radius_px.0.is_finite() {
                (radius_px.0 * ctx.scale_factor).max(0.0)
            } else {
                0.0
            };
            compile_gaussian_blur_in_place(
                passes,
                scratch_targets,
                srcdst,
                quality,
                scissor,
                downsample,
                radius_px,
                ctx,
                budget_bytes,
                effect_degradations,
                effect_blur_quality,
            );
        }
        fret_core::EffectStep::BackdropWarpV1(w) => {
            apply_backdrop_warp_v1_step(
                passes,
                scratch_targets,
                srcdst,
                mode,
                scissor,
                w,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::BackdropWarpV2(w) => {
            apply_backdrop_warp_v2_step(
                passes,
                scratch_targets,
                srcdst,
                mode,
                scissor,
                w,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::NoiseV1(n) => {
            apply_noise_step(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                n,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::ColorAdjust {
            saturation,
            brightness,
            contrast,
        } => {
            apply_color_adjust_step(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                saturation,
                brightness,
                contrast,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::ColorMatrix { m } => {
            apply_color_matrix_step(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                m,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::AlphaThreshold { cutoff, soft } => {
            apply_alpha_threshold_step(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                cutoff,
                soft,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::Pixelate { scale } => {
            apply_pixelate_step(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                scale,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::Dither { mode } => {
            apply_dither_step(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                mode,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::CustomV1 {
            id,
            params,
            max_sample_offset_px: _,
        } => {
            apply_custom_v1_step(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                id,
                params,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::CustomV2 {
            id,
            params,
            max_sample_offset_px: _,
            input_image,
        } => {
            apply_custom_v2_step(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                id,
                params,
                input_image,
                ctx,
                budget_bytes,
                effect_degradations,
            );
        }
        fret_core::EffectStep::CustomV3 {
            id,
            params,
            max_sample_offset_px: _,
            user0,
            user1,
            sources,
        } => {
            apply_custom_v3_step(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                id,
                params,
                user0,
                user1,
                sources,
                ctx,
                budget_bytes,
                effect_degradations,
                custom_v3_chain_raw,
                backdrop_source_group,
                custom_chain_budget,
            );
        }
        fret_core::EffectStep::DropShadowV1(_) => {}
    }
}

#[cfg(test)]
mod tests;
