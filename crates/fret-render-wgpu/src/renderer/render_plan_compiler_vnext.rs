// VNext RenderPlan compiler.
//
// This module exists as a refactor seam: it allows us to iteratively rewrite the plan compiler
// while keeping the legacy compiler available for pixel-compare conformance gates.

use super::render_plan_effects as effects;
use super::{
    EffectMarkerKind, OrderedDraw, RenderPlanDegradation, RenderPlanDegradationKind,
    RenderPlanDegradationReason, RenderPlanPass, SceneDrawRangePass, SceneEncoding, ScissorRect,
};
use crate::renderer::estimate_texture_bytes;

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

#[derive(Clone, Copy, Debug)]
struct CompositeGroupScope {
    mode: fret_core::BlendMode,
    quality: fret_core::EffectQuality,
    scissor: ScissorRect,
    uniform_index: u32,
    opacity: f32,
    parent_target: super::PlanTarget,
    parent_origin: (u32, u32),
    parent_size: (u32, u32),
    content_target: Option<super::PlanTarget>,
    content_origin: (u32, u32),
    content_size: (u32, u32),
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct ClipPathScope {
    scissor: ScissorRect,
    uniform_index: u32,
    mask_draw_index: u32,
    parent_target: super::PlanTarget,
    parent_origin: (u32, u32),
    parent_size: (u32, u32),
    content_target: Option<super::PlanTarget>,
    content_origin: (u32, u32),
    content_size: (u32, u32),
    mask_target: Option<super::PlanTarget>,
    mask_size: (u32, u32),
}

pub(super) fn compile_for_scene_vnext(
    encoding: &SceneEncoding,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    clear: wgpu::Color,
    path_samples: u32,
    postprocess: super::DebugPostprocess,
    intermediate_budget_bytes: u64,
) -> super::RenderPlan {
    let mut postprocess = postprocess;

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
            fret_core::EffectStep::GaussianBlur { downsample, .. } => {
                effects::choose_effect_blur_downsample_scale(
                    viewport_size,
                    format,
                    intermediate_budget_bytes,
                    downsample,
                    quality,
                )
                .is_some()
            }
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
            fret_core::EffectStep::Dither { .. } => false,
        })
    });

    let needs_intermediate = backdrop_effect_enabled
        || matches!(
            postprocess,
            super::DebugPostprocess::OffscreenBlit
                | super::DebugPostprocess::Pixelate { .. }
                | super::DebugPostprocess::Blur { .. }
        );

    if needs_intermediate && matches!(postprocess, super::DebugPostprocess::None) {
        postprocess = super::DebugPostprocess::OffscreenBlit;
    }

    let scene_target = if needs_intermediate {
        super::PlanTarget::Intermediate0
    } else {
        super::PlanTarget::Output
    };

    if encoding.effect_markers.is_empty() {
        return super::RenderPlan::compile_for_scene(
            encoding,
            viewport_size,
            format,
            clear,
            path_samples,
            postprocess,
            intermediate_budget_bytes,
        );
    }

    compile_for_scene_vnext_with_markers(
        encoding,
        viewport_size,
        format,
        clear,
        path_samples,
        postprocess,
        intermediate_budget_bytes,
        scene_target,
    )
}

fn compile_for_scene_vnext_with_markers(
    encoding: &SceneEncoding,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    clear: wgpu::Color,
    path_samples: u32,
    postprocess: super::DebugPostprocess,
    intermediate_budget_bytes: u64,
    scene_target: super::PlanTarget,
) -> super::RenderPlan {
    compile_for_scene_vnext_effects_only(
        encoding,
        viewport_size,
        format,
        clear,
        path_samples,
        postprocess,
        intermediate_budget_bytes,
        scene_target,
    )
}

fn compile_for_scene_vnext_effects_only(
    encoding: &SceneEncoding,
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

    let mut passes: Vec<RenderPlanPass> = Vec::new();
    let mut degradations: Vec<RenderPlanDegradation> = Vec::new();
    let mut draw_scopes: Vec<DrawScope> = vec![DrawScope {
        target: scene_target,
        origin: (0, 0),
        size: viewport_size,
        needs_clear: true,
        clear_color: clear,
    }];
    let mut effect_scopes: Vec<EffectScope> = Vec::new();
    let mut composite_group_scopes: Vec<CompositeGroupScope> = Vec::new();
    let mut clip_path_scopes: Vec<ClipPathScope> = Vec::new();
    let mut clip_path_mask_in_use_bytes: u64 = 0;

    let can_allocate_intermediate_bytes =
        |draw_scopes: &[DrawScope], required: u64, extra_in_use: u64| -> bool {
            let in_use: u64 = draw_scopes
                .iter()
                .filter(|s| {
                    matches!(
                        s.target,
                        super::PlanTarget::Intermediate0
                            | super::PlanTarget::Intermediate1
                            | super::PlanTarget::Intermediate2
                    )
                })
                .map(|s| estimate_texture_bytes(s.size, format, 1))
                .sum();
            in_use.saturating_add(extra_in_use).saturating_add(required)
                <= intermediate_budget_bytes
        };

    let flush_scene_range = |end: usize,
                             passes: &mut Vec<RenderPlanPass>,
                             draw_scopes: &mut Vec<DrawScope>,
                             scene_range_start: &mut usize| {
        let scope = draw_scopes.last_mut().expect("draw scope");
        if scope.needs_clear {
            passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                segment: super::SceneSegmentId(0),
                target: scope.target,
                target_origin: scope.origin,
                target_size: scope.size,
                load: wgpu::LoadOp::Clear(scope.clear_color),
                draw_range: *scene_range_start..end,
            }));
            scope.needs_clear = false;
        } else if *scene_range_start < end {
            passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                segment: super::SceneSegmentId(0),
                target: scope.target,
                target_origin: scope.origin,
                target_size: scope.size,
                load: wgpu::LoadOp::Load,
                draw_range: *scene_range_start..end,
            }));
        }
        *scene_range_start = end;
    };

    let apply_chain_in_place = |passes: &mut Vec<RenderPlanPass>,
                                draw_scopes: &[DrawScope],
                                srcdst: super::PlanTarget,
                                chain: fret_core::EffectChain,
                                quality: fret_core::EffectQuality,
                                ctx_viewport_size: (u32, u32),
                                scissor: ScissorRect,
                                mask_uniform_index: Option<u32>| {
        if srcdst == super::PlanTarget::Output || scissor.w == 0 || scissor.h == 0 {
            return;
        }

        let in_use_targets: Vec<super::PlanTarget> = draw_scopes.iter().map(|s| s.target).collect();
        effects::apply_chain_in_place(
            passes,
            &in_use_targets,
            srcdst,
            chain,
            quality,
            scissor,
            mask_uniform_index,
            effects::EffectCompileCtx {
                viewport_size: ctx_viewport_size,
                format,
                intermediate_budget_bytes,
                clear,
            },
        );
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
            flush_scene_range(
                cursor,
                &mut passes,
                &mut draw_scopes,
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
                                ]
                                .into_iter()
                                .any(|t| {
                                    t != parent_target && !draw_scopes.iter().any(|s| s.target == t)
                                });

                                let before = passes.len();
                                apply_chain_in_place(
                                    &mut passes,
                                    &draw_scopes,
                                    parent_target,
                                    chain,
                                    quality,
                                    parent_size,
                                    scissor,
                                    Some(uniform_index),
                                );
                                if before == passes.len()
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
                                    degradations.push(RenderPlanDegradation {
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
                                    ] {
                                        if draw_scopes.iter().any(|s| s.target == t) {
                                            continue;
                                        }
                                        content_target = Some(t);
                                        had_free_target = true;
                                        break;
                                    }

                                    if content_target.is_some()
                                        && !can_allocate_intermediate_bytes(
                                            &draw_scopes,
                                            estimate_texture_bytes(content_size, format, 1),
                                            clip_path_mask_in_use_bytes,
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
                                    degradations.push(RenderPlanDegradation {
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

                            apply_chain_in_place(
                                &mut passes,
                                &draw_scopes,
                                content_target,
                                scope.chain,
                                scope.quality,
                                scope.content_size,
                                if scope.content_size == viewport_size {
                                    scope.scissor
                                } else {
                                    ScissorRect::full(scope.content_size.0, scope.content_size.1)
                                },
                                None,
                            );

                            let cropped = scope.content_origin != (0, 0)
                                || scope.content_size != viewport_size;
                            passes.push(RenderPlanPass::CompositePremul(
                                super::CompositePremulPass {
                                    src: content_target,
                                    src_origin: scope.content_origin,
                                    dst: scope.parent_target,
                                    src_size: scope.content_size,
                                    dst_origin: scope.parent_origin,
                                    dst_size: scope.parent_size,
                                    dst_scissor: cropped.then_some(scope.scissor),
                                    mask_uniform_index: Some(scope.uniform_index),
                                    mask: None,
                                    blend_mode: fret_core::BlendMode::Over,
                                    opacity: 1.0,
                                    load: wgpu::LoadOp::Load,
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
                        let parent_scope = draw_scopes.last().expect("draw scope");
                        let parent_target = parent_scope.target;
                        let parent_origin = parent_scope.origin;
                        let parent_size = parent_scope.size;

                        let mut content_target: Option<super::PlanTarget> = None;
                        let mut mask_target: Option<super::PlanTarget> = None;
                        let mut had_free_content_target = false;
                        let mut had_free_mask_target = false;

                        let (content_origin, content_size) = if scissor_sized_intermediates {
                            ((scissor.x, scissor.y), (scissor.w, scissor.h))
                        } else {
                            ((0, 0), viewport_size)
                        };
                        let mask_size = (scissor.w, scissor.h);

                        if content_size.0 != 0
                            && content_size.1 != 0
                            && mask_size.0 != 0
                            && mask_size.1 != 0
                        {
                            for t in [
                                super::PlanTarget::Intermediate0,
                                super::PlanTarget::Intermediate1,
                                super::PlanTarget::Intermediate2,
                            ] {
                                if draw_scopes.iter().any(|s| s.target == t) {
                                    continue;
                                }
                                content_target = Some(t);
                                had_free_content_target = true;
                                break;
                            }

                            for t in [
                                super::PlanTarget::Mask0,
                                super::PlanTarget::Mask1,
                                super::PlanTarget::Mask2,
                            ] {
                                if clip_path_scopes.iter().any(|s| s.mask_target == Some(t)) {
                                    continue;
                                }
                                mask_target = Some(t);
                                had_free_mask_target = true;
                                break;
                            }

                            if let (Some(_content_target), Some(_mask_target)) =
                                (content_target, mask_target)
                            {
                                let required_color =
                                    estimate_texture_bytes(content_size, format, 1);
                                let required_mask = estimate_texture_bytes(
                                    mask_size,
                                    wgpu::TextureFormat::R8Unorm,
                                    1,
                                );
                                if !can_allocate_intermediate_bytes(
                                    &draw_scopes,
                                    required_color.saturating_add(required_mask),
                                    clip_path_mask_in_use_bytes,
                                ) {
                                    content_target = None;
                                    mask_target = None;
                                }
                            }
                        }

                        if (content_target.is_none() || mask_target.is_none())
                            && content_size.0 != 0
                            && content_size.1 != 0
                            && mask_size.0 != 0
                            && mask_size.1 != 0
                        {
                            let reason = if intermediate_budget_bytes == 0 {
                                RenderPlanDegradationReason::BudgetZero
                            } else if !had_free_content_target || !had_free_mask_target {
                                RenderPlanDegradationReason::TargetExhausted
                            } else {
                                RenderPlanDegradationReason::BudgetInsufficient
                            };
                            degradations.push(RenderPlanDegradation {
                                draw_ix: cursor,
                                kind: RenderPlanDegradationKind::ClipPathDisabled,
                                reason,
                            });
                        }

                        if let (Some(content_target), Some(mask_target)) =
                            (content_target, mask_target)
                        {
                            let mask_draw = encoding.clip_path_masks[mask_draw_index as usize];
                            debug_assert_eq!(mask_draw.scissor, scissor);
                            debug_assert_eq!(mask_draw.uniform_index, uniform_index);
                            passes.push(RenderPlanPass::PathClipMask(super::PathClipMaskPass {
                                dst: mask_target,
                                dst_origin: (scissor.x, scissor.y),
                                dst_size: mask_size,
                                scissor,
                                uniform_index,
                                first_vertex: mask_draw.first_vertex,
                                vertex_count: mask_draw.vertex_count,
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            }));

                            draw_scopes.push(DrawScope {
                                target: content_target,
                                origin: content_origin,
                                size: content_size,
                                needs_clear: true,
                                clear_color: wgpu::Color::TRANSPARENT,
                            });

                            clip_path_mask_in_use_bytes = clip_path_mask_in_use_bytes
                                .saturating_add(estimate_texture_bytes(
                                    mask_size,
                                    wgpu::TextureFormat::R8Unorm,
                                    1,
                                ));
                        }

                        clip_path_scopes.push(ClipPathScope {
                            scissor,
                            uniform_index,
                            mask_draw_index,
                            parent_target,
                            parent_origin,
                            parent_size,
                            content_target,
                            content_origin,
                            content_size,
                            mask_target,
                            mask_size,
                        });
                    }
                    EffectMarkerKind::ClipPathPop => {
                        let Some(scope) = clip_path_scopes.pop() else {
                            marker_ix += 1;
                            continue;
                        };

                        if let (Some(content_target), Some(mask_target)) =
                            (scope.content_target, scope.mask_target)
                        {
                            debug_assert_eq!(
                                draw_scopes.last().expect("draw scope").target,
                                content_target
                            );

                            passes.push(RenderPlanPass::CompositePremul(
                                super::CompositePremulPass {
                                    src: content_target,
                                    src_origin: scope.content_origin,
                                    dst: scope.parent_target,
                                    src_size: scope.content_size,
                                    dst_origin: scope.parent_origin,
                                    dst_size: scope.parent_size,
                                    dst_scissor: Some(scope.scissor),
                                    mask_uniform_index: Some(scope.uniform_index),
                                    mask: Some(super::MaskRef {
                                        target: mask_target,
                                        size: scope.mask_size,
                                        viewport_rect: scope.scissor,
                                    }),
                                    blend_mode: fret_core::BlendMode::Over,
                                    opacity: 1.0,
                                    load: wgpu::LoadOp::Load,
                                },
                            ));

                            let _ = draw_scopes.pop();

                            clip_path_mask_in_use_bytes = clip_path_mask_in_use_bytes
                                .saturating_sub(estimate_texture_bytes(
                                    scope.mask_size,
                                    wgpu::TextureFormat::R8Unorm,
                                    1,
                                ));
                        } else {
                            let _ = scope.mask_draw_index;
                        }
                    }
                    EffectMarkerKind::CompositeGroupPush {
                        scissor,
                        uniform_index,
                        mode,
                        quality,
                        opacity,
                    } => {
                        let parent_scope = draw_scopes.last().expect("draw scope");
                        let parent_target = parent_scope.target;
                        let parent_origin = parent_scope.origin;
                        let parent_size = parent_scope.size;

                        let (content_origin, content_size) = if scissor_sized_intermediates {
                            ((scissor.x, scissor.y), (scissor.w, scissor.h))
                        } else {
                            ((0, 0), viewport_size)
                        };
                        let mut content_target: Option<super::PlanTarget> = None;
                        let mut had_free_target = false;
                        if content_size.0 != 0 && content_size.1 != 0 {
                            for t in [
                                super::PlanTarget::Intermediate0,
                                super::PlanTarget::Intermediate1,
                                super::PlanTarget::Intermediate2,
                            ] {
                                if draw_scopes.iter().any(|s| s.target == t) {
                                    continue;
                                }
                                content_target = Some(t);
                                had_free_target = true;
                                break;
                            }

                            if content_target.is_some()
                                && !can_allocate_intermediate_bytes(
                                    &draw_scopes,
                                    estimate_texture_bytes(content_size, format, 1),
                                    clip_path_mask_in_use_bytes,
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
                        } else if mode != fret_core::BlendMode::Over
                            && content_size.0 != 0
                            && content_size.1 != 0
                        {
                            degradations.push(RenderPlanDegradation {
                                draw_ix: cursor,
                                kind: RenderPlanDegradationKind::CompositeGroupBlendDegradedToOver,
                                reason: if !had_free_target {
                                    RenderPlanDegradationReason::TargetExhausted
                                } else if intermediate_budget_bytes == 0 {
                                    RenderPlanDegradationReason::BudgetZero
                                } else {
                                    RenderPlanDegradationReason::BudgetInsufficient
                                },
                            });
                        }

                        composite_group_scopes.push(CompositeGroupScope {
                            mode,
                            quality,
                            scissor,
                            uniform_index,
                            opacity,
                            parent_target,
                            parent_origin,
                            parent_size,
                            content_target,
                            content_origin,
                            content_size,
                        });
                    }
                    EffectMarkerKind::CompositeGroupPop => {
                        let Some(scope) = composite_group_scopes.pop() else {
                            marker_ix += 1;
                            continue;
                        };

                        if let Some(content_target) = scope.content_target {
                            debug_assert_eq!(
                                draw_scopes.last().expect("draw scope").target,
                                content_target
                            );

                            passes.push(RenderPlanPass::CompositePremul(
                                super::CompositePremulPass {
                                    src: content_target,
                                    src_origin: scope.content_origin,
                                    dst: scope.parent_target,
                                    src_size: scope.content_size,
                                    dst_origin: scope.parent_origin,
                                    dst_size: scope.parent_size,
                                    dst_scissor: Some(scope.scissor),
                                    mask_uniform_index: Some(scope.uniform_index),
                                    mask: None,
                                    blend_mode: scope.mode,
                                    opacity: scope.opacity,
                                    load: wgpu::LoadOp::Load,
                                },
                            ));

                            let _ = draw_scopes.pop();
                        } else if scope.mode != fret_core::BlendMode::Over {
                            // Degraded: no free intermediate targets, so behave as if the group
                            // was not isolated and the blend mode was `Over` (ADR 0247).
                            let _ = scope.quality;
                        }
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
            flush_scene_range(
                cursor,
                &mut passes,
                &mut draw_scopes,
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
            passes.push(RenderPlanPass::PathMsaaBatch(super::PathMsaaBatchPass {
                segment: super::SceneSegmentId(0),
                target,
                target_origin: scope.origin,
                target_size: scope.size,
                draw_range: cursor..end,
                union_scissor: union,
                batch_uniform_index,
            }));

            cursor = end;
            scene_range_start = cursor;
            continue;
        }

        cursor += 1;
    }

    super::RenderPlan::finalize(
        passes,
        viewport_size,
        postprocess,
        clear,
        format,
        degradations,
    )
}
