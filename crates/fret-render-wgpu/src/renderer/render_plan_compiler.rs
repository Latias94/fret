// RenderPlan compiler.
//
// This module translates a SceneEncoding into a RenderPlan, choosing targets/passes and applying
// deterministic budget-driven degradations.

mod backdrop_source_group;
mod clip_path;
mod composite_group;
mod context;
mod draw_scope;
mod effect_scope;
mod marker_dispatch;
mod path_msaa;
mod preflight;
mod target_budget;

use super::SceneEncoding;
use super::render_plan_effects as effects;
use context::RenderPlanCompilerCtx;
use draw_scope::DrawScope;

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
    let preflight = preflight::plan_render_targets(
        encoding,
        viewport_size,
        format,
        postprocess,
        intermediate_budget_bytes,
    );

    compile_for_scene_inner(
        encoding,
        scale_factor,
        viewport_size,
        format,
        clear,
        path_samples,
        preflight.postprocess,
        intermediate_budget_bytes,
        preflight.scene_target,
        preflight.scissor_sized_intermediates,
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
    scissor_sized_intermediates: bool,
) -> super::RenderPlan {
    let draws = &encoding.ordered_draws;
    let markers = &encoding.effect_markers;
    let mut ctx = RenderPlanCompilerCtx::new();
    let mut draw_scopes: Vec<DrawScope> = vec![DrawScope {
        target: scene_target,
        origin: (0, 0),
        size: viewport_size,
        needs_clear: true,
        clear_color: clear,
    }];
    let mut marker_dispatch_state = marker_dispatch::MarkerDispatchState::new();

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
                marker_dispatch_state.compile_marker(
                    &mut ctx,
                    &mut draw_scopes,
                    encoding,
                    cursor,
                    marker,
                    marker_dispatch::MarkerDispatchCtx {
                        viewport_size,
                        scissor_sized_intermediates,
                        format,
                        clear,
                        scale_factor,
                        intermediate_budget_bytes,
                    },
                );

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

    let (effect_degradations, effect_blur_quality, effect_chain_budget_stats) =
        marker_dispatch_state.into_parts();
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
