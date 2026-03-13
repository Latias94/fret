use super::blur::padded_chain_step_scissors;
use super::builtin::{choose_clip_mask_target_capped, effect_blur_desired_downsample};
use super::custom::append_padded_chain_final_custom_step;
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

fn initial_custom_chain_budget(
    steps: &[fret_core::EffectStep],
    ctx: EffectCompileCtx,
) -> Option<CustomEffectChainBudgetEvidence> {
    let full_target_bytes = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    steps
        .iter()
        .any(super::is_custom_effect_step)
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
    let scratch_targets = super::available_scratch_targets(in_use_targets, srcdst);
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
    let last_step_is_custom = steps.last().is_some_and(super::is_custom_effect_step);
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
    let chain_wants_raw = steps.iter().any(super::step_wants_custom_v3_raw);
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
        super::apply_step_in_place_with_scratch_targets(
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
        super::apply_step_in_place_with_scratch_targets(
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
