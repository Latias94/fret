use super::super::TextSystem;
use super::glyph_render::prepared_glyph_has_normalized_coords;
use fret_render_text::font_instance_key::{FontFaceKey, variation_key_from_normalized_coords};
use fret_render_text::parley_shaper::ParleyGlyph;
use std::collections::HashMap;

pub(super) struct PreparedGlyphContext {
    pub(super) glyph_id: u16,
    pub(super) face_key: FontFaceKey,
    pub(super) size_bits: u32,
}

impl TextSystem {
    pub(super) fn prepare_prepared_glyph_context(
        &mut self,
        glyph: &ParleyGlyph,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
    ) -> Option<PreparedGlyphContext> {
        let glyph_id = prepared_glyph_id(glyph)?;
        let face_key = self.register_prepared_glyph_face(glyph, face_usage);
        Some(prepared_glyph_context(glyph, glyph_id, face_key))
    }

    fn register_prepared_glyph_face(
        &mut self,
        glyph: &ParleyGlyph,
        face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
    ) -> FontFaceKey {
        let (font_data_id, face_index) = prepared_glyph_font_identity(glyph);
        let face_key = prepared_glyph_face_key(glyph, font_data_id, face_index);
        self.cache_prepared_glyph_face_data(glyph, face_key, font_data_id, face_index);
        record_prepared_glyph_face_usage(face_usage, face_key, glyph.id);
        face_key
    }

    fn cache_prepared_glyph_face_data(
        &mut self,
        glyph: &ParleyGlyph,
        face_key: FontFaceKey,
        font_data_id: u64,
        face_index: u32,
    ) {
        self.cache_prepared_glyph_font_data(glyph, font_data_id, face_index);
        self.cache_prepared_glyph_instance_coords(glyph, face_key);
    }

    fn cache_prepared_glyph_font_data(
        &mut self,
        glyph: &ParleyGlyph,
        font_data_id: u64,
        face_index: u32,
    ) {
        self.face_cache
            .font_data_by_face
            .entry((font_data_id, face_index))
            .or_insert_with(|| glyph.font.clone());
    }

    fn cache_prepared_glyph_instance_coords(&mut self, glyph: &ParleyGlyph, face_key: FontFaceKey) {
        if prepared_glyph_has_normalized_coords(glyph) {
            self.face_cache
                .font_instance_coords_by_face
                .entry(face_key)
                .or_insert_with(|| glyph.normalized_coords.clone());
        }
    }
}

fn prepared_glyph_context(
    glyph: &ParleyGlyph,
    glyph_id: u16,
    face_key: FontFaceKey,
) -> PreparedGlyphContext {
    PreparedGlyphContext {
        glyph_id,
        face_key,
        size_bits: prepared_glyph_size_bits(glyph),
    }
}

fn prepared_glyph_id(glyph: &ParleyGlyph) -> Option<u16> {
    u16::try_from(glyph.id).ok()
}

fn prepared_glyph_size_bits(glyph: &ParleyGlyph) -> u32 {
    glyph.font_size.to_bits()
}

fn prepared_glyph_face_key(glyph: &ParleyGlyph, font_data_id: u64, face_index: u32) -> FontFaceKey {
    FontFaceKey {
        font_data_id,
        face_index,
        variation_key: prepared_glyph_variation_key(glyph),
        synthesis_embolden: prepared_glyph_synthesis_embolden(glyph),
        synthesis_skew_degrees: prepared_glyph_synthesis_skew_degrees(glyph),
    }
}

fn prepared_glyph_font_identity(glyph: &ParleyGlyph) -> (u64, u32) {
    (glyph.font.data.id(), glyph.font.index)
}

fn prepared_glyph_variation_key(glyph: &ParleyGlyph) -> u64 {
    variation_key_from_normalized_coords(&glyph.normalized_coords)
}

fn prepared_glyph_synthesis_embolden(glyph: &ParleyGlyph) -> bool {
    glyph.synthesis.embolden()
}

fn prepared_glyph_synthesis_skew_degrees(glyph: &ParleyGlyph) -> i8 {
    glyph
        .synthesis
        .skew()
        .unwrap_or(0.0)
        .clamp(i8::MIN as f32, i8::MAX as f32) as i8
}

fn record_prepared_glyph_face_usage(
    face_usage: &mut HashMap<FontFaceKey, (u32, u32)>,
    face_key: FontFaceKey,
    glyph_id: u32,
) {
    let usage = face_usage.entry(face_key).or_insert((0, 0));
    usage.0 = usage.0.saturating_add(1);
    if glyph_id == 0 {
        usage.1 = usage.1.saturating_add(1);
    }
}
