use super::atlas::GlyphQuadKind;
use fret_render_text::FontFaceKey;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct GlyphKindLookupKey {
    font: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
}

impl GlyphKindLookupKey {
    pub(super) fn new(font: FontFaceKey, glyph_id: u32, size_bits: u32) -> Self {
        Self {
            font,
            glyph_id,
            size_bits,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct TextGlyphKindLookupCache {
    kinds: FxHashMap<GlyphKindLookupKey, GlyphQuadKind>,
}

impl TextGlyphKindLookupCache {
    pub(super) fn clear(&mut self) {
        self.kinds.clear();
    }

    pub(super) fn get(&self, key: GlyphKindLookupKey) -> Option<GlyphQuadKind> {
        self.kinds.get(&key).copied()
    }

    pub(super) fn insert(&mut self, key: GlyphKindLookupKey, kind: GlyphQuadKind) {
        self.kinds.insert(key, kind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face() -> FontFaceKey {
        FontFaceKey::new(1, 0, 0, false, 0)
    }

    #[test]
    fn glyph_kind_lookup_key_tracks_content_kind_not_subpixel_slot() {
        let mut cache = TextGlyphKindLookupCache::default();
        let key = GlyphKindLookupKey::new(face(), 42, 16.0f32.to_bits());

        cache.insert(key, GlyphQuadKind::Mask);

        assert_eq!(cache.get(key), Some(GlyphQuadKind::Mask));
    }
}
