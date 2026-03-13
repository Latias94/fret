use super::*;

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
    budget_bytes >= base_required_bytes_for_srcdst_and_single_scratch(full)
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

fn prepare_effect_single_scratch(
    scratch_targets: &[PlanTarget],
    counters: &mut super::super::EffectDegradationCounters,
    budget_bytes: u64,
    enabled: bool,
) -> Option<PlanTarget> {
    counters.requested = counters.requested.saturating_add(1);
    if !enabled {
        if budget_bytes == 0 {
            counters.degraded_budget_zero = counters.degraded_budget_zero.saturating_add(1);
        } else {
            counters.degraded_budget_insufficient =
                counters.degraded_budget_insufficient.saturating_add(1);
        }
        return None;
    }

    let Some(&scratch) = scratch_targets.first() else {
        counters.degraded_target_exhausted = counters.degraded_target_exhausted.saturating_add(1);
        return None;
    };
    counters.applied = counters.applied.saturating_add(1);
    Some(scratch)
}

pub(super) fn choose_clip_mask_target_capped(
    _viewport_size: (u32, u32),
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
        let size = mask_target_size_in_viewport_rect(viewport_rect, *candidate);
        let bytes = estimate_clip_mask_bytes(size);
        if srcdst_bytes.saturating_add(bytes) <= budget_bytes {
            return Some((*candidate, size, bytes));
        }
    }

    None
}

fn mask_target_size_in_viewport_rect(viewport_rect: ScissorRect, target: PlanTarget) -> (u32, u32) {
    let rect_size = (viewport_rect.w.max(1), viewport_rect.h.max(1));
    match target {
        PlanTarget::Mask0 => rect_size,
        PlanTarget::Mask1 => downsampled_size(rect_size, 2),
        PlanTarget::Mask2 => downsampled_size(rect_size, 4),
        _ => unreachable!("mask_target_size expects a mask PlanTarget"),
    }
}

pub(super) fn append_scissored_blur_in_place_two_scratch(
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

pub(super) fn append_scissored_blur_in_place_single_scratch(
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

pub(super) fn scale_backdrop_warp_v1(
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

pub(super) fn append_color_adjust_in_place_single_scratch(
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

pub(super) fn append_dither_in_place_single_scratch(
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

pub(super) fn append_noise_in_place_single_scratch(
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

pub(super) fn append_backdrop_warp_in_place_single_scratch(
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

fn prepare_backdrop_warp_single_scratch(
    scratch_targets: &[PlanTarget],
    mode: fret_core::EffectMode,
    budget_bytes: u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
) -> Option<PlanTarget> {
    if mode != fret_core::EffectMode::Backdrop {
        return None;
    }

    prepare_effect_single_scratch(
        scratch_targets,
        &mut effect_degradations.backdrop_warp,
        budget_bytes,
        backdrop_warp_enabled(viewport_size, format, budget_bytes),
    )
}

pub(super) fn apply_backdrop_warp_v1_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    scissor: ScissorRect,
    warp: fret_core::scene::BackdropWarpV1,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    let Some(scratch) = prepare_backdrop_warp_single_scratch(
        scratch_targets,
        mode,
        *budget_bytes,
        effect_degradations,
        ctx.viewport_size,
        ctx.format,
    ) else {
        return;
    };

    append_backdrop_warp_in_place_single_scratch(
        passes,
        srcdst,
        scratch,
        ctx.viewport_size,
        scissor,
        scale_backdrop_warp_v1(warp.sanitize(), ctx.scale_factor),
        ctx.clear,
        None,
        None,
    );
}

pub(super) fn apply_backdrop_warp_v2_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    mode: fret_core::EffectMode,
    scissor: ScissorRect,
    warp: fret_core::scene::BackdropWarpV2,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    let Some(scratch) = prepare_backdrop_warp_single_scratch(
        scratch_targets,
        mode,
        *budget_bytes,
        effect_degradations,
        ctx.viewport_size,
        ctx.format,
    ) else {
        return;
    };

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

    let base = warp.base.sanitize();
    let (warp_image, warp_uv, warp_sampling, warp_encoding) = match warp.field {
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

pub(super) fn apply_noise_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    noise: fret_core::scene::NoiseV1,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    let Some(scratch) = prepare_effect_single_scratch(
        scratch_targets,
        &mut effect_degradations.noise,
        *budget_bytes,
        noise_enabled(ctx.viewport_size, ctx.format, *budget_bytes),
    ) else {
        return;
    };

    append_noise_in_place_single_scratch(
        passes,
        srcdst,
        scratch,
        ctx.viewport_size,
        Some(scissor),
        noise.strength,
        (noise.scale_px.0 * ctx.scale_factor).max(1.0),
        noise.phase,
        ctx.clear,
        None,
        None,
    );
}

pub(super) fn apply_color_adjust_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    saturation: f32,
    brightness: f32,
    contrast: f32,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    let Some(scratch) = prepare_effect_single_scratch(
        scratch_targets,
        &mut effect_degradations.color_adjust,
        *budget_bytes,
        color_adjust_enabled(ctx.viewport_size, ctx.format, *budget_bytes),
    ) else {
        return;
    };

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

pub(super) fn apply_color_matrix_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    matrix: [f32; 20],
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    let Some(scratch) = prepare_effect_single_scratch(
        scratch_targets,
        &mut effect_degradations.color_matrix,
        *budget_bytes,
        color_adjust_enabled(ctx.viewport_size, ctx.format, *budget_bytes),
    ) else {
        return;
    };

    append_color_matrix_in_place_single_scratch(
        passes,
        srcdst,
        scratch,
        ctx.viewport_size,
        Some(scissor),
        matrix,
        ctx.clear,
        None,
        None,
    );
}

pub(super) fn apply_alpha_threshold_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    cutoff: f32,
    soft: f32,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    let Some(scratch) = prepare_effect_single_scratch(
        scratch_targets,
        &mut effect_degradations.alpha_threshold,
        *budget_bytes,
        color_adjust_enabled(ctx.viewport_size, ctx.format, *budget_bytes),
    ) else {
        return;
    };

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

pub(super) fn apply_pixelate_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    scale: u32,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    let Some(scratch) = prepare_effect_single_scratch(
        scratch_targets,
        &mut effect_degradations.pixelate,
        *budget_bytes,
        pixelate_enabled(
            ctx.viewport_size,
            Some(scissor),
            ctx.format,
            *budget_bytes,
            scale,
        ),
    ) else {
        return;
    };

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

pub(super) fn apply_dither_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    mode: fret_core::DitherMode,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    let Some(scratch) = prepare_effect_single_scratch(
        scratch_targets,
        &mut effect_degradations.dither,
        *budget_bytes,
        dither_enabled(ctx.viewport_size, ctx.format, *budget_bytes),
    ) else {
        return;
    };

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

#[allow(dead_code)]
pub(super) fn append_color_matrix_in_place_single_scratch(
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
pub(super) fn append_alpha_threshold_in_place_single_scratch(
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

pub(super) fn append_pixelate_in_place_single_scratch(
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
