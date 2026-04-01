use super::super::state::{EncodeState, transform_rows};
use super::super::*;

use fret_core::geometry::{Corners, Edges};
use fret_core::scene::{DashPatternV1, MAX_STOPS, PaintBindingV1, PaintEvalSpaceV1};

use super::paint::{PaintMaterialPolicy, paint_is_visible, paint_to_gpu};

fn normalize_eval_space_for_quad_fill(p: PaintBindingV1) -> PaintBindingV1 {
    match p.eval_space {
        PaintEvalSpaceV1::StrokeS01 => PaintBindingV1 {
            paint: p.paint,
            eval_space: PaintEvalSpaceV1::LocalPx,
        },
        _ => p,
    }
}

pub(in super::super) fn encode_quad(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    background: PaintBindingV1,
    border: Edges,
    border_paint: PaintBindingV1,
    corner_radii: Corners,
    dash: Option<DashPatternV1>,
) {
    let opacity = state.current_opacity();

    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);

    let background = normalize_eval_space_for_quad_fill(background);
    if !paint_is_visible(background, opacity) && !paint_is_visible(border_paint, opacity) {
        return;
    }
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let t_px = state.to_physical_px(state.current_transform());
    let (transform0, transform1) = transform_rows(t_px);

    let corner_radii = corners_to_vec4(corner_radii).map(|r| r * state.scale_factor);
    let corner_radii = clamp_corner_radii_for_rect(w, h, corner_radii);
    let border = edges_to_vec4(border).map(|e| e * state.scale_factor);
    let border_present = border[0] > 0.0 || border[1] > 0.0 || border[2] > 0.0 || border[3] > 0.0;

    let fill_visible = paint_is_visible(background, opacity);
    let border_visible = border_present && paint_is_visible(border_paint, opacity);
    if !fill_visible && !border_visible {
        return;
    }

    let dash_params = dash.map(|d| {
        let dash_px = (d.dash.0 * state.scale_factor).max(0.0);
        let gap_px = (d.gap.0 * state.scale_factor).max(0.0);
        let phase_px = d.phase.0 * state.scale_factor;
        let enabled = (dash_px > 0.0 && gap_px.is_finite() && phase_px.is_finite()) as u32;
        [
            dash_px,
            gap_px,
            phase_px,
            if enabled != 0 { 1.0 } else { 0.0 },
        ]
    });
    let dash_params = dash_params.unwrap_or([0.0, 0.0, 0.0, 0.0]);

    let fill_paint_gpu = paint_to_gpu(
        renderer,
        state,
        background,
        opacity,
        state.scale_factor,
        PaintMaterialPolicy::Allow,
    );
    let border_paint_gpu = if border_present {
        paint_to_gpu(
            renderer,
            state,
            border_paint,
            opacity,
            state.scale_factor,
            PaintMaterialPolicy::Allow,
        )
    } else {
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
            stop_colors: [[0.0; 4]; MAX_STOPS],
            stop_offsets0: [0.0; 4],
            stop_offsets1: [0.0; 4],
        }
    };
    if fill_paint_gpu.kind == 3 || (border_present && border_paint_gpu.kind == 3) {
        *state.material_quad_ops = state.material_quad_ops.saturating_add(1);
        if (fill_paint_gpu.kind == 3 && fill_paint_gpu.stop_count == 1)
            || (border_present && border_paint_gpu.kind == 3 && border_paint_gpu.stop_count == 1)
        {
            *state.material_sampled_quad_ops = state.material_sampled_quad_ops.saturating_add(1);
        }
    }

    let dash_enabled = border_present && dash_params[3] > 0.5;

    let fill_kind = fill_paint_gpu.kind.min(4) as u8;
    let border_kind = if border_present {
        border_paint_gpu.kind.min(4) as u8
    } else {
        0
    };

    let fill_material_sampled = fill_kind == 3 && fill_paint_gpu.stop_count == 1;
    let border_material_sampled =
        border_present && border_kind == 3 && border_paint_gpu.stop_count == 1;

    let pipeline = QuadPipelineKey {
        fill_kind,
        border_kind,
        border_present,
        dash_enabled,
        fill_material_sampled,
        border_material_sampled,
        shadow_mode: false,
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

    state.instances.push(QuadInstance {
        rect: [x, y, w, h],
        transform0,
        transform1,
        fill_paint: fill_paint_gpu,
        border_paint: border_paint_gpu,
        corner_radii,
        border,
        dash_params,
        shadow_params: [0.0; 4],
    });
}
