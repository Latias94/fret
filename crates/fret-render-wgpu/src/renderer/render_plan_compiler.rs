// RenderPlan compiler.
//
// This module translates a SceneEncoding into a RenderPlan, choosing targets/passes and applying
// deterministic budget-driven degradations.

mod backdrop_source_group;
mod clip_path;
mod composite_group;
mod context;
mod effect_scope;
mod path_msaa;
mod target_budget;

use super::render_plan_effects as effects;
use super::{BlurQualitySnapshot, EffectDegradationSnapshot, EffectMarkerKind, SceneEncoding};
use context::RenderPlanCompilerCtx;

#[derive(Clone, Copy, Debug)]
struct DrawScope {
    target: super::PlanTarget,
    origin: (u32, u32),
    size: (u32, u32),
    needs_clear: bool,
    clear_color: wgpu::Color,
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
    let mut effect_scopes: Vec<effect_scope::EffectScope> = Vec::new();
    let mut composite_group_scopes: Vec<composite_group::CompositeGroupScope> = Vec::new();
    let mut clip_path_scopes: Vec<clip_path::ClipPathScope> = Vec::new();
    let mut clip_path_mask_in_use_bytes: u64 = 0;
    let mut backdrop_source_group_scopes: Vec<backdrop_source_group::BackdropSourceGroupScope> =
        Vec::new();
    let mut backdrop_source_group_reserved_targets: Vec<super::PlanTarget> = Vec::new();
    let mut backdrop_source_group_in_use_bytes: u64 = 0;
    let mut effect_chain_budget_stats = effect_scope::EffectChainBudgetStats::default();

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
                        effect_scope::compile_effect_scope_push(
                            &mut ctx,
                            &mut draw_scopes,
                            &mut effect_scopes,
                            &mut effect_chain_budget_stats,
                            &mut effect_degradations,
                            &mut effect_blur_quality,
                            cursor,
                            effect_scope::EffectScopePushCtx {
                                scissor,
                                uniform_index,
                                mode,
                                chain,
                                quality,
                                viewport_size,
                                format,
                                clear,
                                scale_factor,
                                intermediate_budget_bytes,
                                clip_path_mask_in_use_bytes,
                                clip_path_scopes: &clip_path_scopes,
                                backdrop_source_group_scopes: &backdrop_source_group_scopes,
                                backdrop_source_group_reserved_targets:
                                    &backdrop_source_group_reserved_targets,
                                backdrop_source_group_in_use_bytes,
                            },
                        );
                    }
                    EffectMarkerKind::Pop => {
                        effect_scope::compile_effect_scope_pop(
                            &mut ctx,
                            &mut draw_scopes,
                            &mut effect_scopes,
                            &mut effect_chain_budget_stats,
                            &mut effect_degradations,
                            &mut effect_blur_quality,
                            cursor,
                            effect_scope::EffectScopePopCtx {
                                viewport_size,
                                format,
                                clear,
                                scale_factor,
                                intermediate_budget_bytes,
                                clip_path_mask_in_use_bytes,
                                backdrop_source_group_reserved_targets:
                                    &backdrop_source_group_reserved_targets,
                                backdrop_source_group_in_use_bytes,
                            },
                        );
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

        if let Some(end) = path_msaa::try_compile_path_msaa_batch(
            &mut ctx,
            &mut draw_scopes,
            draws,
            encoding,
            cursor,
            next_marker_at,
            path_samples,
            &mut scene_range_start,
        ) {
            cursor = end;
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

    effect_chain_budget_stats.apply_to_plan(&mut plan);

    plan
}
