use super::blur::padded_chain_step_scissors;
use super::blur::{
    compile_drop_shadow_in_place_masked, compile_gaussian_blur_in_place,
    compile_gaussian_blur_in_place_masked,
};
use super::builtin::{
    apply_alpha_threshold_step, apply_alpha_threshold_step_masked, apply_backdrop_warp_v1_step,
    apply_backdrop_warp_v1_step_masked, apply_backdrop_warp_v2_step,
    apply_backdrop_warp_v2_step_masked, apply_color_adjust_step, apply_color_adjust_step_masked,
    apply_color_matrix_step, apply_color_matrix_step_masked, apply_dither_step,
    apply_dither_step_masked, apply_noise_step, apply_noise_step_masked, apply_pixelate_step,
    apply_pixelate_step_masked,
};
use super::builtin::{choose_clip_mask_target_capped, effect_blur_desired_downsample};
use super::custom::{
    append_padded_chain_final_custom_step, apply_custom_v1_step, apply_custom_v1_step_masked,
    apply_custom_v2_step, apply_custom_v2_step_masked, apply_custom_v3_step,
    apply_custom_v3_step_masked,
};
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct ChainCoverage {
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
}

#[derive(Debug)]
pub(super) struct PreparedChainStart {
    pub(super) budget_bytes: u64,
    pub(super) scratch_targets: Vec<PlanTarget>,
    pub(super) coverage: ChainCoverage,
    pub(super) custom_chain_budget: Option<CustomEffectChainBudgetEvidence>,
}

fn available_scratch_targets(in_use_targets: &[PlanTarget], srcdst: PlanTarget) -> Vec<PlanTarget> {
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

pub(super) fn backdrop_source_group_parts(
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

#[allow(clippy::too_many_arguments)]
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
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::super::BlurQualitySnapshot,
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

    let mut chain_start = prepare_chain_start(
        passes,
        in_use_targets,
        srcdst,
        &steps,
        scissor,
        mask_uniform_index,
        unavailable_mask_targets,
        quality,
        ctx,
    );

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
        chain_start.scratch_targets.as_slice(),
        srcdst,
        mode,
        quality,
        scissor,
        chain_start.budget_bytes,
        chain_start.coverage,
        effect_degradations,
        effect_blur_quality,
        ctx,
        backdrop_source_group,
        &mut chain_start.custom_chain_budget,
    ) {
        return chain_start.custom_chain_budget;
    }

    // Clip/shape masks are coverage (alpha) multipliers. If we apply them at every effect step in
    // a chain, coverage compounds (e.g. clip^2) and produces visible edge darkening (especially
    // around rounded corners) for common chains like blur -> custom refraction.
    //
    // To avoid this, we apply clip/mask only on the final step of the chain and keep intermediate
    // steps unmasked.
    apply_unpadded_chain_in_place(
        passes,
        &steps,
        srcdst,
        mode,
        quality,
        scissor,
        group_raw,
        effect_degradations,
        effect_blur_quality,
        ctx,
        backdrop_source_group,
        &mut chain_start,
    )
}

fn scaled_effect_px(px: fret_core::Px, scale_factor: f32) -> f32 {
    if px.0.is_finite() {
        (px.0 * scale_factor).max(0.0)
    } else {
        0.0
    }
}

fn initial_custom_chain_budget(
    steps: &[fret_core::EffectStep],
    ctx: EffectCompileCtx,
) -> Option<CustomEffectChainBudgetEvidence> {
    let full_target_bytes = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    steps
        .iter()
        .any(is_custom_effect_step)
        .then_some(CustomEffectChainBudgetEvidence {
            effective_budget_bytes: ctx.intermediate_budget_bytes,
            base_required_bytes: base_required_bytes_for_srcdst_and_single_scratch(
                full_target_bytes,
            ),
            base_required_full_targets: 2,
            optional_mask_bytes: 0,
            optional_pyramid_bytes: 0,
        })
}

fn chain_forces_quarter_blur(
    steps: &[fret_core::EffectStep],
    scratch_targets: &[PlanTarget],
    quality: fret_core::EffectQuality,
    budget_bytes: u64,
    ctx: EffectCompileCtx,
) -> bool {
    scratch_targets.len() >= 2
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
        })
}

#[allow(clippy::too_many_arguments)]
fn prepare_chain_coverage(
    passes: &mut Vec<RenderPlanPass>,
    steps: &[fret_core::EffectStep],
    scissor: ScissorRect,
    mask_uniform_index: Option<u32>,
    unavailable_mask_targets: &[PlanTarget],
    scratch_targets: &[PlanTarget],
    quality: fret_core::EffectQuality,
    budget_bytes: &mut u64,
    custom_chain_budget: &mut Option<CustomEffectChainBudgetEvidence>,
    ctx: EffectCompileCtx,
) -> ChainCoverage {
    let srcdst_bytes = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    let mask_tier_cap =
        chain_forces_quarter_blur(steps, scratch_targets, quality, *budget_bytes, ctx)
            .then_some(PlanTarget::Mask2);

    let mut chosen_mask_bytes = 0;
    let mask = if let Some(uniform_index) = mask_uniform_index
        && let Some((mask_target, mask_size, mask_bytes)) = choose_clip_mask_target_capped(
            ctx.viewport_size,
            scissor,
            *budget_bytes,
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
        *budget_bytes = (*budget_bytes).saturating_sub(mask_bytes);
        Some(MaskRef {
            target: mask_target,
            size: mask_size,
            viewport_rect: scissor,
        })
    } else {
        None
    };

    if let Some(evidence) = custom_chain_budget.as_mut() {
        evidence.optional_mask_bytes = chosen_mask_bytes;
    }

    ChainCoverage {
        mask_uniform_index,
        mask,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_chain_start(
    passes: &mut Vec<RenderPlanPass>,
    in_use_targets: &[PlanTarget],
    srcdst: PlanTarget,
    steps: &[fret_core::EffectStep],
    scissor: ScissorRect,
    mask_uniform_index: Option<u32>,
    unavailable_mask_targets: &[PlanTarget],
    quality: fret_core::EffectQuality,
    ctx: EffectCompileCtx,
) -> PreparedChainStart {
    let mut budget_bytes = ctx.intermediate_budget_bytes;
    let scratch_targets = available_scratch_targets(in_use_targets, srcdst);
    let mut custom_chain_budget = initial_custom_chain_budget(steps, ctx);
    let coverage = prepare_chain_coverage(
        passes,
        steps,
        scissor,
        mask_uniform_index,
        unavailable_mask_targets,
        scratch_targets.as_slice(),
        quality,
        &mut budget_bytes,
        &mut custom_chain_budget,
        ctx,
    );

    PreparedChainStart {
        budget_bytes,
        scratch_targets,
        coverage,
        custom_chain_budget,
    }
}

fn reserve_unpadded_chain_raw_target(
    passes: &mut Vec<RenderPlanPass>,
    steps: &[fret_core::EffectStep],
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    group_raw: Option<PlanTarget>,
    budget_bytes: &mut u64,
    custom_chain_budget: &mut Option<CustomEffectChainBudgetEvidence>,
    ctx: EffectCompileCtx,
) -> (Option<PlanTarget>, usize) {
    let chain_wants_raw =
        group_raw.is_none() && steps.len() >= 2 && steps.iter().any(step_wants_custom_v3_raw);
    let full_target_bytes = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);

    if !chain_wants_raw
        || scratch_targets.len() < 2
        || *budget_bytes < base_required_bytes_for_srcdst_and_single_scratch(full_target_bytes)
    {
        return (None, 0);
    }

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
    *budget_bytes = budget_excluding_full_size_targets(*budget_bytes, full_target_bytes, 1);
    if let Some(evidence) = custom_chain_budget.as_mut() {
        evidence.base_required_full_targets = evidence.base_required_full_targets.max(3);
        evidence.base_required_bytes = required_bytes_for_full_size_targets(full_target_bytes, 3);
    }

    (Some(chain_raw), 1)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_step_in_place_with_scratch_targets(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    step: fret_core::EffectStep,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::super::BlurQualitySnapshot,
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
            compile_gaussian_blur_in_place(
                passes,
                scratch_targets,
                srcdst,
                quality,
                scissor,
                downsample,
                scaled_effect_px(radius_px, ctx.scale_factor),
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

#[allow(clippy::too_many_arguments)]
fn apply_unpadded_chain_step_masked(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    step: fret_core::EffectStep,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::super::BlurQualitySnapshot,
    ctx: EffectCompileCtx,
    chain_raw: Option<PlanTarget>,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
    custom_chain_budget: &mut Option<CustomEffectChainBudgetEvidence>,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    match step {
        fret_core::EffectStep::GaussianBlur {
            radius_px,
            downsample,
        } => {
            compile_gaussian_blur_in_place_masked(
                passes,
                scratch_targets,
                srcdst,
                quality,
                scissor,
                downsample,
                scaled_effect_px(radius_px, ctx.scale_factor),
                ctx,
                budget_bytes,
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
                budget_bytes,
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
                budget_bytes,
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
                budget_bytes,
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
                budget_bytes,
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
                budget_bytes,
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
                budget_bytes,
                effect_degradations,
                mask_uniform_index,
                mask,
            );
        }
        fret_core::EffectStep::Pixelate { scale } => {
            if scale <= 1 {
                return;
            }
            apply_pixelate_step_masked(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                scale,
                ctx,
                budget_bytes,
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
                budget_bytes,
                effect_degradations,
                mask_uniform_index,
                mask,
            );
        }
        fret_core::EffectStep::NoiseV1(n) => {
            let n = n.sanitize();
            if n.strength <= 0.0 {
                return;
            }
            apply_noise_step_masked(
                passes,
                scratch_targets,
                srcdst,
                scissor,
                n,
                ctx,
                budget_bytes,
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
                budget_bytes,
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
                budget_bytes,
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
                budget_bytes,
                effect_degradations,
                chain_raw,
                backdrop_source_group,
                custom_chain_budget,
                mask_uniform_index,
                mask,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_unpadded_chain_in_place(
    passes: &mut Vec<RenderPlanPass>,
    steps: &[fret_core::EffectStep],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    group_raw: Option<PlanTarget>,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::super::BlurQualitySnapshot,
    ctx: EffectCompileCtx,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
    chain_start: &mut PreparedChainStart,
) -> Option<CustomEffectChainBudgetEvidence> {
    let scratch_targets_all = chain_start.scratch_targets.as_slice();
    let coverage = chain_start.coverage;
    let budget_bytes = &mut chain_start.budget_bytes;
    let custom_chain_budget = &mut chain_start.custom_chain_budget;
    let (chain_raw, scratch_start) = reserve_unpadded_chain_raw_target(
        passes,
        steps,
        scratch_targets_all,
        srcdst,
        group_raw,
        budget_bytes,
        custom_chain_budget,
        ctx,
    );
    let scratch_targets = &scratch_targets_all[scratch_start..];

    for (step_index, step) in steps.iter().copied().enumerate() {
        let apply_mask = step_index + 1 == steps.len();
        let mask_uniform_index = apply_mask.then_some(coverage.mask_uniform_index).flatten();
        let mask = apply_mask.then_some(coverage.mask).flatten();

        apply_unpadded_chain_step_masked(
            passes,
            scratch_targets,
            srcdst,
            mode,
            step,
            quality,
            scissor,
            budget_bytes,
            effect_degradations,
            effect_blur_quality,
            ctx,
            chain_raw,
            backdrop_source_group,
            custom_chain_budget,
            mask_uniform_index,
            mask,
        );
    }

    *custom_chain_budget
}

fn push_padded_chain_commit_pass(
    passes: &mut Vec<RenderPlanPass>,
    work: PlanTarget,
    srcdst: PlanTarget,
    scissor: ScissorRect,
    coverage: ChainCoverage,
    ctx: EffectCompileCtx,
) {
    // For clip/mask coverage, we want the "masked blend" semantics used by effect passes
    // (blend RGB, keep destination alpha), so we reuse a no-op ColorAdjust pass when a mask
    // is present. This avoids having to introduce a dedicated masked blit pass.
    if coverage.mask_uniform_index.is_some() || coverage.mask.is_some() {
        passes.push(RenderPlanPass::ColorAdjust(ColorAdjustPass {
            src: work,
            dst: srcdst,
            src_size: ctx.viewport_size,
            dst_size: ctx.viewport_size,
            dst_scissor: Some(LocalScissorRect(scissor)),
            mask_uniform_index: coverage.mask_uniform_index,
            mask: coverage.mask,
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

#[allow(clippy::too_many_arguments)]
pub(super) fn try_apply_padded_chain_in_place(
    passes: &mut Vec<RenderPlanPass>,
    steps: &[fret_core::EffectStep],
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    budget_bytes: u64,
    coverage: ChainCoverage,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::super::BlurQualitySnapshot,
    ctx: EffectCompileCtx,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
    custom_chain_budget: &mut Option<CustomEffectChainBudgetEvidence>,
) -> bool {
    if steps.len() < 2 {
        return false;
    }

    let step_scissors = padded_chain_step_scissors(steps, scissor, ctx.viewport_size, ctx);
    let needs_padding = step_scissors
        .first()
        .is_some_and(|step_scissor| *step_scissor != scissor);
    let has_drop_shadow = steps
        .iter()
        .any(|step| matches!(*step, fret_core::EffectStep::DropShadowV1(_)));
    let full_target_bytes = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    let min_budget_for_work = base_required_bytes_for_srcdst_and_two_scratch(full_target_bytes);
    let has_work = scratch_targets.len() >= 2;
    let has_mask = coverage.mask_uniform_index.is_some() || coverage.mask.is_some();
    let last_step_is_custom = steps.last().is_some_and(is_custom_effect_step);
    let can_commit_with_mask = !has_mask
        || last_step_is_custom
        || color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes);

    if !needs_padding
        || has_drop_shadow
        || !has_work
        || budget_bytes < min_budget_for_work
        || !can_commit_with_mask
    {
        return false;
    }

    let work = scratch_targets[0];
    let chain_wants_raw = steps.iter().any(step_wants_custom_v3_raw);
    let chain_raw = (chain_wants_raw
        && backdrop_source_group.is_none()
        && scratch_targets.len() >= 3
        && budget_bytes >= base_required_bytes_for_srcdst_and_three_scratch(full_target_bytes))
    .then_some(scratch_targets[1]);

    if let Some(evidence) = custom_chain_budget.as_mut() {
        let required_full_targets = if chain_raw.is_some() { 4 } else { 3 };
        evidence.base_required_full_targets = required_full_targets;
        evidence.base_required_bytes =
            required_bytes_for_full_size_targets(full_target_bytes, required_full_targets as u64);
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
        budget_excluding_full_size_targets(budget_bytes, full_target_bytes, excluded_targets);
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
            custom_chain_budget,
        );
    }

    if let Some(&final_step) = tail_step.first()
        && let Some(&final_scissor) = tail_scissor.first()
        && append_padded_chain_final_custom_step(
            passes,
            work,
            srcdst,
            final_step,
            final_scissor,
            scissor,
            coverage.mask_uniform_index,
            coverage.mask,
            chain_raw,
            backdrop_source_group,
            ctx,
            &mut work_budget_bytes,
            effect_degradations,
            custom_chain_budget,
        )
    {
        return true;
    }

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
            custom_chain_budget,
        );
    }

    push_padded_chain_commit_pass(passes, work, srcdst, scissor, coverage, ctx);
    true
}
