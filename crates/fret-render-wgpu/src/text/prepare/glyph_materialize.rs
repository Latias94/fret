use super::super::atlas::GlyphAtlas;
use super::super::{GlyphInstance, GlyphQuadKind, TextLine, TextSystem};
use super::glyph_raster::{PreparedGlyphRaster, insert_prepared_glyph_raster_into_atlas};
use fret_render_text::FontFaceKey;
use fret_render_text::{ParleyGlyph, PreparedLine, ResolvedSpan, paint_span_for_text_range};
use std::collections::HashMap;

impl TextSystem {
    pub(in super::super) fn materialize_prepared_line(
        &mut self,
        prepared_line: PreparedLine,
        resolved_spans: Option<&[ResolvedSpan]>,
        scale: f32,
        epoch: u64,
        glyphs: &mut Vec<GlyphInstance>,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
        lines: &mut Vec<TextLine>,
    ) {
        let (layout, prepared_glyphs) = prepared_line.into_parts();
        lines.push(layout);
        self.materialize_prepared_line_glyphs(
            prepared_glyphs,
            resolved_spans,
            scale,
            epoch,
            glyphs,
            face_usage,
        );
    }

    fn materialize_prepared_line_glyphs(
        &mut self,
        prepared_glyphs: Vec<ParleyGlyph>,
        resolved_spans: Option<&[ResolvedSpan]>,
        scale: f32,
        epoch: u64,
        glyphs: &mut Vec<GlyphInstance>,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
    ) {
        for glyph in prepared_glyphs {
            let Some(instance) = self.materialize_prepared_line_glyph(
                &glyph,
                resolved_spans,
                scale,
                epoch,
                face_usage,
            ) else {
                continue;
            };
            glyphs.push(instance);
        }
    }

    fn materialize_prepared_line_glyph(
        &mut self,
        glyph: &ParleyGlyph,
        resolved_spans: Option<&[ResolvedSpan]>,
        scale: f32,
        epoch: u64,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
    ) -> Option<GlyphInstance> {
        let context = self.prepare_prepared_glyph_context(glyph, face_usage)?;
        let (x, x_bin, y, y_bin) = prepared_glyph_origin_bins(glyph);
        let paint_span = prepared_glyph_paint_span(resolved_spans, glyph);
        let (glyph_key, x0_px, y0_px, w_px, h_px) = self.resolve_prepared_glyph_bounds(
            glyph,
            context.glyph_id,
            context.face_key,
            context.size_bits,
            x_bin,
            y_bin,
            x,
            y,
            epoch,
        )?;
        Some(prepared_glyph_instance(
            glyph_key, x0_px, y0_px, w_px, h_px, paint_span, scale,
        ))
    }

    fn insert_prepared_glyph_raster(&mut self, raster: PreparedGlyphRaster, epoch: u64) {
        let atlas = self.prepared_glyph_atlas_mut(raster.kind());
        insert_prepared_glyph_raster_into_atlas(atlas, raster, epoch);
    }

    pub(super) fn commit_prepared_glyph_raster(
        &mut self,
        raster: PreparedGlyphRaster,
        x: i32,
        y: i32,
        epoch: u64,
    ) -> (super::super::atlas::GlyphKey, f32, f32, f32, f32) {
        let bounds = raster.bounds(x, y);
        self.insert_prepared_glyph_raster(raster, epoch);
        bounds
    }

    pub(super) fn prepared_glyph_atlas_mut(&mut self, kind: GlyphQuadKind) -> &mut GlyphAtlas {
        self.atlas_runtime.atlas_mut(kind)
    }
}

fn prepared_glyph_paint_span(
    resolved_spans: Option<&[ResolvedSpan]>,
    glyph: &ParleyGlyph,
) -> Option<u16> {
    resolved_spans
        .and_then(|spans| paint_span_for_text_range(spans, &glyph.text_range, glyph.is_rtl))
}

fn prepared_glyph_instance(
    glyph_key: super::super::atlas::GlyphKey,
    x0_px: f32,
    y0_px: f32,
    w_px: f32,
    h_px: f32,
    paint_span: Option<u16>,
    scale: f32,
) -> GlyphInstance {
    GlyphInstance {
        rect: [x0_px / scale, y0_px / scale, w_px / scale, h_px / scale],
        paint_span,
        key: glyph_key,
    }
}

fn prepared_glyph_origin_bins(glyph: &ParleyGlyph) -> (i32, u8, i32, u8) {
    let (x, x_bin) = super::super::atlas::subpixel_bin_q4(glyph.x);
    let (y, y_bin) = super::super::atlas::subpixel_bin_y(glyph.y);
    (x, x_bin, y, y_bin)
}
