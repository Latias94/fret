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

mod builtin;
mod custom;
mod scissor;

use self::builtin::{
    append_alpha_threshold_in_place_single_scratch, append_backdrop_warp_in_place_single_scratch,
    append_color_adjust_in_place_single_scratch, append_color_matrix_in_place_single_scratch,
    append_dither_in_place_single_scratch, append_noise_in_place_single_scratch,
    append_pixelate_in_place_single_scratch, append_scissored_blur_in_place_single_scratch,
    append_scissored_blur_in_place_two_scratch, choose_clip_mask_target_capped,
    effect_blur_desired_downsample, scale_backdrop_warp_v1,
};
use self::custom::{
    append_custom_effect_in_place_single_scratch, append_custom_effect_v2_in_place_single_scratch,
    append_custom_effect_v3_in_place_single_scratch, plan_custom_v3_sources_and_charge_budget,
};

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

    let group_raw = backdrop_source_group.map(|g| g.raw_target);
    let group_pyramid = backdrop_source_group.and_then(|g| g.pyramid);
    let group_pyramid_roi =
        backdrop_source_group.and_then(|g| g.pyramid.map(|_| (g.scissor, g.pyramid_pad_px)));

    let mut budget_bytes = ctx.intermediate_budget_bytes;
    let srcdst_bytes = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    let chain_has_custom_effect = steps.iter().any(|s| {
        matches!(
            *s,
            fret_core::EffectStep::CustomV1 { .. }
                | fret_core::EffectStep::CustomV2 { .. }
                | fret_core::EffectStep::CustomV3 { .. }
        )
    });
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
    if steps.len() >= 2 {
        let step_scissors = padded_chain_step_scissors(&steps, scissor, ctx.viewport_size, ctx);
        let needs_padding = step_scissors.first().is_some_and(|s| *s != scissor);
        let has_drop_shadow = steps
            .iter()
            .any(|s| matches!(*s, fret_core::EffectStep::DropShadowV1(_)));

        let full = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
        let min_budget_for_work = base_required_bytes_for_srcdst_and_two_scratch(full);
        let has_work = scratch_targets.len() >= 2;
        let has_mask = mask_uniform_index.is_some() || mask.is_some();
        let last_step_is_custom = steps.last().is_some_and(|s| {
            matches!(
                *s,
                fret_core::EffectStep::CustomV1 { .. }
                    | fret_core::EffectStep::CustomV2 { .. }
                    | fret_core::EffectStep::CustomV3 { .. }
            )
        });
        let can_commit_with_mask = !has_mask
            || last_step_is_custom
            || color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes);

        if needs_padding
            && !has_drop_shadow
            && has_work
            && budget_bytes >= min_budget_for_work
            && can_commit_with_mask
        {
            let work = scratch_targets[0];
            let chain_wants_raw = steps.iter().any(|s| match *s {
                fret_core::EffectStep::CustomV3 { sources, .. } => sources.want_raw,
                _ => false,
            });
            let chain_raw = (chain_wants_raw
                && group_raw.is_none()
                && scratch_targets.len() >= 3
                && budget_bytes >= base_required_bytes_for_srcdst_and_three_scratch(full))
            .then_some(scratch_targets[1]);

            if let Some(e) = custom_chain_budget.as_mut() {
                let required_full_targets = if chain_raw.is_some() { 4 } else { 3 };
                e.base_required_full_targets = required_full_targets;
                e.base_required_bytes =
                    required_bytes_for_full_size_targets(full, required_full_targets as u64);
            }

            let work_scratch_targets = if chain_raw.is_some() {
                &scratch_targets[2..]
            } else {
                &scratch_targets[1..]
            };

            if let Some(chain_raw) = chain_raw {
                passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                    src: srcdst,
                    dst: chain_raw,
                    src_size: ctx.viewport_size,
                    dst_size: ctx.viewport_size,
                    dst_scissor: None,
                    encode_output_srgb: false,
                    load: wgpu::LoadOp::Clear(ctx.clear),
                }));
            }

            passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                src: srcdst,
                dst: work,
                src_size: ctx.viewport_size,
                dst_size: ctx.viewport_size,
                dst_scissor: None,
                encode_output_srgb: false,
                load: wgpu::LoadOp::Clear(ctx.clear),
            }));

            let excluded_targets = 1u64.saturating_add(chain_raw.is_some() as u64);
            let mut work_budget_bytes =
                budget_excluding_full_size_targets(budget_bytes, full, excluded_targets);

            // Apply all steps except the final one in-place on the work buffer using per-step
            // padded scissors. The final step is handled separately so we can commit the result
            // back into `srcdst` while applying clip/mask coverage exactly once.
            let (head_steps, tail_step) = steps.split_at(steps.len().saturating_sub(1));
            let (head_scissors, tail_scissor) =
                step_scissors.split_at(step_scissors.len().saturating_sub(1));

            for (step, step_scissor) in head_steps
                .iter()
                .copied()
                .zip(head_scissors.iter().copied())
            {
                apply_step_in_place_with_scratch_targets(
                    passes,
                    work_scratch_targets,
                    work,
                    mode,
                    step,
                    quality,
                    step_scissor,
                    &mut work_budget_bytes,
                    effect_degradations,
                    effect_blur_quality,
                    ctx,
                    chain_raw,
                    backdrop_source_group,
                    &mut custom_chain_budget,
                );
            }

            if let Some(&final_step) = tail_step.first()
                && let Some(&final_scissor) = tail_scissor.first()
                && last_step_is_custom
                && matches!(
                    final_step,
                    fret_core::EffectStep::CustomV1 { .. }
                        | fret_core::EffectStep::CustomV2 { .. }
                        | fret_core::EffectStep::CustomV3 { .. }
                )
            {
                // Optimized path: commit the final Custom effect step directly into `srcdst`, reading
                // from the padded work buffer and applying clip/mask coverage exactly once.
                debug_assert_eq!(
                    final_scissor, scissor,
                    "final scissor must be the original bounds"
                );
                match final_step {
                    fret_core::EffectStep::CustomV1 {
                        id,
                        params,
                        max_sample_offset_px: _,
                    } => {
                        passes.push(RenderPlanPass::CustomEffect(CustomEffectPass {
                            common: super::CustomEffectPassCommon {
                                src: work,
                                dst: srcdst,
                                src_size: ctx.viewport_size,
                                dst_size: ctx.viewport_size,
                                dst_scissor: Some(LocalScissorRect(scissor)),
                                mask_uniform_index,
                                mask,
                                effect: id,
                                params,
                                load: wgpu::LoadOp::Load,
                            },
                        }));
                    }
                    fret_core::EffectStep::CustomV2 {
                        id,
                        params,
                        max_sample_offset_px: _,
                        input_image,
                    } => {
                        let (input_image, input_uv, input_sampling) = match input_image {
                            None => (
                                None,
                                fret_core::scene::UvRect::FULL,
                                fret_core::scene::ImageSamplingHint::Default,
                            ),
                            Some(input) => (Some(input.image), input.uv, input.sampling),
                        };
                        passes.push(RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                            common: super::CustomEffectPassCommon {
                                src: work,
                                dst: srcdst,
                                src_size: ctx.viewport_size,
                                dst_size: ctx.viewport_size,
                                dst_scissor: Some(LocalScissorRect(scissor)),
                                mask_uniform_index,
                                mask,
                                effect: id,
                                params,
                                load: wgpu::LoadOp::Load,
                            },
                            input_image,
                            input_uv,
                            input_sampling,
                        }));
                    }
                    fret_core::EffectStep::CustomV3 {
                        id,
                        params,
                        max_sample_offset_px: _,
                        user0,
                        user1,
                        sources,
                    } => {
                        let (user0_image, user0_uv, user0_sampling) = match user0 {
                            None => (
                                None,
                                fret_core::scene::UvRect::FULL,
                                fret_core::scene::ImageSamplingHint::Default,
                            ),
                            Some(input) => (Some(input.image), input.uv, input.sampling),
                        };
                        let (user1_image, user1_uv, user1_sampling) = match user1 {
                            None => (
                                None,
                                fret_core::scene::UvRect::FULL,
                                fret_core::scene::ImageSamplingHint::Default,
                            ),
                            Some(input) => (Some(input.image), input.uv, input.sampling),
                        };

                        let v3_chain_raw = group_raw.or(chain_raw);
                        let v3_sources_plan = plan_custom_v3_sources_and_charge_budget(
                            sources,
                            work,
                            v3_chain_raw,
                            group_pyramid,
                            group_pyramid_roi,
                            scissor,
                            ctx,
                            &mut work_budget_bytes,
                            estimate_texture_bytes(ctx.viewport_size, ctx.format, 1),
                            &mut effect_degradations.custom_effect_v3_sources,
                        );

                        if let Some(e) = custom_chain_budget.as_mut()
                            && group_pyramid.is_none()
                            && sources.pyramid.is_some()
                            && v3_sources_plan.pyramid_levels >= 2
                        {
                            e.optional_pyramid_bytes = e.optional_pyramid_bytes.saturating_add(
                                estimate_custom_v3_pyramid_bytes(
                                    ctx.viewport_size,
                                    ctx.format,
                                    v3_sources_plan.pyramid_levels,
                                ),
                            );
                        }

                        passes.push(RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
                            src_raw: v3_sources_plan.src_raw,
                            src_pyramid: v3_sources_plan.src_raw,
                            pyramid_levels: v3_sources_plan.pyramid_levels,
                            pyramid_build_scissor: v3_sources_plan.pyramid_build_scissor,
                            raw_wanted: sources.want_raw,
                            pyramid_wanted: sources.pyramid.is_some(),
                            common: super::CustomEffectPassCommon {
                                src: work,
                                dst: srcdst,
                                src_size: ctx.viewport_size,
                                dst_size: ctx.viewport_size,
                                dst_scissor: Some(LocalScissorRect(scissor)),
                                mask_uniform_index,
                                mask,
                                effect: id,
                                params,
                                load: wgpu::LoadOp::Load,
                            },
                            user0_image,
                            user0_uv,
                            user0_sampling,
                            user1_image,
                            user1_uv,
                            user1_sampling,
                        }));
                    }
                    _ => {}
                }
            } else {
                // Fallback: apply the final step in-place on the work buffer, then composite the
                // chain result back into `srcdst` under the original scissor.
                if let Some(&final_step) = tail_step.first()
                    && let Some(&final_scissor) = tail_scissor.first()
                {
                    apply_step_in_place_with_scratch_targets(
                        passes,
                        work_scratch_targets,
                        work,
                        mode,
                        final_step,
                        quality,
                        final_scissor,
                        &mut work_budget_bytes,
                        effect_degradations,
                        effect_blur_quality,
                        ctx,
                        chain_raw,
                        backdrop_source_group,
                        &mut custom_chain_budget,
                    );
                }

                // For clip/mask coverage, we want the "masked blend" semantics used by effect passes
                // (blend RGB, keep destination alpha), so we reuse a no-op ColorAdjust pass when a mask
                // is present. This avoids having to introduce a dedicated masked blit pass.
                if has_mask {
                    passes.push(RenderPlanPass::ColorAdjust(ColorAdjustPass {
                        src: work,
                        dst: srcdst,
                        src_size: ctx.viewport_size,
                        dst_size: ctx.viewport_size,
                        dst_scissor: Some(LocalScissorRect(scissor)),
                        mask_uniform_index,
                        mask,
                        saturation: 1.0,
                        brightness: 1.0,
                        contrast: 1.0,
                        load: wgpu::LoadOp::Load,
                    }));
                } else {
                    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                        src: work,
                        dst: srcdst,
                        src_size: ctx.viewport_size,
                        dst_size: ctx.viewport_size,
                        dst_scissor: Some(LocalScissorRect(scissor)),
                        encode_output_srgb: false,
                        load: wgpu::LoadOp::Load,
                    }));
                }
            }

            return custom_chain_budget;
        }
    }

    let chain_wants_raw = group_raw.is_none()
        && steps.len() >= 2
        && steps.iter().any(|s| match *s {
            fret_core::EffectStep::CustomV3 { sources, .. } => sources.want_raw,
            _ => false,
        });

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
                if radius_px <= 0.0 {
                    continue;
                }
                effect_degradations.gaussian_blur.requested = effect_degradations
                    .gaussian_blur
                    .requested
                    .saturating_add(1);

                let requested_downsample = if downsample >= 4 { 4 } else { 2 };
                let desired_downsample =
                    effect_blur_desired_downsample(requested_downsample, quality);
                if scratch_targets.len() >= 2 {
                    let Some(downsample_scale) = choose_effect_blur_downsample_scale(
                        ctx.viewport_size,
                        ctx.format,
                        budget_bytes,
                        requested_downsample,
                        quality,
                    ) else {
                        // Downsampled two-scratch blur does not fit. Fall back to a single-scratch
                        // blur if possible, otherwise degrade to no-op.
                        let Some(&scratch) = scratch_targets.first() else {
                            effect_degradations.gaussian_blur.degraded_target_exhausted =
                                effect_degradations
                                    .gaussian_blur
                                    .degraded_target_exhausted
                                    .saturating_add(1);
                            effect_blur_quality.gaussian_blur.record_applied(
                                1,
                                0,
                                desired_downsample,
                            );
                            effect_blur_quality
                                .gaussian_blur
                                .quality_degraded_blur_removed = effect_blur_quality
                                .gaussian_blur
                                .quality_degraded_blur_removed
                                .saturating_add(1);
                            continue;
                        };
                        if budget_bytes == 0 {
                            effect_degradations.gaussian_blur.degraded_budget_zero =
                                effect_degradations
                                    .gaussian_blur
                                    .degraded_budget_zero
                                    .saturating_add(1);
                            effect_blur_quality.gaussian_blur.record_applied(
                                1,
                                0,
                                desired_downsample,
                            );
                            effect_blur_quality
                                .gaussian_blur
                                .quality_degraded_blur_removed = effect_blur_quality
                                .gaussian_blur
                                .quality_degraded_blur_removed
                                .saturating_add(1);
                            continue;
                        }
                        let full = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
                        let required = base_required_bytes_for_srcdst_and_single_scratch(full);
                        if required > budget_bytes {
                            effect_degradations
                                .gaussian_blur
                                .degraded_budget_insufficient = effect_degradations
                                .gaussian_blur
                                .degraded_budget_insufficient
                                .saturating_add(1);
                            effect_blur_quality.gaussian_blur.record_applied(
                                1,
                                0,
                                desired_downsample,
                            );
                            effect_blur_quality
                                .gaussian_blur
                                .quality_degraded_blur_removed = effect_blur_quality
                                .gaussian_blur
                                .quality_degraded_blur_removed
                                .saturating_add(1);
                            continue;
                        }

                        let iterations =
                            blur_primitive::blur_iterations_for_radius(radius_px, 1, quality);
                        effect_degradations.gaussian_blur.applied =
                            effect_degradations.gaussian_blur.applied.saturating_add(1);
                        effect_blur_quality.gaussian_blur.record_applied(
                            1,
                            iterations,
                            desired_downsample,
                        );
                        append_scissored_blur_in_place_single_scratch(
                            passes,
                            srcdst,
                            scratch,
                            ctx.viewport_size,
                            iterations,
                            scissor,
                            ctx.clear,
                            mask_uniform_index,
                            mask,
                        );
                        continue;
                    };
                    let iterations = blur_primitive::blur_iterations_for_radius(
                        radius_px,
                        downsample_scale,
                        quality,
                    );
                    effect_degradations.gaussian_blur.applied =
                        effect_degradations.gaussian_blur.applied.saturating_add(1);
                    effect_blur_quality.gaussian_blur.record_applied(
                        downsample_scale,
                        iterations,
                        desired_downsample,
                    );
                    append_scissored_blur_in_place_two_scratch(
                        passes,
                        srcdst,
                        scratch_targets[0],
                        scratch_targets[1],
                        ctx.viewport_size,
                        downsample_scale,
                        iterations,
                        scissor,
                        ctx.clear,
                        mask_uniform_index,
                        mask,
                    );
                    continue;
                }

                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.gaussian_blur.degraded_target_exhausted =
                        effect_degradations
                            .gaussian_blur
                            .degraded_target_exhausted
                            .saturating_add(1);
                    effect_blur_quality
                        .gaussian_blur
                        .record_applied(1, 0, desired_downsample);
                    effect_blur_quality
                        .gaussian_blur
                        .quality_degraded_blur_removed = effect_blur_quality
                        .gaussian_blur
                        .quality_degraded_blur_removed
                        .saturating_add(1);
                    continue;
                };
                if budget_bytes == 0 {
                    effect_degradations.gaussian_blur.degraded_budget_zero = effect_degradations
                        .gaussian_blur
                        .degraded_budget_zero
                        .saturating_add(1);
                    effect_blur_quality
                        .gaussian_blur
                        .record_applied(1, 0, desired_downsample);
                    effect_blur_quality
                        .gaussian_blur
                        .quality_degraded_blur_removed = effect_blur_quality
                        .gaussian_blur
                        .quality_degraded_blur_removed
                        .saturating_add(1);
                    continue;
                }
                let full = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
                let required = base_required_bytes_for_srcdst_and_single_scratch(full);
                if required > budget_bytes {
                    effect_degradations
                        .gaussian_blur
                        .degraded_budget_insufficient = effect_degradations
                        .gaussian_blur
                        .degraded_budget_insufficient
                        .saturating_add(1);
                    effect_blur_quality
                        .gaussian_blur
                        .record_applied(1, 0, desired_downsample);
                    effect_blur_quality
                        .gaussian_blur
                        .quality_degraded_blur_removed = effect_blur_quality
                        .gaussian_blur
                        .quality_degraded_blur_removed
                        .saturating_add(1);
                    continue;
                }
                let iterations = blur_primitive::blur_iterations_for_radius(radius_px, 1, quality);
                effect_degradations.gaussian_blur.applied =
                    effect_degradations.gaussian_blur.applied.saturating_add(1);
                effect_blur_quality
                    .gaussian_blur
                    .record_applied(1, iterations, desired_downsample);
                append_scissored_blur_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    iterations,
                    scissor,
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::BackdropWarpV1(w) => {
                if mode != fret_core::EffectMode::Backdrop {
                    continue;
                }
                effect_degradations.backdrop_warp.requested = effect_degradations
                    .backdrop_warp
                    .requested
                    .saturating_add(1);
                if !color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.backdrop_warp.degraded_budget_zero =
                            effect_degradations
                                .backdrop_warp
                                .degraded_budget_zero
                                .saturating_add(1);
                    } else {
                        effect_degradations
                            .backdrop_warp
                            .degraded_budget_insufficient = effect_degradations
                            .backdrop_warp
                            .degraded_budget_insufficient
                            .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.backdrop_warp.degraded_target_exhausted =
                        effect_degradations
                            .backdrop_warp
                            .degraded_target_exhausted
                            .saturating_add(1);
                    continue;
                };
                effect_degradations.backdrop_warp.applied =
                    effect_degradations.backdrop_warp.applied.saturating_add(1);
                append_backdrop_warp_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    scissor,
                    scale_backdrop_warp_v1(w.sanitize(), ctx.scale_factor),
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::BackdropWarpV2(w) => {
                if mode != fret_core::EffectMode::Backdrop {
                    continue;
                }
                effect_degradations.backdrop_warp.requested = effect_degradations
                    .backdrop_warp
                    .requested
                    .saturating_add(1);
                if !color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.backdrop_warp.degraded_budget_zero =
                            effect_degradations
                                .backdrop_warp
                                .degraded_budget_zero
                                .saturating_add(1);
                    } else {
                        effect_degradations
                            .backdrop_warp
                            .degraded_budget_insufficient = effect_degradations
                            .backdrop_warp
                            .degraded_budget_insufficient
                            .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.backdrop_warp.degraded_target_exhausted =
                        effect_degradations
                            .backdrop_warp
                            .degraded_target_exhausted
                            .saturating_add(1);
                    continue;
                };
                effect_degradations.backdrop_warp.applied =
                    effect_degradations.backdrop_warp.applied.saturating_add(1);

                // Scissored in-place pattern: preserve outside-region content by pre-blitting into scratch.
                passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                    src: srcdst,
                    dst: scratch,
                    src_size: ctx.viewport_size,
                    dst_size: ctx.viewport_size,
                    dst_scissor: None,
                    encode_output_srgb: false,
                    load: wgpu::LoadOp::Clear(ctx.clear),
                }));

                let base = w.base.sanitize();
                let (warp_image, warp_uv, warp_sampling, warp_encoding) = match w.field {
                    fret_core::scene::BackdropWarpFieldV2::Procedural => (
                        None,
                        fret_core::scene::UvRect::FULL,
                        fret_core::scene::ImageSamplingHint::Default,
                        fret_core::scene::WarpMapEncodingV1::RgSigned,
                    ),
                    fret_core::scene::BackdropWarpFieldV2::ImageDisplacementMap {
                        image,
                        uv,
                        sampling,
                        encoding,
                    } => (Some(image), uv, sampling, encoding),
                };

                passes.push(RenderPlanPass::BackdropWarp(BackdropWarpPass {
                    src: scratch,
                    dst: srcdst,
                    src_size: ctx.viewport_size,
                    dst_size: ctx.viewport_size,
                    origin_px: (scissor.x, scissor.y),
                    bounds_size_px: (scissor.w, scissor.h),
                    dst_scissor: Some(LocalScissorRect(scissor)),
                    mask_uniform_index,
                    mask,
                    strength_px: base.strength_px.0 * ctx.scale_factor,
                    scale_px: base.scale_px.0 * ctx.scale_factor,
                    phase: base.phase,
                    chromatic_aberration_px: base.chromatic_aberration_px.0 * ctx.scale_factor,
                    kind: base.kind,
                    warp_image,
                    warp_uv,
                    warp_sampling,
                    warp_encoding,
                    load: wgpu::LoadOp::Load,
                }));
            }
            fret_core::EffectStep::DropShadowV1(s) => {
                if mode != fret_core::EffectMode::FilterContent {
                    continue;
                }

                let s = s.sanitize();
                if s.color.a <= 0.0 {
                    continue;
                }
                let blur_radius_px = if s.blur_radius_px.0.is_finite() {
                    (s.blur_radius_px.0 * ctx.scale_factor).max(0.0)
                } else {
                    0.0
                };
                if blur_radius_px <= 0.0 {
                    continue;
                }
                effect_degradations.drop_shadow.requested =
                    effect_degradations.drop_shadow.requested.saturating_add(1);
                if budget_bytes == 0 {
                    effect_degradations.drop_shadow.degraded_budget_zero = effect_degradations
                        .drop_shadow
                        .degraded_budget_zero
                        .saturating_add(1);
                    continue;
                }

                let full = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
                let requested_downsample = if s.downsample >= 4 {
                    4
                } else if s.downsample >= 2 {
                    2
                } else {
                    1
                };
                let desired_downsample = if requested_downsample <= 1 {
                    1
                } else {
                    effect_blur_desired_downsample(requested_downsample, quality)
                };

                // Preferred path: blurred drop shadow, which needs two scratch targets:
                // - one to preserve the original content (for later restore),
                // - one to store the blurred coverage that the shadow is sampled from.
                //
                // If budgets are too tight to hold the full blurred pipeline deterministically,
                // we fall back to a hard shadow (no blur) that uses only a single scratch target.
                let can_blur = scratch_targets.len() >= 2
                    && budget_bytes >= base_required_bytes_for_srcdst_and_two_scratch(full);
                if !can_blur {
                    let Some(&scratch_original) = scratch_targets.first() else {
                        effect_degradations.drop_shadow.degraded_target_exhausted =
                            effect_degradations
                                .drop_shadow
                                .degraded_target_exhausted
                                .saturating_add(1);
                        continue;
                    };
                    if budget_bytes < base_required_bytes_for_srcdst_and_single_scratch(full) {
                        effect_degradations.drop_shadow.degraded_budget_insufficient =
                            effect_degradations
                                .drop_shadow
                                .degraded_budget_insufficient
                                .saturating_add(1);
                        continue;
                    }

                    effect_degradations.drop_shadow.applied =
                        effect_degradations.drop_shadow.applied.saturating_add(1);
                    effect_blur_quality
                        .drop_shadow
                        .record_applied(1, 0, desired_downsample);
                    effect_blur_quality
                        .drop_shadow
                        .quality_degraded_blur_removed = effect_blur_quality
                        .drop_shadow
                        .quality_degraded_blur_removed
                        .saturating_add(1);

                    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                        src: srcdst,
                        dst: scratch_original,
                        src_size: ctx.viewport_size,
                        dst_size: ctx.viewport_size,
                        dst_scissor: None,
                        encode_output_srgb: false,
                        load: wgpu::LoadOp::Clear(ctx.clear),
                    }));
                    passes.push(RenderPlanPass::DropShadow(DropShadowPass {
                        src: scratch_original,
                        dst: srcdst,
                        src_size: ctx.viewport_size,
                        dst_size: ctx.viewport_size,
                        dst_scissor: Some(LocalScissorRect(scissor)),
                        mask_uniform_index,
                        mask,
                        offset_px: (
                            s.offset_px.x.0 * ctx.scale_factor,
                            s.offset_px.y.0 * ctx.scale_factor,
                        ),
                        color: s.color,
                        load: wgpu::LoadOp::Load,
                    }));
                    continue;
                }

                effect_degradations.drop_shadow.applied =
                    effect_degradations.drop_shadow.applied.saturating_add(1);

                let scratch_original = scratch_targets[0];
                let scratch_blurred = scratch_targets[1];

                // Preserve the original content, since we will reuse `srcdst` as a scratch target
                // during the blur stage.
                passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                    src: srcdst,
                    dst: scratch_original,
                    src_size: ctx.viewport_size,
                    dst_size: ctx.viewport_size,
                    dst_scissor: None,
                    encode_output_srgb: false,
                    load: wgpu::LoadOp::Clear(ctx.clear),
                }));

                // Build blurred coverage into `scratch_blurred`, treating outside-bounds as transparent.
                let downsample_scale = if requested_downsample <= 1 {
                    1
                } else {
                    choose_effect_blur_downsample_scale(
                        ctx.viewport_size,
                        ctx.format,
                        budget_bytes,
                        requested_downsample,
                        quality,
                    )
                    .unwrap_or(1)
                };
                let iterations = blur_primitive::blur_iterations_for_radius(
                    blur_radius_px,
                    downsample_scale,
                    quality,
                );
                effect_blur_quality.drop_shadow.record_applied(
                    downsample_scale,
                    iterations,
                    desired_downsample,
                );

                if downsample_scale <= 1 {
                    passes.push(RenderPlanPass::Blur(BlurPass {
                        src: scratch_original,
                        dst: srcdst,
                        src_size: ctx.viewport_size,
                        dst_size: ctx.viewport_size,
                        dst_scissor: Some(LocalScissorRect(scissor)),
                        mask_uniform_index: None,
                        mask: None,
                        axis: BlurAxis::Horizontal,
                        load: wgpu::LoadOp::Clear(ctx.clear),
                    }));
                    passes.push(RenderPlanPass::Blur(BlurPass {
                        src: srcdst,
                        dst: scratch_blurred,
                        src_size: ctx.viewport_size,
                        dst_size: ctx.viewport_size,
                        dst_scissor: Some(LocalScissorRect(scissor)),
                        mask_uniform_index: None,
                        mask: None,
                        axis: BlurAxis::Vertical,
                        load: wgpu::LoadOp::Clear(ctx.clear),
                    }));

                    if iterations > 1 {
                        blur_primitive::append_pingpong_blur_passes(
                            passes,
                            scratch_blurred,
                            srcdst,
                            ctx.viewport_size,
                            Some(LocalScissorRect(scissor)),
                            iterations - 1,
                            ctx.clear,
                            wgpu::LoadOp::Clear(ctx.clear),
                        );
                    }
                } else {
                    let downsample_scale = if downsample_scale >= 4 { 4 } else { 2 };
                    let blur_size = downsampled_size(ctx.viewport_size, downsample_scale);

                    let down_scissor =
                        map_scissor_downsample_nearest(Some(scissor), downsample_scale, blur_size);
                    let down_origin = down_scissor.map(|s| (s.x, s.y)).unwrap_or((0, 0));
                    passes.push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
                        src: scratch_original,
                        dst: srcdst,
                        src_size: ctx.viewport_size,
                        dst_size: blur_size,
                        src_origin: (scissor.x, scissor.y),
                        dst_scissor: down_scissor.map(LocalScissorRect),
                        dst_origin: down_origin,
                        mask_uniform_index: None,
                        mask: None,
                        mode: ScaleMode::Downsample,
                        scale: downsample_scale,
                        load: wgpu::LoadOp::Clear(ctx.clear),
                    }));

                    let blur_scissor = down_scissor.map(LocalScissorRect);
                    blur_primitive::append_pingpong_blur_passes(
                        passes,
                        srcdst,
                        scratch_blurred,
                        blur_size,
                        blur_scissor,
                        iterations,
                        ctx.clear,
                        wgpu::LoadOp::Clear(ctx.clear),
                    );

                    let final_scissor =
                        map_scissor_to_size(Some(scissor), ctx.viewport_size, ctx.viewport_size);
                    passes.push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
                        src: srcdst,
                        dst: scratch_blurred,
                        src_size: blur_size,
                        dst_size: ctx.viewport_size,
                        src_origin: down_origin,
                        dst_scissor: final_scissor.map(LocalScissorRect),
                        dst_origin: (scissor.x, scissor.y),
                        mask_uniform_index: None,
                        mask: None,
                        mode: ScaleMode::Upscale,
                        scale: downsample_scale,
                        load: wgpu::LoadOp::Clear(ctx.clear),
                    }));
                }

                // Restore original content into `srcdst` (outside the effect bounds must remain untouched).
                passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                    src: scratch_original,
                    dst: srcdst,
                    src_size: ctx.viewport_size,
                    dst_size: ctx.viewport_size,
                    dst_scissor: None,
                    encode_output_srgb: false,
                    load: wgpu::LoadOp::Clear(ctx.clear),
                }));

                // Composite the shadow behind the original content, within the computation bounds.
                passes.push(RenderPlanPass::DropShadow(DropShadowPass {
                    src: scratch_blurred,
                    dst: srcdst,
                    src_size: ctx.viewport_size,
                    dst_size: ctx.viewport_size,
                    dst_scissor: Some(LocalScissorRect(scissor)),
                    mask_uniform_index,
                    mask,
                    offset_px: (
                        s.offset_px.x.0 * ctx.scale_factor,
                        s.offset_px.y.0 * ctx.scale_factor,
                    ),
                    color: s.color,
                    load: wgpu::LoadOp::Load,
                }));
            }
            fret_core::EffectStep::ColorAdjust {
                saturation,
                brightness,
                contrast,
            } => {
                effect_degradations.color_adjust.requested =
                    effect_degradations.color_adjust.requested.saturating_add(1);
                if !color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.color_adjust.degraded_budget_zero = effect_degradations
                            .color_adjust
                            .degraded_budget_zero
                            .saturating_add(1);
                    } else {
                        effect_degradations
                            .color_adjust
                            .degraded_budget_insufficient = effect_degradations
                            .color_adjust
                            .degraded_budget_insufficient
                            .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.color_adjust.degraded_target_exhausted =
                        effect_degradations
                            .color_adjust
                            .degraded_target_exhausted
                            .saturating_add(1);
                    continue;
                };
                effect_degradations.color_adjust.applied =
                    effect_degradations.color_adjust.applied.saturating_add(1);
                append_color_adjust_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    Some(scissor),
                    saturation,
                    brightness,
                    contrast,
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::ColorMatrix { m } => {
                effect_degradations.color_matrix.requested =
                    effect_degradations.color_matrix.requested.saturating_add(1);
                if !color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.color_matrix.degraded_budget_zero = effect_degradations
                            .color_matrix
                            .degraded_budget_zero
                            .saturating_add(1);
                    } else {
                        effect_degradations
                            .color_matrix
                            .degraded_budget_insufficient = effect_degradations
                            .color_matrix
                            .degraded_budget_insufficient
                            .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.color_matrix.degraded_target_exhausted =
                        effect_degradations
                            .color_matrix
                            .degraded_target_exhausted
                            .saturating_add(1);
                    continue;
                };
                effect_degradations.color_matrix.applied =
                    effect_degradations.color_matrix.applied.saturating_add(1);
                append_color_matrix_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    Some(scissor),
                    m,
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::AlphaThreshold { cutoff, soft } => {
                effect_degradations.alpha_threshold.requested = effect_degradations
                    .alpha_threshold
                    .requested
                    .saturating_add(1);
                if !color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.alpha_threshold.degraded_budget_zero =
                            effect_degradations
                                .alpha_threshold
                                .degraded_budget_zero
                                .saturating_add(1);
                    } else {
                        effect_degradations
                            .alpha_threshold
                            .degraded_budget_insufficient = effect_degradations
                            .alpha_threshold
                            .degraded_budget_insufficient
                            .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations
                        .alpha_threshold
                        .degraded_target_exhausted = effect_degradations
                        .alpha_threshold
                        .degraded_target_exhausted
                        .saturating_add(1);
                    continue;
                };
                effect_degradations.alpha_threshold.applied = effect_degradations
                    .alpha_threshold
                    .applied
                    .saturating_add(1);
                append_alpha_threshold_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    Some(scissor),
                    cutoff,
                    soft,
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::Pixelate { scale } => {
                if scale <= 1 {
                    continue;
                }
                effect_degradations.pixelate.requested =
                    effect_degradations.pixelate.requested.saturating_add(1);
                if !pixelate_enabled(
                    ctx.viewport_size,
                    Some(scissor),
                    ctx.format,
                    budget_bytes,
                    scale,
                ) {
                    if budget_bytes == 0 {
                        effect_degradations.pixelate.degraded_budget_zero = effect_degradations
                            .pixelate
                            .degraded_budget_zero
                            .saturating_add(1);
                    } else {
                        effect_degradations.pixelate.degraded_budget_insufficient =
                            effect_degradations
                                .pixelate
                                .degraded_budget_insufficient
                                .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.pixelate.degraded_target_exhausted = effect_degradations
                        .pixelate
                        .degraded_target_exhausted
                        .saturating_add(1);
                    continue;
                };
                effect_degradations.pixelate.applied =
                    effect_degradations.pixelate.applied.saturating_add(1);
                append_pixelate_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    Some(scissor),
                    scale,
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::Dither { mode } => {
                effect_degradations.dither.requested =
                    effect_degradations.dither.requested.saturating_add(1);
                if !dither_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.dither.degraded_budget_zero = effect_degradations
                            .dither
                            .degraded_budget_zero
                            .saturating_add(1);
                    } else {
                        effect_degradations.dither.degraded_budget_insufficient =
                            effect_degradations
                                .dither
                                .degraded_budget_insufficient
                                .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.dither.degraded_target_exhausted = effect_degradations
                        .dither
                        .degraded_target_exhausted
                        .saturating_add(1);
                    continue;
                };
                effect_degradations.dither.applied =
                    effect_degradations.dither.applied.saturating_add(1);
                append_dither_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    Some(scissor),
                    mode,
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::NoiseV1(n) => {
                let n = n.sanitize();
                if n.strength <= 0.0 {
                    continue;
                }
                effect_degradations.noise.requested =
                    effect_degradations.noise.requested.saturating_add(1);
                if !noise_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.noise.degraded_budget_zero = effect_degradations
                            .noise
                            .degraded_budget_zero
                            .saturating_add(1);
                    } else {
                        effect_degradations.noise.degraded_budget_insufficient =
                            effect_degradations
                                .noise
                                .degraded_budget_insufficient
                                .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.noise.degraded_target_exhausted = effect_degradations
                        .noise
                        .degraded_target_exhausted
                        .saturating_add(1);
                    continue;
                };
                effect_degradations.noise.applied =
                    effect_degradations.noise.applied.saturating_add(1);
                append_noise_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    Some(scissor),
                    n.strength,
                    (n.scale_px.0 * ctx.scale_factor).max(1.0),
                    n.phase,
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::CustomV1 {
                id,
                params,
                max_sample_offset_px: _,
            } => {
                effect_degradations.custom_effect.requested = effect_degradations
                    .custom_effect
                    .requested
                    .saturating_add(1);
                if !color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.custom_effect.degraded_budget_zero =
                            effect_degradations
                                .custom_effect
                                .degraded_budget_zero
                                .saturating_add(1);
                    } else {
                        effect_degradations
                            .custom_effect
                            .degraded_budget_insufficient = effect_degradations
                            .custom_effect
                            .degraded_budget_insufficient
                            .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.custom_effect.degraded_target_exhausted =
                        effect_degradations
                            .custom_effect
                            .degraded_target_exhausted
                            .saturating_add(1);
                    continue;
                };
                effect_degradations.custom_effect.applied =
                    effect_degradations.custom_effect.applied.saturating_add(1);
                append_custom_effect_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    Some(scissor),
                    id,
                    params,
                    ctx.clear,
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
                effect_degradations.custom_effect.requested = effect_degradations
                    .custom_effect
                    .requested
                    .saturating_add(1);
                if !color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.custom_effect.degraded_budget_zero =
                            effect_degradations
                                .custom_effect
                                .degraded_budget_zero
                                .saturating_add(1);
                    } else {
                        effect_degradations
                            .custom_effect
                            .degraded_budget_insufficient = effect_degradations
                            .custom_effect
                            .degraded_budget_insufficient
                            .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.custom_effect.degraded_target_exhausted =
                        effect_degradations
                            .custom_effect
                            .degraded_target_exhausted
                            .saturating_add(1);
                    continue;
                };
                effect_degradations.custom_effect.applied =
                    effect_degradations.custom_effect.applied.saturating_add(1);

                let (input_image, input_uv, input_sampling) = match input_image {
                    None => (
                        None,
                        fret_core::scene::UvRect::FULL,
                        fret_core::scene::ImageSamplingHint::Default,
                    ),
                    Some(input) => (Some(input.image), input.uv, input.sampling),
                };
                append_custom_effect_v2_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    Some(scissor),
                    id,
                    params,
                    input_image,
                    input_uv,
                    input_sampling,
                    ctx.clear,
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
                effect_degradations.custom_effect.requested = effect_degradations
                    .custom_effect
                    .requested
                    .saturating_add(1);
                if !color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes) {
                    if budget_bytes == 0 {
                        effect_degradations.custom_effect.degraded_budget_zero =
                            effect_degradations
                                .custom_effect
                                .degraded_budget_zero
                                .saturating_add(1);
                    } else {
                        effect_degradations
                            .custom_effect
                            .degraded_budget_insufficient = effect_degradations
                            .custom_effect
                            .degraded_budget_insufficient
                            .saturating_add(1);
                    }
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    effect_degradations.custom_effect.degraded_target_exhausted =
                        effect_degradations
                            .custom_effect
                            .degraded_target_exhausted
                            .saturating_add(1);
                    continue;
                };
                effect_degradations.custom_effect.applied =
                    effect_degradations.custom_effect.applied.saturating_add(1);

                let (user0_image, user0_uv, user0_sampling) = match user0 {
                    None => (
                        None,
                        fret_core::scene::UvRect::FULL,
                        fret_core::scene::ImageSamplingHint::Default,
                    ),
                    Some(input) => (Some(input.image), input.uv, input.sampling),
                };
                let (user1_image, user1_uv, user1_sampling) = match user1 {
                    None => (
                        None,
                        fret_core::scene::UvRect::FULL,
                        fret_core::scene::ImageSamplingHint::Default,
                    ),
                    Some(input) => (Some(input.image), input.uv, input.sampling),
                };

                let v3_chain_raw = group_raw.or(chain_raw);
                let v3_sources_plan = plan_custom_v3_sources_and_charge_budget(
                    sources,
                    scratch,
                    v3_chain_raw,
                    group_pyramid,
                    group_pyramid_roi,
                    scissor,
                    ctx,
                    &mut budget_bytes,
                    base_required_bytes_for_srcdst_and_single_scratch(estimate_texture_bytes(
                        ctx.viewport_size,
                        ctx.format,
                        1,
                    )),
                    &mut effect_degradations.custom_effect_v3_sources,
                );
                if let Some(e) = custom_chain_budget.as_mut()
                    && group_pyramid.is_none()
                    && sources.pyramid.is_some()
                    && v3_sources_plan.pyramid_levels >= 2
                {
                    e.optional_pyramid_bytes =
                        e.optional_pyramid_bytes
                            .saturating_add(estimate_custom_v3_pyramid_bytes(
                                ctx.viewport_size,
                                ctx.format,
                                v3_sources_plan.pyramid_levels,
                            ));
                }
                append_custom_effect_v3_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    Some(scissor),
                    id,
                    params,
                    user0_image,
                    user0_uv,
                    user0_sampling,
                    user1_image,
                    user1_uv,
                    user1_sampling,
                    sources,
                    v3_sources_plan.src_raw,
                    v3_sources_plan.pyramid_levels,
                    v3_sources_plan.pyramid_build_scissor,
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
        }
    }

    custom_chain_budget
}

fn padded_chain_step_scissors(
    steps: &[fret_core::EffectStep],
    final_scissor: ScissorRect,
    viewport_size: (u32, u32),
    ctx: EffectCompileCtx,
) -> Vec<ScissorRect> {
    if steps.is_empty() {
        return Vec::new();
    }

    let mut out = vec![final_scissor; steps.len()];
    let mut required = final_scissor;

    for (idx, step) in steps.iter().copied().enumerate().rev() {
        out[idx] = required;
        let pad_px = effect_step_max_sample_pad_px(step, ctx.scale_factor);
        required = inflate_scissor_to_viewport(required, pad_px, viewport_size);
    }

    out
}

fn effect_step_max_sample_pad_px(step: fret_core::EffectStep, scale_factor: f32) -> u32 {
    let logical_px = match step {
        fret_core::EffectStep::GaussianBlur { radius_px, .. } => radius_px.0,
        fret_core::EffectStep::BackdropWarpV1(w) => w.strength_px.0 + w.chromatic_aberration_px.0,
        fret_core::EffectStep::BackdropWarpV2(w) => {
            w.base.strength_px.0 + w.base.chromatic_aberration_px.0
        }
        fret_core::EffectStep::CustomV1 {
            max_sample_offset_px,
            ..
        } => max_sample_offset_px.0,
        fret_core::EffectStep::CustomV2 {
            max_sample_offset_px,
            ..
        } => max_sample_offset_px.0,
        fret_core::EffectStep::CustomV3 {
            max_sample_offset_px,
            ..
        } => max_sample_offset_px.0,
        _ => 0.0,
    };

    if !logical_px.is_finite()
        || logical_px <= 0.0
        || !scale_factor.is_finite()
        || scale_factor <= 0.0
    {
        return 0;
    }

    ((logical_px * scale_factor).max(0.0)).ceil() as u32
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
            if mode != fret_core::EffectMode::Backdrop {
                return;
            }
            effect_degradations.backdrop_warp.requested = effect_degradations
                .backdrop_warp
                .requested
                .saturating_add(1);
            if !backdrop_warp_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.backdrop_warp.degraded_budget_zero = effect_degradations
                        .backdrop_warp
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations
                        .backdrop_warp
                        .degraded_budget_insufficient = effect_degradations
                        .backdrop_warp
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.backdrop_warp.degraded_target_exhausted = effect_degradations
                    .backdrop_warp
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.backdrop_warp.applied =
                effect_degradations.backdrop_warp.applied.saturating_add(1);
            append_backdrop_warp_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                scissor,
                scale_backdrop_warp_v1(w.sanitize(), ctx.scale_factor),
                ctx.clear,
                None,
                None,
            );
        }
        fret_core::EffectStep::BackdropWarpV2(w) => {
            if mode != fret_core::EffectMode::Backdrop {
                return;
            }
            effect_degradations.backdrop_warp.requested = effect_degradations
                .backdrop_warp
                .requested
                .saturating_add(1);
            if !backdrop_warp_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.backdrop_warp.degraded_budget_zero = effect_degradations
                        .backdrop_warp
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations
                        .backdrop_warp
                        .degraded_budget_insufficient = effect_degradations
                        .backdrop_warp
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.backdrop_warp.degraded_target_exhausted = effect_degradations
                    .backdrop_warp
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.backdrop_warp.applied =
                effect_degradations.backdrop_warp.applied.saturating_add(1);

            // Scissored in-place pattern: preserve outside-region content by pre-blitting into scratch.
            passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                src: srcdst,
                dst: scratch,
                src_size: ctx.viewport_size,
                dst_size: ctx.viewport_size,
                dst_scissor: None,
                encode_output_srgb: false,
                load: wgpu::LoadOp::Clear(ctx.clear),
            }));

            let base = w.base.sanitize();
            let (warp_image, warp_uv, warp_sampling, warp_encoding) = match w.field {
                fret_core::scene::BackdropWarpFieldV2::Procedural => (
                    None,
                    fret_core::scene::UvRect::FULL,
                    fret_core::scene::ImageSamplingHint::Default,
                    fret_core::scene::WarpMapEncodingV1::RgSigned,
                ),
                fret_core::scene::BackdropWarpFieldV2::ImageDisplacementMap {
                    image,
                    uv,
                    sampling,
                    encoding,
                } => (Some(image), uv, sampling, encoding),
            };

            passes.push(RenderPlanPass::BackdropWarp(BackdropWarpPass {
                src: scratch,
                dst: srcdst,
                src_size: ctx.viewport_size,
                dst_size: ctx.viewport_size,
                origin_px: (scissor.x, scissor.y),
                bounds_size_px: (scissor.w, scissor.h),
                dst_scissor: Some(LocalScissorRect(scissor)),
                mask_uniform_index: None,
                mask: None,
                strength_px: base.strength_px.0 * ctx.scale_factor,
                scale_px: base.scale_px.0 * ctx.scale_factor,
                phase: base.phase,
                chromatic_aberration_px: base.chromatic_aberration_px.0 * ctx.scale_factor,
                kind: base.kind,
                warp_image,
                warp_uv,
                warp_sampling,
                warp_encoding,
                load: wgpu::LoadOp::Load,
            }));
        }
        fret_core::EffectStep::NoiseV1(n) => {
            effect_degradations.noise.requested =
                effect_degradations.noise.requested.saturating_add(1);
            if !noise_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.noise.degraded_budget_zero = effect_degradations
                        .noise
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations.noise.degraded_budget_insufficient = effect_degradations
                        .noise
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.noise.degraded_target_exhausted = effect_degradations
                    .noise
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.noise.applied = effect_degradations.noise.applied.saturating_add(1);
            append_noise_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                Some(scissor),
                n.strength,
                (n.scale_px.0 * ctx.scale_factor).max(1.0),
                n.phase,
                ctx.clear,
                None,
                None,
            );
        }
        fret_core::EffectStep::ColorAdjust {
            saturation,
            brightness,
            contrast,
        } => {
            effect_degradations.color_adjust.requested =
                effect_degradations.color_adjust.requested.saturating_add(1);
            if !color_adjust_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.color_adjust.degraded_budget_zero = effect_degradations
                        .color_adjust
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations
                        .color_adjust
                        .degraded_budget_insufficient = effect_degradations
                        .color_adjust
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.color_adjust.degraded_target_exhausted = effect_degradations
                    .color_adjust
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.color_adjust.applied =
                effect_degradations.color_adjust.applied.saturating_add(1);
            append_color_adjust_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                Some(scissor),
                saturation,
                brightness,
                contrast,
                ctx.clear,
                None,
                None,
            );
        }
        fret_core::EffectStep::ColorMatrix { m } => {
            effect_degradations.color_matrix.requested =
                effect_degradations.color_matrix.requested.saturating_add(1);
            if !color_adjust_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.color_matrix.degraded_budget_zero = effect_degradations
                        .color_matrix
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations
                        .color_matrix
                        .degraded_budget_insufficient = effect_degradations
                        .color_matrix
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.color_matrix.degraded_target_exhausted = effect_degradations
                    .color_matrix
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.color_matrix.applied =
                effect_degradations.color_matrix.applied.saturating_add(1);
            append_color_matrix_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                Some(scissor),
                m,
                ctx.clear,
                None,
                None,
            );
        }
        fret_core::EffectStep::AlphaThreshold { cutoff, soft } => {
            effect_degradations.alpha_threshold.requested = effect_degradations
                .alpha_threshold
                .requested
                .saturating_add(1);
            if !color_adjust_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.alpha_threshold.degraded_budget_zero = effect_degradations
                        .alpha_threshold
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations
                        .alpha_threshold
                        .degraded_budget_insufficient = effect_degradations
                        .alpha_threshold
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations
                    .alpha_threshold
                    .degraded_target_exhausted = effect_degradations
                    .alpha_threshold
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.alpha_threshold.applied = effect_degradations
                .alpha_threshold
                .applied
                .saturating_add(1);
            append_alpha_threshold_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                Some(scissor),
                cutoff,
                soft,
                ctx.clear,
                None,
                None,
            );
        }
        fret_core::EffectStep::Pixelate { scale } => {
            effect_degradations.pixelate.requested =
                effect_degradations.pixelate.requested.saturating_add(1);
            if !pixelate_enabled(
                ctx.viewport_size,
                Some(scissor),
                ctx.format,
                *budget_bytes,
                scale,
            ) {
                if *budget_bytes == 0 {
                    effect_degradations.pixelate.degraded_budget_zero = effect_degradations
                        .pixelate
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations.pixelate.degraded_budget_insufficient = effect_degradations
                        .pixelate
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.pixelate.degraded_target_exhausted = effect_degradations
                    .pixelate
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.pixelate.applied =
                effect_degradations.pixelate.applied.saturating_add(1);
            append_pixelate_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                Some(scissor),
                scale,
                ctx.clear,
                None,
                None,
            );
        }
        fret_core::EffectStep::Dither { mode } => {
            effect_degradations.dither.requested =
                effect_degradations.dither.requested.saturating_add(1);
            if !dither_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.dither.degraded_budget_zero = effect_degradations
                        .dither
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations.dither.degraded_budget_insufficient = effect_degradations
                        .dither
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.dither.degraded_target_exhausted = effect_degradations
                    .dither
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.dither.applied =
                effect_degradations.dither.applied.saturating_add(1);
            append_dither_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                Some(scissor),
                mode,
                ctx.clear,
                None,
                None,
            );
        }
        fret_core::EffectStep::CustomV1 {
            id,
            params,
            max_sample_offset_px: _,
        } => {
            effect_degradations.custom_effect.requested = effect_degradations
                .custom_effect
                .requested
                .saturating_add(1);
            if !color_adjust_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.custom_effect.degraded_budget_zero = effect_degradations
                        .custom_effect
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations
                        .custom_effect
                        .degraded_budget_insufficient = effect_degradations
                        .custom_effect
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.custom_effect.degraded_target_exhausted = effect_degradations
                    .custom_effect
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.custom_effect.applied =
                effect_degradations.custom_effect.applied.saturating_add(1);
            append_custom_effect_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                Some(scissor),
                id,
                params,
                ctx.clear,
                None,
                None,
            );
        }
        fret_core::EffectStep::CustomV2 {
            id,
            params,
            max_sample_offset_px: _,
            input_image,
        } => {
            effect_degradations.custom_effect.requested = effect_degradations
                .custom_effect
                .requested
                .saturating_add(1);
            if !color_adjust_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.custom_effect.degraded_budget_zero = effect_degradations
                        .custom_effect
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations
                        .custom_effect
                        .degraded_budget_insufficient = effect_degradations
                        .custom_effect
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.custom_effect.degraded_target_exhausted = effect_degradations
                    .custom_effect
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.custom_effect.applied =
                effect_degradations.custom_effect.applied.saturating_add(1);

            let (input_image, input_uv, input_sampling) = match input_image {
                None => (
                    None,
                    fret_core::scene::UvRect::FULL,
                    fret_core::scene::ImageSamplingHint::Default,
                ),
                Some(input) => (Some(input.image), input.uv, input.sampling),
            };
            append_custom_effect_v2_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                Some(scissor),
                id,
                params,
                input_image,
                input_uv,
                input_sampling,
                ctx.clear,
                None,
                None,
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
            effect_degradations.custom_effect.requested = effect_degradations
                .custom_effect
                .requested
                .saturating_add(1);
            if !color_adjust_enabled(ctx.viewport_size, ctx.format, *budget_bytes) {
                if *budget_bytes == 0 {
                    effect_degradations.custom_effect.degraded_budget_zero = effect_degradations
                        .custom_effect
                        .degraded_budget_zero
                        .saturating_add(1);
                } else {
                    effect_degradations
                        .custom_effect
                        .degraded_budget_insufficient = effect_degradations
                        .custom_effect
                        .degraded_budget_insufficient
                        .saturating_add(1);
                }
                return;
            }
            let Some(&scratch) = scratch_targets.first() else {
                effect_degradations.custom_effect.degraded_target_exhausted = effect_degradations
                    .custom_effect
                    .degraded_target_exhausted
                    .saturating_add(1);
                return;
            };
            effect_degradations.custom_effect.applied =
                effect_degradations.custom_effect.applied.saturating_add(1);

            let (user0_image, user0_uv, user0_sampling) = match user0 {
                None => (
                    None,
                    fret_core::scene::UvRect::FULL,
                    fret_core::scene::ImageSamplingHint::Default,
                ),
                Some(input) => (Some(input.image), input.uv, input.sampling),
            };
            let (user1_image, user1_uv, user1_sampling) = match user1 {
                None => (
                    None,
                    fret_core::scene::UvRect::FULL,
                    fret_core::scene::ImageSamplingHint::Default,
                ),
                Some(input) => (Some(input.image), input.uv, input.sampling),
            };

            let group_raw = backdrop_source_group.map(|g| g.raw_target);
            let group_pyramid = backdrop_source_group.and_then(|g| g.pyramid);
            let group_pyramid_roi = backdrop_source_group
                .and_then(|g| g.pyramid.map(|_| (g.scissor, g.pyramid_pad_px)));

            let v3_chain_raw = group_raw.or(custom_v3_chain_raw);
            let v3_sources_plan = plan_custom_v3_sources_and_charge_budget(
                sources,
                scratch,
                v3_chain_raw,
                group_pyramid,
                group_pyramid_roi,
                scissor,
                ctx,
                budget_bytes,
                base_required_bytes_for_srcdst_and_single_scratch(estimate_texture_bytes(
                    ctx.viewport_size,
                    ctx.format,
                    1,
                )),
                &mut effect_degradations.custom_effect_v3_sources,
            );
            if let Some(e) = custom_chain_budget.as_mut()
                && group_pyramid.is_none()
                && sources.pyramid.is_some()
                && v3_sources_plan.pyramid_levels >= 2
            {
                e.optional_pyramid_bytes =
                    e.optional_pyramid_bytes
                        .saturating_add(estimate_custom_v3_pyramid_bytes(
                            ctx.viewport_size,
                            ctx.format,
                            v3_sources_plan.pyramid_levels,
                        ));
            }
            append_custom_effect_v3_in_place_single_scratch(
                passes,
                srcdst,
                scratch,
                ctx.viewport_size,
                Some(scissor),
                id,
                params,
                user0_image,
                user0_uv,
                user0_sampling,
                user1_image,
                user1_uv,
                user1_sampling,
                sources,
                v3_sources_plan.src_raw,
                v3_sources_plan.pyramid_levels,
                v3_sources_plan.pyramid_build_scissor,
                ctx.clear,
                None,
                None,
            );
        }
        fret_core::EffectStep::DropShadowV1(_) => {}
    }
}

fn inflate_scissor_to_viewport(
    scissor: ScissorRect,
    pad_px: u32,
    viewport_size: (u32, u32),
) -> ScissorRect {
    if pad_px == 0 {
        return scissor;
    }

    let max_w = viewport_size.0;
    let max_h = viewport_size.1;

    let x0 = scissor.x.saturating_sub(pad_px).min(max_w);
    let y0 = scissor.y.saturating_sub(pad_px).min(max_h);
    let x1 = scissor
        .x
        .saturating_add(scissor.w)
        .saturating_add(pad_px)
        .min(max_w);
    let y1 = scissor
        .y
        .saturating_add(scissor.h)
        .saturating_add(pad_px)
        .min(max_h);

    if x1 <= x0 || y1 <= y0 {
        return scissor;
    }

    ScissorRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

fn compile_gaussian_blur_in_place(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    downsample: u32,
    radius_px: f32,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::BlurQualitySnapshot,
) {
    if radius_px <= 0.0 || scissor.w == 0 || scissor.h == 0 {
        return;
    }
    effect_degradations.gaussian_blur.requested = effect_degradations
        .gaussian_blur
        .requested
        .saturating_add(1);

    let requested_downsample = if downsample >= 4 { 4 } else { 2 };
    let desired_downsample = effect_blur_desired_downsample(requested_downsample, quality);

    // Prefer two-scratch downsampled blur when available.
    if scratch_targets.len() >= 2
        && let Some(downsample_scale) = choose_effect_blur_downsample_scale(
            ctx.viewport_size,
            ctx.format,
            *budget_bytes,
            requested_downsample,
            quality,
        )
    {
        let iterations =
            blur_primitive::blur_iterations_for_radius(radius_px, downsample_scale, quality);
        effect_degradations.gaussian_blur.applied =
            effect_degradations.gaussian_blur.applied.saturating_add(1);
        effect_blur_quality.gaussian_blur.record_applied(
            downsample_scale,
            iterations,
            desired_downsample,
        );
        append_scissored_blur_in_place_two_scratch(
            passes,
            srcdst,
            scratch_targets[0],
            scratch_targets[1],
            ctx.viewport_size,
            downsample_scale,
            iterations,
            scissor,
            ctx.clear,
            None,
            None,
        );
        return;
    }

    // Fallback: single-scratch blur (still deterministic, but may be more expensive).
    let Some(&scratch) = scratch_targets.first() else {
        effect_degradations.gaussian_blur.degraded_target_exhausted = effect_degradations
            .gaussian_blur
            .degraded_target_exhausted
            .saturating_add(1);
        effect_blur_quality
            .gaussian_blur
            .record_applied(1, 0, desired_downsample);
        effect_blur_quality
            .gaussian_blur
            .quality_degraded_blur_removed = effect_blur_quality
            .gaussian_blur
            .quality_degraded_blur_removed
            .saturating_add(1);
        return;
    };

    if *budget_bytes == 0 {
        effect_degradations.gaussian_blur.degraded_budget_zero = effect_degradations
            .gaussian_blur
            .degraded_budget_zero
            .saturating_add(1);
        effect_blur_quality
            .gaussian_blur
            .record_applied(1, 0, desired_downsample);
        effect_blur_quality
            .gaussian_blur
            .quality_degraded_blur_removed = effect_blur_quality
            .gaussian_blur
            .quality_degraded_blur_removed
            .saturating_add(1);
        return;
    }

    let full = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    let required = base_required_bytes_for_srcdst_and_single_scratch(full);
    if required > *budget_bytes {
        effect_degradations
            .gaussian_blur
            .degraded_budget_insufficient = effect_degradations
            .gaussian_blur
            .degraded_budget_insufficient
            .saturating_add(1);
        effect_blur_quality
            .gaussian_blur
            .record_applied(1, 0, desired_downsample);
        effect_blur_quality
            .gaussian_blur
            .quality_degraded_blur_removed = effect_blur_quality
            .gaussian_blur
            .quality_degraded_blur_removed
            .saturating_add(1);
        return;
    }

    let iterations = blur_primitive::blur_iterations_for_radius(radius_px, 1, quality);
    effect_degradations.gaussian_blur.applied =
        effect_degradations.gaussian_blur.applied.saturating_add(1);
    effect_blur_quality
        .gaussian_blur
        .record_applied(1, iterations, desired_downsample);
    append_scissored_blur_in_place_single_scratch(
        passes,
        srcdst,
        scratch,
        ctx.viewport_size,
        iterations,
        scissor,
        ctx.clear,
        None,
        None,
    );
}

#[cfg(test)]
mod tests;
