use super::{TextBlob, TextDecoration, TextFrameResidency, TextRenderGlyph, TextSystem};
use fret_core::{Color, TextBlobId, geometry::Px};
use std::sync::Arc;

pub(crate) struct TextBlobRenderData<'a> {
    glyphs: Arc<[TextRenderGlyph]>,
    baseline: Px,
    decorations: &'a [TextDecoration],
    paint_palette: Option<&'a [Option<Color>]>,
}

impl<'a> TextBlobRenderData<'a> {
    fn new(text_system: &'a TextSystem, blob: &'a TextBlob) -> Self {
        let shape = blob.shape();
        let glyphs = shape.render_glyphs(text_system.atlas_revision(), |glyph| {
            text_system.glyph_uv_for_instance(glyph)
        });
        Self {
            glyphs,
            baseline: shape.metrics().baseline,
            decorations: blob.decorations(),
            paint_palette: blob.paint_palette(),
        }
    }

    pub(crate) fn baseline(&self) -> Px {
        self.baseline
    }

    pub(crate) fn decorations(&self) -> &'a [TextDecoration] {
        self.decorations
    }

    pub(crate) fn paint_palette(&self) -> Option<&'a [Option<Color>]> {
        self.paint_palette
    }

    pub(crate) fn glyphs(&self) -> impl Iterator<Item = TextRenderGlyph> + '_ {
        self.glyphs.iter().copied()
    }
}

impl TextSystem {
    pub(super) fn blob(&self, id: TextBlobId) -> Option<&TextBlob> {
        self.blob_state.blobs.get(id)
    }

    pub(super) fn shape_for_blob(&self, id: TextBlobId) -> Option<&super::TextShape> {
        Some(self.blob(id)?.shape())
    }

    pub(crate) fn text_residency_for_blobs(
        &self,
        text_blob_ids: &[TextBlobId],
    ) -> TextFrameResidency {
        let mut residency = TextFrameResidency::new();
        for text_blob in text_blob_ids.iter().copied() {
            let Some(shape) = self.shape_for_blob(text_blob) else {
                residency.note_missing_entry();
                continue;
            };
            residency.push_glyphs(text_blob, shape.glyphs().iter().map(|glyph| glyph.key));
        }
        residency
    }

    pub(crate) fn push_glyph_residency_for_blob(
        &self,
        residency: &mut TextFrameResidency,
        text_blob: TextBlobId,
        mut predicate: impl FnMut([f32; 4]) -> bool,
    ) -> bool {
        let Some(shape) = self.shape_for_blob(text_blob) else {
            residency.note_missing_entry();
            return false;
        };
        let glyphs = shape
            .glyphs()
            .iter()
            .filter(|glyph| predicate(glyph.rect()))
            .map(|glyph| glyph.key)
            .collect::<Vec<_>>();
        let pushed = !glyphs.is_empty();
        residency.push_glyphs(text_blob, glyphs);
        pushed
    }

    pub(crate) fn render_data_for_blob(&self, id: TextBlobId) -> Option<TextBlobRenderData<'_>> {
        Some(TextBlobRenderData::new(self, self.blob(id)?))
    }

    #[cfg(test)]
    pub(super) fn shape_handle_for_blob(&self, id: TextBlobId) -> Option<&Arc<super::TextShape>> {
        Some(self.blob(id)?.shape_handle())
    }

    pub fn release(&mut self, blob: TextBlobId) {
        let entries = fret_render_text::released_blob_cache_entries();

        let Some(b) = self.blob_state.blobs.get_mut(blob) else {
            return;
        };

        if b.ref_count() > 1 {
            b.decrement_ref_count();
            return;
        }

        if b.is_released() {
            return;
        }

        if entries > 0 {
            b.mark_released();
            self.insert_released_blob(blob, entries);
            return;
        }

        self.evict_blob(blob);
    }

    pub(super) fn remove_released_blob(&mut self, id: TextBlobId) {
        if !self.blob_state.released_blob_set.remove(&id) {
            return;
        }
        if let Some(pos) = self
            .blob_state
            .released_blob_lru
            .iter()
            .position(|v| *v == id)
        {
            self.blob_state.released_blob_lru.remove(pos);
        }
    }

    fn insert_released_blob(&mut self, id: TextBlobId, entries: usize) {
        if entries == 0 {
            return;
        }

        if !self.blob_state.released_blob_set.insert(id)
            && let Some(pos) = self
                .blob_state
                .released_blob_lru
                .iter()
                .position(|v| *v == id)
        {
            self.blob_state.released_blob_lru.remove(pos);
        }
        self.blob_state.released_blob_lru.push_back(id);

        while self.blob_state.released_blob_lru.len() > entries {
            let Some(evict) = self.blob_state.released_blob_lru.pop_front() else {
                break;
            };
            self.blob_state.released_blob_set.remove(&evict);
            if self
                .blob_state
                .blobs
                .get(evict)
                .is_some_and(|b| b.ref_count() > 0)
            {
                continue;
            }
            self.evict_blob(evict);
        }
    }

    fn evict_blob(&mut self, blob: TextBlobId) {
        self.remove_released_blob(blob);

        let remove_shape = self
            .blob_state
            .blobs
            .get(blob)
            .is_some_and(|b| Arc::strong_count(b.shape_handle()) == 2);

        if let Some(key) = self.blob_state.blob_key_by_id.remove(&blob) {
            self.blob_state.blob_cache.remove(&key);
            if remove_shape {
                let shape_key = fret_render_text::TextShapeKey::from_blob_key(&key);
                self.layout_cache.remove_shape(&shape_key);
            }
        }
        let _ = self.blob_state.blobs.remove(blob);
    }
}
