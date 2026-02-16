use super::super::state::{
    EncodeState, apply_transform_px, bounds_of_quad_points, transform_quad_points_px,
};
use super::super::*;
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
    paint: fret_core::scene::Paint,
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

    fn mul_alpha(color: Color, opacity: f32) -> Color {
        let mut c = color;
        c.a = (c.a * opacity).clamp(0.0, 1.0);
        c
    }

    fn tile_mode_to_u32(m: fret_core::scene::TileMode) -> u32 {
        match m {
            fret_core::scene::TileMode::Clamp => 0,
            fret_core::scene::TileMode::Repeat => 1,
            fret_core::scene::TileMode::Mirror => 2,
        }
    }

    fn color_space_to_u32(c: fret_core::scene::ColorSpace) -> u32 {
        match c {
            fret_core::scene::ColorSpace::Srgb => 0,
            fret_core::scene::ColorSpace::Oklab => 1,
        }
    }

    fn paint_to_gpu(p: fret_core::scene::Paint, opacity: f32, scale_factor: f32) -> PaintGpu {
        use fret_core::scene::{MAX_STOPS, Paint};

        let mut out = PaintGpu {
            kind: 0,
            tile_mode: 0,
            color_space: 0,
            stop_count: 0,
            params0: [0.0; 4],
            params1: [0.0; 4],
            params2: [0.0; 4],
            params3: [0.0; 4],
            stop_colors: [[0.0; 4]; MAX_STOPS],
            stop_offsets0: [0.0; 4],
            stop_offsets1: [0.0; 4],
        };

        // v1: path paint supports solid + gradients. Material is deterministically degraded to
        // a solid base color (params.vec4s[0]).
        let p = match p {
            Paint::Material { params, .. } => {
                let base = params.vec4s[0];
                Paint::Solid(Color {
                    r: base[0],
                    g: base[1],
                    b: base[2],
                    a: base[3],
                })
            }
            other => other,
        };

        match p {
            Paint::Solid(c) => {
                let c = mul_alpha(c, opacity);
                out.kind = 0;
                out.params0 = color_to_linear_rgba_premul(c);
            }
            Paint::LinearGradient(g) => {
                out.kind = 1;
                out.tile_mode = tile_mode_to_u32(g.tile_mode);
                out.color_space = color_space_to_u32(g.color_space);
                out.stop_count = u32::from(g.stop_count);
                out.params0 = [
                    g.start.x.0 * scale_factor,
                    g.start.y.0 * scale_factor,
                    g.end.x.0 * scale_factor,
                    g.end.y.0 * scale_factor,
                ];

                let n = usize::from(g.stop_count).min(MAX_STOPS);
                for i in 0..n {
                    let stop = g.stops[i];
                    let c = mul_alpha(stop.color, opacity);
                    out.stop_colors[i] = color_to_linear_rgba_premul(c);
                    let offset = stop.offset.clamp(0.0, 1.0);
                    if i < 4 {
                        out.stop_offsets0[i] = offset;
                    } else {
                        out.stop_offsets1[i - 4] = offset;
                    }
                }
            }
            Paint::RadialGradient(g) => {
                out.kind = 2;
                out.tile_mode = tile_mode_to_u32(g.tile_mode);
                out.color_space = color_space_to_u32(g.color_space);
                out.stop_count = u32::from(g.stop_count);
                out.params0 = [
                    g.center.x.0 * scale_factor,
                    g.center.y.0 * scale_factor,
                    g.radius.width.0 * scale_factor,
                    g.radius.height.0 * scale_factor,
                ];

                let n = usize::from(g.stop_count).min(MAX_STOPS);
                for i in 0..n {
                    let stop = g.stops[i];
                    let c = mul_alpha(stop.color, opacity);
                    out.stop_colors[i] = color_to_linear_rgba_premul(c);
                    let offset = stop.offset.clamp(0.0, 1.0);
                    if i < 4 {
                        out.stop_offsets0[i] = offset;
                    } else {
                        out.stop_offsets1[i - 4] = offset;
                    }
                }
            }
            Paint::Material { .. } => {}
        }

        out
    }

    let paint_gpu = paint_to_gpu(paint, group_opacity, state.scale_factor);
    // Visibility early-out: for solid, alpha is encoded in params0.w (premul). For gradients,
    // scan stop alphas cheaply.
    let visible = if paint_gpu.kind == 0 {
        paint_gpu.params0[3] > 0.0
    } else {
        let mut any = false;
        for c in paint_gpu.stop_colors {
            if c[3] > 0.0 {
                any = true;
                break;
            }
        }
        any
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

    for p in &prepared.triangles {
        let lx = ox + p[0] * state.scale_factor;
        let ly = oy + p[1] * state.scale_factor;
        let (wx, wy) = apply_transform_px(t_px, lx, ly);
        state.path_vertices.push(PathVertex {
            pos_px: [wx, wy],
            local_pos_px: [lx, ly],
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

    let Some(prepared) = renderer.paths.get(path) else {
        return None;
    };
    if prepared.triangles.is_empty() {
        return None;
    }

    let t_px = state.current_transform_px();

    let first_vertex = state.path_vertices.len() as u32;
    let ox = origin.x.0 * state.scale_factor;
    let oy = origin.y.0 * state.scale_factor;

    for p in &prepared.triangles {
        let lx = ox + p[0] * state.scale_factor;
        let ly = oy + p[1] * state.scale_factor;
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
