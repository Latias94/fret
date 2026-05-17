// RenderPlan compiler.
//
// This module translates a SceneEncoding into a RenderPlan, choosing targets/passes and applying
// deterministic budget-driven degradations.

mod backdrop_source_group;
mod clip_path;
mod composite_group;
mod context;
mod target_budget;

use super::render_plan_effects as effects;
use super::{
    BlurQualitySnapshot, EffectDegradationSnapshot, EffectMarkerKind, OrderedDraw,
    RenderPlanDegradation, RenderPlanDegradationKind, RenderPlanDegradationReason, RenderPlanPass,
    SceneEncoding, ScissorRect,
};
use crate::renderer::estimate_texture_bytes;
use context::RenderPlanCompilerCtx;
use target_budget::{can_allocate_intermediate_bytes, intermediate_budget_breakdown_for_chain};

#[derive(Clone, Copy, Debug)]
struct DrawScope {
    target: super::PlanTarget,
    origin: (u32, u32),
    size: (u32, u32),
    needs_clear: bool,
    clear_color: wgpu::Color,
}

#[derive(Clone, Copy, Debug)]
struct EffectScope {
    mode: fret_core::EffectMode,
    chain: fret_core::EffectChain,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    uniform_index: u32,
    parent_target: super::PlanTarget,
    parent_origin: (u32, u32),
    parent_size: (u32, u32),
    content_target: Option<super::PlanTarget>,
    content_origin: (u32, u32),
    content_size: (u32, u32),
}

fn take_scope_load_for_write(
    draw_scopes: &mut Vec<DrawScope>,
    dst: super::PlanTarget,
) -> wgpu::LoadOp<wgpu::Color> {
    let Some(index) = draw_scopes.iter().rposition(|s| s.target == dst) else {
        return wgpu::LoadOp::Load;
    };
    if draw_scopes[index].needs_clear {
        draw_scopes[index].needs_clear = false;
        wgpu::LoadOp::Clear(draw_scopes[index].clear_color)
    } else {
        wgpu::LoadOp::Load
    }
}

pub(super) fn compile_for_scene(
    encoding: &SceneEncoding,
    scale_factor: f32,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    clear: wgpu::Color,
    path_samples: u32,
    postprocess: super::DebugPostprocess,
    intermediate_budget_bytes: u64,
) -> super::RenderPlan {
    let mut postprocess = postprocess;
    let output_transfer_needed = super::output_requires_explicit_srgb_encode(format);

    let backdrop_effect_enabled = encoding.effect_markers.iter().any(|m| {
        let EffectMarkerKind::Push {
            mode,
            chain,
            quality,
            scissor,
            ..
        } = m.kind
        else {
            return false;
        };
        if mode != fret_core::EffectMode::Backdrop {
            return false;
        }

        chain.iter().any(|step| match step {
            fret_core::EffectStep::GaussianBlur {
                radius_px,
                downsample,
            } => {
                if !radius_px.0.is_finite() || radius_px.0 <= 0.0 {
                    return false;
                }
                effects::choose_effect_blur_downsample_scale(
                    viewport_size,
                    format,
                    intermediate_budget_bytes,
                    downsample,
                    quality,
                )
                .is_some()
            }
            fret_core::EffectStep::BackdropWarpV1(_w) => {
                effects::backdrop_warp_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::BackdropWarpV2(_w) => {
                effects::backdrop_warp_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::DropShadowV1(_s) => false,
            fret_core::EffectStep::ColorAdjust { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::ColorMatrix { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::AlphaThreshold { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::Pixelate { scale } => effects::pixelate_enabled(
                viewport_size,
                Some(scissor),
                format,
                intermediate_budget_bytes,
                scale,
            ),
            fret_core::EffectStep::Dither { .. } => {
                effects::dither_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::NoiseV1(_n) => {
                effects::noise_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::CustomV1 { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::CustomV2 { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
            fret_core::EffectStep::CustomV3 { .. } => {
                effects::color_adjust_enabled(viewport_size, format, intermediate_budget_bytes)
            }
        })
    });

    let needs_intermediate = backdrop_effect_enabled
        || matches!(
            postprocess,
            super::DebugPostprocess::OffscreenBlit { .. }
                | super::DebugPostprocess::Pixelate { .. }
                | super::DebugPostprocess::Blur { .. }
        );

    if needs_intermediate && matches!(postprocess, super::DebugPostprocess::None) {
        postprocess = super::DebugPostprocess::OffscreenBlit {
            src: super::PlanTarget::Intermediate0,
        };
    }

    let mut scene_target = if needs_intermediate {
        super::PlanTarget::Intermediate0
    } else {
        super::PlanTarget::Output
    };

    if scene_target == super::PlanTarget::Output
        && output_transfer_needed
        && matches!(postprocess, super::DebugPostprocess::None)
    {
        scene_target = super::PlanTarget::Intermediate3;
        postprocess = super::DebugPostprocess::OffscreenBlit { src: scene_target };
    }

    compile_for_scene_inner(
        encoding,
        scale_factor,
        viewport_size,
        format,
        clear,
        path_samples,
        postprocess,
        intermediate_budget_bytes,
        scene_target,
    )
}

fn compile_for_scene_inner(
    encoding: &SceneEncoding,
    scale_factor: f32,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    clear: wgpu::Color,
    path_samples: u32,
    postprocess: super::DebugPostprocess,
    intermediate_budget_bytes: u64,
    scene_target: super::PlanTarget,
) -> super::RenderPlan {
    let draws = &encoding.ordered_draws;
    let markers = &encoding.effect_markers;
    let scissor_sized_intermediates = !markers.iter().any(|m| match m.kind {
        EffectMarkerKind::Push { mode, .. } => mode == fret_core::EffectMode::Backdrop,
        _ => false,
    });
    let mut ctx = RenderPlanCompilerCtx::new();
    let mut effect_degradations = EffectDegradationSnapshot::default();
    let mut effect_blur_quality = BlurQualitySnapshot::default();
    let mut draw_scopes: Vec<DrawScope> = vec![DrawScope {
        target: scene_target,
        origin: (0, 0),
        size: viewport_size,
        needs_clear: true,
        clear_color: clear,
    }];
    let mut effect_scopes: Vec<EffectScope> = Vec::new();
    let mut composite_group_scopes: Vec<composite_group::CompositeGroupScope> = Vec::new();
    let mut clip_path_scopes: Vec<clip_path::ClipPathScope> = Vec::new();
    let mut clip_path_mask_in_use_bytes: u64 = 0;
    let mut backdrop_source_group_scopes: Vec<backdrop_source_group::BackdropSourceGroupScope> =
        Vec::new();
    let mut backdrop_source_group_reserved_targets: Vec<super::PlanTarget> = Vec::new();
    let mut backdrop_source_group_in_use_bytes: u64 = 0;

    let mut effect_chain_budget_samples: u64 = 0;
    let mut effect_chain_effective_budget_min_bytes: u64 = u64::MAX;
    let mut effect_chain_effective_budget_max_bytes: u64 = 0;
    let mut effect_chain_other_live_max_bytes: u64 = 0;

    let mut custom_effect_chain_budget_samples: u64 = 0;
    let mut custom_effect_chain_effective_budget_min_bytes: u64 = u64::MAX;
    let mut custom_effect_chain_effective_budget_max_bytes: u64 = 0;
    let mut custom_effect_chain_other_live_max_bytes: u64 = 0;
    let mut custom_effect_chain_base_required_max_bytes: u64 = 0;
    let mut custom_effect_chain_optional_required_max_bytes: u64 = 0;
    let mut custom_effect_chain_base_required_full_targets_max: u32 = 0;
    let mut custom_effect_chain_optional_mask_max_bytes: u64 = 0;
    let mut custom_effect_chain_optional_pyramid_max_bytes: u64 = 0;

    let mut apply_chain_in_place =
        |passes: &mut Vec<RenderPlanPass>,
         draw_scopes: &[DrawScope],
         srcdst: super::PlanTarget,
         mode: fret_core::EffectMode,
         chain: fret_core::EffectChain,
         quality: fret_core::EffectQuality,
         ctx_viewport_size: (u32, u32),
         scissor: ScissorRect,
         mask_uniform_index: Option<u32>,
         extra_in_use_bytes: u64,
         unavailable_mask_targets: &[super::PlanTarget],
         reserved_targets: &[super::PlanTarget],
         backdrop_source_group: Option<effects::BackdropSourceGroupCtx>,
         effect_degradations: &mut EffectDegradationSnapshot,
         effect_blur_quality: &mut BlurQualitySnapshot| {
            if srcdst == super::PlanTarget::Output || scissor.w == 0 || scissor.h == 0 {
                return;
            }

            let breakdown = intermediate_budget_breakdown_for_chain(
                intermediate_budget_bytes,
                draw_scopes,
                srcdst,
                format,
                extra_in_use_bytes,
            );
            let effective_budget_bytes = breakdown.effective_budget_bytes;

            if !chain.is_empty() {
                effect_chain_budget_samples = effect_chain_budget_samples.saturating_add(1);
                effect_chain_effective_budget_min_bytes =
                    effect_chain_effective_budget_min_bytes.min(effective_budget_bytes);
                effect_chain_effective_budget_max_bytes =
                    effect_chain_effective_budget_max_bytes.max(effective_budget_bytes);
                effect_chain_other_live_max_bytes =
                    effect_chain_other_live_max_bytes.max(breakdown.other_live_bytes);
            }

            let mut in_use_targets: Vec<super::PlanTarget> = Vec::new();
            for s in draw_scopes {
                if !in_use_targets.contains(&s.target) {
                    in_use_targets.push(s.target);
                }
            }
            for &t in reserved_targets {
                if !in_use_targets.contains(&t) {
                    in_use_targets.push(t);
                }
            }
            let custom_evidence = effects::apply_chain_in_place(
                passes,
                &in_use_targets,
                srcdst,
                mode,
                chain,
                quality,
                scissor,
                mask_uniform_index,
                unavailable_mask_targets,
                effect_degradations,
                effect_blur_quality,
                effects::EffectCompileCtx {
                    viewport_size: ctx_viewport_size,
                    format,
                    intermediate_budget_bytes: effective_budget_bytes,
                    clear,
                    scale_factor,
                },
                backdrop_source_group,
            );

            if let Some(e) = custom_evidence {
                custom_effect_chain_budget_samples =
                    custom_effect_chain_budget_samples.saturating_add(1);
                let effective_budget_bytes = e.effective_budget_bytes;
                custom_effect_chain_effective_budget_min_bytes =
                    custom_effect_chain_effective_budget_min_bytes.min(effective_budget_bytes);
                custom_effect_chain_effective_budget_max_bytes =
                    custom_effect_chain_effective_budget_max_bytes.max(effective_budget_bytes);
                custom_effect_chain_other_live_max_bytes =
                    custom_effect_chain_other_live_max_bytes.max(breakdown.other_live_bytes);
                custom_effect_chain_base_required_max_bytes =
                    custom_effect_chain_base_required_max_bytes.max(e.base_required_bytes);
                custom_effect_chain_optional_required_max_bytes =
                    custom_effect_chain_optional_required_max_bytes
                        .max(e.optional_required_bytes());
                custom_effect_chain_base_required_full_targets_max =
                    custom_effect_chain_base_required_full_targets_max
                        .max(e.base_required_full_targets);
                custom_effect_chain_optional_mask_max_bytes =
                    custom_effect_chain_optional_mask_max_bytes.max(e.optional_mask_bytes);
                custom_effect_chain_optional_pyramid_max_bytes =
                    custom_effect_chain_optional_pyramid_max_bytes.max(e.optional_pyramid_bytes);
            }
        };

    let mut scene_range_start: usize = 0;
    let mut cursor: usize = 0;
    let mut marker_ix: usize = 0;

    while cursor <= draws.len() {
        let next_marker_at = markers
            .get(marker_ix)
            .map(|m| m.draw_ix)
            .unwrap_or(usize::MAX);

        if cursor == next_marker_at || cursor == draws.len() {
            ctx.flush_scene_range(
                cursor,
                &mut draw_scopes,
                draws,
                encoding,
                &mut scene_range_start,
            );

            while marker_ix < markers.len() && markers[marker_ix].draw_ix == cursor {
                let marker = markers[marker_ix];
                match marker.kind {
                    EffectMarkerKind::Push {
                        scissor,
                        uniform_index,
                        mode,
                        chain,
                        quality,
                    } => {
                        let parent_scope = draw_scopes.last().expect("draw scope");
                        let parent_target = parent_scope.target;
                        let parent_origin = parent_scope.origin;
                        let parent_size = parent_scope.size;
                        match mode {
                            fret_core::EffectMode::Backdrop => {
                                let had_free_scratch_target = [
                                    super::PlanTarget::Intermediate0,
                                    super::PlanTarget::Intermediate1,
                                    super::PlanTarget::Intermediate2,
                                    super::PlanTarget::Intermediate3,
                                ]
                                .into_iter()
                                .any(|t| {
                                    t != parent_target
                                        && !draw_scopes.iter().any(|s| s.target == t)
                                        && !backdrop_source_group_reserved_targets.contains(&t)
                                });

                                let before = ctx.passes_len();
                                let unavailable_mask_targets: Vec<super::PlanTarget> =
                                    clip_path::active_mask_targets(&clip_path_scopes).collect();
                                let backdrop_source_group = backdrop_source_group_scopes
                                    .last()
                                    .and_then(|s| s.effect_ctx());
                                apply_chain_in_place(
                                    ctx.passes_mut(),
                                    &draw_scopes,
                                    parent_target,
                                    mode,
                                    chain,
                                    quality,
                                    parent_size,
                                    scissor,
                                    Some(uniform_index),
                                    clip_path_mask_in_use_bytes
                                        .saturating_add(backdrop_source_group_in_use_bytes),
                                    &unavailable_mask_targets,
                                    &backdrop_source_group_reserved_targets,
                                    backdrop_source_group,
                                    &mut effect_degradations,
                                    &mut effect_blur_quality,
                                );
                                if before == ctx.passes_len()
                                    && !chain.is_empty()
                                    && parent_target != super::PlanTarget::Output
                                    && scissor.w != 0
                                    && scissor.h != 0
                                {
                                    let reason = if intermediate_budget_bytes == 0 {
                                        RenderPlanDegradationReason::BudgetZero
                                    } else if !had_free_scratch_target {
                                        RenderPlanDegradationReason::TargetExhausted
                                    } else {
                                        RenderPlanDegradationReason::BudgetInsufficient
                                    };
                                    ctx.push_degradation(RenderPlanDegradation {
                                        draw_ix: cursor,
                                        kind: RenderPlanDegradationKind::BackdropEffectNoOp,
                                        reason,
                                    });
                                }

                                effect_scopes.push(EffectScope {
                                    mode,
                                    chain,
                                    quality,
                                    scissor,
                                    uniform_index,
                                    parent_target,
                                    parent_origin,
                                    parent_size,
                                    content_target: None,
                                    content_origin: (0, 0),
                                    content_size: (0, 0),
                                });
                            }
                            fret_core::EffectMode::FilterContent => {
                                // `bounds` are computation bounds (ADR 0117), not an implicit clip.
                                // FilterContent therefore must preserve unfiltered content outside
                                // `bounds`, which requires a full-viewport content target (the
                                // postprocess passes themselves remain scissored to `bounds`).
                                let (content_origin, content_size) = ((0, 0), viewport_size);
                                let mut content_target: Option<super::PlanTarget> = None;
                                let mut had_free_target = false;
                                if content_size.0 != 0 && content_size.1 != 0 {
                                    for t in [
                                        super::PlanTarget::Intermediate0,
                                        super::PlanTarget::Intermediate1,
                                        super::PlanTarget::Intermediate2,
                                        super::PlanTarget::Intermediate3,
                                    ] {
                                        if draw_scopes.iter().any(|s| s.target == t)
                                            || backdrop_source_group_reserved_targets.contains(&t)
                                        {
                                            continue;
                                        }
                                        content_target = Some(t);
                                        had_free_target = true;
                                        break;
                                    }

                                    if content_target.is_some()
                                        && !can_allocate_intermediate_bytes(
                                            intermediate_budget_bytes,
                                            &draw_scopes,
                                            estimate_texture_bytes(content_size, format, 1),
                                            clip_path_mask_in_use_bytes
                                                .saturating_add(backdrop_source_group_in_use_bytes),
                                            format,
                                        )
                                    {
                                        content_target = None;
                                    }
                                }

                                if let Some(content_target) = content_target {
                                    draw_scopes.push(DrawScope {
                                        target: content_target,
                                        origin: content_origin,
                                        size: content_size,
                                        needs_clear: true,
                                        clear_color: wgpu::Color::TRANSPARENT,
                                    });
                                } else if content_size.0 != 0 && content_size.1 != 0 {
                                    ctx.push_degradation(RenderPlanDegradation {
                                        draw_ix: cursor,
                                        kind: RenderPlanDegradationKind::FilterContentDisabled,
                                        reason: if !had_free_target {
                                            RenderPlanDegradationReason::TargetExhausted
                                        } else if intermediate_budget_bytes == 0 {
                                            RenderPlanDegradationReason::BudgetZero
                                        } else {
                                            RenderPlanDegradationReason::BudgetInsufficient
                                        },
                                    });
                                }

                                effect_scopes.push(EffectScope {
                                    mode,
                                    chain,
                                    quality,
                                    scissor,
                                    uniform_index,
                                    parent_target,
                                    parent_origin,
                                    parent_size,
                                    content_target,
                                    content_origin,
                                    content_size,
                                });
                            }
                        }
                    }
                    EffectMarkerKind::Pop => {
                        let Some(scope) = effect_scopes.pop() else {
                            marker_ix += 1;
                            continue;
                        };

                        if scope.mode == fret_core::EffectMode::FilterContent
                            && let Some(content_target) = scope.content_target
                        {
                            debug_assert_eq!(
                                draw_scopes.last().expect("draw scope").target,
                                content_target
                            );

                            let had_free_scratch_target = [
                                super::PlanTarget::Intermediate0,
                                super::PlanTarget::Intermediate1,
                                super::PlanTarget::Intermediate2,
                                super::PlanTarget::Intermediate3,
                            ]
                            .into_iter()
                            .any(|t| {
                                t != content_target
                                    && !draw_scopes.iter().any(|s| s.target == t)
                                    && !backdrop_source_group_reserved_targets.contains(&t)
                            });

                            let chain_scissor = if scope.content_size == viewport_size {
                                scope.scissor
                            } else {
                                ScissorRect::full(scope.content_size.0, scope.content_size.1)
                            };

                            let before = ctx.passes_len();
                            apply_chain_in_place(
                                ctx.passes_mut(),
                                &draw_scopes,
                                content_target,
                                scope.mode,
                                scope.chain,
                                scope.quality,
                                scope.content_size,
                                chain_scissor,
                                None,
                                clip_path_mask_in_use_bytes
                                    .saturating_add(backdrop_source_group_in_use_bytes),
                                &[],
                                &backdrop_source_group_reserved_targets,
                                None,
                                &mut effect_degradations,
                                &mut effect_blur_quality,
                            );
                            if before == ctx.passes_len()
                                && !scope.chain.is_empty()
                                && chain_scissor.w != 0
                                && chain_scissor.h != 0
                            {
                                ctx.push_degradation(RenderPlanDegradation {
                                    draw_ix: cursor,
                                    kind: RenderPlanDegradationKind::FilterContentDisabled,
                                    reason: if intermediate_budget_bytes == 0 {
                                        RenderPlanDegradationReason::BudgetZero
                                    } else if !had_free_scratch_target {
                                        RenderPlanDegradationReason::TargetExhausted
                                    } else {
                                        RenderPlanDegradationReason::BudgetInsufficient
                                    },
                                });
                            }

                            let cropped = scope.content_origin != (0, 0)
                                || scope.content_size != viewport_size;
                            let load =
                                take_scope_load_for_write(&mut draw_scopes, scope.parent_target);
                            ctx.push_pass(RenderPlanPass::CompositePremul(
                                super::CompositePremulPass {
                                    src: content_target,
                                    src_origin: scope.content_origin,
                                    dst: scope.parent_target,
                                    src_size: scope.content_size,
                                    dst_origin: scope.parent_origin,
                                    dst_size: scope.parent_size,
                                    dst_scissor: cropped
                                        .then_some(super::AbsoluteScissorRect(scope.scissor)),
                                    mask_uniform_index: Some(scope.uniform_index),
                                    mask: None,
                                    blend_mode: fret_core::BlendMode::Over,
                                    opacity: 1.0,
                                    load,
                                },
                            ));

                            let _ = draw_scopes.pop();
                        }
                    }
                    EffectMarkerKind::ClipPathPush {
                        scissor,
                        uniform_index,
                        mask_draw_index,
                    } => {
                        clip_path::compile_clip_path_push(
                            &mut ctx,
                            &mut draw_scopes,
                            &mut clip_path_scopes,
                            &mut clip_path_mask_in_use_bytes,
                            encoding,
                            cursor,
                            clip_path::ClipPathPushCtx {
                                scissor,
                                uniform_index,
                                mask_draw_index,
                                viewport_size,
                                scissor_sized_intermediates,
                                format,
                                intermediate_budget_bytes,
                                backdrop_source_group_reserved_targets:
                                    &backdrop_source_group_reserved_targets,
                                backdrop_source_group_in_use_bytes,
                            },
                        );
                    }
                    EffectMarkerKind::ClipPathPop => {
                        clip_path::compile_clip_path_pop(
                            &mut ctx,
                            &mut draw_scopes,
                            &mut clip_path_scopes,
                            &mut clip_path_mask_in_use_bytes,
                        );
                    }
                    EffectMarkerKind::BackdropSourceGroupPush {
                        scissor,
                        pyramid,
                        quality,
                    } => {
                        backdrop_source_group::compile_backdrop_source_group_push(
                            &mut ctx,
                            &draw_scopes,
                            &mut backdrop_source_group_scopes,
                            &mut backdrop_source_group_reserved_targets,
                            &mut backdrop_source_group_in_use_bytes,
                            &mut effect_degradations.backdrop_source_groups,
                            backdrop_source_group::BackdropSourceGroupPushCtx {
                                scissor,
                                pyramid,
                                quality,
                                scale_factor,
                                viewport_size,
                                format,
                                intermediate_budget_bytes,
                                clip_path_mask_in_use_bytes,
                            },
                        );
                    }
                    EffectMarkerKind::BackdropSourceGroupPop => {
                        backdrop_source_group::compile_backdrop_source_group_pop(
                            &mut backdrop_source_group_scopes,
                            &mut backdrop_source_group_reserved_targets,
                            &mut backdrop_source_group_in_use_bytes,
                        );
                    }
                    EffectMarkerKind::CompositeGroupPush {
                        scissor,
                        uniform_index,
                        mode,
                        quality,
                        opacity,
                    } => {
                        composite_group::compile_composite_group_push(
                            &mut ctx,
                            &mut draw_scopes,
                            &mut composite_group_scopes,
                            cursor,
                            composite_group::CompositeGroupPushCtx {
                                scissor,
                                uniform_index,
                                mode,
                                quality,
                                opacity,
                                viewport_size,
                                scissor_sized_intermediates,
                                format,
                                intermediate_budget_bytes,
                                clip_path_mask_in_use_bytes,
                                backdrop_source_group_reserved_targets:
                                    &backdrop_source_group_reserved_targets,
                                backdrop_source_group_in_use_bytes,
                            },
                        );
                    }
                    EffectMarkerKind::CompositeGroupPop => {
                        composite_group::compile_composite_group_pop(
                            &mut ctx,
                            &mut draw_scopes,
                            &mut composite_group_scopes,
                        );
                    }
                }

                marker_ix += 1;
            }

            if cursor == draws.len() {
                break;
            }

            continue;
        }

        if path_samples > 1
            && let OrderedDraw::Path(first) = &draws[cursor]
        {
            ctx.flush_scene_range(
                cursor,
                &mut draw_scopes,
                draws,
                encoding,
                &mut scene_range_start,
            );

            let batch_uniform_index = first.uniform_index;
            let mut union = first.scissor;
            let mut end = cursor + 1;
            while end < draws.len() && end < next_marker_at {
                match &draws[end] {
                    OrderedDraw::Path(d) if d.uniform_index == batch_uniform_index => {
                        union = super::union_scissor(union, d.scissor);
                        end += 1;
                    }
                    _ => break,
                }
            }

            let scope = draw_scopes.last().expect("draw scope");
            let target = scope.target;
            let segment = ctx.alloc_segment(cursor..end, draws, encoding);
            ctx.push_pass(RenderPlanPass::PathMsaaBatch(super::PathMsaaBatchPass {
                segment,
                target,
                target_origin: scope.origin,
                target_size: scope.size,
                draw_range: cursor..end,
                union_scissor: super::AbsoluteScissorRect(union),
                batch_uniform_index,
                load: wgpu::LoadOp::Load,
            }));

            cursor = end;
            scene_range_start = cursor;
            continue;
        }

        cursor += 1;
    }

    let (segments, passes, degradations) = ctx.into_parts();
    let mut plan = super::RenderPlan::finalize(
        segments,
        passes,
        viewport_size,
        postprocess,
        clear,
        format,
        degradations,
        effect_degradations,
        effect_blur_quality,
    );

    if effect_chain_budget_samples > 0 {
        plan.compile_stats.effect_chain_budget_samples = effect_chain_budget_samples;
        plan.compile_stats.effect_chain_effective_budget_min_bytes =
            effect_chain_effective_budget_min_bytes;
        plan.compile_stats.effect_chain_effective_budget_max_bytes =
            effect_chain_effective_budget_max_bytes;
        plan.compile_stats.effect_chain_other_live_max_bytes = effect_chain_other_live_max_bytes;
    }

    if custom_effect_chain_budget_samples > 0 {
        plan.compile_stats.custom_effect_chain_budget_samples = custom_effect_chain_budget_samples;
        plan.compile_stats
            .custom_effect_chain_effective_budget_min_bytes =
            custom_effect_chain_effective_budget_min_bytes;
        plan.compile_stats
            .custom_effect_chain_effective_budget_max_bytes =
            custom_effect_chain_effective_budget_max_bytes;
        plan.compile_stats.custom_effect_chain_other_live_max_bytes =
            custom_effect_chain_other_live_max_bytes;
        plan.compile_stats
            .custom_effect_chain_base_required_max_bytes =
            custom_effect_chain_base_required_max_bytes;
        plan.compile_stats
            .custom_effect_chain_optional_required_max_bytes =
            custom_effect_chain_optional_required_max_bytes;
        plan.compile_stats
            .custom_effect_chain_base_required_full_targets_max =
            custom_effect_chain_base_required_full_targets_max;
        plan.compile_stats
            .custom_effect_chain_optional_mask_max_bytes =
            custom_effect_chain_optional_mask_max_bytes;
        plan.compile_stats
            .custom_effect_chain_optional_pyramid_max_bytes =
            custom_effect_chain_optional_pyramid_max_bytes;
    }

    plan
}
