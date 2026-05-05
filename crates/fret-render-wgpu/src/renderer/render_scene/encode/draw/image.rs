use super::super::state::{
    EncodeState, apply_transform_px, bounds_of_quad_points, bounds_of_triangle_points,
    transform_quad_points_px,
};
use super::super::*;
use crate::images::AlphaMode;

const WHITE_VERTEX_COLOR: [f32; 4] = [1.0; 4];

pub(in super::super) fn encode_image(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    image: fret_core::ImageId,
    fit: fret_core::ViewportFit,
    sampling: fret_core::scene::ImageSamplingHint,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    if renderer.gpu_resources.image_view(image).is_none() {
        return;
    }
    let Some(source_px_size) = renderer.gpu_resources.image_size_px(image) else {
        return;
    };
    let Some(mapped) = fret_core::scene::map_image_object_fit(rect, source_px_size, fit) else {
        return;
    };

    let (x, y, w, h) = rect_to_pixels(mapped.draw_rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let t_px = state.current_transform_px();
    let quad = transform_quad_points_px(t_px, x, y, w, h);
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
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
    let premul = matches!(
        renderer.gpu_resources.image_alpha_mode(image),
        Some(AlphaMode::Premultiplied)
    );
    let premul_flag = if premul { 1.0 } else { 0.0 };

    let (u0, v0, u1, v1) = (mapped.uv.u0, mapped.uv.v0, mapped.uv.u1, mapped.uv.v1);
    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [u1, v0],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [u0, v1],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
    ]);

    state.ordered_draws.push(OrderedDraw::Image(ImageDraw {
        scissor: clipped_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image,
        sampling,
    }));
}

pub(in super::super) fn encode_image_region(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    image: fret_core::ImageId,
    uv: UvRect,
    sampling: fret_core::scene::ImageSamplingHint,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    if renderer.gpu_resources.image_view(image).is_none() {
        return;
    }
    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let t_px = state.current_transform_px();
    let quad = transform_quad_points_px(t_px, x, y, w, h);
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
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
    let premul = matches!(
        renderer.gpu_resources.image_alpha_mode(image),
        Some(AlphaMode::Premultiplied)
    );
    let premul_flag = if premul { 1.0 } else { 0.0 };

    let (u0, v0, u1, v1) = (uv.u0, uv.v0, uv.u1, uv.v1);
    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [u1, v0],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [u0, v1],
            opacity: o,
            premul: premul_flag,
            color: WHITE_VERTEX_COLOR,
        },
    ]);

    state.ordered_draws.push(OrderedDraw::Image(ImageDraw {
        scissor: clipped_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image,
        sampling,
    }));
}

pub(in super::super) fn encode_image_quad(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    points: [Point; 4],
    image: fret_core::ImageId,
    uvs: [UvPoint; 4],
    sampling: fret_core::scene::ImageSamplingHint,
    tint: Color,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || tint.a <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    if renderer.gpu_resources.image_view(image).is_none() {
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
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
    let premul = matches!(
        renderer.gpu_resources.image_alpha_mode(image),
        Some(AlphaMode::Premultiplied)
    );
    let premul_flag = if premul { 1.0 } else { 0.0 };
    let tint = vertex_color(tint);
    let uv = |i: usize| [uvs[i].u, uvs[i].v];

    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: uv(0),
            opacity: o,
            premul: premul_flag,
            color: tint,
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: uv(1),
            opacity: o,
            premul: premul_flag,
            color: tint,
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: uv(2),
            opacity: o,
            premul: premul_flag,
            color: tint,
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: uv(0),
            opacity: o,
            premul: premul_flag,
            color: tint,
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: uv(2),
            opacity: o,
            premul: premul_flag,
            color: tint,
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: uv(3),
            opacity: o,
            premul: premul_flag,
            color: tint,
        },
    ]);

    state.ordered_draws.push(OrderedDraw::Image(ImageDraw {
        scissor: clipped_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image,
        sampling,
    }));
}

pub(in super::super) fn encode_image_triangle(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    image: fret_core::ImageId,
    vertices: [fret_core::SceneMeshVertex; 3],
    sampling: fret_core::scene::ImageSamplingHint,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 || vertices.iter().all(|vertex| vertex.color.a <= 0.0)
    {
        return;
    }
    if renderer.gpu_resources.image_view(image).is_none() {
        return;
    }

    let t_px = state.current_transform_px();
    let triangle = vertices.map(|vertex| {
        apply_transform_px(
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
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
    let premul = matches!(
        renderer.gpu_resources.image_alpha_mode(image),
        Some(AlphaMode::Premultiplied)
    );
    let premul_flag = if premul { 1.0 } else { 0.0 };
    let colors = vertices.map(|vertex| vertex_color(vertex.color));

    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [triangle[0].0, triangle[0].1],
            uv: [vertices[0].uv.u, vertices[0].uv.v],
            opacity: o,
            premul: premul_flag,
            color: colors[0],
        },
        ViewportVertex {
            pos_px: [triangle[1].0, triangle[1].1],
            uv: [vertices[1].uv.u, vertices[1].uv.v],
            opacity: o,
            premul: premul_flag,
            color: colors[1],
        },
        ViewportVertex {
            pos_px: [triangle[2].0, triangle[2].1],
            uv: [vertices[2].uv.u, vertices[2].uv.v],
            opacity: o,
            premul: premul_flag,
            color: colors[2],
        },
    ]);

    state.ordered_draws.push(OrderedDraw::Image(ImageDraw {
        scissor: clipped_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 3,
        image,
        sampling,
    }));
}

pub(super) fn transform_points_px(
    t_px: Transform2D,
    scale_factor: f32,
    points: [Point; 4],
) -> [(f32, f32); 4] {
    points.map(|point| apply_transform_px(t_px, point.x.0 * scale_factor, point.y.0 * scale_factor))
}

pub(super) fn vertex_color(color: Color) -> [f32; 4] {
    [
        color.r.clamp(0.0, 1.0),
        color.g.clamp(0.0, 1.0),
        color.b.clamp(0.0, 1.0),
        color.a.clamp(0.0, 1.0),
    ]
}
