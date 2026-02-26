use super::blur_primitive;
use super::frame_targets::downsampled_size;
use super::intermediate_pool::estimate_texture_bytes;
use super::{
    AlphaThresholdPass, BackdropWarpPass, BlurAxis, BlurPass, ClipMaskPass, ColorAdjustPass,
    ColorMatrixPass, CustomEffectPass, DitherPass, DropShadowPass, FullscreenBlitPass,
    LocalScissorRect, MaskRef, NoisePass, PlanTarget, RenderPlanPass, ScaleMode, ScaleNearestPass,
    ScissorRect,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct EffectCompileCtx {
    pub(super) viewport_size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) clear: wgpu::Color,
    pub(super) scale_factor: f32,
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
) {
    if srcdst == PlanTarget::Output || scissor.w == 0 || scissor.h == 0 {
        return;
    }

    let steps: Vec<fret_core::EffectStep> = chain.iter().collect();
    if steps.is_empty() {
        return;
    }

    let mut budget_bytes = ctx.intermediate_budget_bytes;
    let srcdst_bytes = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);

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
        budget_bytes = budget_bytes.saturating_sub(mask_bytes);
        Some(MaskRef {
            target: mask_target,
            size: mask_size,
            viewport_rect: scissor,
        })
    } else {
        None
    };

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
        let min_budget_for_work = full.saturating_mul(3);
        let has_work = scratch_targets.len() >= 2;
        let can_commit_with_mask = mask_uniform_index.is_none()
            || color_adjust_enabled(ctx.viewport_size, ctx.format, budget_bytes);

        if needs_padding
            && !has_drop_shadow
            && has_work
            && budget_bytes >= min_budget_for_work
            && can_commit_with_mask
        {
            let work = scratch_targets[0];
            let work_scratch_targets = &scratch_targets[1..];

            passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                src: srcdst,
                dst: work,
                src_size: ctx.viewport_size,
                dst_size: ctx.viewport_size,
                dst_scissor: None,
                encode_output_srgb: false,
                load: wgpu::LoadOp::Clear(ctx.clear),
            }));

            let mut work_budget_bytes = budget_bytes.saturating_sub(full);
            for (step, step_scissor) in steps.iter().copied().zip(step_scissors.iter().copied()) {
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
                );
            }

            // Composite the chain result back into `srcdst` under the original scissor.
            //
            // For clip/mask coverage, we want the "masked blend" semantics used by effect passes
            // (blend RGB, keep destination alpha), so we reuse a no-op ColorAdjust pass when a mask
            // is present. This avoids having to introduce a dedicated masked blit pass.
            if mask_uniform_index.is_some() || mask.is_some() {
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

            return;
        }
    }

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
                        let required = full.saturating_mul(2);
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
                let required = full.saturating_mul(2);
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
                let can_blur = scratch_targets.len() >= 2 && full.saturating_mul(3) <= budget_bytes;
                if !can_blur {
                    let Some(&scratch_original) = scratch_targets.first() else {
                        effect_degradations.drop_shadow.degraded_target_exhausted =
                            effect_degradations
                                .drop_shadow
                                .degraded_target_exhausted
                                .saturating_add(1);
                        continue;
                    };
                    if full.saturating_mul(2) > budget_bytes {
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
        }
    }
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
    if scratch_targets.len() >= 2 {
        if let Some(downsample_scale) = choose_effect_blur_downsample_scale(
            ctx.viewport_size,
            ctx.format,
            *budget_bytes,
            requested_downsample,
            quality,
        ) {
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
    let required = full.saturating_mul(2);
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

pub(super) fn choose_effect_blur_downsample_scale(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    requested_downsample: u32,
    quality: fret_core::EffectQuality,
) -> Option<u32> {
    blur_primitive::choose_effect_blur_downsample_scale(
        viewport_size,
        format,
        budget_bytes,
        requested_downsample,
        quality,
    )
}

pub(super) fn effect_blur_desired_downsample(
    requested_downsample: u32,
    quality: fret_core::EffectQuality,
) -> u32 {
    blur_primitive::effect_blur_desired_downsample(requested_downsample, quality)
}

pub(super) fn color_adjust_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    if budget_bytes == 0 {
        return false;
    }
    let full = estimate_texture_bytes(viewport_size, format, 1);
    full.saturating_mul(2) <= budget_bytes
}

pub(super) fn dither_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    // Dither uses the same single-scratch in-place pattern as color-adjust/matrix.
    color_adjust_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn noise_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    // Noise uses the same single-scratch in-place pattern as color-adjust/matrix.
    color_adjust_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn backdrop_warp_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    // BackdropWarp v1 uses the same single-scratch texture pattern as color-adjust/matrix.
    color_adjust_enabled(viewport_size, format, budget_bytes)
}

pub(super) fn pixelate_enabled(
    viewport_size: (u32, u32),
    scissor: Option<ScissorRect>,
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    scale: u32,
) -> bool {
    if budget_bytes == 0 {
        return false;
    }
    let scale = scale.max(1);
    if scale <= 1 {
        return true;
    }

    let full = estimate_texture_bytes(viewport_size, format, 1);
    let down_base = scissor
        .filter(|s| s.w != 0 && s.h != 0)
        .map(|s| (s.w, s.h))
        .unwrap_or(viewport_size);
    let down = estimate_texture_bytes(downsampled_size(down_base, scale), format, 1);
    full.saturating_add(down) <= budget_bytes
}

fn choose_clip_mask_target_capped(
    viewport_size: (u32, u32),
    viewport_rect: ScissorRect,
    budget_bytes: u64,
    srcdst_bytes: u64,
    quality: fret_core::EffectQuality,
    tier_cap: Option<PlanTarget>,
    unavailable_mask_targets: &[PlanTarget],
) -> Option<(PlanTarget, (u32, u32), u64)> {
    if budget_bytes == 0 {
        return None;
    }

    let mut desired = match quality {
        fret_core::EffectQuality::High => PlanTarget::Mask0,
        fret_core::EffectQuality::Medium => PlanTarget::Mask1,
        fret_core::EffectQuality::Low => PlanTarget::Mask2,
        fret_core::EffectQuality::Auto => PlanTarget::Mask0,
    };

    if let Some(tier_cap) = tier_cap {
        desired = match (desired, tier_cap) {
            (PlanTarget::Mask0, PlanTarget::Mask1 | PlanTarget::Mask2) => tier_cap,
            (PlanTarget::Mask1, PlanTarget::Mask2) => PlanTarget::Mask2,
            _ => desired,
        };
    }

    for candidate in match desired {
        PlanTarget::Mask0 => [PlanTarget::Mask0, PlanTarget::Mask1, PlanTarget::Mask2].as_slice(),
        PlanTarget::Mask1 => [PlanTarget::Mask1, PlanTarget::Mask2].as_slice(),
        PlanTarget::Mask2 => [PlanTarget::Mask2].as_slice(),
        _ => unreachable!("desired mask tier must be a mask PlanTarget"),
    } {
        if unavailable_mask_targets.contains(candidate) {
            continue;
        }
        let size = mask_target_size_in_viewport_rect(viewport_size, viewport_rect, *candidate);
        let bytes = estimate_texture_bytes(size, wgpu::TextureFormat::R8Unorm, 1);
        if srcdst_bytes.saturating_add(bytes) <= budget_bytes {
            return Some((*candidate, size, bytes));
        }
    }

    None
}

fn mask_target_size_in_viewport_rect(
    _viewport_size: (u32, u32),
    viewport_rect: ScissorRect,
    target: PlanTarget,
) -> (u32, u32) {
    let rect_size = (viewport_rect.w.max(1), viewport_rect.h.max(1));
    match target {
        PlanTarget::Mask0 => rect_size,
        PlanTarget::Mask1 => downsampled_size(rect_size, 2),
        PlanTarget::Mask2 => downsampled_size(rect_size, 4),
        _ => unreachable!("mask_target_size expects a mask PlanTarget"),
    }
}

fn append_scissored_blur_in_place_two_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch_a: PlanTarget,
    scratch_b: PlanTarget,
    full_size: (u32, u32),
    downsample_scale: u32,
    iterations: u32,
    scissor: ScissorRect,
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch_a, PlanTarget::Output);
    debug_assert_ne!(scratch_b, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch_a);
    debug_assert_ne!(srcdst, scratch_b);
    debug_assert_ne!(scratch_a, scratch_b);

    if scissor.w == 0 || scissor.h == 0 || iterations == 0 {
        return;
    }

    let downsample_scale = if downsample_scale >= 4 { 4 } else { 2 };
    let blur_size = downsampled_size(full_size, downsample_scale);

    let down_scissor = map_scissor_downsample_nearest(Some(scissor), downsample_scale, blur_size);
    let down_origin = down_scissor.map(|s| (s.x, s.y)).unwrap_or((0, 0));
    passes.push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
        src: srcdst,
        dst: scratch_a,
        src_size: full_size,
        dst_size: blur_size,
        src_origin: (scissor.x, scissor.y),
        dst_scissor: down_scissor.map(LocalScissorRect),
        dst_origin: down_origin,
        mask_uniform_index: None,
        mask: None,
        mode: ScaleMode::Downsample,
        scale: downsample_scale,
        load: wgpu::LoadOp::Clear(clear),
    }));

    let blur_scissor = down_scissor.map(LocalScissorRect);
    blur_primitive::append_pingpong_blur_passes(
        passes,
        scratch_a,
        scratch_b,
        blur_size,
        blur_scissor,
        iterations,
        clear,
        wgpu::LoadOp::Clear(clear),
    );

    let final_scissor = map_scissor_to_size(Some(scissor), full_size, full_size);
    passes.push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
        src: scratch_a,
        dst: srcdst,
        src_size: blur_size,
        dst_size: full_size,
        src_origin: down_origin,
        dst_scissor: final_scissor.map(LocalScissorRect),
        dst_origin: (scissor.x, scissor.y),
        mask_uniform_index,
        mask,
        mode: ScaleMode::Upscale,
        scale: downsample_scale,
        load: wgpu::LoadOp::Load,
    }));
}

fn append_scissored_blur_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    iterations: u32,
    scissor: ScissorRect,
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if scissor.w == 0 || scissor.h == 0 || iterations == 0 {
        return;
    }

    for ix in 0..iterations {
        let apply_mask = ix + 1 == iterations;
        passes.push(RenderPlanPass::Blur(BlurPass {
            src: srcdst,
            dst: scratch,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(LocalScissorRect(scissor)),
            mask_uniform_index: None,
            mask: None,
            axis: BlurAxis::Horizontal,
            load: wgpu::LoadOp::Clear(clear),
        }));
        passes.push(RenderPlanPass::Blur(BlurPass {
            src: scratch,
            dst: srcdst,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(LocalScissorRect(scissor)),
            mask_uniform_index: apply_mask.then_some(mask_uniform_index).flatten(),
            mask: apply_mask.then_some(mask).flatten(),
            axis: BlurAxis::Vertical,
            load: wgpu::LoadOp::Load,
        }));
    }
}

fn scale_backdrop_warp_v1(
    warp: fret_core::scene::BackdropWarpV1,
    scale_factor: f32,
) -> fret_core::scene::BackdropWarpV1 {
    // The core contract uses logical px (pre-scale-factor). The wgpu backend operates in
    // physical pixels, so we scale here while keeping sanitization in logical space.
    fret_core::scene::BackdropWarpV1 {
        strength_px: fret_core::Px(warp.strength_px.0 * scale_factor),
        scale_px: fret_core::Px(warp.scale_px.0 * scale_factor),
        phase: warp.phase,
        chromatic_aberration_px: fret_core::Px(warp.chromatic_aberration_px.0 * scale_factor),
        kind: warp.kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_applies_clip_only_on_final_step() {
        let ctx = EffectCompileCtx {
            viewport_size: (64, 64),
            format: wgpu::TextureFormat::Rgba8Unorm,
            intermediate_budget_bytes: 1u64 << 60,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect::full(64, 64);

        let mut passes = Vec::new();
        let mut degradations = super::super::EffectDegradationSnapshot::default();
        let mut blur_quality = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes,
            &[],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::Backdrop,
            fret_core::EffectChain::from_steps(&[
                fret_core::EffectStep::GaussianBlur {
                    radius_px: fret_core::Px(14.0),
                    downsample: 2,
                },
                fret_core::EffectStep::CustomV1 {
                    id: fret_core::EffectId::default(),
                    params: fret_core::scene::EffectParamsV1 {
                        vec4s: [[0.0; 4]; 4],
                    },
                    max_sample_offset_px: fret_core::Px(0.0),
                },
            ]),
            fret_core::EffectQuality::Medium,
            scissor,
            Some(7),
            &[],
            &mut degradations,
            &mut blur_quality,
            ctx,
        );

        let blur_masked = passes.iter().any(|p| match p {
            RenderPlanPass::Blur(pass) => pass.mask_uniform_index.is_some() || pass.mask.is_some(),
            _ => false,
        });
        assert!(
            !blur_masked,
            "intermediate blur passes must not apply clip coverage; apply it once at chain end"
        );

        let custom = passes.iter().find_map(|p| match p {
            RenderPlanPass::CustomEffect(pass) => Some(pass),
            _ => None,
        });
        assert!(
            custom.is_some_and(|p| p.mask_uniform_index.is_some() || p.mask.is_some()),
            "the final step must apply clip coverage"
        );
    }

    #[test]
    fn padded_blur_then_custom_uses_work_buffer() {
        let ctx = EffectCompileCtx {
            viewport_size: (64, 64),
            format: wgpu::TextureFormat::Rgba8Unorm,
            intermediate_budget_bytes: 1u64 << 60,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect {
            x: 10,
            y: 12,
            w: 20,
            h: 18,
        };

        let mut passes = Vec::new();
        let mut degradations = super::super::EffectDegradationSnapshot::default();
        let mut blur_quality = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes,
            &[],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::Backdrop,
            fret_core::EffectChain::from_steps(&[
                fret_core::EffectStep::GaussianBlur {
                    radius_px: fret_core::Px(14.0),
                    downsample: 2,
                },
                fret_core::EffectStep::CustomV1 {
                    id: fret_core::EffectId::default(),
                    params: fret_core::scene::EffectParamsV1 {
                        vec4s: [[0.0; 4]; 4],
                    },
                    max_sample_offset_px: fret_core::Px(12.0),
                },
            ]),
            fret_core::EffectQuality::Medium,
            scissor,
            Some(7),
            &[],
            &mut degradations,
            &mut blur_quality,
            ctx,
        );

        let copied_to_work = passes.iter().any(|p| {
            matches!(
                p,
                RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                    src: PlanTarget::Intermediate0,
                    dst: PlanTarget::Intermediate1,
                    ..
                })
            )
        });
        assert!(
            copied_to_work,
            "padded blur->custom should copy srcdst into a work buffer"
        );

        let custom = passes.iter().find_map(|p| match p {
            RenderPlanPass::CustomEffect(pass) => Some(pass),
            _ => None,
        });
        assert!(
            custom.is_some_and(|p| {
                p.src == PlanTarget::Intermediate1
                    && p.dst == PlanTarget::Intermediate0
                    && p.dst_scissor == Some(LocalScissorRect(scissor))
                    && (p.mask_uniform_index.is_some() || p.mask.is_some())
            }),
            "final CustomEffect should read from the work buffer and apply clip coverage once"
        );
    }

    #[test]
    fn gaussian_blur_radius_affects_pass_count() {
        let ctx = EffectCompileCtx {
            viewport_size: (64, 64),
            format: wgpu::TextureFormat::Rgba8Unorm,
            intermediate_budget_bytes: 1u64 << 60,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect::full(64, 64);

        let mut passes_small = Vec::new();
        let mut degr_small = super::super::EffectDegradationSnapshot::default();
        let mut blur_small = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes_small,
            &[],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::FilterContent,
            fret_core::EffectChain::from_steps(&[fret_core::EffectStep::GaussianBlur {
                radius_px: fret_core::Px(8.0),
                downsample: 2,
            }]),
            fret_core::EffectQuality::Medium,
            scissor,
            None,
            &[],
            &mut degr_small,
            &mut blur_small,
            ctx,
        );

        let mut passes_large = Vec::new();
        let mut degr_large = super::super::EffectDegradationSnapshot::default();
        let mut blur_large = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes_large,
            &[],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::FilterContent,
            fret_core::EffectChain::from_steps(&[fret_core::EffectStep::GaussianBlur {
                radius_px: fret_core::Px(16.0),
                downsample: 2,
            }]),
            fret_core::EffectQuality::Medium,
            scissor,
            None,
            &[],
            &mut degr_large,
            &mut blur_large,
            ctx,
        );

        assert!(
            passes_large.len() > passes_small.len(),
            "larger blur radius should compile to more passes"
        );
    }

    #[test]
    fn dither_compiles_to_pass() {
        let ctx = EffectCompileCtx {
            viewport_size: (64, 64),
            format: wgpu::TextureFormat::Rgba8Unorm,
            intermediate_budget_bytes: 1u64 << 60,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect::full(64, 64);

        let mut passes = Vec::new();
        let mut degradations = super::super::EffectDegradationSnapshot::default();
        let mut blur_quality = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes,
            &[],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::FilterContent,
            fret_core::EffectChain::from_steps(&[fret_core::EffectStep::Dither {
                mode: fret_core::DitherMode::Bayer4x4,
            }]),
            fret_core::EffectQuality::Medium,
            scissor,
            None,
            &[],
            &mut degradations,
            &mut blur_quality,
            ctx,
        );

        assert!(
            passes
                .iter()
                .any(|p| matches!(p, RenderPlanPass::Dither(_))),
            "dither step should compile to a Dither pass"
        );
    }

    #[test]
    fn noise_compiles_to_pass() {
        let ctx = EffectCompileCtx {
            viewport_size: (64, 64),
            format: wgpu::TextureFormat::Rgba8Unorm,
            intermediate_budget_bytes: 1u64 << 60,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect::full(64, 64);

        let mut passes = Vec::new();
        let mut degradations = super::super::EffectDegradationSnapshot::default();
        let mut blur_quality = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes,
            &[],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::FilterContent,
            fret_core::EffectChain::from_steps(&[fret_core::EffectStep::NoiseV1(
                fret_core::scene::NoiseV1 {
                    strength: 0.1,
                    scale_px: fret_core::Px(4.0),
                    phase: 0.0,
                },
            )]),
            fret_core::EffectQuality::Medium,
            scissor,
            None,
            &[],
            &mut degradations,
            &mut blur_quality,
            ctx,
        );

        assert!(
            passes.iter().any(|p| matches!(p, RenderPlanPass::Noise(_))),
            "noise step should compile to a Noise pass"
        );
    }

    #[test]
    fn gaussian_blur_budget_zero_increments_effect_degradations() {
        let ctx = EffectCompileCtx {
            viewport_size: (64, 64),
            format: wgpu::TextureFormat::Rgba8Unorm,
            intermediate_budget_bytes: 0,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect::full(64, 64);

        let mut passes = Vec::new();
        let mut degradations = super::super::EffectDegradationSnapshot::default();
        let mut blur_quality = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes,
            &[],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::FilterContent,
            fret_core::EffectChain::from_steps(&[fret_core::EffectStep::GaussianBlur {
                radius_px: fret_core::Px(8.0),
                downsample: 2,
            }]),
            fret_core::EffectQuality::Medium,
            scissor,
            None,
            &[],
            &mut degradations,
            &mut blur_quality,
            ctx,
        );

        assert_eq!(degradations.gaussian_blur.requested, 1);
        assert_eq!(degradations.gaussian_blur.applied, 0);
        assert_eq!(degradations.gaussian_blur.degraded_budget_zero, 1);
    }

    #[test]
    fn color_adjust_missing_scratch_increments_effect_degradations() {
        let ctx = EffectCompileCtx {
            viewport_size: (64, 64),
            format: wgpu::TextureFormat::Rgba8Unorm,
            intermediate_budget_bytes: 1u64 << 60,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect::full(64, 64);

        let mut passes = Vec::new();
        let mut degradations = super::super::EffectDegradationSnapshot::default();
        let mut blur_quality = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes,
            &[
                PlanTarget::Intermediate1,
                PlanTarget::Intermediate2,
                PlanTarget::Intermediate3,
            ],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::FilterContent,
            fret_core::EffectChain::from_steps(&[fret_core::EffectStep::ColorAdjust {
                saturation: 1.0,
                brightness: 1.0,
                contrast: 1.0,
            }]),
            fret_core::EffectQuality::Medium,
            scissor,
            None,
            &[],
            &mut degradations,
            &mut blur_quality,
            ctx,
        );

        assert_eq!(degradations.color_adjust.requested, 1);
        assert_eq!(degradations.color_adjust.applied, 0);
        assert_eq!(degradations.color_adjust.degraded_target_exhausted, 1);
        assert!(passes.is_empty());
    }

    #[test]
    fn gaussian_blur_quality_records_applied_downsample_scale() {
        let viewport_size = (256, 256);
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let full = estimate_texture_bytes(viewport_size, format, 1);
        let half = estimate_texture_bytes(downsampled_size(viewport_size, 2), format, 1);
        let quarter = estimate_texture_bytes(downsampled_size(viewport_size, 4), format, 1);
        let required_half = full.saturating_add(half.saturating_mul(2));
        let required_quarter = full.saturating_add(quarter.saturating_mul(2));
        let budget_bytes = required_quarter.min(required_half.saturating_sub(1));

        let ctx = EffectCompileCtx {
            viewport_size,
            format,
            intermediate_budget_bytes: budget_bytes,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect::full(viewport_size.0, viewport_size.1);

        let mut passes = Vec::new();
        let mut degradations = super::super::EffectDegradationSnapshot::default();
        let mut blur_quality = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes,
            &[],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::FilterContent,
            fret_core::EffectChain::from_steps(&[fret_core::EffectStep::GaussianBlur {
                radius_px: fret_core::Px(16.0),
                downsample: 2,
            }]),
            fret_core::EffectQuality::Medium,
            scissor,
            None,
            &[],
            &mut degradations,
            &mut blur_quality,
            ctx,
        );

        assert_eq!(blur_quality.gaussian_blur.applied, 1);
        assert_eq!(blur_quality.gaussian_blur.applied_downsample_4, 1);
        assert_eq!(blur_quality.gaussian_blur.quality_degraded_downsample, 1);
        assert!(passes.iter().any(|p| matches!(p, RenderPlanPass::Blur(_))));
    }

    #[test]
    fn drop_shadow_budget_pressure_degrades_to_hard_shadow() {
        let viewport_size = (128, 128);
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let full = estimate_texture_bytes(viewport_size, format, 1);
        let budget_bytes = full.saturating_mul(2);

        let ctx = EffectCompileCtx {
            viewport_size,
            format,
            intermediate_budget_bytes: budget_bytes,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect::full(viewport_size.0, viewport_size.1);

        let shadow = fret_core::scene::DropShadowV1 {
            offset_px: fret_core::Point::new(fret_core::Px(2.0), fret_core::Px(3.0)),
            blur_radius_px: fret_core::Px(8.0),
            downsample: 2,
            color: fret_core::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        };

        let mut passes = Vec::new();
        let mut degradations = super::super::EffectDegradationSnapshot::default();
        let mut blur_quality = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes,
            &[],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::FilterContent,
            fret_core::EffectChain::from_steps(&[fret_core::EffectStep::DropShadowV1(shadow)]),
            fret_core::EffectQuality::Medium,
            scissor,
            None,
            &[],
            &mut degradations,
            &mut blur_quality,
            ctx,
        );

        assert_eq!(degradations.drop_shadow.requested, 1);
        assert_eq!(degradations.drop_shadow.applied, 1);
        assert_eq!(degradations.drop_shadow.degraded_budget_insufficient, 0);
        assert!(
            passes
                .iter()
                .any(|p| matches!(p, RenderPlanPass::DropShadow(_))),
            "hard drop shadow fallback should still emit a DropShadow pass"
        );
        assert!(
            !passes.iter().any(|p| matches!(p, RenderPlanPass::Blur(_))),
            "hard drop shadow fallback must not emit blur passes"
        );
        assert_eq!(blur_quality.drop_shadow.applied, 1);
        assert_eq!(blur_quality.drop_shadow.applied_iterations_zero, 1);
        assert_eq!(blur_quality.drop_shadow.quality_degraded_blur_removed, 1);
    }

    #[test]
    fn gaussian_blur_target_pressure_falls_back_to_single_scratch_blur() {
        let viewport_size = (256, 256);
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let full = estimate_texture_bytes(viewport_size, format, 1);
        let budget_bytes = full.saturating_mul(2);
        let ctx = EffectCompileCtx {
            viewport_size,
            format,
            intermediate_budget_bytes: budget_bytes,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        };
        let scissor = ScissorRect::full(viewport_size.0, viewport_size.1);

        let mut passes = Vec::new();
        let mut degradations = super::super::EffectDegradationSnapshot::default();
        let mut blur_quality = super::super::BlurQualitySnapshot::default();
        apply_chain_in_place(
            &mut passes,
            &[PlanTarget::Intermediate1, PlanTarget::Intermediate2],
            PlanTarget::Intermediate0,
            fret_core::EffectMode::FilterContent,
            fret_core::EffectChain::from_steps(&[fret_core::EffectStep::GaussianBlur {
                radius_px: fret_core::Px(16.0),
                downsample: 2,
            }]),
            fret_core::EffectQuality::Medium,
            scissor,
            None,
            &[],
            &mut degradations,
            &mut blur_quality,
            ctx,
        );

        assert_eq!(degradations.gaussian_blur.requested, 1);
        assert_eq!(degradations.gaussian_blur.applied, 1);
        assert_eq!(degradations.gaussian_blur.degraded_target_exhausted, 0);
        assert_eq!(blur_quality.gaussian_blur.applied, 1);
        assert_eq!(blur_quality.gaussian_blur.applied_downsample_1, 1);
        assert_eq!(blur_quality.gaussian_blur.quality_degraded_blur_removed, 0);
        assert_eq!(blur_quality.gaussian_blur.quality_degraded_downsample, 1);
        assert!(
            passes.iter().any(|p| matches!(p, RenderPlanPass::Blur(_))),
            "single-scratch blur fallback should still emit blur passes"
        );
        assert!(
            !passes
                .iter()
                .any(|p| matches!(p, RenderPlanPass::ScaleNearest(_))),
            "single-scratch blur fallback must not emit downsample scale passes"
        );
    }
}

fn append_color_adjust_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: Option<ScissorRect>,
    saturation: f32,
    brightness: f32,
    contrast: f32,
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if let Some(scissor) = scissor {
        if scissor.w == 0 || scissor.h == 0 {
            return;
        }

        passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: srcdst,
            dst: scratch,
            src_size: size,
            dst_size: size,
            dst_scissor: None,
            encode_output_srgb: false,
            load: wgpu::LoadOp::Clear(clear),
        }));
        passes.push(RenderPlanPass::ColorAdjust(ColorAdjustPass {
            src: scratch,
            dst: srcdst,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(LocalScissorRect(scissor)),
            mask_uniform_index,
            mask,
            saturation,
            brightness,
            contrast,
            load: wgpu::LoadOp::Load,
        }));
        return;
    }

    passes.push(RenderPlanPass::ColorAdjust(ColorAdjustPass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        mask_uniform_index: None,
        mask: None,
        saturation,
        brightness,
        contrast,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
        src: scratch,
        dst: srcdst,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        encode_output_srgb: false,
        load: wgpu::LoadOp::Clear(clear),
    }));
}

fn append_dither_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: Option<ScissorRect>,
    mode: fret_core::DitherMode,
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if let Some(scissor) = scissor {
        if scissor.w == 0 || scissor.h == 0 {
            return;
        }

        passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: srcdst,
            dst: scratch,
            src_size: size,
            dst_size: size,
            dst_scissor: None,
            encode_output_srgb: false,
            load: wgpu::LoadOp::Clear(clear),
        }));
        passes.push(RenderPlanPass::Dither(DitherPass {
            src: scratch,
            dst: srcdst,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(LocalScissorRect(scissor)),
            mask_uniform_index,
            mask,
            mode,
            load: wgpu::LoadOp::Load,
        }));
        return;
    }

    passes.push(RenderPlanPass::Dither(DitherPass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        mask_uniform_index: None,
        mask: None,
        mode,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
        src: scratch,
        dst: srcdst,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        encode_output_srgb: false,
        load: wgpu::LoadOp::Clear(clear),
    }));
}

fn append_noise_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: Option<ScissorRect>,
    strength: f32,
    scale_px: f32,
    phase: f32,
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if let Some(scissor) = scissor {
        if scissor.w == 0 || scissor.h == 0 {
            return;
        }

        passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: srcdst,
            dst: scratch,
            src_size: size,
            dst_size: size,
            dst_scissor: None,
            encode_output_srgb: false,
            load: wgpu::LoadOp::Clear(clear),
        }));
        passes.push(RenderPlanPass::Noise(NoisePass {
            src: scratch,
            dst: srcdst,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(LocalScissorRect(scissor)),
            mask_uniform_index,
            mask,
            strength,
            scale_px,
            phase,
            load: wgpu::LoadOp::Load,
        }));
        return;
    }

    passes.push(RenderPlanPass::Noise(NoisePass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        mask_uniform_index: None,
        mask: None,
        strength,
        scale_px,
        phase,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
        src: scratch,
        dst: srcdst,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        encode_output_srgb: false,
        load: wgpu::LoadOp::Clear(clear),
    }));
}

fn append_custom_effect_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: Option<ScissorRect>,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if let Some(scissor) = scissor {
        if scissor.w == 0 || scissor.h == 0 {
            return;
        }

        passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: srcdst,
            dst: scratch,
            src_size: size,
            dst_size: size,
            dst_scissor: None,
            encode_output_srgb: false,
            load: wgpu::LoadOp::Clear(clear),
        }));
        passes.push(RenderPlanPass::CustomEffect(CustomEffectPass {
            src: scratch,
            dst: srcdst,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(LocalScissorRect(scissor)),
            mask_uniform_index,
            mask,
            effect,
            params,
            load: wgpu::LoadOp::Load,
        }));
        return;
    }

    passes.push(RenderPlanPass::CustomEffect(CustomEffectPass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        mask_uniform_index: None,
        mask: None,
        effect,
        params,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
        src: scratch,
        dst: srcdst,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        encode_output_srgb: false,
        load: wgpu::LoadOp::Clear(clear),
    }));
}

fn append_backdrop_warp_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: ScissorRect,
    warp: fret_core::scene::BackdropWarpV1,
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if scissor.w == 0 || scissor.h == 0 {
        return;
    }

    // Scissored in-place pattern: preserve outside-region content by pre-blitting into scratch.
    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        encode_output_srgb: false,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::BackdropWarp(BackdropWarpPass {
        src: scratch,
        dst: srcdst,
        src_size: size,
        dst_size: size,
        origin_px: (scissor.x, scissor.y),
        bounds_size_px: (scissor.w, scissor.h),
        dst_scissor: Some(LocalScissorRect(scissor)),
        mask_uniform_index,
        mask,
        strength_px: warp.strength_px.0,
        scale_px: warp.scale_px.0,
        phase: warp.phase,
        chromatic_aberration_px: warp.chromatic_aberration_px.0,
        kind: warp.kind,
        warp_image: None,
        warp_uv: fret_core::scene::UvRect::FULL,
        warp_sampling: fret_core::scene::ImageSamplingHint::Default,
        warp_encoding: fret_core::scene::WarpMapEncodingV1::RgSigned,
        load: wgpu::LoadOp::Load,
    }));
}

#[allow(dead_code)]
fn append_color_matrix_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: Option<ScissorRect>,
    matrix: [f32; 20],
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if let Some(scissor) = scissor {
        if scissor.w == 0 || scissor.h == 0 {
            return;
        }

        passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: srcdst,
            dst: scratch,
            src_size: size,
            dst_size: size,
            dst_scissor: None,
            encode_output_srgb: false,
            load: wgpu::LoadOp::Clear(clear),
        }));
        passes.push(RenderPlanPass::ColorMatrix(ColorMatrixPass {
            src: scratch,
            dst: srcdst,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(LocalScissorRect(scissor)),
            mask_uniform_index,
            mask,
            matrix,
            load: wgpu::LoadOp::Load,
        }));
        return;
    }

    passes.push(RenderPlanPass::ColorMatrix(ColorMatrixPass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        mask_uniform_index: None,
        mask: None,
        matrix,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
        src: scratch,
        dst: srcdst,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        encode_output_srgb: false,
        load: wgpu::LoadOp::Clear(clear),
    }));
}

#[allow(dead_code)]
fn append_alpha_threshold_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: Option<ScissorRect>,
    cutoff: f32,
    soft: f32,
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if let Some(scissor) = scissor {
        if scissor.w == 0 || scissor.h == 0 {
            return;
        }

        passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: srcdst,
            dst: scratch,
            src_size: size,
            dst_size: size,
            dst_scissor: None,
            encode_output_srgb: false,
            load: wgpu::LoadOp::Clear(clear),
        }));
        passes.push(RenderPlanPass::AlphaThreshold(AlphaThresholdPass {
            src: scratch,
            dst: srcdst,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(LocalScissorRect(scissor)),
            mask_uniform_index,
            mask,
            cutoff,
            soft,
            load: wgpu::LoadOp::Load,
        }));
        return;
    }

    passes.push(RenderPlanPass::AlphaThreshold(AlphaThresholdPass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        mask_uniform_index: None,
        mask: None,
        cutoff,
        soft,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
        src: scratch,
        dst: srcdst,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        encode_output_srgb: false,
        load: wgpu::LoadOp::Clear(clear),
    }));
}

fn append_pixelate_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    full_size: (u32, u32),
    scissor: Option<ScissorRect>,
    scale: u32,
    clear: wgpu::Color,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    let scale = scale.max(1);
    if scale <= 1 {
        return;
    }

    let scissor = scissor.filter(|s| s.w != 0 && s.h != 0);
    let effect_rect = scissor.unwrap_or(ScissorRect::full(full_size.0, full_size.1));
    let down_size = downsampled_size((effect_rect.w, effect_rect.h), scale);

    passes.push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
        src: srcdst,
        dst: scratch,
        src_size: full_size,
        dst_size: down_size,
        src_origin: (effect_rect.x, effect_rect.y),
        dst_scissor: None,
        dst_origin: (0, 0),
        mask_uniform_index: None,
        mask: None,
        mode: ScaleMode::Downsample,
        scale,
        load: wgpu::LoadOp::Clear(clear),
    }));

    passes.push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
        src: scratch,
        dst: srcdst,
        src_size: down_size,
        dst_size: full_size,
        src_origin: (0, 0),
        dst_scissor: scissor.map(LocalScissorRect),
        dst_origin: (effect_rect.x, effect_rect.y),
        mask_uniform_index: scissor.is_some().then_some(mask_uniform_index).flatten(),
        mask: scissor.is_some().then_some(mask).flatten(),
        mode: ScaleMode::Upscale,
        scale,
        load: if scissor.is_some() {
            wgpu::LoadOp::Load
        } else {
            wgpu::LoadOp::Clear(clear)
        },
    }));
}

pub(super) fn map_scissor_to_size(
    scissor: Option<ScissorRect>,
    src_size: (u32, u32),
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    let scissor = scissor?;
    if scissor.w == 0 || scissor.h == 0 {
        return None;
    }

    let src_w = src_size.0.max(1);
    let src_h = src_size.1.max(1);
    let dst_w = dst_size.0.max(1);
    let dst_h = dst_size.1.max(1);

    let x0 = scissor.x;
    let y0 = scissor.y;
    let x1 = x0.saturating_add(scissor.w);
    let y1 = y0.saturating_add(scissor.h);

    let sx0 = x0.saturating_mul(dst_w) / src_w;
    let sy0 = y0.saturating_mul(dst_h) / src_h;
    let sx1 = x1.saturating_mul(dst_w).div_ceil(src_w);
    let sy1 = y1.saturating_mul(dst_h).div_ceil(src_h);

    let sx0 = sx0.min(dst_w);
    let sy0 = sy0.min(dst_h);
    let sx1 = sx1.min(dst_w);
    let sy1 = sy1.min(dst_h);

    if sx1 <= sx0 || sy1 <= sy0 {
        return None;
    }

    Some(ScissorRect {
        x: sx0,
        y: sy0,
        w: sx1 - sx0,
        h: sy1 - sy0,
    })
}

pub(super) fn map_scissor_downsample_nearest(
    scissor: Option<ScissorRect>,
    scale: u32,
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    let scissor = scissor?;
    if scissor.w == 0 || scissor.h == 0 {
        return None;
    }
    let scale = scale.max(1);
    if scale <= 1 {
        return map_scissor_to_size(Some(scissor), dst_size, dst_size);
    }

    let dst_w = dst_size.0.max(1);
    let dst_h = dst_size.1.max(1);

    let x0 = scissor.x;
    let y0 = scissor.y;
    let x1 = x0.saturating_add(scissor.w);
    let y1 = y0.saturating_add(scissor.h);

    let sx0 = x0 / scale;
    let sy0 = y0 / scale;
    let sx1 = x1.div_ceil(scale);
    let sy1 = y1.div_ceil(scale);

    let sx0 = sx0.min(dst_w);
    let sy0 = sy0.min(dst_h);
    let sx1 = sx1.min(dst_w);
    let sy1 = sy1.min(dst_h);

    if sx1 <= sx0 || sy1 <= sy0 {
        return None;
    }

    Some(ScissorRect {
        x: sx0,
        y: sy0,
        w: sx1 - sx0,
        h: sy1 - sy0,
    })
}
