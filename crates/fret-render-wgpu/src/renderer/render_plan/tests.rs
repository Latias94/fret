#![allow(clippy::field_reassign_with_default)]

use super::super::intermediate_pool::estimate_texture_bytes;
use super::super::render_plan_effects as effects;
use super::super::{
    ClipPathMaskDraw, EffectMarker, EffectMarkerKind, OrderedDraw, PathDraw, QuadDraw,
    QuadPipelineKey,
};
use super::*;

fn strip_releases(passes: &[RenderPlanPass]) -> Vec<&RenderPlanPass> {
    passes
        .iter()
        .filter(|p| !matches!(p, RenderPlanPass::ReleaseTarget(_)))
        .collect()
}

fn apply_single_step_effect_with_scissor(
    step: fret_core::EffectStep,
    mode: fret_core::EffectMode,
    scissor: ScissorRect,
) -> Vec<RenderPlanPass> {
    let mut passes: Vec<RenderPlanPass> = Vec::new();
    effects::apply_chain_in_place(
        &mut passes,
        &[],
        PlanTarget::Intermediate0,
        mode,
        fret_core::EffectChain::from_steps(&[step]),
        fret_core::EffectQuality::Auto,
        scissor,
        None,
        &[],
        effects::EffectCompileCtx {
            viewport_size: (100, 100),
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            intermediate_budget_bytes: u64::MAX,
            clear: wgpu::Color::TRANSPARENT,
            scale_factor: 1.0,
        },
    );
    passes
}

fn first_output_write(passes: &[RenderPlanPass]) -> Option<&RenderPlanPass> {
    passes.iter().find(|p| match p {
        RenderPlanPass::SceneDrawRange(p) => p.target == PlanTarget::Output,
        RenderPlanPass::PathMsaaBatch(p) => p.target == PlanTarget::Output,
        RenderPlanPass::PathClipMask(_) => false,
        RenderPlanPass::FullscreenBlit(p) => p.dst == PlanTarget::Output,
        RenderPlanPass::CompositePremul(p) => p.dst == PlanTarget::Output,
        RenderPlanPass::ScaleNearest(p) => p.dst == PlanTarget::Output,
        RenderPlanPass::Blur(p) => p.dst == PlanTarget::Output,
        RenderPlanPass::BackdropWarp(p) => p.dst == PlanTarget::Output,
        RenderPlanPass::ColorAdjust(p) => p.dst == PlanTarget::Output,
        RenderPlanPass::ColorMatrix(p) => p.dst == PlanTarget::Output,
        RenderPlanPass::AlphaThreshold(p) => p.dst == PlanTarget::Output,
        RenderPlanPass::DropShadow(p) => p.dst == PlanTarget::Output,
        RenderPlanPass::ClipMask(_) => false,
        RenderPlanPass::ReleaseTarget(_) => false,
    })
}

fn assert_first_output_write_is_clear(passes: &[RenderPlanPass]) {
    let pass = first_output_write(passes).expect("expected at least one Output write");
    match pass {
        RenderPlanPass::SceneDrawRange(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::PathMsaaBatch(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::FullscreenBlit(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::CompositePremul(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::ScaleNearest(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::Blur(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::BackdropWarp(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::ColorAdjust(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::ColorMatrix(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::AlphaThreshold(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::DropShadow(p) => {
            assert!(matches!(p.load, wgpu::LoadOp::Clear(_)));
        }
        RenderPlanPass::PathClipMask(_)
        | RenderPlanPass::ClipMask(_)
        | RenderPlanPass::ReleaseTarget(_) => {
            unreachable!("first_output_write filtered these out")
        }
    }
}

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
fn compile_for_scene_path_msaa_batch_initializes_output_via_empty_clear_pass() {
    let mut encoding = SceneEncoding::default();
    encoding.ordered_draws.push(OrderedDraw::Path(PathDraw {
        scissor: ScissorRect {
            x: 1,
            y: 2,
            w: 3,
            h: 4,
        },
        uniform_index: 0,
        first_vertex: 0,
        vertex_count: 0,
        paint_index: 0,
    }));

    let viewport_size = (64, 64);
    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        viewport_size,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        4,
        DebugPostprocess::None,
        u64::MAX,
    );

    let core = strip_releases(&plan.passes);
    assert_eq!(core.len(), 2);

    let RenderPlanPass::SceneDrawRange(scene) = core[0] else {
        panic!("expected SceneDrawRange init pass");
    };
    assert_eq!(scene.target, PlanTarget::Output);
    assert!(matches!(scene.load, wgpu::LoadOp::Clear(_)));
    assert_eq!(scene.draw_range, 0..0);

    let RenderPlanPass::PathMsaaBatch(batch) = core[1] else {
        panic!("expected PathMsaaBatch pass");
    };
    assert_eq!(batch.target, PlanTarget::Output);
    assert!(matches!(batch.load, wgpu::LoadOp::Load));
    assert_eq!(batch.draw_range, 0..1);
    assert_eq!(
        batch.union_scissor.0,
        ScissorRect {
            x: 1,
            y: 2,
            w: 3,
            h: 4,
        }
    );

    assert_first_output_write_is_clear(&plan.passes);
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

#[test]
fn insert_early_releases_inserts_release_after_last_use() {
    let mut passes = vec![
        RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
            segment: SceneSegmentId(0),
            target: PlanTarget::Intermediate0,
            target_origin: (0, 0),
            target_size: (64, 64),
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            draw_range: 0..0,
        }),
        RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: PlanTarget::Intermediate0,
            dst: PlanTarget::Output,
            src_size: (64, 64),
            dst_size: (64, 64),
            dst_scissor: None,
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        }),
    ];

    let inserted = insert_early_releases(&mut passes);
    assert_eq!(inserted, 1);

    let last_use = passes
        .iter()
        .rposition(|p| match p {
            RenderPlanPass::SceneDrawRange(p) => p.target == PlanTarget::Intermediate0,
            RenderPlanPass::FullscreenBlit(p) => p.src == PlanTarget::Intermediate0,
            _ => false,
        })
        .unwrap();
    let release_at = passes
        .iter()
        .position(|p| matches!(p, RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate0)))
        .unwrap();
    assert!(release_at > last_use);
}

#[test]
fn scissored_color_adjust_preserves_outside_region_via_pre_blit() {
    let scissor = ScissorRect {
        x: 10,
        y: 11,
        w: 20,
        h: 21,
    };
    let passes = apply_single_step_effect_with_scissor(
        fret_core::EffectStep::ColorAdjust {
            saturation: 1.0,
            brightness: 1.0,
            contrast: 1.0,
        },
        fret_core::EffectMode::FilterContent,
        scissor,
    );

    assert_eq!(passes.len(), 2);
    let RenderPlanPass::FullscreenBlit(pre) = passes[0] else {
        panic!("expected pre-blit");
    };
    assert_eq!(pre.src, PlanTarget::Intermediate0);
    assert_eq!(pre.dst, PlanTarget::Intermediate1);
    assert_eq!(pre.dst_scissor, None);
    assert!(matches!(pre.load, wgpu::LoadOp::Clear(_)));

    let RenderPlanPass::ColorAdjust(pass) = passes[1] else {
        panic!("expected ColorAdjust pass");
    };
    assert_eq!(pass.src, PlanTarget::Intermediate1);
    assert_eq!(pass.dst, PlanTarget::Intermediate0);
    assert_eq!(pass.dst_scissor.map(|s| s.0), Some(scissor));
    assert!(matches!(pass.load, wgpu::LoadOp::Load));
}

#[test]
fn scissored_color_matrix_preserves_outside_region_via_pre_blit() {
    let scissor = ScissorRect {
        x: 2,
        y: 3,
        w: 70,
        h: 31,
    };
    let passes = apply_single_step_effect_with_scissor(
        fret_core::EffectStep::ColorMatrix { m: [0.0; 20] },
        fret_core::EffectMode::FilterContent,
        scissor,
    );

    assert_eq!(passes.len(), 2);
    assert!(matches!(passes[0], RenderPlanPass::FullscreenBlit(_)));
    let RenderPlanPass::ColorMatrix(pass) = passes[1] else {
        panic!("expected ColorMatrix pass");
    };
    assert_eq!(pass.dst_scissor.map(|s| s.0), Some(scissor));
    assert!(matches!(pass.load, wgpu::LoadOp::Load));
}

#[test]
fn scissored_alpha_threshold_preserves_outside_region_via_pre_blit() {
    let scissor = ScissorRect {
        x: 8,
        y: 9,
        w: 10,
        h: 11,
    };
    let passes = apply_single_step_effect_with_scissor(
        fret_core::EffectStep::AlphaThreshold {
            cutoff: 0.5,
            soft: 0.25,
        },
        fret_core::EffectMode::FilterContent,
        scissor,
    );

    assert_eq!(passes.len(), 2);
    assert!(matches!(passes[0], RenderPlanPass::FullscreenBlit(_)));
    let RenderPlanPass::AlphaThreshold(pass) = passes[1] else {
        panic!("expected AlphaThreshold pass");
    };
    assert_eq!(pass.dst_scissor.map(|s| s.0), Some(scissor));
    assert!(matches!(pass.load, wgpu::LoadOp::Load));
}

#[test]
fn scissored_backdrop_warp_preserves_outside_region_via_pre_blit() {
    let scissor = ScissorRect {
        x: 1,
        y: 1,
        w: 33,
        h: 44,
    };
    let warp = fret_core::scene::BackdropWarpV1 {
        strength_px: fret_core::Px(10.0),
        scale_px: fret_core::Px(12.0),
        phase: 0.0,
        chromatic_aberration_px: fret_core::Px(0.0),
        kind: fret_core::scene::BackdropWarpKindV1::Wave,
    };
    let passes = apply_single_step_effect_with_scissor(
        fret_core::EffectStep::BackdropWarpV1(warp),
        fret_core::EffectMode::Backdrop,
        scissor,
    );

    assert_eq!(passes.len(), 2);
    assert!(matches!(passes[0], RenderPlanPass::FullscreenBlit(_)));
    let RenderPlanPass::BackdropWarp(pass) = passes[1] else {
        panic!("expected BackdropWarp pass");
    };
    assert_eq!(pass.dst_scissor.map(|s| s.0), Some(scissor));
    assert!(matches!(pass.load, wgpu::LoadOp::Load));
}

#[test]
fn scissored_drop_shadow_restores_original_outside_region() {
    let scissor = ScissorRect {
        x: 6,
        y: 7,
        w: 80,
        h: 10,
    };
    let shadow = fret_core::scene::DropShadowV1 {
        offset_px: fret_core::Point::new(fret_core::Px(2.0), fret_core::Px(3.0)),
        blur_radius_px: fret_core::Px(8.0),
        downsample: 1,
        color: fret_core::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    };

    let passes = apply_single_step_effect_with_scissor(
        fret_core::EffectStep::DropShadowV1(shadow),
        fret_core::EffectMode::FilterContent,
        scissor,
    );

    let pre_blit = passes
        .iter()
        .find_map(|p| match p {
            RenderPlanPass::FullscreenBlit(p) if p.dst == PlanTarget::Intermediate1 => Some(*p),
            _ => None,
        })
        .expect("expected preserve-original pre-blit to Intermediate1");
    assert_eq!(pre_blit.src, PlanTarget::Intermediate0);
    assert!(matches!(pre_blit.load, wgpu::LoadOp::Clear(_)));

    let restore = passes
        .iter()
        .find_map(|p| match p {
            RenderPlanPass::FullscreenBlit(p) if p.src == PlanTarget::Intermediate1 => Some(*p),
            _ => None,
        })
        .expect("expected restore blit from Intermediate1 to srcdst");
    assert_eq!(restore.dst, PlanTarget::Intermediate0);
    assert!(matches!(restore.load, wgpu::LoadOp::Clear(_)));

    let RenderPlanPass::DropShadow(pass) = passes
        .last()
        .expect("expected at least one pass for drop shadow")
    else {
        panic!("expected final DropShadow pass");
    };
    assert_eq!(pass.dst, PlanTarget::Intermediate0);
    assert_eq!(pass.dst_scissor.map(|s| s.0), Some(scissor));
    assert!(matches!(pass.load, wgpu::LoadOp::Load));
}

#[test]
fn compile_for_scene_none_targets_output() {
    let encoding = SceneEncoding::default();
    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        (100, 100),
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        1,
        DebugPostprocess::None,
        u64::MAX,
    );

    assert_eq!(plan.passes.len(), 1);
    let RenderPlanPass::SceneDrawRange(pass) = &plan.passes[0] else {
        panic!("expected SceneDrawRange pass");
    };
    assert_eq!(pass.target, PlanTarget::Output);
    assert!(matches!(pass.load, wgpu::LoadOp::Clear(_)));
    assert_first_output_write_is_clear(&plan.passes);
}

#[test]
fn compile_for_scene_offscreen_blit_adds_fullscreen_blit() {
    let encoding = SceneEncoding::default();
    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        (100, 100),
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        1,
        DebugPostprocess::OffscreenBlit,
        u64::MAX,
    );

    let core = strip_releases(&plan.passes);
    assert_eq!(core.len(), 2);
    let RenderPlanPass::SceneDrawRange(scene) = core[0] else {
        panic!("expected SceneDrawRange pass");
    };
    assert_eq!(scene.target, PlanTarget::Intermediate0);
    let RenderPlanPass::FullscreenBlit(blit) = core[1] else {
        panic!("expected FullscreenBlit pass");
    };
    assert_eq!(blit.src, PlanTarget::Intermediate0);
    assert_eq!(blit.dst, PlanTarget::Output);
    assert_eq!(blit.src_size, (100, 100));
    assert_eq!(blit.dst_size, (100, 100));
    assert_eq!(blit.dst_scissor, None);

    assert!(
        plan.passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate0))),
        "expected ReleaseTarget(Intermediate0)"
    );

    assert_first_output_write_is_clear(&plan.passes);
}

#[test]
fn compile_for_scene_pixelate_adds_scale_chain_then_blit() {
    let encoding = SceneEncoding::default();
    let viewport_size = (128, 64);
    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        viewport_size,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        1,
        DebugPostprocess::Pixelate { scale: 4 },
        u64::MAX,
    );

    let core = strip_releases(&plan.passes);
    assert_eq!(core.len(), 6);

    let RenderPlanPass::SceneDrawRange(scene) = core[0] else {
        panic!("expected SceneDrawRange pass");
    };
    assert_eq!(scene.target, PlanTarget::Intermediate0);

    let RenderPlanPass::ScaleNearest(down0) = core[1] else {
        panic!("expected ScaleNearest downsample pass 0");
    };
    assert_eq!(down0.src, PlanTarget::Intermediate0);
    assert_eq!(down0.dst, PlanTarget::Intermediate2);
    assert_eq!(down0.mode, ScaleMode::Downsample);
    assert_eq!(down0.scale, 2);
    assert_eq!(down0.src_size, viewport_size);
    assert_eq!(down0.dst_size, downsampled_size(viewport_size, 2));
    assert_eq!(down0.dst_scissor, None);

    let release0 = plan
        .passes
        .iter()
        .position(|p| matches!(p, RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate0)))
        .expect("expected ReleaseTarget(Intermediate0)");
    let down0_idx = plan
        .passes
        .iter()
        .position(|p| {
            matches!(
                p,
                RenderPlanPass::ScaleNearest(p)
                    if p.mode == ScaleMode::Downsample
                        && p.src == PlanTarget::Intermediate0
                        && p.dst == PlanTarget::Intermediate2
            )
        })
        .unwrap();
    assert!(release0 > down0_idx);

    let RenderPlanPass::ScaleNearest(down1) = core[2] else {
        panic!("expected ScaleNearest downsample pass 1");
    };
    assert_eq!(down1.src, PlanTarget::Intermediate2);
    assert_eq!(down1.dst, PlanTarget::Intermediate1);
    assert_eq!(down1.mode, ScaleMode::Downsample);
    assert_eq!(down1.scale, 2);
    assert_eq!(down1.src_size, down0.dst_size);
    assert_eq!(down1.dst_size, downsampled_size(down0.dst_size, 2));
    assert_eq!(down1.dst_scissor, None);

    let RenderPlanPass::ScaleNearest(up0) = core[3] else {
        panic!("expected ScaleNearest upscale pass 0");
    };
    assert_eq!(up0.src, PlanTarget::Intermediate1);
    assert_eq!(up0.dst, PlanTarget::Intermediate2);
    assert_eq!(up0.mode, ScaleMode::Upscale);
    assert_eq!(up0.scale, 2);
    assert_eq!(up0.src_size, down1.dst_size);
    assert_eq!(up0.dst_size, down1.src_size);
    assert_eq!(up0.dst_scissor, None);

    let RenderPlanPass::ScaleNearest(up1) = core[4] else {
        panic!("expected ScaleNearest upscale pass 1");
    };
    assert_eq!(up1.src, PlanTarget::Intermediate2);
    assert_eq!(up1.dst, PlanTarget::Intermediate1);
    assert_eq!(up1.mode, ScaleMode::Upscale);
    assert_eq!(up1.scale, 2);
    assert_eq!(up1.src_size, up0.dst_size);
    assert_eq!(up1.dst_size, viewport_size);
    assert_eq!(up1.dst_scissor, None);

    let RenderPlanPass::FullscreenBlit(blit) = core[5] else {
        panic!("expected FullscreenBlit pass");
    };
    assert_eq!(blit.src, PlanTarget::Intermediate1);
    assert_eq!(blit.dst, PlanTarget::Output);
    assert_eq!(blit.src_size, viewport_size);
    assert_eq!(blit.dst_size, viewport_size);
    assert_eq!(blit.dst_scissor, None);
    let releases: Vec<PlanTarget> = plan
        .passes
        .iter()
        .filter_map(|p| match p {
            RenderPlanPass::ReleaseTarget(t) => Some(*t),
            _ => None,
        })
        .collect();
    assert!(releases.contains(&PlanTarget::Intermediate0));
    assert!(releases.contains(&PlanTarget::Intermediate1));
    assert!(releases.contains(&PlanTarget::Intermediate2));

    assert_first_output_write_is_clear(&plan.passes);
}

#[test]
fn compile_for_scene_backdrop_color_adjust_emits_mask_target_when_budget_allows() {
    let viewport_size = (100, 100);
    let scissor = ScissorRect::full(viewport_size.0, viewport_size.1);

    let mut encoding = SceneEncoding::default();
    encoding.effect_markers = vec![
        EffectMarker {
            draw_ix: 0,
            kind: EffectMarkerKind::Push {
                scissor,
                uniform_index: 0,
                mode: fret_core::EffectMode::Backdrop,
                chain: fret_core::EffectChain::from_steps(&[fret_core::EffectStep::ColorAdjust {
                    saturation: 1.0,
                    brightness: 1.0,
                    contrast: 1.0,
                }]),
                quality: fret_core::EffectQuality::Auto,
            },
        },
        EffectMarker {
            draw_ix: 0,
            kind: EffectMarkerKind::Pop,
        },
    ];

    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        viewport_size,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        1,
        DebugPostprocess::OffscreenBlit,
        u64::MAX,
    );

    let count = plan
        .passes
        .iter()
        .filter(|p| matches!(p, RenderPlanPass::ClipMask(_)))
        .count();
    assert_eq!(count, 1);

    assert_first_output_write_is_clear(&plan.passes);
}

#[test]
fn compile_for_scene_backdrop_blur_caps_clip_mask_tier_when_forced_to_quarter() {
    let viewport_size = (256, 256);
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let clear = wgpu::Color::TRANSPARENT;
    let scissor = ScissorRect::full(viewport_size.0, viewport_size.1);

    let mut encoding = SceneEncoding::default();
    encoding.effect_markers = vec![
        EffectMarker {
            draw_ix: 0,
            kind: EffectMarkerKind::Push {
                scissor,
                uniform_index: 0,
                mode: fret_core::EffectMode::Backdrop,
                chain: fret_core::EffectChain::from_steps(&[fret_core::EffectStep::GaussianBlur {
                    radius_px: fret_core::Px(8.0),
                    downsample: 2,
                }]),
                quality: fret_core::EffectQuality::Auto,
            },
        },
        EffectMarker {
            draw_ix: 0,
            kind: EffectMarkerKind::Pop,
        },
    ];

    let full = estimate_texture_bytes(viewport_size, format, 1);
    let half = estimate_texture_bytes(downsampled_size(viewport_size, 2), format, 1);
    let quarter = estimate_texture_bytes(downsampled_size(viewport_size, 4), format, 1);
    let required_quarter = full.saturating_add(quarter.saturating_mul(2));
    let required_half = full.saturating_add(half.saturating_mul(2));
    assert!(required_quarter <= required_half);

    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        viewport_size,
        format,
        clear,
        1,
        DebugPostprocess::OffscreenBlit,
        required_quarter,
    );

    let clip_mask_tiers: Vec<PlanTarget> = plan
        .passes
        .iter()
        .filter_map(|p| match p {
            RenderPlanPass::ClipMask(p) => Some(p.dst),
            _ => None,
        })
        .collect();
    assert!(!clip_mask_tiers.is_empty());
    assert!(clip_mask_tiers.iter().all(|t| *t == PlanTarget::Mask2));

    assert_first_output_write_is_clear(&plan.passes);
}

#[test]
fn compile_for_scene_filter_content_composite_does_not_allocate_clip_mask() {
    let viewport_size = (100, 100);
    let scissor = ScissorRect::full(viewport_size.0, viewport_size.1);

    let mut encoding = SceneEncoding::default();
    encoding.effect_markers = vec![
        EffectMarker {
            draw_ix: 0,
            kind: EffectMarkerKind::Push {
                scissor,
                uniform_index: 0,
                mode: fret_core::EffectMode::FilterContent,
                chain: fret_core::EffectChain::from_steps(&[fret_core::EffectStep::ColorAdjust {
                    saturation: 1.0,
                    brightness: 1.0,
                    contrast: 1.0,
                }]),
                quality: fret_core::EffectQuality::Auto,
            },
        },
        EffectMarker {
            draw_ix: 0,
            kind: EffectMarkerKind::Pop,
        },
    ];

    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        viewport_size,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        1,
        DebugPostprocess::None,
        u64::MAX,
    );

    let count = plan
        .passes
        .iter()
        .filter(|p| matches!(p, RenderPlanPass::ClipMask(_)))
        .count();
    assert_eq!(count, 0);

    assert_first_output_write_is_clear(&plan.passes);
}

#[test]
fn compile_for_scene_composite_group_preserves_output_clear_guardrail() {
    let viewport_size = (100, 100);
    let scissor = ScissorRect::full(viewport_size.0, viewport_size.1);

    let mut encoding = SceneEncoding::default();
    encoding.effect_markers = vec![
        EffectMarker {
            draw_ix: 0,
            kind: EffectMarkerKind::CompositeGroupPush {
                scissor,
                uniform_index: 0,
                mode: fret_core::BlendMode::Add,
                quality: fret_core::EffectQuality::Auto,
                opacity: 1.0,
            },
        },
        EffectMarker {
            draw_ix: 1,
            kind: EffectMarkerKind::CompositeGroupPop,
        },
    ];
    encoding.ordered_draws.push(OrderedDraw::Quad(QuadDraw {
        scissor,
        uniform_index: 0,
        first_instance: 0,
        instance_count: 0,
        pipeline: QuadPipelineKey {
            fill_kind: 0,
            border_kind: 0,
            border_present: false,
            dash_enabled: false,
            fill_material_sampled: false,
            border_material_sampled: false,
        },
    }));

    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        viewport_size,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        1,
        DebugPostprocess::None,
        u64::MAX,
    );

    assert!(
        plan.passes.iter().any(|p| matches!(
            p,
            RenderPlanPass::CompositePremul(p)
                if p.blend_mode == fret_core::BlendMode::Add
        )),
        "expected at least one CompositePremul(Add) pass"
    );
    assert!(plan.degradations.is_empty(), "unexpected degradations");
    assert_first_output_write_is_clear(&plan.passes);
}

#[test]
fn compile_for_scene_clip_path_preserves_output_clear_guardrail() {
    let viewport_size = (64, 64);
    let scissor = ScissorRect::full(viewport_size.0, viewport_size.1);

    let mut encoding = SceneEncoding::default();
    encoding.effect_markers = vec![
        EffectMarker {
            draw_ix: 0,
            kind: EffectMarkerKind::ClipPathPush {
                scissor,
                uniform_index: 0,
                mask_draw_index: 0,
            },
        },
        EffectMarker {
            draw_ix: 1,
            kind: EffectMarkerKind::ClipPathPop,
        },
    ];
    encoding.clip_path_masks.push(ClipPathMaskDraw {
        scissor,
        uniform_index: 0,
        first_vertex: 0,
        vertex_count: 0,
        cache_key: 123,
    });
    encoding.ordered_draws.push(OrderedDraw::Quad(QuadDraw {
        scissor,
        uniform_index: 0,
        first_instance: 0,
        instance_count: 0,
        pipeline: QuadPipelineKey {
            fill_kind: 0,
            border_kind: 0,
            border_present: false,
            dash_enabled: false,
            fill_material_sampled: false,
            border_material_sampled: false,
        },
    }));

    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        viewport_size,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        1,
        DebugPostprocess::None,
        u64::MAX,
    );

    assert!(
        plan.passes
            .iter()
            .any(|p| matches!(p, RenderPlanPass::PathClipMask(_))),
        "expected at least one PathClipMask pass"
    );
    assert!(
        plan.passes.iter().any(|p| matches!(
            p,
            RenderPlanPass::CompositePremul(p) if p.mask.is_some()
        )),
        "expected at least one CompositePremul pass with a mask"
    );
    assert_first_output_write_is_clear(&plan.passes);
}

#[test]
fn compile_for_scene_blur_emits_separable_passes() {
    let encoding = SceneEncoding::default();
    let viewport_size = (128, 64);
    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        viewport_size,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        1,
        DebugPostprocess::Blur {
            radius: 2,
            downsample_scale: 2,
            scissor: None,
        },
        u64::MAX,
    );

    let core = strip_releases(&plan.passes);
    assert_eq!(core.len(), 6);

    let RenderPlanPass::SceneDrawRange(scene) = core[0] else {
        panic!("expected SceneDrawRange pass");
    };
    assert_eq!(scene.target, PlanTarget::Intermediate0);

    let RenderPlanPass::ScaleNearest(down) = core[1] else {
        panic!("expected downsample pass");
    };
    assert_eq!(down.mode, ScaleMode::Downsample);
    assert_eq!(down.src, PlanTarget::Intermediate0);
    assert_eq!(down.dst, PlanTarget::Intermediate2);
    assert_eq!(down.src_size, viewport_size);
    assert_eq!(down.dst_size, (64, 32));

    let release0 = plan
        .passes
        .iter()
        .position(|p| matches!(p, RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate0)))
        .expect("expected ReleaseTarget(Intermediate0)");
    let down0_idx = plan
        .passes
        .iter()
        .position(
            |p| matches!(p, RenderPlanPass::ScaleNearest(p) if p.mode == ScaleMode::Downsample),
        )
        .unwrap();
    assert!(release0 > down0_idx);

    let RenderPlanPass::Blur(blur_h) = core[2] else {
        panic!("expected blur-h pass");
    };
    assert_eq!(blur_h.axis, BlurAxis::Horizontal);
    assert_eq!(blur_h.src, PlanTarget::Intermediate2);
    assert_eq!(blur_h.dst, PlanTarget::Intermediate1);
    assert_eq!(blur_h.src_size, (64, 32));
    assert_eq!(blur_h.dst_size, (64, 32));
    assert_eq!(blur_h.dst_scissor, None);

    let RenderPlanPass::Blur(blur_v) = core[3] else {
        panic!("expected blur-v pass");
    };
    assert_eq!(blur_v.axis, BlurAxis::Vertical);
    assert_eq!(blur_v.src, PlanTarget::Intermediate1);
    assert_eq!(blur_v.dst, PlanTarget::Intermediate2);
    assert_eq!(blur_v.src_size, (64, 32));
    assert_eq!(blur_v.dst_size, (64, 32));
    assert_eq!(blur_v.dst_scissor, None);

    let RenderPlanPass::ScaleNearest(upscale) = core[4] else {
        panic!("expected upscale pass");
    };
    assert_eq!(upscale.src, PlanTarget::Intermediate2);
    assert_eq!(upscale.dst, PlanTarget::Intermediate0);
    assert_eq!(upscale.src_size, (64, 32));
    assert_eq!(upscale.dst_size, viewport_size);
    assert_eq!(upscale.mode, ScaleMode::Upscale);
    assert_eq!(upscale.scale, 2);
    assert_eq!(upscale.dst_scissor, None);

    let RenderPlanPass::FullscreenBlit(blit) = core[5] else {
        panic!("expected blit pass");
    };
    assert_eq!(blit.src, PlanTarget::Intermediate0);
    assert_eq!(blit.dst, PlanTarget::Output);
    assert_eq!(blit.src_size, viewport_size);
    assert_eq!(blit.dst_size, viewport_size);
    assert_eq!(blit.dst_scissor, None);

    let releases: Vec<PlanTarget> = plan
        .passes
        .iter()
        .filter_map(|p| match p {
            RenderPlanPass::ReleaseTarget(t) => Some(*t),
            _ => None,
        })
        .collect();
    assert!(releases.contains(&PlanTarget::Intermediate0));
    assert!(releases.contains(&PlanTarget::Intermediate1));
    assert!(releases.contains(&PlanTarget::Intermediate2));

    assert_first_output_write_is_clear(&plan.passes);
}

#[test]
fn downsample_half_quarter_helper_emits_two_passes() {
    let viewport_size = (100, 100);
    let mut plan = RenderPlan {
        segments: Vec::new(),
        passes: Vec::new(),
        compile_stats: RenderPlanCompileStats::default(),
        degradations: Vec::new(),
    };
    let out = append_downsample_half_quarter(
        &mut plan,
        PlanTarget::Intermediate0,
        viewport_size,
        PlanTarget::Intermediate2,
        PlanTarget::Intermediate1,
        None,
        viewport_size,
        wgpu::Color::TRANSPARENT,
    );
    assert_eq!(out.stack.len(), 2);
    assert_eq!(plan.passes.len(), 2);
}

#[test]
fn downsample_nearest_scissor_mapping_matches_integer_division_for_non_divisible_viewport() {
    let full_size = (1654, 827);
    let scale = 8;
    let scissor = ScissorRect {
        x: 567,
        y: 24,
        w: 500,
        h: 700,
    };

    let down_size = downsampled_size(full_size, scale);
    assert_eq!(down_size, (207, 104));
    assert_eq!(
        effects::map_scissor_downsample_nearest(Some(scissor), scale, down_size),
        Some(ScissorRect {
            x: 70,
            y: 3,
            w: 64,
            h: 88
        })
    );
}

#[test]
fn downsample_scissor_mapping_does_not_expand_across_steps() {
    let full_size = (3, 5);
    let scissor_in_full = Some(ScissorRect {
        x: 1,
        y: 3,
        w: 1,
        h: 2,
    });

    let mut plan = RenderPlan {
        segments: Vec::new(),
        passes: Vec::new(),
        compile_stats: RenderPlanCompileStats::default(),
        degradations: Vec::new(),
    };

    let out = append_downsample_half_quarter(
        &mut plan,
        PlanTarget::Intermediate0,
        full_size,
        PlanTarget::Intermediate2,
        PlanTarget::Intermediate1,
        scissor_in_full,
        full_size,
        wgpu::Color::TRANSPARENT,
    );

    let expected_half_scissor =
        effects::map_scissor_to_size(scissor_in_full, full_size, downsampled_size(full_size, 2));
    let expected_quarter_scissor =
        effects::map_scissor_to_size(scissor_in_full, full_size, out.quarter_size);
    let chained_quarter_scissor = effects::map_scissor_to_size(
        expected_half_scissor,
        downsampled_size(full_size, 2),
        out.quarter_size,
    );
    assert_ne!(expected_quarter_scissor, chained_quarter_scissor);

    let RenderPlanPass::ScaleNearest(half_pass) = plan.passes[0] else {
        panic!("expected half downsample pass");
    };
    assert_eq!(half_pass.dst_scissor.map(|s| s.0), expected_half_scissor);

    let RenderPlanPass::ScaleNearest(quarter_pass) = plan.passes[1] else {
        panic!("expected quarter downsample pass");
    };
    assert_eq!(
        quarter_pass.dst_scissor.map(|s| s.0),
        expected_quarter_scissor
    );
}

#[test]
fn blur_scissor_is_mapped_per_pass_dst_size() {
    let encoding = SceneEncoding::default();
    let viewport_size = (100, 100);
    let plan = RenderPlan::compile_for_scene(
        &encoding,
        1.0,
        viewport_size,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::Color::TRANSPARENT,
        1,
        DebugPostprocess::Blur {
            radius: 2,
            downsample_scale: 2,
            scissor: Some(ScissorRect {
                x: 10,
                y: 10,
                w: 50,
                h: 50,
            }),
        },
        u64::MAX,
    );

    // Half target is (50, 50) for 100x100.
    let half = plan
        .passes
        .iter()
        .find_map(|p| match p {
            RenderPlanPass::ScaleNearest(p) if p.mode == ScaleMode::Downsample => Some(*p),
            _ => None,
        })
        .expect("expected half downsample pass");
    assert_eq!(
        half.dst_scissor.map(|s| s.0),
        Some(ScissorRect {
            x: 5,
            y: 5,
            w: 25,
            h: 25
        })
    );
    let blur_h = plan
        .passes
        .iter()
        .find_map(|p| match p {
            RenderPlanPass::Blur(p) if p.axis == BlurAxis::Horizontal => Some(*p),
            _ => None,
        })
        .expect("expected blur-h pass");
    assert_eq!(
        blur_h.dst_scissor.map(|s| s.0),
        Some(ScissorRect {
            x: 5,
            y: 5,
            w: 25,
            h: 25
        })
    );
    let base_blit = plan
        .passes
        .iter()
        .find_map(|p| match p {
            RenderPlanPass::FullscreenBlit(p)
                if p.src == PlanTarget::Intermediate0 && p.dst == PlanTarget::Output =>
            {
                Some(*p)
            }
            _ => None,
        })
        .expect("expected base blit pass");
    assert_eq!(base_blit.dst_scissor, None);

    let upscale = plan
        .passes
        .iter()
        .find_map(|p| match p {
            RenderPlanPass::ScaleNearest(p)
                if p.mode == ScaleMode::Upscale && p.dst == PlanTarget::Output =>
            {
                Some(*p)
            }
            _ => None,
        })
        .expect("expected upscale-to-output pass");
    assert_eq!(
        upscale.dst_scissor.map(|s| s.0),
        Some(ScissorRect {
            x: 10,
            y: 10,
            w: 50,
            h: 50
        })
    );

    assert_first_output_write_is_clear(&plan.passes);
}
