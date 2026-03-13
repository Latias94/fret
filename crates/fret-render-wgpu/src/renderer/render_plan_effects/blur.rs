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

pub(super) fn compile_drop_shadow_in_place_masked(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    shadow: fret_core::scene::DropShadowV1,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    effect_blur_quality: &mut super::super::BlurQualitySnapshot,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    if mode != fret_core::EffectMode::FilterContent {
        return;
    }

    let shadow = shadow.sanitize();
    if shadow.color.a <= 0.0 {
        return;
    }
    let blur_radius_px = if shadow.blur_radius_px.0.is_finite() {
        (shadow.blur_radius_px.0 * ctx.scale_factor).max(0.0)
    } else {
        0.0
    };
    if blur_radius_px <= 0.0 {
        return;
    }

    effect_degradations.drop_shadow.requested =
        effect_degradations.drop_shadow.requested.saturating_add(1);
    if *budget_bytes == 0 {
        effect_degradations.drop_shadow.degraded_budget_zero = effect_degradations
            .drop_shadow
            .degraded_budget_zero
            .saturating_add(1);
        return;
    }

    let full = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
    let requested_downsample = if shadow.downsample >= 4 {
        4
    } else if shadow.downsample >= 2 {
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
        && *budget_bytes >= base_required_bytes_for_srcdst_and_two_scratch(full);
    if !can_blur {
        let Some(&scratch_original) = scratch_targets.first() else {
            effect_degradations.drop_shadow.degraded_target_exhausted = effect_degradations
                .drop_shadow
                .degraded_target_exhausted
                .saturating_add(1);
            return;
        };
        if *budget_bytes < base_required_bytes_for_srcdst_and_single_scratch(full) {
            effect_degradations.drop_shadow.degraded_budget_insufficient = effect_degradations
                .drop_shadow
                .degraded_budget_insufficient
                .saturating_add(1);
            return;
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
                shadow.offset_px.x.0 * ctx.scale_factor,
                shadow.offset_px.y.0 * ctx.scale_factor,
            ),
            color: shadow.color,
            load: wgpu::LoadOp::Load,
        }));
        return;
    }

    effect_degradations.drop_shadow.applied =
        effect_degradations.drop_shadow.applied.saturating_add(1);

    let scratch_original = scratch_targets[0];
    let scratch_blurred = scratch_targets[1];

    // Preserve the original content, since we will reuse `srcdst` as a scratch target during the
    // blur stage.
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
            *budget_bytes,
            requested_downsample,
            quality,
        )
        .unwrap_or(1)
    };
    let iterations =
        blur_primitive::blur_iterations_for_radius(blur_radius_px, downsample_scale, quality);
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
            shadow.offset_px.x.0 * ctx.scale_factor,
            shadow.offset_px.y.0 * ctx.scale_factor,
        ),
        color: shadow.color,
        load: wgpu::LoadOp::Load,
    }));
}
