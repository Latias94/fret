use super::super::state::{EncodeState, transform_rows};
use super::super::*;

use fret_core::geometry::{Corners, Edges};

pub(in super::super) fn encode_quad(
    state: &mut EncodeState<'_>,
    rect: Rect,
    background: Color,
    border: Edges,
    border_color: Color,
    corner_radii: Corners,
) {
    let opacity = state.current_opacity();

    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    let background = EncodeState::color_with_opacity(background, opacity);
    let border_color = EncodeState::color_with_opacity(border_color, opacity);

    if background.a <= 0.0 && border_color.a <= 0.0 {
        return;
    }
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let needs_new_batch = match state.quad_batch {
        Some((scissor, uniform_index, _)) => {
            scissor != state.current_scissor || uniform_index != state.current_uniform_index
        }
        None => true,
    };

    if needs_new_batch {
        state.flush_quad_batch();
        state.quad_batch = Some((
            state.current_scissor,
            state.current_uniform_index,
            state.instances.len() as u32,
        ));
    }

    let t_px = state.to_physical_px(state.current_transform());
    let (transform0, transform1) = transform_rows(t_px);

    let corner_radii = corners_to_vec4(corner_radii).map(|r| r * state.scale_factor);
    let corner_radii = clamp_corner_radii_for_rect(w, h, corner_radii);
    let border = edges_to_vec4(border).map(|e| e * state.scale_factor);
    state.instances.push(QuadInstance {
        rect: [x, y, w, h],
        transform0,
        transform1,
        color: color_to_linear_rgba_premul(background),
        corner_radii,
        border,
        border_color: color_to_linear_rgba_premul(border_color),
    });
}
