use super::super::state::{EncodeState, transform_rows};
use super::super::*;

use fret_core::geometry::{Corners, Edges};
use fret_core::scene::{ColorSpace, MAX_STOPS, MaterialParams, Paint, TileMode};

pub(in super::super) fn encode_quad(
    renderer: &Renderer,
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

    fn material_kind_to_u32(kind: fret_core::MaterialKind) -> u32 {
        match kind {
            fret_core::MaterialKind::DotGrid => 0,
            fret_core::MaterialKind::Grid => 1,
            fret_core::MaterialKind::Checkerboard => 2,
            fret_core::MaterialKind::Stripe => 3,
            fret_core::MaterialKind::Noise => 4,
            fret_core::MaterialKind::Beam => 5,
            fret_core::MaterialKind::Sparkle => 6,
            fret_core::MaterialKind::ConicSweep => 7,
        }
    }

    fn catalog_texture_kind_to_layer(kind: fret_core::MaterialCatalogTextureKind) -> u32 {
        match kind {
            fret_core::MaterialCatalogTextureKind::BlueNoise64x64R8 => 0,
            fret_core::MaterialCatalogTextureKind::Bayer8x8R8 => 1,
        }
    }

    fn material_colors_from_params(params: MaterialParams) -> (Color, Color) {
        let base = params.vec4s[0];
        let fg = params.vec4s[1];
        (
            Color {
                r: base[0],
                g: base[1],
                b: base[2],
                a: base[3],
            },
            Color {
                r: fg[0],
                g: fg[1],
                b: fg[2],
                a: fg[3],
            },
        )
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
            Paint::Material { params, .. } => {
                let (base, fg) = material_colors_from_params(params);
                (base.a * opacity) > 0.0 || (fg.a * opacity) > 0.0
            }
        }
    }

    fn paint_to_gpu(
        renderer: &Renderer,
        state: &mut EncodeState<'_>,
        p: Paint,
        opacity: f32,
        scale_factor: f32,
    ) -> PaintGpu {
        let mut out = PaintGpu {
            kind: 0,
            tile_mode: 0,
            color_space: 0,
            stop_count: 0,
            params0: [0.0; 4],
            params1: [0.0; 4],
            params2: [0.0; 4],
            params3: [0.0; 4],
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
            Paint::Material { id, params } => {
                let Some(entry) = renderer.materials.get(id) else {
                    *state.material_unknown_ids = state.material_unknown_ids.saturating_add(1);
                    out.kind = 0;
                    out.params0 = [0.0; 4];
                    return out;
                };

                let (base, fg) = material_colors_from_params(params);

                let is_new_distinct = !state.material_seen.contains(&id);
                if is_new_distinct
                    && state.material_seen.len() >= state.material_distinct_budget_per_frame
                {
                    *state.material_degraded_due_to_budget =
                        state.material_degraded_due_to_budget.saturating_add(1);
                    out.kind = 0;
                    out.params0 = color_to_linear_rgba_premul(mul_alpha(base, opacity));
                    return out;
                }

                if state.material_paints_used >= state.material_paint_budget_per_frame {
                    *state.material_degraded_due_to_budget =
                        state.material_degraded_due_to_budget.saturating_add(1);
                    out.kind = 0;
                    out.params0 = color_to_linear_rgba_premul(mul_alpha(base, opacity));
                    return out;
                }

                state.material_paints_used = state.material_paints_used.saturating_add(1);
                if is_new_distinct {
                    state.material_seen.push(id);
                    *state.material_distinct = state.material_seen.len() as u64;
                }

                out.kind = 3;
                out.tile_mode = material_kind_to_u32(entry.desc.kind);
                match entry.desc.binding {
                    fret_core::MaterialBindingShape::ParamsOnly => {
                        out.stop_count = 0;
                        out.color_space = 0;
                    }
                    fret_core::MaterialBindingShape::ParamsPlusCatalogTexture { texture } => {
                        // For materials, `stop_count` and `color_space` are repurposed as a small
                        // aux channel to select the fixed v2 binding behavior without changing the
                        // instance payload size.
                        out.stop_count = 1;
                        out.color_space = catalog_texture_kind_to_layer(texture);
                    }
                }
                out.params0 = color_to_linear_rgba_premul(mul_alpha(base, opacity));
                out.params1 = color_to_linear_rgba_premul(mul_alpha(fg, opacity));

                // params2/3 are material-kind-specific. v1 treats values as logical px by default.
                // Each material kind may reinterpret these values (angles, normalized coords, etc).
                out.params2 = params.vec4s[2];
                out.params3 = params.vec4s[3];

                // Apply a conservative default scaling to px-like fields where it is unambiguous.
                // For v1 baseline kinds, we treat `params2.xy` as logical px sizes/spacing.
                out.params2[0] *= scale_factor;
                out.params2[1] *= scale_factor;
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
    let fill_paint_gpu = paint_to_gpu(renderer, state, background, opacity, state.scale_factor);
    let border_paint_gpu = paint_to_gpu(renderer, state, border_paint, opacity, state.scale_factor);
    if fill_paint_gpu.kind == 3 || border_paint_gpu.kind == 3 {
        *state.material_quad_ops = state.material_quad_ops.saturating_add(1);
        if (fill_paint_gpu.kind == 3 && fill_paint_gpu.stop_count == 1)
            || (border_paint_gpu.kind == 3 && border_paint_gpu.stop_count == 1)
        {
            *state.material_sampled_quad_ops = state.material_sampled_quad_ops.saturating_add(1);
        }
    }
    state.instances.push(QuadInstance {
        rect: [x, y, w, h],
        transform0,
        transform1,
        fill_paint: fill_paint_gpu,
        border_paint: border_paint_gpu,
        corner_radii,
        border,
    });
}
