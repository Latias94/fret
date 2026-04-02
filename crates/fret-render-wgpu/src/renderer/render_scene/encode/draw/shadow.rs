use super::super::state::{EncodeState, transform_rows};
use super::super::*;

use fret_core::geometry::{Corners, Point, Px, Rect};
use fret_core::scene::Color;

fn transparent_paint() -> PaintGpu {
    PaintGpu {
        kind: 0,
        tile_mode: 0,
        color_space: 0,
        stop_count: 0,
        eval_space: 0,
        _pad_eval_space: [0; 3],
        params0: [0.0; 4],
        params1: [0.0; 4],
        params2: [0.0; 4],
        params3: [0.0; 4],
        stop_colors: [[0.0; 4]; fret_core::scene::MAX_STOPS],
        stop_offsets0: [0.0; 4],
        stop_offsets1: [0.0; 4],
    }
}

pub(in super::super) fn encode_shadow_rrect(
    _renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    corner_radii: Corners,
    offset: Point,
    spread: Px,
    blur_radius: Px,
    color: Color,
) {
    let opacity = state.current_opacity();
    if (color.a * opacity) <= 0.0 {
        return;
    }

    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let spread_px = spread.0 * state.scale_factor;
    let source_w = w + spread_px * 2.0;
    let source_h = h + spread_px * 2.0;
    if source_w <= 0.0 || source_h <= 0.0 {
        return;
    }

    let offset_px = [
        offset.x.0 * state.scale_factor,
        offset.y.0 * state.scale_factor,
    ];
    let blur_radius_px = blur_radius.0.max(0.0) * state.scale_factor;

    let t_px = state.to_physical_px(state.current_transform());
    let (transform0, transform1) = transform_rows(t_px);

    let base_corner_radii = corners_to_vec4(corner_radii).map(|r| r * state.scale_factor);
    let base_corner_radii = clamp_corner_radii_for_rect(w, h, base_corner_radii);

    let pipeline = QuadPipelineKey {
        fill_kind: 0,
        border_kind: 0,
        border_present: false,
        dash_enabled: false,
        fill_material_sampled: false,
        border_material_sampled: false,
        shadow_mode: true,
    };

    let needs_new_batch = match state.quad_batch {
        Some((scissor, uniform_index, prev_pipeline, _)) => {
            scissor != state.current_scissor
                || uniform_index != state.current_uniform_index
                || prev_pipeline != pipeline
        }
        None => true,
    };

    if needs_new_batch {
        state.flush_quad_batch();
        state.quad_batch = Some((
            state.current_scissor,
            state.current_uniform_index,
            pipeline,
            state.instances.len() as u32,
        ));
    }

    let shadow_color = Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * opacity).clamp(0.0, 1.0),
    };

    state.instances.push(QuadInstance {
        rect: [x, y, w, h],
        transform0,
        transform1,
        fill_paint: PaintGpu {
            params0: color_to_linear_rgba_premul(shadow_color),
            ..transparent_paint()
        },
        border_paint: transparent_paint(),
        corner_radii: base_corner_radii,
        border: [0.0; 4],
        dash_params: [0.0; 4],
        shadow_params: [offset_px[0], offset_px[1], spread_px, blur_radius_px],
    });
}
