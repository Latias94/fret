use super::super::state::{EncodeState, transform_quad_points_px};
use super::super::*;

pub(in super::super) fn encode_mask_image(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    image: fret_core::ImageId,
    uv: UvRect,
    color: Color,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 || color.a <= 0.0 {
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

    let first_vertex = state.text_vertices.len() as u32;
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
    let mut premul = color_to_linear_rgba_premul(color);
    premul = premul.map(|c| c * o);

    let (u0, v0, u1, v1) = (uv.u0, uv.v0, uv.u1, uv.v1);
    state.text_vertices.extend_from_slice(&[
        TextVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [u1, v0],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [u0, v1],
            color: premul,
        },
    ]);

    state.ordered_draws.push(OrderedDraw::Mask(MaskDraw {
        scissor: state.current_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image,
    }));
}
