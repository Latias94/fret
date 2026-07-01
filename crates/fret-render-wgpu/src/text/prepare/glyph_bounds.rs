use super::super::TextSystem;
use super::super::atlas::GlyphKey;
use fret_render_text::{FontFaceKey, ParleyGlyph};

impl TextSystem {
    fn materialize_prepared_glyph_bounds(
        &mut self,
        glyph: &ParleyGlyph,
        glyph_id: u16,
        face_key: FontFaceKey,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        x: i32,
        y: i32,
    ) -> Option<(GlyphKey, f32, f32, f32, f32)> {
        let raster =
            self.render_prepared_glyph_raster(glyph, glyph_id, face_key, size_bits, x_bin, y_bin)?;
        Some(raster.bounds(x, y))
    }

    pub(super) fn resolve_prepared_glyph_bounds(
        &mut self,
        glyph: &ParleyGlyph,
        glyph_id: u16,
        face_key: FontFaceKey,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        x: i32,
        y: i32,
    ) -> Option<(GlyphKey, f32, f32, f32, f32)> {
        self.materialize_prepared_glyph_bounds(
            glyph, glyph_id, face_key, size_bits, x_bin, y_bin, x, y,
        )
    }
}
