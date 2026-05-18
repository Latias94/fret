use super::blur::inflate_scissor_to_viewport;
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
        let required = estimate_custom_v3_pyramid_bytes(size, format, levels);
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
        let required = estimate_custom_v3_pyramid_bytes(size, format, levels);
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

fn prepare_custom_effect_single_scratch(
    scratch_targets: &[PlanTarget],
    budget_bytes: u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
) -> Option<PlanTarget> {
    effect_degradations.custom_effect.requested = effect_degradations
        .custom_effect
        .requested
        .saturating_add(1);
    if !color_adjust_enabled(viewport_size, format, budget_bytes) {
        if budget_bytes == 0 {
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
        return None;
    }
    let Some(&scratch) = scratch_targets.first() else {
        effect_degradations.custom_effect.degraded_target_exhausted = effect_degradations
            .custom_effect
            .degraded_target_exhausted
            .saturating_add(1);
        return None;
    };
    effect_degradations.custom_effect.applied =
        effect_degradations.custom_effect.applied.saturating_add(1);
    Some(scratch)
}

fn resolve_custom_effect_image_input(
    input: Option<fret_core::scene::CustomEffectImageInputV1>,
) -> (
    Option<fret_core::ImageId>,
    fret_core::scene::UvRect,
    fret_core::scene::ImageSamplingHint,
) {
    match input {
        None => (
            None,
            fret_core::scene::UvRect::FULL,
            fret_core::scene::ImageSamplingHint::Default,
        ),
        Some(input) => (Some(input.image), input.uv, input.sampling),
    }
}

fn apply_custom_v1_step_inner(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    let Some(scratch) = prepare_custom_effect_single_scratch(
        scratch_targets,
        *budget_bytes,
        effect_degradations,
        ctx.viewport_size,
        ctx.format,
    ) else {
        return;
    };

    append_custom_effect_in_place_single_scratch(
        passes,
        srcdst,
        scratch,
        ctx.viewport_size,
        Some(scissor),
        effect,
        params,
        ctx.clear,
        mask_uniform_index,
        mask,
    );
}

pub(super) fn apply_custom_v1_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    apply_custom_v1_step_inner(
        passes,
        scratch_targets,
        srcdst,
        scissor,
        effect,
        params,
        ctx,
        budget_bytes,
        effect_degradations,
        None,
        None,
    );
}

pub(super) fn apply_custom_v1_step_masked(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    apply_custom_v1_step_inner(
        passes,
        scratch_targets,
        srcdst,
        scissor,
        effect,
        params,
        ctx,
        budget_bytes,
        effect_degradations,
        mask_uniform_index,
        mask,
    );
}

#[allow(clippy::too_many_arguments)]
fn apply_custom_v2_step_inner(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    input_image: Option<fret_core::scene::CustomEffectImageInputV1>,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    let Some(scratch) = prepare_custom_effect_single_scratch(
        scratch_targets,
        *budget_bytes,
        effect_degradations,
        ctx.viewport_size,
        ctx.format,
    ) else {
        return;
    };

    let (input_image, input_uv, input_sampling) = resolve_custom_effect_image_input(input_image);
    append_custom_effect_v2_in_place_single_scratch(
        passes,
        srcdst,
        scratch,
        ctx.viewport_size,
        Some(scissor),
        effect,
        params,
        input_image,
        input_uv,
        input_sampling,
        ctx.clear,
        mask_uniform_index,
        mask,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_custom_v2_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    input_image: Option<fret_core::scene::CustomEffectImageInputV1>,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
) {
    apply_custom_v2_step_inner(
        passes,
        scratch_targets,
        srcdst,
        scissor,
        effect,
        params,
        input_image,
        ctx,
        budget_bytes,
        effect_degradations,
        None,
        None,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_custom_v2_step_masked(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    input_image: Option<fret_core::scene::CustomEffectImageInputV1>,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    apply_custom_v2_step_inner(
        passes,
        scratch_targets,
        srcdst,
        scissor,
        effect,
        params,
        input_image,
        ctx,
        budget_bytes,
        effect_degradations,
        mask_uniform_index,
        mask,
    );
}

#[allow(clippy::too_many_arguments)]
fn apply_custom_v3_step_inner(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    user0: Option<fret_core::scene::CustomEffectImageInputV1>,
    user1: Option<fret_core::scene::CustomEffectImageInputV1>,
    sources: fret_core::scene::CustomEffectSourcesV3,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    custom_v3_chain_raw: Option<PlanTarget>,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
    custom_chain_budget: &mut Option<CustomEffectChainBudgetEvidence>,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    let Some(scratch) = prepare_custom_effect_single_scratch(
        scratch_targets,
        *budget_bytes,
        effect_degradations,
        ctx.viewport_size,
        ctx.format,
    ) else {
        return;
    };

    let (user0_image, user0_uv, user0_sampling) = resolve_custom_effect_image_input(user0);
    let (user1_image, user1_uv, user1_sampling) = resolve_custom_effect_image_input(user1);
    let (group_raw, group_pyramid, group_pyramid_roi) =
        super::chain::backdrop_source_group_parts(backdrop_source_group);

    let v3_chain_raw = group_raw.or(custom_v3_chain_raw);
    let v3_sources_plan = plan_custom_v3_sources_and_charge_budget(
        sources,
        scratch,
        v3_chain_raw,
        group_pyramid,
        group_pyramid_roi,
        scissor,
        ctx,
        budget_bytes,
        base_required_bytes_for_srcdst_and_single_scratch(estimate_texture_bytes(
            ctx.viewport_size,
            ctx.format,
            1,
        )),
        &mut effect_degradations.custom_effect_v3_sources,
    );
    if let Some(e) = custom_chain_budget.as_mut()
        && group_pyramid.is_none()
        && sources.pyramid.is_some()
        && v3_sources_plan.pyramid_levels >= 2
    {
        e.optional_pyramid_bytes =
            e.optional_pyramid_bytes
                .saturating_add(estimate_custom_v3_pyramid_bytes(
                    ctx.viewport_size,
                    ctx.format,
                    v3_sources_plan.pyramid_levels,
                ));
    }
    append_custom_effect_v3_in_place_single_scratch(
        passes,
        srcdst,
        scratch,
        ctx.viewport_size,
        Some(scissor),
        effect,
        params,
        user0_image,
        user0_uv,
        user0_sampling,
        user1_image,
        user1_uv,
        user1_sampling,
        sources,
        v3_sources_plan.src_raw,
        v3_sources_plan.pyramid_levels,
        v3_sources_plan.pyramid_build_scissor,
        ctx.clear,
        mask_uniform_index,
        mask,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_padded_chain_final_custom_step(
    passes: &mut Vec<RenderPlanPass>,
    work: PlanTarget,
    srcdst: PlanTarget,
    final_step: fret_core::EffectStep,
    final_scissor: ScissorRect,
    scissor: ScissorRect,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
    chain_raw: Option<PlanTarget>,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
    ctx: EffectCompileCtx,
    work_budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    custom_chain_budget: &mut Option<CustomEffectChainBudgetEvidence>,
) -> bool {
    debug_assert_eq!(
        final_scissor, scissor,
        "final scissor must be the original bounds"
    );

    match final_step {
        fret_core::EffectStep::CustomV1 {
            id,
            params,
            max_sample_offset_px: _,
        } => {
            passes.push(RenderPlanPass::CustomEffect(CustomEffectPass {
                common: CustomEffectPassCommon {
                    src: work,
                    dst: srcdst,
                    src_size: ctx.viewport_size,
                    dst_size: ctx.viewport_size,
                    dst_scissor: Some(LocalScissorRect(scissor)),
                    mask_uniform_index,
                    mask,
                    effect: id,
                    params,
                    load: wgpu::LoadOp::Load,
                },
            }));
            true
        }
        fret_core::EffectStep::CustomV2 {
            id,
            params,
            max_sample_offset_px: _,
            input_image,
        } => {
            let (input_image, input_uv, input_sampling) =
                resolve_custom_effect_image_input(input_image);
            passes.push(RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                common: CustomEffectPassCommon {
                    src: work,
                    dst: srcdst,
                    src_size: ctx.viewport_size,
                    dst_size: ctx.viewport_size,
                    dst_scissor: Some(LocalScissorRect(scissor)),
                    mask_uniform_index,
                    mask,
                    effect: id,
                    params,
                    load: wgpu::LoadOp::Load,
                },
                input_image,
                input_uv,
                input_sampling,
            }));
            true
        }
        fret_core::EffectStep::CustomV3 {
            id,
            params,
            max_sample_offset_px: _,
            user0,
            user1,
            sources,
        } => {
            let (user0_image, user0_uv, user0_sampling) = resolve_custom_effect_image_input(user0);
            let (user1_image, user1_uv, user1_sampling) = resolve_custom_effect_image_input(user1);
            let (group_raw, group_pyramid, group_pyramid_roi) =
                super::chain::backdrop_source_group_parts(backdrop_source_group);

            let v3_chain_raw = group_raw.or(chain_raw);
            let v3_sources_plan = plan_custom_v3_sources_and_charge_budget(
                sources,
                work,
                v3_chain_raw,
                group_pyramid,
                group_pyramid_roi,
                scissor,
                ctx,
                work_budget_bytes,
                estimate_texture_bytes(ctx.viewport_size, ctx.format, 1),
                &mut effect_degradations.custom_effect_v3_sources,
            );

            if let Some(evidence) = custom_chain_budget.as_mut()
                && group_pyramid.is_none()
                && sources.pyramid.is_some()
                && v3_sources_plan.pyramid_levels >= 2
            {
                evidence.optional_pyramid_bytes = evidence.optional_pyramid_bytes.saturating_add(
                    estimate_custom_v3_pyramid_bytes(
                        ctx.viewport_size,
                        ctx.format,
                        v3_sources_plan.pyramid_levels,
                    ),
                );
            }

            passes.push(RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
                src_raw: v3_sources_plan.src_raw,
                src_pyramid: v3_sources_plan.src_raw,
                pyramid_levels: v3_sources_plan.pyramid_levels,
                pyramid_build_scissor: v3_sources_plan.pyramid_build_scissor,
                raw_wanted: sources.want_raw,
                pyramid_wanted: sources.pyramid.is_some(),
                common: CustomEffectPassCommon {
                    src: work,
                    dst: srcdst,
                    src_size: ctx.viewport_size,
                    dst_size: ctx.viewport_size,
                    dst_scissor: Some(LocalScissorRect(scissor)),
                    mask_uniform_index,
                    mask,
                    effect: id,
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
            true
        }
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_custom_v3_step(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    user0: Option<fret_core::scene::CustomEffectImageInputV1>,
    user1: Option<fret_core::scene::CustomEffectImageInputV1>,
    sources: fret_core::scene::CustomEffectSourcesV3,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    custom_v3_chain_raw: Option<PlanTarget>,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
    custom_chain_budget: &mut Option<CustomEffectChainBudgetEvidence>,
) {
    apply_custom_v3_step_inner(
        passes,
        scratch_targets,
        srcdst,
        scissor,
        effect,
        params,
        user0,
        user1,
        sources,
        ctx,
        budget_bytes,
        effect_degradations,
        custom_v3_chain_raw,
        backdrop_source_group,
        custom_chain_budget,
        None,
        None,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_custom_v3_step_masked(
    passes: &mut Vec<RenderPlanPass>,
    scratch_targets: &[PlanTarget],
    srcdst: PlanTarget,
    scissor: ScissorRect,
    effect: fret_core::EffectId,
    params: fret_core::EffectParamsV1,
    user0: Option<fret_core::scene::CustomEffectImageInputV1>,
    user1: Option<fret_core::scene::CustomEffectImageInputV1>,
    sources: fret_core::scene::CustomEffectSourcesV3,
    ctx: EffectCompileCtx,
    budget_bytes: &mut u64,
    effect_degradations: &mut super::super::EffectDegradationSnapshot,
    custom_v3_chain_raw: Option<PlanTarget>,
    backdrop_source_group: Option<BackdropSourceGroupCtx>,
    custom_chain_budget: &mut Option<CustomEffectChainBudgetEvidence>,
    mask_uniform_index: Option<u32>,
    mask: Option<MaskRef>,
) {
    apply_custom_v3_step_inner(
        passes,
        scratch_targets,
        srcdst,
        scissor,
        effect,
        params,
        user0,
        user1,
        sources,
        ctx,
        budget_bytes,
        effect_degradations,
        custom_v3_chain_raw,
        backdrop_source_group,
        custom_chain_budget,
        mask_uniform_index,
        mask,
    );
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
