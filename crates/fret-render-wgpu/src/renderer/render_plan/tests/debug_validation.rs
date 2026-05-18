use super::*;
use crate::renderer::render_plan::debug::{validate_plan_scissors, validate_plan_target_lifetimes};

#[test]
fn debug_validate_rejects_load_before_init() {
    let init_src = RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
        segment: SceneSegmentId(0),
        target: PlanTarget::Intermediate1,
        target_origin: (0, 0),
        target_size: (64, 64),
        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        draw_range: 0..0,
    });
    let composite = RenderPlanPass::CompositePremul(CompositePremulPass {
        src: PlanTarget::Intermediate1,
        src_origin: (0, 0),
        dst: PlanTarget::Intermediate0,
        src_size: (64, 64),
        dst_origin: (0, 0),
        dst_size: (64, 64),
        dst_scissor: None,
        mask_uniform_index: None,
        mask: None,
        blend_mode: fret_core::BlendMode::Over,
        opacity: 1.0,
        load: wgpu::LoadOp::Load,
    });

    let err = validate_plan_target_lifetimes(&[init_src, composite]).unwrap_err();
    assert!(err.contains("writes Intermediate0"), "{err}");
    assert!(err.contains("LoadOp::Load"), "{err}");
}

#[test]
fn debug_validate_rejects_path_msaa_batch_before_init() {
    let pass = RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass {
        segment: SceneSegmentId(0),
        target: PlanTarget::Intermediate0,
        target_origin: (0, 0),
        target_size: (64, 64),
        draw_range: 0..1,
        union_scissor: AbsoluteScissorRect(ScissorRect {
            x: 0,
            y: 0,
            w: 1,
            h: 1,
        }),
        batch_uniform_index: 0,
        load: wgpu::LoadOp::Load,
    });

    let err = validate_plan_target_lifetimes(&[pass]).unwrap_err();
    assert!(err.contains("writes Intermediate0"), "{err}");
    assert!(err.contains("LoadOp::Load"), "{err}");
}

#[test]
fn debug_validate_rejects_absolute_scissor_without_intersection() {
    let pass = RenderPlanPass::PathClipMask(PathClipMaskPass {
        dst: PlanTarget::Mask0,
        dst_origin: (10, 10),
        dst_size: (16, 16),
        scissor: AbsoluteScissorRect(ScissorRect {
            x: 0,
            y: 0,
            w: 5,
            h: 5,
        }),
        uniform_index: 0,
        first_vertex: 0,
        vertex_count: 3,
        cache_key: 0,
        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
    });

    let err = validate_plan_scissors(&[pass]).unwrap_err();
    assert!(err.contains("does not intersect"), "{err}");
}

#[test]
fn debug_validate_rejects_mask_viewport_rect_out_of_bounds() {
    let pass = RenderPlanPass::Blur(BlurPass {
        src: PlanTarget::Intermediate0,
        dst: PlanTarget::Intermediate1,
        src_size: (10, 10),
        dst_size: (10, 10),
        dst_scissor: None,
        mask_uniform_index: Some(0),
        mask: Some(MaskRef {
            target: PlanTarget::Mask0,
            size: (2, 2),
            viewport_rect: ScissorRect {
                x: 9,
                y: 0,
                w: 2,
                h: 1,
            },
        }),
        axis: BlurAxis::Horizontal,
        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
    });

    let err = validate_plan_scissors(&[pass]).unwrap_err();
    assert!(err.contains("mask viewport_rect"), "{err}");
}

#[test]
fn debug_validate_rejects_mask_size_mismatch() {
    let pass = RenderPlanPass::ColorAdjust(ColorAdjustPass {
        src: PlanTarget::Intermediate0,
        dst: PlanTarget::Intermediate1,
        src_size: (10, 10),
        dst_size: (10, 10),
        dst_scissor: None,
        mask_uniform_index: Some(0),
        mask: Some(MaskRef {
            target: PlanTarget::Mask1,
            // Expected for 3x5 viewport rect at Mask1 is downsampled_size((3,5),2) == (2,3).
            size: (1, 1),
            viewport_rect: ScissorRect {
                x: 1,
                y: 2,
                w: 3,
                h: 5,
            },
        }),
        saturation: 1.0,
        brightness: 1.0,
        contrast: 1.0,
        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
    });

    let err = validate_plan_scissors(&[pass]).unwrap_err();
    assert!(err.contains("mask size mismatch"), "{err}");
}

#[test]
fn debug_validate_rejects_origin_size_overflow() {
    let pass = RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
        segment: SceneSegmentId(0),
        target: PlanTarget::Intermediate0,
        target_origin: (u32::MAX, 0),
        target_size: (1, 1),
        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        draw_range: 0..0,
    });

    let err = validate_plan_scissors(&[pass]).unwrap_err();
    assert!(err.contains("overflows"), "{err}");
}

#[test]
fn debug_validate_rejects_clip_mask_load_op_load() {
    let pass = RenderPlanPass::ClipMask(ClipMaskPass {
        dst: PlanTarget::Mask0,
        dst_size: (10, 10),
        dst_scissor: None,
        uniform_index: 0,
        load: wgpu::LoadOp::Load,
    });

    let err = validate_plan_scissors(&[pass]).unwrap_err();
    assert!(err.contains("ClipMask must clear"), "{err}");
}
