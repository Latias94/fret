use super::super::TextSystem;
use super::super::atlas::GlyphKey;
use super::glyph_raster::prepared_glyph_lookup_keys;
use fret_render_text::{FontFaceKey, ParleyGlyph};

impl TextSystem {
    fn lookup_prepared_glyph_atlas(
        &mut self,
        face_key: FontFaceKey,
        glyph_id: u32,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        x: i32,
        y: i32,
        epoch: u64,
    ) -> Option<(GlyphKey, f32, f32, f32, f32)> {
        for glyph_key in prepared_glyph_lookup_keys(face_key, glyph_id, size_bits, x_bin, y_bin) {
            if let Some(hit) = self.lookup_prepared_glyph_bounds_for_key(glyph_key, x, y, epoch) {
                return Some(hit);
            }
        }
        None
    }

    fn lookup_prepared_glyph_bounds_for_key(
        &mut self,
        glyph_key: GlyphKey,
        x: i32,
        y: i32,
        epoch: u64,
    ) -> Option<(GlyphKey, f32, f32, f32, f32)> {
        self.atlas_runtime
            .prepared_bounds_for_key(glyph_key, x, y, epoch)
    }

    fn materialize_prepared_glyph_miss(
        &mut self,
        glyph: &ParleyGlyph,
        glyph_id: u16,
        face_key: FontFaceKey,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        x: i32,
        y: i32,
        epoch: u64,
    ) -> Option<(GlyphKey, f32, f32, f32, f32)> {
        let raster =
            self.render_prepared_glyph_raster(glyph, glyph_id, face_key, size_bits, x_bin, y_bin)?;
        Some(self.commit_prepared_glyph_raster(raster, x, y, epoch))
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
        epoch: u64,
    ) -> Option<(GlyphKey, f32, f32, f32, f32)> {
        self.resolve_prepared_glyph_hit_or_miss_bounds(
            glyph, glyph_id, face_key, size_bits, x_bin, y_bin, x, y, epoch,
        )
    }

    fn resolve_prepared_glyph_atlas_hit_bounds(
        &mut self,
        face_key: FontFaceKey,
        glyph_id: u32,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        x: i32,
        y: i32,
        epoch: u64,
    ) -> Option<(GlyphKey, f32, f32, f32, f32)> {
        self.lookup_prepared_glyph_atlas(face_key, glyph_id, size_bits, x_bin, y_bin, x, y, epoch)
    }

    fn resolve_prepared_glyph_hit_or_miss_bounds(
        &mut self,
        glyph: &ParleyGlyph,
        glyph_id: u16,
        face_key: FontFaceKey,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        x: i32,
        y: i32,
        epoch: u64,
    ) -> Option<(GlyphKey, f32, f32, f32, f32)> {
        self.resolve_prepared_glyph_atlas_hit_bounds(
            face_key,
            glyph.id(),
            size_bits,
            x_bin,
            y_bin,
            x,
            y,
            epoch,
        )
        .or_else(|| {
            self.materialize_prepared_glyph_miss(
                glyph, glyph_id, face_key, size_bits, x_bin, y_bin, x, y, epoch,
            )
        })
    }
}
