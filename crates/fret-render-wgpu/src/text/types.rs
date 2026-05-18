use super::atlas::{GlyphKey, GlyphPinKeys};
use fret_core::{TextMetrics, geometry::Px};
use fret_render_text::{FontFaceKey, TextDecoration};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextRenderGlyphKind {
    Mask,
    Color,
    Subpixel,
}

#[derive(Debug, Clone)]
pub(super) struct GlyphInstance {
    /// Logical-space rect relative to the text baseline origin.
    rect: [f32; 4],
    paint_span: Option<u16>,
    pub(super) key: GlyphKey,
}

impl GlyphInstance {
    pub(super) fn new(rect: [f32; 4], paint_span: Option<u16>, key: GlyphKey) -> Self {
        Self {
            rect,
            paint_span,
            key,
        }
    }

    pub(super) fn is_color(&self) -> bool {
        self.key.is_color()
    }

    #[cfg(test)]
    pub(super) fn is_mask(&self) -> bool {
        self.key.is_mask()
    }

    pub(super) fn is_subpixel(&self) -> bool {
        self.key.is_subpixel()
    }

    pub(crate) fn render_kind(&self) -> TextRenderGlyphKind {
        if self.is_color() {
            TextRenderGlyphKind::Color
        } else if self.is_subpixel() {
            TextRenderGlyphKind::Subpixel
        } else {
            TextRenderGlyphKind::Mask
        }
    }

    #[cfg(test)]
    pub(super) fn is_mask_or_subpixel(&self) -> bool {
        self.is_mask() || self.is_subpixel()
    }

    pub(crate) fn rect(&self) -> [f32; 4] {
        self.rect
    }

    pub(crate) fn paint_span(&self) -> Option<u16> {
        self.paint_span
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
    pub(crate) fn new(
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

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub(crate) struct DebugGlyphAtlasLookup {
    font_data_id: u64,
    face_index: u32,
    variation_key: u64,
    synthesis_embolden: bool,
    synthesis_skew_degrees: i8,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    kind: &'static str,
}

#[cfg(not(target_arch = "wasm32"))]
impl DebugGlyphAtlasLookup {
    pub(crate) fn new(
        font_data_id: u64,
        face_index: u32,
        variation_key: u64,
        synthesis_embolden: bool,
        synthesis_skew_degrees: i8,
        glyph_id: u32,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
        kind: &'static str,
    ) -> Self {
        Self {
            font_data_id,
            face_index,
            variation_key,
            synthesis_embolden,
            synthesis_skew_degrees,
            glyph_id,
            size_bits,
            x_bin,
            y_bin,
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct TextBlob {
    shape: Arc<TextShape>,
    paint_palette: Option<Arc<[Option<fret_core::Color>]>>,
    decorations: Arc<[TextDecoration]>,
    ref_count: u32,
}

impl TextBlob {
    pub(super) fn new(
        shape: Arc<TextShape>,
        paint_palette: Option<Arc<[Option<fret_core::Color>]>>,
        decorations: Arc<[TextDecoration]>,
    ) -> Self {
        Self {
            shape,
            paint_palette,
            decorations,
            ref_count: 1,
        }
    }

    pub(crate) fn shape(&self) -> &TextShape {
        self.shape.as_ref()
    }

    pub(crate) fn shape_handle(&self) -> &Arc<TextShape> {
        &self.shape
    }

    pub(crate) fn paint_palette(&self) -> Option<&[Option<fret_core::Color>]> {
        self.paint_palette.as_deref()
    }

    pub(crate) fn decorations(&self) -> &[TextDecoration] {
        self.decorations.as_ref()
    }

    pub(crate) fn ref_count(&self) -> u32 {
        self.ref_count
    }

    pub(crate) fn increment_ref_count(&mut self) {
        self.ref_count = self.ref_count.saturating_add(1);
    }

    pub(crate) fn decrement_ref_count(&mut self) {
        self.ref_count = self.ref_count.saturating_sub(1);
    }

    pub(crate) fn mark_released(&mut self) {
        self.ref_count = 0;
    }

    pub(crate) fn is_released(&self) -> bool {
        self.ref_count == 0
    }
}

#[derive(Debug)]
pub(super) struct TextShape {
    glyphs: Arc<[GlyphInstance]>,
    pin_keys: GlyphPinKeys,
    metrics: TextMetrics,
    lines: Arc<[TextLine]>,
    caret_stops: Arc<[(usize, Px)]>,
    missing_glyphs: u32,
    font_faces: Arc<[TextFontFaceUsage]>,
    render_cache: RwLock<Option<TextShapeRenderCache>>,
}

impl TextShape {
    pub(super) fn new(
        glyphs: Arc<[GlyphInstance]>,
        metrics: TextMetrics,
        lines: Arc<[TextLine]>,
        caret_stops: Arc<[(usize, Px)]>,
        missing_glyphs: u32,
        font_faces: Arc<[TextFontFaceUsage]>,
    ) -> Self {
        let pin_keys = GlyphPinKeys::from_keys(glyphs.iter().map(|glyph| glyph.key));
        Self {
            glyphs,
            pin_keys,
            metrics,
            lines,
            caret_stops,
            missing_glyphs,
            font_faces,
            render_cache: RwLock::new(None),
        }
    }

    pub(crate) fn glyphs(&self) -> &[GlyphInstance] {
        self.glyphs.as_ref()
    }

    pub(super) fn pin_keys(&self) -> &GlyphPinKeys {
        &self.pin_keys
    }

    pub(crate) fn render_glyphs<F>(
        &self,
        atlas_revision: u64,
        mut resolve_uv: F,
    ) -> Arc<[TextRenderGlyph]>
    where
        F: FnMut(&GlyphInstance) -> Option<(u16, [f32; 4])>,
    {
        let cached = self.render_cache.read().ok().and_then(|cache| {
            cache
                .as_ref()
                .filter(|cache| cache.atlas_revision == atlas_revision)
                .map(|cache| cache.glyphs.clone())
        });
        if let Some(cached) = cached {
            return cached;
        }

        let mut render_glyphs: Vec<TextRenderGlyph> = Vec::with_capacity(self.glyphs.len());
        for glyph in self.glyphs.iter() {
            let Some((atlas_page, uv)) = resolve_uv(glyph) else {
                continue;
            };
            render_glyphs.push(TextRenderGlyph::new(
                glyph.render_kind(),
                glyph.rect(),
                glyph.paint_span(),
                atlas_page,
                uv,
            ));
        }

        let render_glyphs: Arc<[TextRenderGlyph]> = Arc::from(render_glyphs);
        if let Ok(mut cache) = self.render_cache.write() {
            match cache.as_ref() {
                Some(existing) if existing.atlas_revision == atlas_revision => {
                    existing.glyphs.clone()
                }
                _ => {
                    *cache = Some(TextShapeRenderCache {
                        atlas_revision,
                        glyphs: render_glyphs.clone(),
                    });
                    render_glyphs
                }
            }
        } else {
            render_glyphs
        }
    }

    pub(crate) fn metrics(&self) -> TextMetrics {
        self.metrics
    }

    pub(crate) fn lines(&self) -> &[TextLine] {
        self.lines.as_ref()
    }

    pub(crate) fn caret_stops(&self) -> &[(usize, Px)] {
        self.caret_stops.as_ref()
    }

    pub(crate) fn missing_glyphs(&self) -> u32 {
        self.missing_glyphs
    }

    pub(crate) fn font_faces(&self) -> &[TextFontFaceUsage] {
        self.font_faces.as_ref()
    }

    pub(crate) fn render_cache_bytes_estimate(&self) -> u64 {
        let Some(glyphs) = self
            .render_cache
            .read()
            .ok()
            .and_then(|cache| cache.as_ref().map(|cache| cache.glyphs.len()))
        else {
            return 0;
        };

        ((glyphs as u128) * (std::mem::size_of::<TextRenderGlyph>() as u128)).min(u64::MAX as u128)
            as u64
    }
}

#[derive(Debug, Clone)]
struct TextShapeRenderCache {
    atlas_revision: u64,
    glyphs: Arc<[TextRenderGlyph]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TextFontFaceUsage {
    font_data_id: u64,
    face_index: u32,
    variation_key: u64,
    synthesis_embolden: bool,
    /// Faux italic/oblique skew in degrees (fontique synthesis), applied at rasterization time.
    synthesis_skew_degrees: i8,
    glyphs: u32,
    missing_glyphs: u32,
}

impl TextFontFaceUsage {
    pub(super) fn new(
        font_data_id: u64,
        face_index: u32,
        variation_key: u64,
        synthesis_embolden: bool,
        synthesis_skew_degrees: i8,
        glyphs: u32,
        missing_glyphs: u32,
    ) -> Self {
        Self {
            font_data_id,
            face_index,
            variation_key,
            synthesis_embolden,
            synthesis_skew_degrees,
            glyphs,
            missing_glyphs,
        }
    }

    pub(crate) fn font_data_id(&self) -> u64 {
        self.font_data_id
    }

    pub(crate) fn face_index(&self) -> u32 {
        self.face_index
    }

    pub(crate) fn variation_key(&self) -> u64 {
        self.variation_key
    }

    pub(crate) fn synthesis_embolden(&self) -> bool {
        self.synthesis_embolden
    }

    pub(crate) fn synthesis_skew_degrees(&self) -> i8 {
        self.synthesis_skew_degrees
    }

    pub(crate) fn glyphs(&self) -> u32 {
        self.glyphs
    }

    pub(crate) fn missing_glyphs(&self) -> u32 {
        self.missing_glyphs
    }

    pub(crate) fn face_key(&self) -> FontFaceKey {
        FontFaceKey::new(
            self.font_data_id,
            self.face_index,
            self.variation_key,
            self.synthesis_embolden,
            self.synthesis_skew_degrees,
        )
    }
}

pub use fret_render_text::TextLineLayout as TextLine;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TextAtlasPerfSnapshot {
    pub(crate) uploads: u64,
    pub(crate) upload_bytes: u64,
    pub(crate) evicted_glyphs: u64,
    pub(crate) evicted_pages: u64,
    pub(crate) evicted_page_glyphs: u64,
    pub(crate) resets: u64,
}

pub type TextFontFamilyConfig = fret_core::TextFontFamilyConfig;
