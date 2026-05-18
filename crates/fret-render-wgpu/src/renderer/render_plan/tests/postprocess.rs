use super::*;
use crate::renderer::render_plan::postprocess::append_downsample_half_quarter;

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
        DebugPostprocess::OffscreenBlit {
            src: PlanTarget::Intermediate0,
        },
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
                if p.mode == ScaleMode::Upscale && p.dst == PlanTarget::Intermediate0 =>
            {
                Some(*p)
            }
            _ => None,
        })
        .expect("expected upscale-to-intermediate pass");
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
