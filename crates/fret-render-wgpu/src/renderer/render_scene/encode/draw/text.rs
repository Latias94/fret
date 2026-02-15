use super::super::state::{EncodeState, apply_transform_px, bounds_of_quad_points};
use super::super::*;

use crate::text::{GlyphQuadKind, TextDecorationKind};
use fret_core::{Corners, Edges};

pub(in super::super) fn encode_text(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    blob_id: fret_core::TextBlobId,
    color: Color,
) {
    state.flush_quad_batch();

    let Some(blob) = renderer.text_system.blob(blob_id) else {
        return;
    };

    let group_opacity = state.current_opacity();
    if group_opacity <= 0.0 || color.a <= 0.0 {
        return;
    }

    let t_px = state.current_transform_px();

    let base_x = origin.x.0 * state.scale_factor;
    let base_y = origin.y.0 * state.scale_factor;
    let paint_opacity = group_opacity * color.a;
    let baseline = blob.shape.metrics.baseline;

    let resolve_decoration_color = |paint_span: Option<u16>, explicit: Option<Color>| -> Color {
        if let Some(c) = explicit {
            let mut out = c;
            out.a *= color.a;
            return out;
        }

        if let Some(slot) = paint_span
            && let Some(palette) = blob.paint_palette.as_ref()
            && let Some(Some(c)) = palette.get(slot as usize)
        {
            let mut out = *c;
            out.a *= color.a;
            return out;
        }

        color
    };

    if !blob.decorations.is_empty() {
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

    let base_color = EncodeState::color_with_opacity(color, group_opacity);

    let mut active_kind: Option<TextDrawKind> = None;
    let mut active_page: u16 = 0;
    let mut group_first_vertex = state.text_vertices.len() as u32;
    let mut group_bounds_px: Option<(f32, f32, f32, f32)> = None;

    let flush_group = |state: &mut EncodeState<'_>,
                       kind: Option<TextDrawKind>,
                       page: u16,
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

        if active_kind != Some(kind) || (active_kind.is_some() && active_page != atlas_page) {
            flush_group(
                state,
                active_kind,
                active_page,
                &mut group_first_vertex,
                &mut group_bounds_px,
            );
            active_kind = Some(kind);
            active_page = atlas_page;
            group_first_vertex = state.text_vertices.len() as u32;
        }

        let paint_color = match (g.paint_span, blob.paint_palette.as_ref()) {
            (Some(slot), Some(palette)) => palette
                .get(slot as usize)
                .and_then(|c| *c)
                .map(|c| EncodeState::color_with_opacity(c, paint_opacity))
                .unwrap_or(base_color),
            _ => base_color,
        };
        let premul = color_to_linear_rgba_premul(paint_color);
        let vertex_color = match kind {
            TextDrawKind::Mask => premul,
            TextDrawKind::Color => [1.0, 1.0, 1.0, premul[3]],
            TextDrawKind::Subpixel => premul,
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
                uv: [u0, v0],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[1].0, quad[1].1],
                uv: [u1, v0],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[2].0, quad[2].1],
                uv: [u1, v1],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[0].0, quad[0].1],
                uv: [u0, v0],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[2].0, quad[2].1],
                uv: [u1, v1],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[3].0, quad[3].1],
                uv: [u0, v1],
                color: vertex_color,
            },
        ]);
    }

    flush_group(
        state,
        active_kind,
        active_page,
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
