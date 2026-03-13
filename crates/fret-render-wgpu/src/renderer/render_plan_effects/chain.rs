use super::blur::padded_chain_step_scissors;
use super::custom::append_padded_chain_final_custom_step;
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct ChainCoverage {
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
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
