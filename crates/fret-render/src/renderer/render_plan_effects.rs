use super::frame_targets::downsampled_size;
use super::intermediate_pool::estimate_texture_bytes;
use super::{
    BlurAxis, BlurPass, ClipMaskPass, ColorAdjustPass, FullscreenBlitPass, MaskRef, PlanTarget,
    RenderPlanPass, ScaleMode, ScaleNearestPass, ScissorRect,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct EffectCompileCtx {
    pub(super) viewport_size: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) intermediate_budget_bytes: u64,
    pub(super) clear: wgpu::Color,
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
    chain: fret_core::EffectChain,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    mask_uniform_index: Option<u32>,
    ctx: EffectCompileCtx,
) {
    if srcdst == PlanTarget::Output || scissor.w == 0 || scissor.h == 0 {
        return;
    }

    let scratch_targets = available_scratch_targets(in_use_targets, srcdst);
    let forced_quarter_blur = scratch_targets.len() >= 2
        && chain.iter().any(|step| match step {
            fret_core::EffectStep::GaussianBlur { downsample, .. } => {
                let requested_downsample = if downsample >= 4 { 4 } else { 2 };
                let desired_downsample =
                    effect_blur_desired_downsample(requested_downsample, quality);
                if desired_downsample != 2 {
                    return false;
                }
                let Some(chosen) = choose_effect_blur_downsample_scale(
                    ctx.viewport_size,
                    ctx.format,
                    ctx.intermediate_budget_bytes,
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
        && let Some(mask_target) = choose_clip_mask_target_capped(
            ctx.viewport_size,
            scissor,
            ctx.intermediate_budget_bytes,
            quality,
            mask_tier_cap,
        ) {
        let mask_size = mask_target_size_in_viewport_rect(ctx.viewport_size, scissor, mask_target);
        passes.push(RenderPlanPass::ClipMask(ClipMaskPass {
            dst: mask_target,
            dst_size: mask_size,
            dst_scissor: None,
            uniform_index,
        }));
        Some(MaskRef {
            target: mask_target,
            size: mask_size,
            viewport_rect: scissor,
        })
    } else {
        None
    };

    for step in chain.iter() {
        match step {
            fret_core::EffectStep::GaussianBlur { downsample, .. } => {
                let requested_downsample = if downsample >= 4 { 4 } else { 2 };
                if scratch_targets.len() >= 2 {
                    let Some(downsample_scale) = choose_effect_blur_downsample_scale(
                        ctx.viewport_size,
                        ctx.format,
                        ctx.intermediate_budget_bytes,
                        requested_downsample,
                        quality,
                    ) else {
                        continue;
                    };
                    append_scissored_blur_in_place_two_scratch(
                        passes,
                        srcdst,
                        scratch_targets[0],
                        scratch_targets[1],
                        ctx.viewport_size,
                        downsample_scale,
                        scissor,
                        ctx.clear,
                        mask_uniform_index,
                        mask,
                    );
                    continue;
                }

                let Some(&scratch) = scratch_targets.first() else {
                    continue;
                };
                if ctx.intermediate_budget_bytes == 0 {
                    continue;
                }
                let full = estimate_texture_bytes(ctx.viewport_size, ctx.format, 1);
                let required = full.saturating_mul(2);
                if required > ctx.intermediate_budget_bytes {
                    continue;
                }
                append_scissored_blur_in_place_single_scratch(
                    passes,
                    srcdst,
                    scratch,
                    ctx.viewport_size,
                    scissor,
                    ctx.clear,
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::ColorAdjust {
                saturation,
                brightness,
                contrast,
            } => {
                if !color_adjust_enabled(
                    ctx.viewport_size,
                    ctx.format,
                    ctx.intermediate_budget_bytes,
                ) {
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    continue;
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
                    mask_uniform_index,
                    mask,
                );
            }
            fret_core::EffectStep::Pixelate { scale } => {
                if !pixelate_enabled(
                    ctx.viewport_size,
                    Some(scissor),
                    ctx.format,
                    ctx.intermediate_budget_bytes,
                    scale,
                ) {
                    continue;
                }
                let Some(&scratch) = scratch_targets.first() else {
                    continue;
                };
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
            fret_core::EffectStep::Dither { .. } => {
                // Not yet implemented in effect chains (debug-only postprocess exists).
            }
        }
    }
}

pub(super) fn choose_effect_blur_downsample_scale(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    requested_downsample: u32,
    quality: fret_core::EffectQuality,
) -> Option<u32> {
    if budget_bytes == 0 {
        return None;
    }

    let full = estimate_texture_bytes(viewport_size, format, 1);
    let half = estimate_texture_bytes(downsampled_size(viewport_size, 2), format, 1);
    let quarter = estimate_texture_bytes(downsampled_size(viewport_size, 4), format, 1);

    let required_half = full.saturating_add(half.saturating_mul(2));
    let required_quarter = full.saturating_add(quarter.saturating_mul(2));

    let desired = effect_blur_desired_downsample(requested_downsample, quality);

    if desired == 2 && required_half <= budget_bytes {
        return Some(2);
    }
    if required_quarter <= budget_bytes {
        return Some(4);
    }
    None
}

pub(super) fn effect_blur_desired_downsample(
    requested_downsample: u32,
    quality: fret_core::EffectQuality,
) -> u32 {
    let desired = match quality {
        fret_core::EffectQuality::Low => 4,
        fret_core::EffectQuality::Medium | fret_core::EffectQuality::High => 2,
        fret_core::EffectQuality::Auto => requested_downsample,
    };
    if desired >= 4 { 4 } else { 2 }
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
    quality: fret_core::EffectQuality,
    tier_cap: Option<PlanTarget>,
) -> Option<PlanTarget> {
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
        let size = mask_target_size_in_viewport_rect(viewport_size, viewport_rect, *candidate);
        let bytes = estimate_texture_bytes(size, wgpu::TextureFormat::R8Unorm, 1);
        if bytes <= budget_bytes {
            return Some(*candidate);
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

    if scissor.w == 0 || scissor.h == 0 {
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
        dst_scissor: down_scissor,
        dst_origin: down_origin,
        mask_uniform_index: None,
        mask: None,
        mode: ScaleMode::Downsample,
        scale: downsample_scale,
        load: wgpu::LoadOp::Clear(clear),
    }));

    let blur_scissor = down_scissor;
    passes.push(RenderPlanPass::Blur(BlurPass {
        src: scratch_a,
        dst: scratch_b,
        src_size: blur_size,
        dst_size: blur_size,
        dst_scissor: blur_scissor,
        mask_uniform_index: None,
        mask: None,
        axis: BlurAxis::Horizontal,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::Blur(BlurPass {
        src: scratch_b,
        dst: scratch_a,
        src_size: blur_size,
        dst_size: blur_size,
        dst_scissor: blur_scissor,
        mask_uniform_index: None,
        mask: None,
        axis: BlurAxis::Vertical,
        load: wgpu::LoadOp::Clear(clear),
    }));

    let final_scissor = map_scissor_to_size(Some(scissor), full_size, full_size);
    passes.push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
        src: scratch_a,
        dst: srcdst,
        src_size: blur_size,
        dst_size: full_size,
        src_origin: down_origin,
        dst_scissor: final_scissor,
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
    scissor: ScissorRect,
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

    passes.push(RenderPlanPass::Blur(BlurPass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: Some(scissor),
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
        dst_scissor: Some(scissor),
        mask_uniform_index,
        mask,
        axis: BlurAxis::Vertical,
        load: wgpu::LoadOp::Load,
    }));
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
            load: wgpu::LoadOp::Clear(clear),
        }));
        passes.push(RenderPlanPass::ColorAdjust(ColorAdjustPass {
            src: scratch,
            dst: srcdst,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(scissor),
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
        dst_scissor: scissor,
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
