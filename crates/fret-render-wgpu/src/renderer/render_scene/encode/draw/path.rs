use super::super::state::{
    EncodeState, apply_transform_px, bounds_of_quad_points, transform_quad_points_px,
};
use super::super::*;
use super::paint::{PaintMaterialPolicy, paint_is_visible, paint_to_gpu};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

fn debug_path_draw_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("FRET_DEBUG_PATH_DRAW").is_some_and(|v| !v.is_empty()))
}

fn debug_path_draw_should_log() -> bool {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) < 32
}

pub(in super::super) fn encode_path(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    path: fret_core::PathId,
    paint: fret_core::scene::PaintBindingV1,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if group_opacity <= 0.0 {
        return;
    }

    let Some(prepared) = renderer.paths.get(path) else {
        if debug_path_draw_enabled() {
            eprintln!(
                "encode_path: skipped (missing path) id={:?} origin=({:.1},{:.1}) sf={:.2}",
                path, origin.x.0, origin.y.0, state.scale_factor
            );
        }
        return;
    };

    let stroke_s01_supported = paint.eval_space == fret_core::scene::PaintEvalSpaceV1::StrokeS01
        && prepared.stroke_s01_mode == crate::renderer::path::StrokeS01ModeV1::Continuous;

    let paint = if paint.eval_space == fret_core::scene::PaintEvalSpaceV1::StrokeS01
        && !stroke_s01_supported
    {
        fret_core::scene::PaintBindingV1 {
            paint: paint.paint,
            eval_space: fret_core::scene::PaintEvalSpaceV1::LocalPx,
        }
    } else {
        paint
    };

    if !paint_is_visible(paint, group_opacity) {
        return;
    }
    if prepared.triangles.is_empty() {
        if debug_path_draw_enabled() {
            let b = prepared.metrics.bounds;
            eprintln!(
                "encode_path: skipped (empty triangles) id={:?} origin=({:.1},{:.1}) bounds=({:.1},{:.1} {:.1}x{:.1}) sf={:.2}",
                path,
                origin.x.0,
                origin.y.0,
                b.origin.x.0,
                b.origin.y.0,
                b.size.width.0,
                b.size.height.0,
                state.scale_factor
            );
        }
        return;
    }
    let t_px = state.current_transform_px();

    let material_requested = matches!(paint.paint, fret_core::scene::Paint::Material { .. });
    let paint_gpu = paint_to_gpu(
        renderer,
        state,
        paint,
        group_opacity,
        state.scale_factor,
        PaintMaterialPolicy::Allow,
    );

    if material_requested && paint_gpu.kind != 3 {
        *state.path_material_paints_degraded_to_solid_base = state
            .path_material_paints_degraded_to_solid_base
            .saturating_add(1);
    }

    // Visibility early-out (after encoding): for solid, alpha is params0.w (premul). For gradients,
    // scan stop alphas cheaply. For materials, alpha may be carried by both base + fg paints.
    let visible = match paint_gpu.kind {
        0 => paint_gpu.params0[3] > 0.0,
        3 => paint_gpu.params0[3] > 0.0 || paint_gpu.params1[3] > 0.0,
        _ => paint_gpu.stop_colors.iter().any(|c| c[3] > 0.0),
    };
    if !visible {
        return;
    }

    let local_bounds = Rect::new(
        Point::new(
            origin.x + prepared.metrics.bounds.origin.x,
            origin.y + prepared.metrics.bounds.origin.y,
        ),
        prepared.metrics.bounds.size,
    );
    let (bx, by, bw, bh) = rect_to_pixels(local_bounds, state.scale_factor);
    let bounds_quad = transform_quad_points_px(t_px, bx, by, bw, bh);
    let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&bounds_quad);
    let Some(bounds_scissor) =
        scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
    else {
        if debug_path_draw_enabled() {
            eprintln!(
                "encode_path: skipped (scissor_from_bounds_px returned None) id={:?} bounds_px=({:.1},{:.1}..{:.1},{:.1}) viewport={:?}",
                path, min_x, min_y, max_x, max_y, state.viewport_size
            );
        }
        return;
    };
    let clipped_scissor = intersect_scissor(state.current_scissor, bounds_scissor);
    if clipped_scissor.w == 0 || clipped_scissor.h == 0 {
        if debug_path_draw_enabled() {
            eprintln!(
                "encode_path: skipped (empty scissor) id={:?} current={:?} bounds={:?} clipped={:?}",
                path, state.current_scissor, bounds_scissor, clipped_scissor
            );
        }
        return;
    }

    if debug_path_draw_enabled() && debug_path_draw_should_log() {
        let b = prepared.metrics.bounds;
        eprintln!(
            "encode_path: draw id={:?} origin=({:.1},{:.1}) bounds=({:.1},{:.1} {:.1}x{:.1}) tris={} paint_kind={} scissor={:?}",
            path,
            origin.x.0,
            origin.y.0,
            b.origin.x.0,
            b.origin.y.0,
            b.size.width.0,
            b.size.height.0,
            prepared.triangles.len(),
            paint_gpu.kind,
            clipped_scissor
        );
    }

    let first_vertex = state.path_vertices.len() as u32;
    let ox = origin.x.0 * state.scale_factor;
    let oy = origin.y.0 * state.scale_factor;
    let paint_index = state.path_paints.len().min(u32::MAX as usize) as u32;
    state.path_paints.push(paint_gpu);

    for v in &prepared.triangles {
        let lx = ox + v.pos[0] * state.scale_factor;
        let ly = oy + v.pos[1] * state.scale_factor;
        let (wx, wy) = apply_transform_px(t_px, lx, ly);
        state.path_vertices.push(PathVertex {
            pos_px: [wx, wy],
            local_pos_px: if stroke_s01_supported {
                [v.stroke_s01, 0.0]
            } else {
                [lx, ly]
            },
        });
    }

    let vertex_count = (state.path_vertices.len() as u32).saturating_sub(first_vertex);
    if vertex_count > 0 {
        state.ordered_draws.push(OrderedDraw::Path(PathDraw {
            scissor: clipped_scissor,
            uniform_index: state.current_uniform_index,
            first_vertex,
            vertex_count,
            paint_index,
        }));
    }
}

pub(in super::super) fn encode_clip_path_mask(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    path: fret_core::PathId,
) -> Option<(u32, u32)> {
    state.flush_quad_batch();

    let prepared = renderer.paths.get(path)?;
    if prepared.triangles.is_empty() {
        return None;
    }

    let t_px = state.current_transform_px();

    let first_vertex = state.path_vertices.len() as u32;
    let ox = origin.x.0 * state.scale_factor;
    let oy = origin.y.0 * state.scale_factor;

    for v in &prepared.triangles {
        let lx = ox + v.pos[0] * state.scale_factor;
        let ly = oy + v.pos[1] * state.scale_factor;
        let (wx, wy) = apply_transform_px(t_px, lx, ly);
        state.path_vertices.push(PathVertex {
            pos_px: [wx, wy],
            local_pos_px: [0.0; 2],
        });
    }

    let vertex_count = (state.path_vertices.len() as u32).saturating_sub(first_vertex);
    if vertex_count == 0 {
        return None;
    }

    Some((first_vertex, vertex_count))
}
