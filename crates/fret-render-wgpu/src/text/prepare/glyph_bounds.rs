use super::super::atlas::{GlyphAtlasEntry, GlyphKey};
use super::super::{GlyphQuadKind, TextSystem};
use super::glyph_raster::prepared_glyph_lookup_key;
use fret_render_text::font_instance_key::FontFaceKey;
use fret_render_text::parley_shaper::ParleyGlyph;

const PREPARED_GLYPH_ATLAS_LOOKUP_ORDER: [GlyphQuadKind; 3] = [
    GlyphQuadKind::Color,
    GlyphQuadKind::Subpixel,
    GlyphQuadKind::Mask,
];

impl TextSystem {
    fn lookup_prepared_glyph_atlas(
        &mut self,
        face_key: FontFaceKey,
        glyph_id: u32,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        epoch: u64,
    ) -> Option<(GlyphKey, GlyphAtlasEntry)> {
        for kind in PREPARED_GLYPH_ATLAS_LOOKUP_ORDER {
            if let Some(hit) = self.lookup_prepared_glyph_atlas_kind(
                face_key, glyph_id, size_bits, x_bin, y_bin, kind, epoch,
            ) {
                return Some(hit);
            }
        }
        None
    }

    fn lookup_prepared_glyph_atlas_kind(
        &mut self,
        face_key: FontFaceKey,
        glyph_id: u32,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        kind: GlyphQuadKind,
        epoch: u64,
    ) -> Option<(GlyphKey, GlyphAtlasEntry)> {
        let glyph_key =
            prepared_glyph_lookup_key(face_key, glyph_id, size_bits, x_bin, y_bin, kind);
        self.lookup_prepared_glyph_atlas_entry(glyph_key, epoch)
    }

    fn lookup_prepared_glyph_atlas_entry(
        &mut self,
        glyph_key: GlyphKey,
        epoch: u64,
    ) -> Option<(GlyphKey, GlyphAtlasEntry)> {
        self.prepared_glyph_atlas_mut(glyph_key.kind)
            .get(glyph_key, epoch)
            .map(|entry| (glyph_key, entry))
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
        let (glyph_key, entry) =
            self.lookup_prepared_glyph_atlas(face_key, glyph_id, size_bits, x_bin, y_bin, epoch)?;
        Some(prepared_glyph_bounds_from_atlas_entry(
            glyph_key, entry, x, y,
        ))
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
            face_key, glyph.id, size_bits, x_bin, y_bin, x, y, epoch,
        )
        .or_else(|| {
            self.materialize_prepared_glyph_miss(
                glyph, glyph_id, face_key, size_bits, x_bin, y_bin, x, y, epoch,
            )
        })
    }
}

fn prepared_glyph_bounds_from_atlas_entry(
    glyph_key: GlyphKey,
    entry: GlyphAtlasEntry,
    x: i32,
    y: i32,
) -> (GlyphKey, f32, f32, f32, f32) {
    (
        glyph_key,
        x as f32 + entry.placement_left as f32,
        y as f32 - entry.placement_top as f32,
        entry.w as f32,
        entry.h as f32,
    )
}
