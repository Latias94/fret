use super::*;

use super::super::CustomEffectV3SourceDegradationCounters;
use super::super::render_plan::CustomEffectPassCommon;

fn choose_custom_v3_pyramid_levels_from_group_or_budget(
    sources: fret_core::scene::CustomEffectSourcesV3,
    group_pyramid: Option<CustomV3PyramidChoice>,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: &mut u64,
    base_required_bytes: u64,
    v3: &mut CustomEffectV3SourceDegradationCounters,
) -> u32 {
    if sources.pyramid.is_some()
        && let Some(choice) = group_pyramid
    {
        v3.pyramid_requested = v3.pyramid_requested.saturating_add(1);
        if choice.levels >= 2 {
            v3.pyramid_applied_levels_ge2 = v3.pyramid_applied_levels_ge2.saturating_add(1);
        } else if let Some(reason) = choice.degraded_to_one {
            match reason {
                CustomV3PyramidDegradeReason::BudgetZero => {
                    v3.pyramid_degraded_to_one_budget_zero =
                        v3.pyramid_degraded_to_one_budget_zero.saturating_add(1);
                }
                CustomV3PyramidDegradeReason::BudgetInsufficient => {
                    v3.pyramid_degraded_to_one_budget_insufficient = v3
                        .pyramid_degraded_to_one_budget_insufficient
                        .saturating_add(1);
                }
            }
        }
        let req = sources.pyramid.unwrap();
        return choice.levels.min((req.max_levels as u32).max(1));
    }

    choose_custom_v3_pyramid_levels_and_charge(
        sources,
        size,
        format,
        budget_bytes,
        base_required_bytes,
        v3,
    )
}

fn record_custom_v3_raw_choice(
    sources: fret_core::scene::CustomEffectSourcesV3,
    effective_raw: PlanTarget,
    alias_src: PlanTarget,
    v3: &mut CustomEffectV3SourceDegradationCounters,
) {
    if !sources.want_raw {
        return;
    }

    v3.raw_requested = v3.raw_requested.saturating_add(1);
    if effective_raw == alias_src {
        v3.raw_aliased_to_src = v3.raw_aliased_to_src.saturating_add(1);
    } else {
        v3.raw_distinct = v3.raw_distinct.saturating_add(1);
    }
}

fn custom_v3_pyramid_build_scissor(
    sources: fret_core::scene::CustomEffectSourcesV3,
    pyramid_levels: u32,
    group_pyramid_roi: Option<(ScissorRect, u32)>,
    scissor: ScissorRect,
    viewport_size: (u32, u32),
    scale_factor: f32,
) -> Option<LocalScissorRect> {
    sources.pyramid.filter(|_| pyramid_levels >= 2).map(|req| {
        let roi = if let Some((g_scissor, g_pad_px)) = group_pyramid_roi {
            inflate_scissor_to_viewport(g_scissor, g_pad_px, viewport_size)
        } else {
            let pad_px = pyramid_radius_pad_px(req, scale_factor);
            inflate_scissor_to_viewport(scissor, pad_px, viewport_size)
        };
        LocalScissorRect(roi)
    })
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CustomV3SourcePlan {
    pub(super) src_raw: PlanTarget,
    pub(super) pyramid_levels: u32,
    pub(super) pyramid_build_scissor: Option<LocalScissorRect>,
}

pub(super) fn plan_custom_v3_sources_and_charge_budget(
    sources: fret_core::scene::CustomEffectSourcesV3,
    alias_src: PlanTarget,
    chain_raw: Option<PlanTarget>,
    group_pyramid: Option<CustomV3PyramidChoice>,
    group_pyramid_roi: Option<(ScissorRect, u32)>,
    scissor: ScissorRect,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    base_required_bytes: u64,
    v3: &mut CustomEffectV3SourceDegradationCounters,
) -> CustomV3SourcePlan {
    let raw_needed = sources.want_raw || sources.pyramid.is_some();
    let src_raw = if raw_needed {
        chain_raw.unwrap_or(alias_src)
    } else {
        alias_src
    };

    record_custom_v3_raw_choice(sources, src_raw, alias_src, v3);

    let pyramid_levels = choose_custom_v3_pyramid_levels_from_group_or_budget(
        sources,
        group_pyramid,
        ctx.viewport_size,
        ctx.format,
        budget_bytes,
        base_required_bytes,
        v3,
    );
    let pyramid_build_scissor = custom_v3_pyramid_build_scissor(
        sources,
        pyramid_levels,
        group_pyramid_roi,
        scissor,
        ctx.viewport_size,
        ctx.scale_factor,
    );

    CustomV3SourcePlan {
        src_raw,
        pyramid_levels,
        pyramid_build_scissor,
    }
}

fn max_mip_levels_for_size(size: (u32, u32)) -> u32 {
    let mut w = size.0.max(1);
    let mut h = size.1.max(1);
    let mut levels: u32 = 1;
    while w > 1 || h > 1 {
        w = (w / 2).max(1);
        h = (h / 2).max(1);
        levels = levels.saturating_add(1);
        if levels >= 32 {
            break;
        }
    }
    levels
}

fn choose_custom_v3_pyramid_levels_and_charge(
    sources: fret_core::scene::CustomEffectSourcesV3,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: &mut u64,
    base_required_bytes: u64,
    v3: &mut CustomEffectV3SourceDegradationCounters,
) -> u32 {
    let Some(req) = sources.pyramid else {
        return 1;
    };

    v3.pyramid_requested = v3.pyramid_requested.saturating_add(1);

    let max_for_size = max_mip_levels_for_size(size);
    let mut levels = (req.max_levels as u32).max(1).min(max_for_size).min(7);
    if levels < 2 {
        return 1;
    }

    let budget_before = *budget_bytes;
    let headroom_before = budget_before.saturating_sub(base_required_bytes);
    while levels >= 2 {
        let required = estimate_mipped_texture_bytes(size, format, 1, levels);
        if required <= headroom_before {
            *budget_bytes = (*budget_bytes).saturating_sub(required);
            v3.pyramid_applied_levels_ge2 = v3.pyramid_applied_levels_ge2.saturating_add(1);
            return levels;
        }
        levels = levels.saturating_sub(1);
    }

    if budget_before == 0 {
        v3.pyramid_degraded_to_one_budget_zero =
            v3.pyramid_degraded_to_one_budget_zero.saturating_add(1);
    } else {
        v3.pyramid_degraded_to_one_budget_insufficient = v3
            .pyramid_degraded_to_one_budget_insufficient
            .saturating_add(1);
    }

    1
}

pub(super) fn choose_custom_v3_pyramid_choice_for_request(
    req: fret_core::scene::CustomEffectPyramidRequestV1,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    base_required_bytes: u64,
) -> CustomV3PyramidChoice {
    let max_for_size = max_mip_levels_for_size(size);
    let mut levels = (req.max_levels as u32).max(1).min(max_for_size).min(7);
    if levels < 2 {
        return CustomV3PyramidChoice {
            levels: 1,
            degraded_to_one: None,
        };
    }

    let headroom_before = budget_bytes.saturating_sub(base_required_bytes);
    while levels >= 2 {
        let required = estimate_mipped_texture_bytes(size, format, 1, levels);
        if required <= headroom_before {
            return CustomV3PyramidChoice {
                levels,
                degraded_to_one: None,
            };
        }
        levels = levels.saturating_sub(1);
    }

    CustomV3PyramidChoice {
        levels: 1,
        degraded_to_one: Some(if budget_bytes == 0 {
            CustomV3PyramidDegradeReason::BudgetZero
        } else {
            CustomV3PyramidDegradeReason::BudgetInsufficient
        }),
    }
}

pub(super) fn estimate_custom_v3_pyramid_bytes(
    size: (u32, u32),
    format: wgpu::TextureFormat,
    levels: u32,
) -> u64 {
    estimate_mipped_texture_bytes(size, format, 1, levels.max(1))
}

fn pyramid_radius_pad_px(
    req: fret_core::scene::CustomEffectPyramidRequestV1,
    scale_factor: f32,
) -> u32 {
    if !req.max_radius_px.0.is_finite() || req.max_radius_px.0 <= 0.0 {
        return 0;
    }
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return 0;
    }
    (req.max_radius_px.0 * scale_factor).ceil().max(0.0) as u32
}

pub(super) fn append_custom_effect_in_place_single_scratch(
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
            common: CustomEffectPassCommon {
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
            },
        }));
        return;
    }

    passes.push(RenderPlanPass::CustomEffect(CustomEffectPass {
        common: CustomEffectPassCommon {
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
        },
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

#[allow(clippy::too_many_arguments)]
pub(super) fn append_custom_effect_v2_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: Option<ScissorRect>,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    input_image: Option<fret_core::ImageId>,
    input_uv: fret_core::scene::UvRect,
    input_sampling: fret_core::scene::ImageSamplingHint,
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
        passes.push(RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
            common: CustomEffectPassCommon {
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
            },
            input_image,
            input_uv,
            input_sampling,
        }));
        return;
    }

    passes.push(RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
        common: CustomEffectPassCommon {
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
        },
        input_image,
        input_uv,
        input_sampling,
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

#[allow(clippy::too_many_arguments)]
pub(super) fn append_custom_effect_v3_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: Option<ScissorRect>,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    user0_image: Option<fret_core::ImageId>,
    user0_uv: fret_core::scene::UvRect,
    user0_sampling: fret_core::scene::ImageSamplingHint,
    user1_image: Option<fret_core::ImageId>,
    user1_uv: fret_core::scene::UvRect,
    user1_sampling: fret_core::scene::ImageSamplingHint,
    sources: fret_core::scene::CustomEffectSourcesV3,
    src_raw: PlanTarget,
    pyramid_levels: u32,
    pyramid_build_scissor: Option<LocalScissorRect>,
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
        let src_pyramid = src_raw;

        passes.push(RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
            src_raw,
            src_pyramid,
            pyramid_levels,
            pyramid_build_scissor,
            raw_wanted: sources.want_raw,
            pyramid_wanted: sources.pyramid.is_some(),
            common: CustomEffectPassCommon {
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
            },
            user0_image,
            user0_uv,
            user0_sampling,
            user1_image,
            user1_uv,
            user1_sampling,
        }));
        return;
    }
    let src_pyramid = src_raw;

    passes.push(RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
        src_raw,
        src_pyramid,
        pyramid_levels,
        pyramid_build_scissor,
        raw_wanted: sources.want_raw,
        pyramid_wanted: sources.pyramid.is_some(),
        common: CustomEffectPassCommon {
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
        },
        user0_image,
        user0_uv,
        user0_sampling,
        user1_image,
        user1_uv,
        user1_sampling,
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
