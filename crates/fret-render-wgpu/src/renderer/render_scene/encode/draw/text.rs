use super::super::state::{EncodeState, apply_transform_px, bounds_of_quad_points};
use super::super::*;

use super::paint::{PaintMaterialPolicy, paint_to_gpu};
use crate::text::{GlyphQuadKind, TextDecorationKind};
use fret_core::{Corners, Edges};

pub(in super::super) fn encode_text(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    blob_id: fret_core::TextBlobId,
    paint: fret_core::scene::Paint,
    shadow: Option<fret_core::scene::TextShadowV1>,
) {
    state.flush_quad_batch();

    let Some(blob) = renderer.text_system.blob(blob_id) else {
        return;
    };

    if let Some(shadow) = shadow
        && shadow.color.a > 0.0
        && (shadow.offset.x.0 != 0.0 || shadow.offset.y.0 != 0.0)
    {
        let shadow_origin = Point::new(origin.x + shadow.offset.x, origin.y + shadow.offset.y);
        encode_text_blob(
            renderer,
            state,
            shadow_origin,
            blob,
            fret_core::scene::Paint::Solid(shadow.color),
            false,
        );
    }

    encode_text_blob(renderer, state, origin, blob, paint, true);
}

fn encode_text_blob(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    blob: &crate::text::TextBlob,
    paint: fret_core::scene::Paint,
    draw_decorations: bool,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if group_opacity <= 0.0 {
        return;
    }

    let t_px = state.current_transform_px();

    let base_x = origin.x.0 * state.scale_factor;
    let base_y = origin.y.0 * state.scale_factor;
    let baseline = blob.shape.metrics.baseline;

    fn paint_representative_color(p: fret_core::scene::Paint) -> Color {
        use fret_core::scene::{MAX_STOPS, Paint};

        match p {
            Paint::Solid(c) => c,
            Paint::LinearGradient(g) => {
                let n = usize::from(g.stop_count).clamp(0, MAX_STOPS);
                if n == 0 {
                    return Color::TRANSPARENT;
                }
                g.stops[n - 1].color
            }
            Paint::RadialGradient(g) => {
                let n = usize::from(g.stop_count).clamp(0, MAX_STOPS);
                if n == 0 {
                    return Color::TRANSPARENT;
                }
                g.stops[n - 1].color
            }
            Paint::SweepGradient(g) => {
                let n = usize::from(g.stop_count).clamp(0, MAX_STOPS);
                if n == 0 {
                    return Color::TRANSPARENT;
                }
                g.stops[n - 1].color
            }
            Paint::Material { params, .. } => {
                let base = params.vec4s[0];
                Color {
                    r: base[0],
                    g: base[1],
                    b: base[2],
                    a: base[3],
                }
            }
        }
    }

    fn paint_is_visible(p: &PaintGpu) -> bool {
        if p.kind == 0 {
            return p.params0[3] > 0.0;
        }
        for c in p.stop_colors {
            if c[3] > 0.0 {
                return true;
            }
        }
        false
    }

    let base_color_hint = paint_representative_color(paint);
    let paint_opacity = group_opacity * base_color_hint.a;

    let resolve_decoration_color = |paint_span: Option<u16>, explicit: Option<Color>| -> Color {
        if let Some(c) = explicit {
            let mut out = c;
            out.a *= base_color_hint.a;
            return out;
        }

        if let Some(slot) = paint_span
            && let Some(palette) = blob.paint_palette.as_ref()
            && let Some(Some(c)) = palette.get(slot as usize)
        {
            let mut out = *c;
            out.a *= base_color_hint.a;
            return out;
        }

        base_color_hint
    };

    if draw_decorations && !blob.decorations.is_empty() {
        for d in blob
            .decorations
            .as_ref()
            .iter()
            .filter(|d| d.kind == TextDecorationKind::Underline)
        {
            let rect = Rect::new(
                Point::new(
                    Px(origin.x.0 + d.rect.origin.x.0),
                    Px(origin.y.0 + d.rect.origin.y.0 - baseline.0),
                ),
                d.rect.size,
            );
            let bg = resolve_decoration_color(d.paint_span, d.color);
            super::encode_quad(
                renderer,
                state,
                rect,
                fret_core::Paint::Solid(bg),
                Edges::all(Px(0.0)),
                fret_core::Paint::Solid(Color::TRANSPARENT),
                Corners::all(Px(0.0)),
                None,
            );
        }
        state.flush_quad_batch();
    }

    let text_paint = paint_to_gpu(
        renderer,
        state,
        paint,
        group_opacity,
        state.scale_factor,
        PaintMaterialPolicy::DegradeToSolidBase,
    );
    let text_paint_index = state.text_paints.len() as u32;
    state.text_paints.push(text_paint);

    let white_paint_index = state.text_white_paint_index.unwrap_or_else(|| {
        let idx = state.text_paints.len() as u32;
        state.text_paints.push(PaintGpu {
            kind: 0,
            tile_mode: 0,
            color_space: 0,
            stop_count: 0,
            params0: [1.0, 1.0, 1.0, 1.0],
            params1: [0.0; 4],
            params2: [0.0; 4],
            params3: [0.0; 4],
            stop_colors: [[0.0; 4]; fret_core::scene::MAX_STOPS],
            stop_offsets0: [0.0; 4],
            stop_offsets1: [0.0; 4],
        });
        state.text_white_paint_index = Some(idx);
        idx
    });

    let mut active_kind: Option<TextDrawKind> = None;
    let mut active_page: u16 = 0;
    let mut active_paint_index: u32 = 0;
    let mut active_palette: bool = false;
    let mut group_first_vertex = state.text_vertices.len() as u32;
    let mut group_bounds_px: Option<(f32, f32, f32, f32)> = None;

    let flush_group = |state: &mut EncodeState<'_>,
                       kind: Option<TextDrawKind>,
                       page: u16,
                       paint_index: u32,
                       group_first_vertex: &mut u32,
                       group_bounds_px: &mut Option<(f32, f32, f32, f32)>| {
        let Some(kind) = kind else {
            return;
        };

        let first = *group_first_vertex;
        let vertex_count = (state.text_vertices.len() as u32).saturating_sub(first);
        if vertex_count == 0 {
            *group_bounds_px = None;
            return;
        }

        let Some((min_x, min_y, max_x, max_y)) = *group_bounds_px else {
            *group_bounds_px = None;
            return;
        };

        let Some(bounds_scissor) =
            scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
        else {
            state.text_vertices.truncate(first as usize);
            *group_bounds_px = None;
            return;
        };
        let clipped_scissor = intersect_scissor(state.current_scissor, bounds_scissor);
        if clipped_scissor.w == 0 || clipped_scissor.h == 0 {
            state.text_vertices.truncate(first as usize);
            *group_bounds_px = None;
            return;
        }

        state.push_text_draw(TextDraw {
            scissor: clipped_scissor,
            uniform_index: state.current_uniform_index,
            first_vertex: first,
            vertex_count,
            kind,
            atlas_page: page,
            paint_index,
        });

        *group_bounds_px = None;
        *group_first_vertex = state.text_vertices.len() as u32;
    };

    for g in blob.shape.glyphs.as_ref() {
        let kind = match g.kind() {
            GlyphQuadKind::Mask => TextDrawKind::Mask,
            GlyphQuadKind::Color => TextDrawKind::Color,
            GlyphQuadKind::Subpixel => TextDrawKind::Subpixel,
        };

        let Some((atlas_page, uv)) = renderer.text_system.glyph_uv_for_instance(g) else {
            continue;
        };

        let (use_palette_override, palette_color) = if let Some(slot) = g.paint_span {
            let c = blob
                .paint_palette
                .as_ref()
                .and_then(|p| p.get(slot as usize).copied().flatten())
                .unwrap_or(base_color_hint);
            (true, c)
        } else {
            (false, Color::TRANSPARENT)
        };

        let draw_paint_index = if use_palette_override {
            white_paint_index
        } else {
            text_paint_index
        };

        if !use_palette_override && !paint_is_visible(&state.text_paints[text_paint_index as usize])
        {
            continue;
        }

        if active_kind != Some(kind)
            || (active_kind.is_some() && active_page != atlas_page)
            || active_paint_index != draw_paint_index
            || active_palette != use_palette_override
        {
            flush_group(
                state,
                active_kind,
                active_page,
                active_paint_index,
                &mut group_first_vertex,
                &mut group_bounds_px,
            );
            active_kind = Some(kind);
            active_page = atlas_page;
            active_paint_index = draw_paint_index;
            active_palette = use_palette_override;
            group_first_vertex = state.text_vertices.len() as u32;
        }

        let vertex_color = if use_palette_override {
            let c = EncodeState::color_with_opacity(palette_color, paint_opacity);
            let premul = color_to_linear_rgba_premul(c);
            match kind {
                TextDrawKind::Mask => premul,
                TextDrawKind::Color => [1.0, 1.0, 1.0, premul[3]],
                TextDrawKind::Subpixel => premul,
            }
        } else {
            match kind {
                TextDrawKind::Mask => [1.0, 1.0, 1.0, 1.0],
                TextDrawKind::Color => [1.0, 1.0, 1.0, 1.0],
                TextDrawKind::Subpixel => [1.0, 1.0, 1.0, 1.0],
            }
        };

        let lx0 = base_x + g.rect[0] * state.scale_factor;
        let ly0 = base_y + g.rect[1] * state.scale_factor;
        let lx1 = lx0 + g.rect[2] * state.scale_factor;
        let ly1 = ly0 + g.rect[3] * state.scale_factor;
        let quad = [
            apply_transform_px(t_px, lx0, ly0),
            apply_transform_px(t_px, lx1, ly0),
            apply_transform_px(t_px, lx1, ly1),
            apply_transform_px(t_px, lx0, ly1),
        ];

        let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
        group_bounds_px = Some(match group_bounds_px {
            Some((gx0, gy0, gx1, gy1)) => (
                gx0.min(min_x),
                gy0.min(min_y),
                gx1.max(max_x),
                gy1.max(max_y),
            ),
            None => (min_x, min_y, max_x, max_y),
        });

        let (u0, v0, u1, v1) = (uv[0], uv[1], uv[2], uv[3]);

        state.text_vertices.extend_from_slice(&[
            TextVertex {
                pos_px: [quad[0].0, quad[0].1],
                local_pos_px: [lx0, ly0],
                uv: [u0, v0],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[1].0, quad[1].1],
                local_pos_px: [lx1, ly0],
                uv: [u1, v0],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[2].0, quad[2].1],
                local_pos_px: [lx1, ly1],
                uv: [u1, v1],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[0].0, quad[0].1],
                local_pos_px: [lx0, ly0],
                uv: [u0, v0],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[2].0, quad[2].1],
                local_pos_px: [lx1, ly1],
                uv: [u1, v1],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[3].0, quad[3].1],
                local_pos_px: [lx0, ly1],
                uv: [u0, v1],
                color: vertex_color,
            },
        ]);
    }

    flush_group(
        state,
        active_kind,
        active_page,
        active_paint_index,
        &mut group_first_vertex,
        &mut group_bounds_px,
    );

    if !blob.decorations.is_empty() {
        for d in blob
            .decorations
            .as_ref()
            .iter()
            .filter(|d| d.kind == TextDecorationKind::Strikethrough)
        {
            let rect = Rect::new(
                Point::new(
                    Px(origin.x.0 + d.rect.origin.x.0),
                    Px(origin.y.0 + d.rect.origin.y.0 - baseline.0),
                ),
                d.rect.size,
            );
            let bg = resolve_decoration_color(d.paint_span, d.color);
            super::encode_quad(
                renderer,
                state,
                rect,
                fret_core::Paint::Solid(bg),
                Edges::all(Px(0.0)),
                fret_core::Paint::Solid(Color::TRANSPARENT),
                Corners::all(Px(0.0)),
                None,
            );
        }
        state.flush_quad_batch();
    }
}
