use super::super::state::bounds_of_quad_points;
use super::super::state::bounds_of_triangle_points;
use super::super::*;
use super::image::{transform_points_px, vertex_color};

pub(in super::super) fn encode_vertex_color_quad(
    state: &mut EncodeState<'_>,
    points: [Point; 4],
    colors: [Color; 4],
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if group_opacity <= 0.0 || colors.iter().all(|color| color.a <= 0.0) {
        return;
    }

    let t_px = state.current_transform_px();
    let quad = transform_points_px(t_px, state.scale_factor, points);
    let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
    let Some(bounds_scissor) =
        scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
    else {
        return;
    };
    let clipped_scissor = intersect_scissor(state.current_scissor, bounds_scissor);
    if clipped_scissor.w == 0 || clipped_scissor.h == 0 {
        return;
    }

    let first_vertex = state.viewport_vertices.len() as u32;
    let opacity = group_opacity.clamp(0.0, 1.0);
    let colors = colors.map(vertex_color);

    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity,
            premul: 0.0,
            color: colors[0],
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [0.0, 0.0],
            opacity,
            premul: 0.0,
            color: colors[1],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [0.0, 0.0],
            opacity,
            premul: 0.0,
            color: colors[2],
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity,
            premul: 0.0,
            color: colors[0],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [0.0, 0.0],
            opacity,
            premul: 0.0,
            color: colors[2],
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [0.0, 0.0],
            opacity,
            premul: 0.0,
            color: colors[3],
        },
    ]);

    state
        .ordered_draws
        .push(OrderedDraw::VertexColor(VertexColorDraw {
            scissor: clipped_scissor,
            uniform_index: state.current_uniform_index,
            first_vertex,
            vertex_count: 6,
        }));
}

pub(in super::super) fn encode_vertex_color_triangle(
    state: &mut EncodeState<'_>,
    vertices: [fret_core::SceneMeshVertex; 3],
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if group_opacity <= 0.0 || vertices.iter().all(|vertex| vertex.color.a <= 0.0) {
        return;
    }

    let t_px = state.current_transform_px();
    let triangle = vertices.map(|vertex| {
        super::super::state::apply_transform_px(
            t_px,
            vertex.position.x.0 * state.scale_factor,
            vertex.position.y.0 * state.scale_factor,
        )
    });
    let (min_x, min_y, max_x, max_y) = bounds_of_triangle_points(&triangle);
    let Some(bounds_scissor) =
        scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
    else {
        return;
    };
    let clipped_scissor = intersect_scissor(state.current_scissor, bounds_scissor);
    if clipped_scissor.w == 0 || clipped_scissor.h == 0 {
        return;
    }

    let first_vertex = state.viewport_vertices.len() as u32;
    let opacity = group_opacity.clamp(0.0, 1.0);
    let colors = vertices.map(|vertex| vertex_color(vertex.color));

    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [triangle[0].0, triangle[0].1],
            uv: [vertices[0].uv.u, vertices[0].uv.v],
            opacity,
            premul: 0.0,
            color: colors[0],
        },
        ViewportVertex {
            pos_px: [triangle[1].0, triangle[1].1],
            uv: [vertices[1].uv.u, vertices[1].uv.v],
            opacity,
            premul: 0.0,
            color: colors[1],
        },
        ViewportVertex {
            pos_px: [triangle[2].0, triangle[2].1],
            uv: [vertices[2].uv.u, vertices[2].uv.v],
            opacity,
            premul: 0.0,
            color: colors[2],
        },
    ]);

    state
        .ordered_draws
        .push(OrderedDraw::VertexColor(VertexColorDraw {
            scissor: clipped_scissor,
            uniform_index: state.current_uniform_index,
            first_vertex,
            vertex_count: 3,
        }));
}
