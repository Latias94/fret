use super::atlas::GlyphKey;
use fret_core::{TextMetrics, geometry::Px};
use fret_render_text::{FontFaceKey, TextDecoration};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GlyphInstance {
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

    pub fn kind(&self) -> GlyphQuadKind {
        self.key.kind
    }

    pub fn rect(&self) -> [f32; 4] {
        self.rect
    }

    pub fn paint_span(&self) -> Option<u16> {
        self.paint_span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphQuadKind {
    Mask,
    Color,
    Subpixel,
}

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
pub struct TextBlob {
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

    pub fn shape(&self) -> &TextShape {
        self.shape.as_ref()
    }

    pub(crate) fn shape_handle(&self) -> &Arc<TextShape> {
        &self.shape
    }

    pub fn paint_palette(&self) -> Option<&[Option<fret_core::Color>]> {
        self.paint_palette.as_deref()
    }

    pub fn decorations(&self) -> &[TextDecoration] {
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

#[derive(Debug, Clone)]
pub struct TextShape {
    glyphs: Arc<[GlyphInstance]>,
    metrics: TextMetrics,
    lines: Arc<[TextLine]>,
    caret_stops: Arc<[(usize, Px)]>,
    missing_glyphs: u32,
    font_faces: Arc<[TextFontFaceUsage]>,
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
        Self {
            glyphs,
            metrics,
            lines,
            caret_stops,
            missing_glyphs,
            font_faces,
        }
    }

    pub fn glyphs(&self) -> &[GlyphInstance] {
        self.glyphs.as_ref()
    }

    pub fn metrics(&self) -> TextMetrics {
        self.metrics
    }

    pub fn lines(&self) -> &[TextLine] {
        self.lines.as_ref()
    }

    pub fn caret_stops(&self) -> &[(usize, Px)] {
        self.caret_stops.as_ref()
    }

    pub fn missing_glyphs(&self) -> u32 {
        self.missing_glyphs
    }

    pub(crate) fn font_faces(&self) -> &[TextFontFaceUsage] {
        self.font_faces.as_ref()
    }
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

#[allow(dead_code)]
pub(crate) fn subpixel_mask_to_alpha(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 4);
    for rgba in data.chunks_exact(4) {
        out.push(rgba[0].max(rgba[1]).max(rgba[2]));
    }
    out
}

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
