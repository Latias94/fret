use super::{GlyphAtlas, GlyphKey};
use crate::text::DebugGlyphAtlasLookup;

impl GlyphKey {
    fn kind_label(self) -> &'static str {
        if self.is_mask() {
            "mask"
        } else if self.is_color() {
            "color"
        } else if self.is_subpixel() {
            "subpixel"
        } else {
            debug_assert!(false, "unknown glyph quad kind");
            "mask"
        }
    }
}

impl GlyphAtlas {
    pub(in crate::text) fn debug_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub(in crate::text) fn debug_lookup_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        let key = self.find_key_for_bounds(page, x, y, w, h)?;
        Some(DebugGlyphAtlasLookup::new(
            key.font.font_data_id(),
            key.font.face_index(),
            key.font.variation_key(),
            key.font.synthesis_embolden(),
            key.font.synthesis_skew_degrees(),
            key.glyph_id,
            key.size_bits,
            key.x_bin,
            key.y_bin,
            key.kind_label(),
        ))
    }

    fn find_key_for_bounds(&self, page: u16, x: u32, y: u32, w: u32, h: u32) -> Option<GlyphKey> {
        self.glyphs.iter().find_map(|(key, entry)| {
            (entry.page == page && entry.x == x && entry.y == y && entry.w == w && entry.h == h)
                .then_some(*key)
        })
    }
}
