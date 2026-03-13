#[derive(Debug, Default)]
pub(crate) struct TextFramePerfState {
    pub(crate) cache_resets: u64,
    pub(crate) blob_cache_hits: u64,
    pub(crate) blob_cache_misses: u64,
    pub(crate) blobs_created: u64,
    pub(crate) shape_cache_hits: u64,
    pub(crate) shape_cache_misses: u64,
    pub(crate) shapes_created: u64,
    pub(crate) missing_glyphs: u64,
    pub(crate) texts_with_missing_glyphs: u64,
}

impl TextFramePerfState {
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn record_missing_glyphs(&mut self, missing_glyphs: u32) {
        if missing_glyphs == 0 {
            return;
        }

        self.missing_glyphs = self
            .missing_glyphs
            .saturating_add(u64::from(missing_glyphs));
        self.texts_with_missing_glyphs = self.texts_with_missing_glyphs.saturating_add(1);
    }
}
