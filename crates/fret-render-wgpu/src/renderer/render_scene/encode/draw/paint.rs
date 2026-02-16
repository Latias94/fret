use super::super::state::EncodeState;
use super::super::*;

use fret_core::scene::{ColorSpace, MAX_STOPS, MaterialParams, Paint, TileMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PaintMaterialPolicy {
    /// Encode `Paint::Material` as a material paint (`PaintGpu.kind = 3`) and apply the existing
    /// per-frame budgets + deterministic degradations.
    Allow,
    /// Deterministically degrade `Paint::Material` to a solid base color (`params.vec4s[0]`).
    ///
    /// This is the v1 contract behavior for paint surfaces that do not yet support materials
    /// (e.g. path/text), while keeping `Paint` as a shared contract type.
    DegradeToSolidBase,
}

pub(super) fn paint_is_visible(p: Paint, opacity: f32) -> bool {
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
        Paint::SweepGradient(g) => {
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

pub(super) fn paint_to_gpu(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    paint: Paint,
    opacity: f32,
    scale_factor: f32,
    material_policy: PaintMaterialPolicy,
) -> PaintGpu {
    use fret_core::scene::Paint;

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

    let paint = match (material_policy, paint) {
        (PaintMaterialPolicy::DegradeToSolidBase, Paint::Material { params, .. }) => {
            let base = params.vec4s[0];
            Paint::Solid(Color {
                r: base[0],
                g: base[1],
                b: base[2],
                a: base[3],
            })
        }
        (_, other) => other,
    };

    match paint {
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
        Paint::SweepGradient(g) => {
            out.kind = 4;
            out.tile_mode = tile_mode_to_u32(g.tile_mode);
            out.color_space = color_space_to_u32(g.color_space);
            out.stop_count = u32::from(g.stop_count.min(MAX_STOPS as u8));
            out.params0 = [
                g.center.x.0 * scale_factor,
                g.center.y.0 * scale_factor,
                g.start_angle_turns,
                (g.end_angle_turns - g.start_angle_turns).max(1e-6),
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
            if material_policy != PaintMaterialPolicy::Allow {
                return out;
            }

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
                    out.stop_count = 1;
                    out.color_space = catalog_texture_kind_to_layer(texture);
                }
            }
            out.params0 = color_to_linear_rgba_premul(mul_alpha(base, opacity));
            out.params1 = color_to_linear_rgba_premul(mul_alpha(fg, opacity));
            out.params2 = params.vec4s[2];
            out.params3 = params.vec4s[3];

            // Conservative default scaling: v1 baseline materials treat `params2.xy` as logical px.
            out.params2[0] *= scale_factor;
            out.params2[1] *= scale_factor;
        }
    }

    out
}
