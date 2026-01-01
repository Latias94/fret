use super::state::{ClipPop, EncodeState, bounds_of_quad_points, transform_quad_points_px};
use super::*;

use fret_core::geometry::Corners;

pub(super) fn push_clip_rect(state: &mut EncodeState<'_>, rect: Rect) -> bool {
    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    let new_scissor = if w <= 0.0 || h <= 0.0 {
        Some(ScissorRect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        })
    } else {
        let t_px = state.current_transform_px();
        let quad = transform_quad_points_px(t_px, x, y, w, h);
        let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
        scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
    };
    let Some(new_scissor) = new_scissor else {
        return false;
    };

    let combined = intersect_scissor(state.current_scissor, new_scissor);
    if combined != state.current_scissor {
        state.flush_quad_batch();
    }

    state.current_scissor = combined;
    state.scissor_stack.push(state.current_scissor);

    if w <= 0.0 || h <= 0.0 {
        state.clip_pop_stack.push(ClipPop::NoShader);
        return true;
    }

    let t_px = state.current_transform_px();
    let is_axis_aligned = t_px.b == 0.0 && t_px.c == 0.0;
    if is_axis_aligned {
        state.clip_pop_stack.push(ClipPop::NoShader);
        return true;
    }

    let Some(inv_px) = t_px.inverse() else {
        state.clip_pop_stack.push(ClipPop::NoShader);
        return true;
    };

    state.flush_quad_batch();
    let prev_head = if state.clip_count > 0 {
        state.clip_head
    } else {
        u32::MAX
    };
    let node_index = state.clips.len() as u32;
    let parent_bits = f32::from_bits(prev_head);
    state.clips.push(ClipRRectUniform {
        rect: [x, y, w, h],
        corner_radii: [0.0; 4],
        inv0: [inv_px.a, inv_px.c, inv_px.tx, parent_bits],
        inv1: [inv_px.b, inv_px.d, inv_px.ty, 0.0],
    });
    state.clip_head = node_index;
    state.clip_count = state.clip_count.saturating_add(1);
    state.current_uniform_index = state.push_uniform_snapshot(state.clip_head, state.clip_count);
    state.clip_pop_stack.push(ClipPop::Shader { prev_head });
    true
}

pub(super) fn push_clip_rrect(
    state: &mut EncodeState<'_>,
    rect: Rect,
    corner_radii: Corners,
) -> bool {
    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    let radii = corners_to_vec4(corner_radii).map(|r| r * state.scale_factor);
    let radii = if w > 0.0 && h > 0.0 {
        clamp_corner_radii_for_rect(w, h, radii)
    } else {
        [0.0; 4]
    };

    let new_scissor = if w <= 0.0 || h <= 0.0 {
        Some(ScissorRect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        })
    } else {
        let t_px = state.current_transform_px();
        let quad = transform_quad_points_px(t_px, x, y, w, h);
        let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
        scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
    };
    let Some(new_scissor) = new_scissor else {
        return false;
    };

    let combined = intersect_scissor(state.current_scissor, new_scissor);
    if combined != state.current_scissor {
        state.flush_quad_batch();
    }

    state.current_scissor = combined;
    state.scissor_stack.push(state.current_scissor);

    if w <= 0.0 || h <= 0.0 {
        state.clip_pop_stack.push(ClipPop::NoShader);
        return true;
    }

    let t_px = state.current_transform_px();
    let is_axis_aligned = t_px.b == 0.0 && t_px.c == 0.0;
    let is_rect = radii.iter().all(|r| *r <= 0.0);
    if is_axis_aligned && is_rect {
        state.clip_pop_stack.push(ClipPop::NoShader);
        return true;
    }

    let Some(inv_px) = t_px.inverse() else {
        state.clip_pop_stack.push(ClipPop::NoShader);
        return true;
    };

    state.flush_quad_batch();
    let prev_head = if state.clip_count > 0 {
        state.clip_head
    } else {
        u32::MAX
    };
    let node_index = state.clips.len() as u32;
    let parent_bits = f32::from_bits(prev_head);
    state.clips.push(ClipRRectUniform {
        rect: [x, y, w, h],
        corner_radii: radii,
        inv0: [inv_px.a, inv_px.c, inv_px.tx, parent_bits],
        inv1: [inv_px.b, inv_px.d, inv_px.ty, 0.0],
    });
    state.clip_head = node_index;
    state.clip_count = state.clip_count.saturating_add(1);
    state.current_uniform_index = state.push_uniform_snapshot(state.clip_head, state.clip_count);
    state.clip_pop_stack.push(ClipPop::Shader { prev_head });
    true
}

pub(super) fn pop_clip(state: &mut EncodeState<'_>) {
    if state.scissor_stack.len() > 1 {
        state.scissor_stack.pop();
        let new_scissor = *state
            .scissor_stack
            .last()
            .expect("scissor stack must be non-empty");
        if new_scissor != state.current_scissor {
            state.flush_quad_batch();
            state.current_scissor = new_scissor;
        }
    }

    if let Some(ClipPop::Shader { prev_head }) = state.clip_pop_stack.pop() {
        state.flush_quad_batch();
        state.clip_count = state.clip_count.saturating_sub(1);
        state.clip_head = if state.clip_count == 0 || prev_head == u32::MAX {
            0
        } else {
            prev_head
        };
        state.current_uniform_index =
            state.push_uniform_snapshot(state.clip_head, state.clip_count);
    }
}
