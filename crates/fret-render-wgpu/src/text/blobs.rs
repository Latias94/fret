use super::{GlyphInstance, GlyphQuadKind, TextBlob, TextDecoration, TextSystem};
use fret_core::{Color, TextBlobId, geometry::Px};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextRenderGlyphKind {
    Mask,
    Color,
    Subpixel,
}

impl From<GlyphQuadKind> for TextRenderGlyphKind {
    fn from(value: GlyphQuadKind) -> Self {
        match value {
            GlyphQuadKind::Mask => Self::Mask,
            GlyphQuadKind::Color => Self::Color,
            GlyphQuadKind::Subpixel => Self::Subpixel,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextRenderGlyph {
    kind: TextRenderGlyphKind,
    rect: [f32; 4],
    paint_span: Option<u16>,
    atlas_page: u16,
    uv: [f32; 4],
}

impl TextRenderGlyph {
    fn new(
        kind: TextRenderGlyphKind,
        rect: [f32; 4],
        paint_span: Option<u16>,
        atlas_page: u16,
        uv: [f32; 4],
    ) -> Self {
        Self {
            kind,
            rect,
            paint_span,
            atlas_page,
            uv,
        }
    }

    pub(crate) fn kind(&self) -> TextRenderGlyphKind {
        self.kind
    }

    pub(crate) fn rect(&self) -> [f32; 4] {
        self.rect
    }

    pub(crate) fn paint_span(&self) -> Option<u16> {
        self.paint_span
    }

    pub(crate) fn atlas_page(&self) -> u16 {
        self.atlas_page
    }

    pub(crate) fn uv(&self) -> [f32; 4] {
        self.uv
    }
}

pub(crate) struct TextBlobRenderData<'a> {
    text_system: &'a TextSystem,
    glyphs: &'a [GlyphInstance],
    baseline: Px,
    decorations: &'a [TextDecoration],
    paint_palette: Option<&'a [Option<Color>]>,
}

impl<'a> TextBlobRenderData<'a> {
    fn new(text_system: &'a TextSystem, blob: &'a TextBlob) -> Self {
        let shape = blob.shape();
        Self {
            text_system,
            glyphs: shape.glyphs(),
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

    pub(crate) fn glyphs(&'a self) -> impl Iterator<Item = TextRenderGlyph> + 'a {
        self.glyphs.iter().filter_map(|glyph| {
            let (atlas_page, uv) = self.text_system.glyph_uv_for_instance(glyph)?;
            Some(TextRenderGlyph::new(
                glyph.kind().into(),
                glyph.rect(),
                glyph.paint_span(),
                atlas_page,
                uv,
            ))
        })
    }
}

impl TextSystem {
    pub(super) fn blob(&self, id: TextBlobId) -> Option<&TextBlob> {
        self.blob_state.blobs.get(id)
    }

    pub(super) fn shape_for_blob(&self, id: TextBlobId) -> Option<&super::TextShape> {
        Some(self.blob(id)?.shape())
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
                self.layout_cache.shape_cache.remove(&shape_key);
            }
        }
        let _ = self.blob_state.blobs.remove(blob);
    }
}
