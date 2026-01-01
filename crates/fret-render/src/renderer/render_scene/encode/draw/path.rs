use super::super::state::{
    EncodeState, apply_transform_px, bounds_of_quad_points, transform_quad_points_px,
};
use super::super::*;

pub(in super::super) fn encode_path(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    path: fret_core::PathId,
    color: Color,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if color.a <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    let Some(prepared) = renderer.paths.get(path) else {
        return;
    };
    if prepared.triangles.is_empty() {
        return;
    }
    let t_px = state.current_transform_px();

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
        return;
    };
    let clipped_scissor = intersect_scissor(state.current_scissor, bounds_scissor);
    if clipped_scissor.w == 0 || clipped_scissor.h == 0 {
        return;
    }

    let first_vertex = state.path_vertices.len() as u32;
    let ox = origin.x.0 * state.scale_factor;
    let oy = origin.y.0 * state.scale_factor;
    let premul = color_to_linear_rgba_premul(EncodeState::color_with_opacity(color, group_opacity));

    for p in &prepared.triangles {
        let lx = ox + p[0] * state.scale_factor;
        let ly = oy + p[1] * state.scale_factor;
        let (wx, wy) = apply_transform_px(t_px, lx, ly);
        state.path_vertices.push(PathVertex {
            pos_px: [wx, wy],
            color: premul,
        });
    }

    let vertex_count = (state.path_vertices.len() as u32).saturating_sub(first_vertex);
    if vertex_count > 0 {
        state.ordered_draws.push(OrderedDraw::Path(PathDraw {
            scissor: clipped_scissor,
            uniform_index: state.current_uniform_index,
            first_vertex,
            vertex_count,
        }));
    }
}
