use super::super::state::{EncodeState, bounds_of_quad_points, transform_quad_points_px};
use super::super::*;
use crate::images::AlphaMode;

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
    if renderer.registries.images.get(image).is_none() {
        return;
    }
    let Some(source_px_size) = renderer.registries.images.size_px(image) else {
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
        renderer.registries.images.alpha_mode(image),
        Some(AlphaMode::Premultiplied)
    );
    let premul_flag = if premul { 1.0 } else { 0.0 };

    let (u0, v0, u1, v1) = (mapped.uv.u0, mapped.uv.v0, mapped.uv.u1, mapped.uv.v1);
    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [u1, v0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [u0, v1],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
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
    if renderer.registries.images.get(image).is_none() {
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
        renderer.registries.images.alpha_mode(image),
        Some(AlphaMode::Premultiplied)
    );
    let premul_flag = if premul { 1.0 } else { 0.0 };

    let (u0, v0, u1, v1) = (uv.u0, uv.v0, uv.u1, uv.v1);
    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [u1, v0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [u0, v1],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
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
