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
        None,
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
        custom
            .is_some_and(|p| { p.common.mask_uniform_index.is_some() || p.common.mask.is_some() }),
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
        None,
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
            p.common.src == PlanTarget::Intermediate1
                && p.common.dst == PlanTarget::Intermediate0
                && p.common.dst_scissor == Some(LocalScissorRect(scissor))
                && (p.common.mask_uniform_index.is_some() || p.common.mask.is_some())
        }),
        "final CustomEffect should read from the work buffer and apply clip coverage once"
    );
}

#[test]
fn custom_v3_pyramid_budget_pressure_degrades_to_one_and_records_counters() {
    let viewport_size = (64, 64);
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let full = estimate_texture_bytes(viewport_size, format, 1);

    let ctx = EffectCompileCtx {
        viewport_size,
        format,
        // Enough for srcdst + a single scratch target, but not enough headroom for a pyramid.
        intermediate_budget_bytes: full.saturating_mul(2),
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
        fret_core::EffectChain::from_steps(&[fret_core::EffectStep::CustomV3 {
            id: fret_core::EffectId::default(),
            params: fret_core::scene::EffectParamsV1 {
                vec4s: [[0.0; 4]; 4],
            },
            max_sample_offset_px: fret_core::Px(0.0),
            user0: None,
            user1: None,
            sources: fret_core::scene::CustomEffectSourcesV3 {
                want_raw: false,
                pyramid: Some(fret_core::scene::CustomEffectPyramidRequestV1 {
                    max_levels: 6,
                    max_radius_px: fret_core::Px(32.0),
                }),
            },
        }]),
        fret_core::EffectQuality::Auto,
        scissor,
        None,
        &[],
        &mut degradations,
        &mut blur_quality,
        ctx,
        None,
    );

    let custom_v3 = passes.iter().find_map(|p| match p {
        RenderPlanPass::CustomEffectV3(p) => Some(*p),
        _ => None,
    });
    assert!(
        custom_v3.is_some_and(|p| p.pyramid_wanted && p.pyramid_levels == 1),
        "budget pressure should deterministically degrade the pyramid to 1 level"
    );

    assert_eq!(degradations.custom_effect_v3_sources.raw_requested, 0);
    assert_eq!(degradations.custom_effect_v3_sources.pyramid_requested, 1);
    assert_eq!(
        degradations
            .custom_effect_v3_sources
            .pyramid_applied_levels_ge2,
        0
    );
    assert_eq!(
        degradations
            .custom_effect_v3_sources
            .pyramid_degraded_to_one_budget_zero,
        0
    );
    assert_eq!(
        degradations
            .custom_effect_v3_sources
            .pyramid_degraded_to_one_budget_insufficient,
        1
    );
}

#[test]
fn custom_v3_sources_plan_records_raw_aliasing_vs_distinct() {
    let ctx = EffectCompileCtx {
        viewport_size: (64, 64),
        format: wgpu::TextureFormat::Rgba8Unorm,
        intermediate_budget_bytes: 1u64 << 60,
        clear: wgpu::Color::TRANSPARENT,
        scale_factor: 1.0,
    };
    let scissor = ScissorRect::full(64, 64);

    let sources = fret_core::scene::CustomEffectSourcesV3 {
        want_raw: true,
        pyramid: None,
    };

    let mut budget_bytes = 1u64 << 60;
    let mut v3 = super::super::CustomEffectV3SourceDegradationCounters::default();
    let plan = plan_custom_v3_sources_and_charge_budget(
        sources,
        PlanTarget::Intermediate0,
        None,
        None,
        None,
        scissor,
        ctx,
        &mut budget_bytes,
        0,
        &mut v3,
    );
    assert_eq!(plan.src_raw, PlanTarget::Intermediate0);
    assert_eq!(v3.raw_requested, 1);
    assert_eq!(v3.raw_aliased_to_src, 1);
    assert_eq!(v3.raw_distinct, 0);

    let mut budget_bytes = 1u64 << 60;
    let mut v3 = super::super::CustomEffectV3SourceDegradationCounters::default();
    let plan = plan_custom_v3_sources_and_charge_budget(
        sources,
        PlanTarget::Intermediate0,
        Some(PlanTarget::Intermediate1),
        None,
        None,
        scissor,
        ctx,
        &mut budget_bytes,
        0,
        &mut v3,
    );
    assert_eq!(plan.src_raw, PlanTarget::Intermediate1);
    assert_eq!(v3.raw_requested, 1);
    assert_eq!(v3.raw_aliased_to_src, 0);
    assert_eq!(v3.raw_distinct, 1);
}

#[test]
fn custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi() {
    let ctx = EffectCompileCtx {
        viewport_size: (64, 64),
        format: wgpu::TextureFormat::Rgba8Unorm,
        intermediate_budget_bytes: 1u64 << 60,
        clear: wgpu::Color::TRANSPARENT,
        scale_factor: 1.0,
    };
    let scissor = ScissorRect {
        x: 0,
        y: 0,
        w: 10,
        h: 10,
    };

    let sources = fret_core::scene::CustomEffectSourcesV3 {
        want_raw: false,
        pyramid: Some(fret_core::scene::CustomEffectPyramidRequestV1 {
            max_levels: 3,
            max_radius_px: fret_core::Px(32.0),
        }),
    };

    let group_choice = CustomV3PyramidChoice {
        levels: 6,
        degraded_to_one: None,
    };
    let group_roi = (
        ScissorRect {
            x: 20,
            y: 20,
            w: 10,
            h: 10,
        },
        8,
    );

    let mut budget_bytes = 123;
    let mut v3 = super::super::CustomEffectV3SourceDegradationCounters::default();
    let plan = plan_custom_v3_sources_and_charge_budget(
        sources,
        PlanTarget::Intermediate0,
        None,
        Some(group_choice),
        Some(group_roi),
        scissor,
        ctx,
        &mut budget_bytes,
        0,
        &mut v3,
    );

    assert_eq!(
        plan.pyramid_levels, 3,
        "group choice must be clamped by max_levels"
    );
    assert_eq!(v3.pyramid_requested, 1);
    assert_eq!(v3.pyramid_applied_levels_ge2, 1);
    assert_eq!(v3.pyramid_degraded_to_one_budget_zero, 0);
    assert_eq!(v3.pyramid_degraded_to_one_budget_insufficient, 0);

    let expected_roi = inflate_scissor_to_viewport(group_roi.0, group_roi.1, ctx.viewport_size);
    assert_eq!(
        plan.pyramid_build_scissor,
        Some(LocalScissorRect(expected_roi))
    );
}

#[test]
fn custom_v3_sources_plan_group_pyramid_degrade_to_one_records_reason() {
    let ctx = EffectCompileCtx {
        viewport_size: (64, 64),
        format: wgpu::TextureFormat::Rgba8Unorm,
        intermediate_budget_bytes: 1u64 << 60,
        clear: wgpu::Color::TRANSPARENT,
        scale_factor: 1.0,
    };
    let scissor = ScissorRect::full(64, 64);

    let sources = fret_core::scene::CustomEffectSourcesV3 {
        want_raw: false,
        pyramid: Some(fret_core::scene::CustomEffectPyramidRequestV1 {
            max_levels: 6,
            max_radius_px: fret_core::Px(32.0),
        }),
    };
    let group_choice = CustomV3PyramidChoice {
        levels: 1,
        degraded_to_one: Some(CustomV3PyramidDegradeReason::BudgetZero),
    };

    let mut budget_bytes = 123;
    let mut v3 = super::super::CustomEffectV3SourceDegradationCounters::default();
    let plan = plan_custom_v3_sources_and_charge_budget(
        sources,
        PlanTarget::Intermediate0,
        None,
        Some(group_choice),
        None,
        scissor,
        ctx,
        &mut budget_bytes,
        0,
        &mut v3,
    );

    assert_eq!(plan.pyramid_levels, 1);
    assert_eq!(plan.pyramid_build_scissor, None);
    assert_eq!(v3.pyramid_requested, 1);
    assert_eq!(v3.pyramid_degraded_to_one_budget_zero, 1);
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
        None,
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
        None,
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
        None,
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
        None,
    );

    assert!(
        passes.iter().any(|p| matches!(p, RenderPlanPass::Noise(_))),
        "noise step should compile to a Noise pass"
    );
}

#[test]
fn backdrop_warp_v2_image_field_compiles_to_backdrop_warp_pass() {
    let ctx = EffectCompileCtx {
        viewport_size: (64, 64),
        format: wgpu::TextureFormat::Rgba8Unorm,
        intermediate_budget_bytes: 1u64 << 60,
        clear: wgpu::Color::TRANSPARENT,
        scale_factor: 2.0,
    };
    let scissor = ScissorRect {
        x: 4,
        y: 6,
        w: 20,
        h: 18,
    };
    let image = fret_core::ImageId::default();
    let uv = fret_core::scene::UvRect {
        u0: 0.1,
        v0: 0.2,
        u1: 0.8,
        v1: 0.9,
    };

    let mut passes = Vec::new();
    let mut degradations = super::super::EffectDegradationSnapshot::default();
    let mut blur_quality = super::super::BlurQualitySnapshot::default();
    apply_chain_in_place(
        &mut passes,
        &[],
        PlanTarget::Intermediate0,
        fret_core::EffectMode::Backdrop,
        fret_core::EffectChain::from_steps(&[fret_core::EffectStep::BackdropWarpV2(
            fret_core::scene::BackdropWarpV2 {
                base: fret_core::scene::BackdropWarpV1 {
                    strength_px: fret_core::Px(3.0),
                    scale_px: fret_core::Px(12.0),
                    phase: 0.5,
                    chromatic_aberration_px: fret_core::Px(1.0),
                    kind: fret_core::scene::BackdropWarpKindV1::Wave,
                },
                field: fret_core::scene::BackdropWarpFieldV2::ImageDisplacementMap {
                    image,
                    uv,
                    sampling: fret_core::scene::ImageSamplingHint::Nearest,
                    encoding: fret_core::scene::WarpMapEncodingV1::NormalRgb,
                },
            },
        )]),
        fret_core::EffectQuality::Medium,
        scissor,
        None,
        &[],
        &mut degradations,
        &mut blur_quality,
        ctx,
        None,
    );

    let pass = passes.iter().find_map(|p| match p {
        RenderPlanPass::BackdropWarp(pass) => Some(pass),
        _ => None,
    });
    assert!(
        pass.is_some_and(|pass| {
            pass.warp_image == Some(image)
                && pass.warp_uv == uv
                && pass.warp_sampling == fret_core::scene::ImageSamplingHint::Nearest
                && pass.warp_encoding == fret_core::scene::WarpMapEncodingV1::NormalRgb
                && pass.origin_px == (scissor.x, scissor.y)
                && pass.bounds_size_px == (scissor.w, scissor.h)
                && pass.dst_scissor == Some(LocalScissorRect(scissor))
        }),
        "BackdropWarpV2 should preserve image-field settings when compiling the pass"
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
        None,
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
        None,
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
        None,
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
        None,
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
        None,
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
