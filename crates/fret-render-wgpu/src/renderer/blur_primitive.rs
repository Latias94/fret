use super::frame_targets::downsampled_size;
use super::intermediate_pool::estimate_texture_bytes;
use super::render_plan::{BlurAxis, BlurPass, LocalScissorRect, PlanTarget, RenderPlanPass};

const BLUR_KERNEL_RADIUS_PX: f32 = 4.0;

pub(super) fn blur_iterations_for_radius(
    radius_px: f32,
    downsample_scale: u32,
    quality: fret_core::EffectQuality,
) -> u32 {
    let radius_px = if radius_px.is_finite() {
        radius_px.max(0.0)
    } else {
        0.0
    };
    if radius_px <= 0.0 {
        return 0;
    }

    let downsample_scale = downsample_scale.max(1) as f32;
    let per_iter = BLUR_KERNEL_RADIUS_PX * downsample_scale;
    let mut iterations = (radius_px / per_iter).ceil() as u32;
    iterations = iterations.max(1);

    let max_iterations = match quality {
        fret_core::EffectQuality::Low => 2,
        fret_core::EffectQuality::Auto | fret_core::EffectQuality::Medium => 4,
        fret_core::EffectQuality::High => 8,
    };
    iterations.min(max_iterations)
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

pub(super) fn append_pingpong_blur_passes(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    dst_scissor: Option<LocalScissorRect>,
    iterations: u32,
    clear: wgpu::Color,
    vertical_load: wgpu::LoadOp<wgpu::Color>,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if iterations == 0 {
        return;
    }

    for _ in 0..iterations {
        passes.push(RenderPlanPass::Blur(BlurPass {
            src: srcdst,
            dst: scratch,
            src_size: size,
            dst_size: size,
            dst_scissor,
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
            dst_scissor,
            mask_uniform_index: None,
            mask: None,
            axis: BlurAxis::Vertical,
            load: vertical_load,
        }));
    }
}
