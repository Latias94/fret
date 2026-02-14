use super::state::{EncodeState, MaskPop};
use super::*;

use fret_core::scene::{MAX_STOPS, Mask, TileMode};

fn tile_mode_to_u32(m: TileMode) -> u32 {
    match m {
        TileMode::Clamp => 0,
        TileMode::Repeat => 1,
        TileMode::Mirror => 2,
    }
}

pub(super) fn push_mask(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    bounds: Rect,
    mask: Mask,
) -> bool {
    let Some(mask) = mask.sanitize() else {
        state.mask_pop_stack.push(MaskPop::NoShader);
        return true;
    };

    let (x, y, w, h) = rect_to_pixels(bounds, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        state.mask_pop_stack.push(MaskPop::NoShader);
        return true;
    }

    let t_px = state.current_transform_px();
    let Some(inv_px) = t_px.inverse() else {
        state.mask_pop_stack.push(MaskPop::NoShader);
        return true;
    };

    state.flush_quad_batch();
    let prev_head = if state.mask_count > 0 {
        state.mask_head
    } else {
        u32::MAX
    };
    let prev_mask_image = state.mask_image;

    let node_index = state.masks.len() as u32;
    let parent_bits = f32::from_bits(prev_head);

    let mut uniform = MaskGradientUniform {
        bounds: [x, y, w, h],
        kind: 0,
        tile_mode: 0,
        stop_count: 0,
        _pad0: 0,
        params0: [0.0; 4],
        inv0: [inv_px.a, inv_px.c, inv_px.tx, parent_bits],
        inv1: [inv_px.b, inv_px.d, inv_px.ty, 0.0],
        stop_alphas0: [0.0; 4],
        stop_alphas1: [0.0; 4],
        stop_offsets0: [0.0; 4],
        stop_offsets1: [0.0; 4],
    };

    match mask {
        Mask::LinearGradient(g) => {
            uniform.kind = 1;
            uniform.tile_mode = tile_mode_to_u32(g.tile_mode);
            uniform.stop_count = u32::from(g.stop_count.min(MAX_STOPS as u8));
            uniform.params0 = [
                g.start.x.0 * state.scale_factor,
                g.start.y.0 * state.scale_factor,
                g.end.x.0 * state.scale_factor,
                g.end.y.0 * state.scale_factor,
            ];

            let n = usize::from(g.stop_count).min(MAX_STOPS);
            for i in 0..n {
                let alpha = g.stops[i].color.a.clamp(0.0, 1.0);
                if i < 4 {
                    uniform.stop_alphas0[i] = alpha;
                    uniform.stop_offsets0[i] = g.stops[i].offset;
                } else {
                    let j = i - 4;
                    uniform.stop_alphas1[j] = alpha;
                    uniform.stop_offsets1[j] = g.stops[i].offset;
                }
            }
        }
        Mask::RadialGradient(g) => {
            uniform.kind = 2;
            uniform.tile_mode = tile_mode_to_u32(g.tile_mode);
            uniform.stop_count = u32::from(g.stop_count.min(MAX_STOPS as u8));
            uniform.params0 = [
                g.center.x.0 * state.scale_factor,
                g.center.y.0 * state.scale_factor,
                g.radius.width.0 * state.scale_factor,
                g.radius.height.0 * state.scale_factor,
            ];

            let n = usize::from(g.stop_count).min(MAX_STOPS);
            for i in 0..n {
                let alpha = g.stops[i].color.a.clamp(0.0, 1.0);
                if i < 4 {
                    uniform.stop_alphas0[i] = alpha;
                    uniform.stop_offsets0[i] = g.stops[i].offset;
                } else {
                    let j = i - 4;
                    uniform.stop_alphas1[j] = alpha;
                    uniform.stop_offsets1[j] = g.stops[i].offset;
                }
            }
        }
        Mask::Image { image, uv } => {
            // v1 wgpu renderer limitation: support at most one concurrently-active image mask.
            if state.mask_image.is_some() {
                state.mask_pop_stack.push(MaskPop::NoShader);
                return true;
            }

            // Missing mask source is a deterministic degrade-to-unmasked.
            if renderer.images.get(image).is_none() {
                state.mask_pop_stack.push(MaskPop::NoShader);
                return true;
            }

            uniform.kind = 3;
            uniform.stop_count = 0;
            uniform.params0 = [uv.u0, uv.v0, uv.u1, uv.v1];

            // Prefer alpha for sRGB textures (avoid sRGB conversion on RGB channels).
            let use_alpha_channel = renderer
                .images
                .format(image)
                .is_some_and(|fmt| fmt.is_srgb());
            uniform.tile_mode = u32::from(use_alpha_channel);

            state.mask_image = Some(image);
        }
    }

    state.masks.push(uniform);
    state.mask_head = node_index;
    state.mask_count = state.mask_count.saturating_add(1);
    state.current_uniform_index = state.push_uniform_snapshot(
        state.clip_head,
        state.clip_count,
        state.mask_head,
        state.mask_count,
        state.mask_scope_head,
        state.mask_scope_count,
    );
    state.mask_pop_stack.push(MaskPop::Shader {
        prev_head,
        prev_mask_image,
    });
    true
}

pub(super) fn pop_mask(state: &mut EncodeState<'_>) {
    let Some(pop) = state.mask_pop_stack.pop() else {
        return;
    };

    match pop {
        MaskPop::NoShader => {}
        MaskPop::Shader {
            prev_head,
            prev_mask_image,
        } => {
            state.flush_quad_batch();
            state.mask_count = state.mask_count.saturating_sub(1);
            state.mask_head = if state.mask_count == 0 || prev_head == u32::MAX {
                0
            } else {
                prev_head
            };
            state.mask_image = prev_mask_image;
            state.current_uniform_index = state.push_uniform_snapshot(
                state.clip_head,
                state.clip_count,
                state.mask_head,
                state.mask_count,
                state.mask_scope_head,
                state.mask_scope_count,
            );
        }
    }
}
