use super::super::state::{EncodeState, bounds_of_quad_points, transform_quad_points_px};
use super::super::*;

use super::paint::{PaintMaterialPolicy, paint_to_gpu};
use crate::text::{TextDecorationKind, TextRenderGlyphKind};
use fret_core::time::Instant;
use fret_core::{Corners, Edges};

pub(in super::super) fn encode_text(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    blob_id: fret_core::TextBlobId,
    paint: fret_core::scene::PaintBindingV1,
    outline: Option<fret_core::scene::TextOutlineV1>,
    shadow: Option<fret_core::scene::TextShadowV1>,
    perf_enabled: bool,
    frame_perf: &mut RenderPerfStats,
) {
    state.flush_quad_batch();

    let Some(blob) = renderer.text_system.render_data_for_blob(blob_id) else {
        return;
    };

    if let Some(shadow) = shadow
        && shadow.color.a > 0.0
        && (shadow.offset.x.0 != 0.0 || shadow.offset.y.0 != 0.0)
    {
        let shadow_origin = Point::new(origin.x + shadow.offset.x, origin.y + shadow.offset.y);
        let shadow_start = perf_enabled.then(Instant::now);
        encode_text_blob(
            renderer,
            state,
            shadow_origin,
            &blob,
            fret_core::scene::Paint::Solid(shadow.color).into(),
            None,
            false,
            false,
            perf_enabled,
            frame_perf,
        );
        if let Some(shadow_start) = shadow_start {
            frame_perf.record_encode_scene_text_phase(
                EncodeSceneTextPhase::Shadow,
                Some(shadow_start.elapsed()),
            );
        }
    }

    encode_text_blob(
        renderer,
        state,
        origin,
        &blob,
        paint,
        outline,
        true,
        true,
        perf_enabled,
        frame_perf,
    );
}

fn encode_text_blob(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    blob: &crate::text::TextBlobRenderData<'_>,
    paint: fret_core::scene::PaintBindingV1,
    outline: Option<fret_core::scene::TextOutlineV1>,
    draw_decorations: bool,
    profile_text_phases: bool,
    perf_enabled: bool,
    frame_perf: &mut RenderPerfStats,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if group_opacity <= 0.0 {
        return;
    }

    let setup_start = (profile_text_phases && perf_enabled).then(Instant::now);
    let t_px = state.current_transform_px();
    let t_fast = t_px.as_translation_uniform_scale();

    let base_x = origin.x.0 * state.scale_factor;
    let base_y = origin.y.0 * state.scale_factor;
    let baseline = blob.baseline();

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

    let paint = match paint.eval_space {
        fret_core::scene::PaintEvalSpaceV1::StrokeS01 => fret_core::scene::PaintBindingV1 {
            paint: paint.paint,
            eval_space: fret_core::scene::PaintEvalSpaceV1::LocalPx,
        },
        _ => paint,
    };

    let base_color_hint = paint_representative_color(paint.paint);
    let paint_opacity = group_opacity * base_color_hint.a;

    let resolve_decoration_color = |paint_span: Option<u16>, explicit: Option<Color>| -> Color {
        if let Some(c) = explicit {
            let mut out = c;
            out.a *= base_color_hint.a;
            return out;
        }

        if let Some(slot) = paint_span
            && let Some(palette) = blob.paint_palette()
            && let Some(Some(c)) = palette.get(slot as usize)
        {
            let mut out = *c;
            out.a *= base_color_hint.a;
            return out;
        }

        base_color_hint
    };

    if draw_decorations && !blob.decorations().is_empty() {
        for d in blob
            .decorations()
            .iter()
            .filter(|d| d.kind() == TextDecorationKind::Underline)
        {
            let decoration_rect = d.rect();
            let rect = Rect::new(
                Point::new(
                    Px(origin.x.0 + decoration_rect.origin.x.0),
                    Px(origin.y.0 + decoration_rect.origin.y.0 - baseline.0),
                ),
                decoration_rect.size,
            );
            let bg = resolve_decoration_color(d.paint_span(), d.color());
            super::encode_quad(
                renderer,
                state,
                rect,
                fret_core::Paint::Solid(bg).into(),
                Edges::all(Px(0.0)),
                fret_core::Paint::Solid(Color::TRANSPARENT).into(),
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

    let mut outline_params_mask: u32 = 0;
    if let Some(outline) = outline.and_then(|o| o.sanitize()) {
        let outline_width_px = outline.width_px.0 * state.scale_factor;
        let outline_radius_px = outline_width_px.round().clamp(0.0, 3.0) as u32;
        if outline_radius_px > 0 {
            let outline_paint = paint_to_gpu(
                renderer,
                state,
                outline.paint,
                group_opacity,
                state.scale_factor,
                PaintMaterialPolicy::DegradeToSolidBase,
            );
            if paint_is_visible(&outline_paint) {
                let outline_paint_index = state.text_paints.len() as u32;
                state.text_paints.push(outline_paint);
                outline_params_mask = (outline_paint_index << 2) | (outline_radius_px & 3);
            }
        }
    }

    let white_paint_index = state.text_white_paint_index.unwrap_or_else(|| {
        let idx = state.text_paints.len() as u32;
        state.text_paints.push(PaintGpu {
            kind: 0,
            tile_mode: 0,
            color_space: 0,
            stop_count: 0,
            eval_space: 0,
            _pad_eval_space: [0; 3],
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

    if let Some(setup_start) = setup_start {
        frame_perf.record_encode_scene_text_phase(
            EncodeSceneTextPhase::Setup,
            Some(setup_start.elapsed()),
        );
    }

    let mut active_kind: Option<TextDrawKind> = None;
    let mut active_page: u16 = 0;
    let mut active_paint_index: u32 = 0;
    let mut active_palette: bool = false;
    let mut group_first_instance = state.text_glyph_instances.len() as u32;
    let mut group_bounds_local: Option<(f32, f32, f32, f32)> = None;
    let glyphs_start = (profile_text_phases && perf_enabled).then(Instant::now);

    let flush_group =
        |state: &mut EncodeState<'_>,
         kind: Option<TextDrawKind>,
         page: u16,
         paint_index: u32,
         group_first_instance: &mut u32,
         group_bounds_local: &mut Option<(f32, f32, f32, f32)>| {
            let Some(kind) = kind else {
                return;
            };

            let first = *group_first_instance;
            let instance_count = (state.text_glyph_instances.len() as u32).saturating_sub(first);
            if instance_count == 0 {
                *group_bounds_local = None;
                return;
            }

            let Some((min_x, min_y, max_x, max_y)) = *group_bounds_local else {
                *group_bounds_local = None;
                return;
            };

            let t_px = state.current_transform_px();
            let quad = transform_quad_points_px(t_px, min_x, min_y, max_x - min_x, max_y - min_y);
            let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
            let Some(bounds_scissor) =
                scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
            else {
                state.text_glyph_instances.truncate(first as usize);
                *group_bounds_local = None;
                return;
            };
            let clipped_scissor = intersect_scissor(state.current_scissor, bounds_scissor);
            if clipped_scissor.w == 0 || clipped_scissor.h == 0 {
                state.text_glyph_instances.truncate(first as usize);
                *group_bounds_local = None;
                return;
            }

            state.push_text_draw(TextDraw {
                scissor: clipped_scissor,
                uniform_index: state.current_uniform_index,
                first_instance: first,
                instance_count,
                kind,
                atlas_page: page,
                paint_index,
            });

            *group_bounds_local = None;
            *group_first_instance = state.text_glyph_instances.len() as u32;
        };

    for g in blob.glyphs() {
        let kind = match g.kind() {
            TextRenderGlyphKind::Mask => {
                if outline_params_mask != 0 {
                    TextDrawKind::MaskOutline
                } else {
                    TextDrawKind::Mask
                }
            }
            TextRenderGlyphKind::Color => TextDrawKind::Color,
            TextRenderGlyphKind::Subpixel => {
                if outline_params_mask != 0 {
                    TextDrawKind::SubpixelOutline
                } else {
                    TextDrawKind::Subpixel
                }
            }
        };

        let atlas_page = g.atlas_page();
        let uv = g.uv();

        let (use_palette_override, palette_color) = if let Some(slot) = g.paint_span() {
            let c = blob
                .paint_palette()
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
            let flush_group_start = (profile_text_phases && perf_enabled).then(Instant::now);
            flush_group(
                state,
                active_kind,
                active_page,
                active_paint_index,
                &mut group_first_instance,
                &mut group_bounds_local,
            );
            if let Some(flush_group_start) = flush_group_start {
                frame_perf.record_encode_scene_text_phase(
                    EncodeSceneTextPhase::GroupFlush,
                    Some(flush_group_start.elapsed()),
                );
            }
            active_kind = Some(kind);
            active_page = atlas_page;
            active_paint_index = draw_paint_index;
            active_palette = use_palette_override;
            group_first_instance = state.text_glyph_instances.len() as u32;
        }

        let vertex_color = if use_palette_override {
            let c = EncodeState::color_with_opacity(palette_color, paint_opacity);
            let premul = color_to_linear_rgba_premul(c);
            match kind {
                TextDrawKind::Mask => premul,
                TextDrawKind::MaskOutline => premul,
                TextDrawKind::Color => [1.0, 1.0, 1.0, premul[3]],
                TextDrawKind::Subpixel => premul,
                TextDrawKind::SubpixelOutline => premul,
            }
        } else {
            match kind {
                TextDrawKind::Mask => [1.0, 1.0, 1.0, 1.0],
                TextDrawKind::MaskOutline => [1.0, 1.0, 1.0, 1.0],
                TextDrawKind::Color => [1.0, 1.0, 1.0, 1.0],
                TextDrawKind::Subpixel => [1.0, 1.0, 1.0, 1.0],
                TextDrawKind::SubpixelOutline => [1.0, 1.0, 1.0, 1.0],
            }
        };

        let rect = g.rect();
        let lx0 = base_x + rect[0] * state.scale_factor;
        let ly0 = base_y + rect[1] * state.scale_factor;
        let lx1 = lx0 + rect[2] * state.scale_factor;
        let ly1 = ly0 + rect[3] * state.scale_factor;
        if t_fast.is_some() {
            frame_perf.encode_scene_text_transform_fast_path_glyphs = frame_perf
                .encode_scene_text_transform_fast_path_glyphs
                .saturating_add(1);
        } else {
            frame_perf.encode_scene_text_transform_generic_glyphs = frame_perf
                .encode_scene_text_transform_generic_glyphs
                .saturating_add(1);
        }

        let (u0, v0, u1, v1) = (uv[0], uv[1], uv[2], uv[3]);

        let local_min_x = lx0.min(lx1);
        let local_min_y = ly0.min(ly1);
        let local_max_x = lx0.max(lx1);
        let local_max_y = ly0.max(ly1);
        group_bounds_local = Some(match group_bounds_local {
            Some((gx0, gy0, gx1, gy1)) => (
                gx0.min(local_min_x),
                gy0.min(local_min_y),
                gx1.max(local_max_x),
                gy1.max(local_max_y),
            ),
            None => (local_min_x, local_min_y, local_max_x, local_max_y),
        });

        if state
            .text_glyph_instances
            .capacity()
            .saturating_sub(state.text_glyph_instances.len())
            < 1
        {
            frame_perf.encode_scene_text_vertex_grow_events = frame_perf
                .encode_scene_text_vertex_grow_events
                .saturating_add(1);
        }

        let glyph_emit_start = (profile_text_phases && perf_enabled).then(Instant::now);
        state.text_glyph_instances.push(TextGlyphInstance {
            local_rect: [lx0, ly0, lx1, ly1],
            uv: [u0, v0, u1, v1],
            color: vertex_color,
            outline_params: if matches!(
                kind,
                TextDrawKind::MaskOutline | TextDrawKind::SubpixelOutline
            ) {
                outline_params_mask
            } else {
                0
            },
            paint_index: draw_paint_index,
        });
        if let Some(glyph_emit_start) = glyph_emit_start {
            frame_perf.record_encode_scene_text_phase(
                EncodeSceneTextPhase::GlyphEmit,
                Some(glyph_emit_start.elapsed()),
            );
        }
    }

    let flush_group_start = (profile_text_phases && perf_enabled).then(Instant::now);
    flush_group(
        state,
        active_kind,
        active_page,
        active_paint_index,
        &mut group_first_instance,
        &mut group_bounds_local,
    );
    if let Some(flush_group_start) = flush_group_start {
        frame_perf.record_encode_scene_text_phase(
            EncodeSceneTextPhase::GroupFlush,
            Some(flush_group_start.elapsed()),
        );
    }

    if !blob.decorations().is_empty() {
        for d in blob
            .decorations()
            .iter()
            .filter(|d| d.kind() == TextDecorationKind::Strikethrough)
        {
            let decoration_rect = d.rect();
            let rect = Rect::new(
                Point::new(
                    Px(origin.x.0 + decoration_rect.origin.x.0),
                    Px(origin.y.0 + decoration_rect.origin.y.0 - baseline.0),
                ),
                decoration_rect.size,
            );
            let bg = resolve_decoration_color(d.paint_span(), d.color());
            super::encode_quad(
                renderer,
                state,
                rect,
                fret_core::Paint::Solid(bg).into(),
                Edges::all(Px(0.0)),
                fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                Corners::all(Px(0.0)),
                None,
            );
        }
        state.flush_quad_batch();
    }

    if let Some(glyphs_start) = glyphs_start {
        frame_perf.record_encode_scene_text_phase(
            EncodeSceneTextPhase::Glyphs,
            Some(glyphs_start.elapsed()),
        );
    }
}
