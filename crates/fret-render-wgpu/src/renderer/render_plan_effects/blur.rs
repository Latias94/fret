use super::*;

pub(super) fn padded_chain_step_scissors(
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

pub(super) fn inflate_scissor_to_viewport(
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

fn compile_gaussian_blur_in_place_inner(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    downsample: u32,
    radius_px: f32,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::super::BlurQualitySnapshot,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
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
            mask_uniform_index,
            mask,
        );
        return;
    }

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
        mask_uniform_index,
        mask,
    );
}

pub(super) fn compile_gaussian_blur_in_place(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    downsample: u32,
    radius_px: f32,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::super::BlurQualitySnapshot,
) {
    compile_gaussian_blur_in_place_inner(
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
        None,
        None,
    );
}

pub(super) fn compile_gaussian_blur_in_place_masked(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    downsample: u32,
    radius_px: f32,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::super::BlurQualitySnapshot,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    compile_gaussian_blur_in_place_inner(
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
        mask_uniform_index,
        mask,
    );
}
