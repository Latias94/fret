use super::super::state::{EncodeState, transform_rows};
use super::super::*;

use fret_core::geometry::{Corners, Edges};
use fret_core::scene::{ColorSpace, MAX_STOPS, Paint, TileMode};

pub(in super::super) fn encode_quad(
    state: &mut EncodeState<'_>,
    rect: Rect,
    background: Paint,
    border: Edges,
    border_paint: Paint,
    corner_radii: Corners,
) {
    let opacity = state.current_opacity();

    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);

    fn tile_mode_to_u32(m: TileMode) -> u32 {
        match m {
            TileMode::Clamp => 0,
            TileMode::Repeat => 1,
            TileMode::Mirror => 2,
        }
    }

    fn color_space_to_u32(c: ColorSpace) -> u32 {
        match c {
            ColorSpace::Srgb => 0,
            ColorSpace::Oklab => 1,
        }
    }

    fn mul_alpha(color: Color, opacity: f32) -> Color {
        let mut c = color;
        c.a = (c.a * opacity).clamp(0.0, 1.0);
        c
    }

    fn paint_is_visible(p: Paint, opacity: f32) -> bool {
        match p {
            Paint::Solid(c) => (c.a * opacity) > 0.0,
            Paint::LinearGradient(g) => {
                let n = usize::from(g.stop_count).min(MAX_STOPS);
                for i in 0..n {
                    if (g.stops[i].color.a * opacity) > 0.0 {
                        return true;
                    }
                }
                false
            }
            Paint::RadialGradient(g) => {
                let n = usize::from(g.stop_count).min(MAX_STOPS);
                for i in 0..n {
                    if (g.stops[i].color.a * opacity) > 0.0 {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn paint_to_gpu(p: Paint, opacity: f32, scale_factor: f32) -> PaintGpu {
        let mut out = PaintGpu {
            kind: 0,
            tile_mode: 0,
            color_space: 0,
            stop_count: 0,
            params0: [0.0; 4],
            params1: [0.0; 4],
            stop_colors: [[0.0; 4]; MAX_STOPS],
            stop_offsets0: [0.0; 4],
            stop_offsets1: [0.0; 4],
        };

        match p {
            Paint::Solid(c) => {
                let c = mul_alpha(c, opacity);
                out.kind = 0;
                out.params0 = color_to_linear_rgba_premul(c);
            }
            Paint::LinearGradient(g) => {
                out.kind = 1;
                out.tile_mode = tile_mode_to_u32(g.tile_mode);
                out.color_space = color_space_to_u32(g.color_space);
                out.stop_count = u32::from(g.stop_count.min(MAX_STOPS as u8));
                out.params0 = [
                    g.start.x.0 * scale_factor,
                    g.start.y.0 * scale_factor,
                    g.end.x.0 * scale_factor,
                    g.end.y.0 * scale_factor,
                ];

                let n = usize::from(g.stop_count).min(MAX_STOPS);
                for i in 0..n {
                    let stop = g.stops[i];
                    let c = mul_alpha(stop.color, opacity);
                    out.stop_colors[i] = color_to_linear_rgba_premul(c);
                    let offset = stop.offset.clamp(0.0, 1.0);
                    if i < 4 {
                        out.stop_offsets0[i] = offset;
                    } else {
                        out.stop_offsets1[i - 4] = offset;
                    }
                }
            }
            Paint::RadialGradient(g) => {
                out.kind = 2;
                out.tile_mode = tile_mode_to_u32(g.tile_mode);
                out.color_space = color_space_to_u32(g.color_space);
                out.stop_count = u32::from(g.stop_count.min(MAX_STOPS as u8));
                out.params0 = [
                    g.center.x.0 * scale_factor,
                    g.center.y.0 * scale_factor,
                    g.radius.width.0 * scale_factor,
                    g.radius.height.0 * scale_factor,
                ];

                let n = usize::from(g.stop_count).min(MAX_STOPS);
                for i in 0..n {
                    let stop = g.stops[i];
                    let c = mul_alpha(stop.color, opacity);
                    out.stop_colors[i] = color_to_linear_rgba_premul(c);
                    let offset = stop.offset.clamp(0.0, 1.0);
                    if i < 4 {
                        out.stop_offsets0[i] = offset;
                    } else {
                        out.stop_offsets1[i - 4] = offset;
                    }
                }
            }
        }

        out
    }

    if !paint_is_visible(background, opacity) && !paint_is_visible(border_paint, opacity) {
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
        fill_paint: paint_to_gpu(background, opacity, state.scale_factor),
        border_paint: paint_to_gpu(border_paint, opacity, state.scale_factor),
        corner_radii,
        border,
    });
}
