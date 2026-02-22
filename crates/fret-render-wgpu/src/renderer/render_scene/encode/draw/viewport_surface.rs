use super::super::state::{EncodeState, bounds_of_quad_points, transform_quad_points_px};
use super::super::*;
use fret_render_core::{RenderTargetAlphaMode, RenderTargetOrientation, RenderTargetRotation};

fn orient_uv(uv: [f32; 2], orientation: RenderTargetOrientation) -> [f32; 2] {
    let mut u = uv[0];
    let mut v = uv[1];

    if orientation.mirror_x {
        u = 1.0 - u;
    }

    match orientation.rotation {
        RenderTargetRotation::R0 => {}
        // Rotate the sampled content clockwise around the UV center.
        RenderTargetRotation::R90 => {
            let uu = u;
            u = 1.0 - v;
            v = uu;
        }
        RenderTargetRotation::R180 => {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        RenderTargetRotation::R270 => {
            let uu = u;
            u = v;
            v = 1.0 - uu;
        }
    }

    [u, v]
}

pub(in super::super) fn encode_viewport_surface(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    target: fret_core::RenderTargetId,
    opacity: f32,
) {
    state.flush_quad_batch();
    let group_opacity = state.current_opacity();

    if opacity <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    if renderer
        .gpu_resources
        .registries
        .render_targets
        .get(target)
        .is_none()
    {
        return;
    }
    let metadata = renderer
        .gpu_resources
        .registries
        .render_targets
        .metadata(target)
        .unwrap_or_default();
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
    let premul_flag = match metadata.alpha_mode {
        RenderTargetAlphaMode::Premultiplied => 1.0,
        RenderTargetAlphaMode::Straight => 0.0,
    };

    let tl = orient_uv([0.0, 0.0], metadata.orientation);
    let tr = orient_uv([1.0, 0.0], metadata.orientation);
    let br = orient_uv([1.0, 1.0], metadata.orientation);
    let bl = orient_uv([0.0, 1.0], metadata.orientation);

    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: tl,
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: tr,
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: br,
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: tl,
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: br,
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: bl,
            opacity: o,
            _pad: [premul_flag, 0.0, 0.0],
        },
    ]);

    state
        .ordered_draws
        .push(OrderedDraw::Viewport(ViewportDraw {
            scissor: clipped_scissor,
            uniform_index: state.current_uniform_index,
            first_vertex,
            vertex_count: 6,
            target,
        }));
}
