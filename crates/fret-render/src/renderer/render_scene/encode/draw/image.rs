use super::super::state::{EncodeState, transform_quad_points_px};
use super::super::*;
use crate::images::AlphaMode;

pub(in super::super) fn encode_image(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    image: fret_core::ImageId,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    if renderer.images.get(image).is_none() {
        return;
    }
    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let t_px = state.current_transform_px();
    let quad = transform_quad_points_px(t_px, x, y, w, h);

    let first_vertex = state.viewport_vertices.len() as u32;
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
    let premul = matches!(
        renderer.images.alpha_mode(image),
        Some(AlphaMode::Premultiplied)
    );
    let premul_flag = if premul { 1.0 } else { 0.0 };

    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [1.0, 0.0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [1.0, 1.0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [1.0, 1.0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [0.0, 1.0],
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
    ]);

    state.ordered_draws.push(OrderedDraw::Image(ImageDraw {
        scissor: state.current_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image,
    }));
}

pub(in super::super) fn encode_image_region(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    image: fret_core::ImageId,
    uv: UvRect,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    if renderer.images.get(image).is_none() {
        return;
    }
    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let t_px = state.current_transform_px();
    let quad = transform_quad_points_px(t_px, x, y, w, h);

    let first_vertex = state.viewport_vertices.len() as u32;
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
    let premul = matches!(
        renderer.images.alpha_mode(image),
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
        scissor: state.current_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image,
    }));
}
