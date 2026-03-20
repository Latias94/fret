use super::atlas::GlyphKey;
use fret_core::{TextMetrics, geometry::Px};
use fret_render_text::TextDecoration;
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
    pub(crate) font_data_id: u64,
    pub(crate) face_index: u32,
    pub(crate) variation_key: u64,
    pub(crate) synthesis_embolden: bool,
    pub(crate) synthesis_skew_degrees: i8,
    pub(crate) glyph_id: u32,
    pub(crate) size_bits: u32,
    pub(crate) x_bin: u8,
    pub(crate) y_bin: u8,
    pub(crate) kind: &'static str,
}

#[derive(Debug, Clone)]
pub struct TextBlob {
    pub shape: Arc<TextShape>,
    pub paint_palette: Option<Arc<[Option<fret_core::Color>]>>,
    pub decorations: Arc<[TextDecoration]>,
    pub(super) ref_count: u32,
}

#[derive(Debug, Clone)]
pub struct TextShape {
    pub glyphs: Arc<[GlyphInstance]>,
    pub metrics: TextMetrics,
    pub lines: Arc<[TextLine]>,
    pub caret_stops: Arc<[(usize, Px)]>,
    pub missing_glyphs: u32,
    pub(crate) font_faces: Arc<[TextFontFaceUsage]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TextFontFaceUsage {
    pub(crate) font_data_id: u64,
    pub(crate) face_index: u32,
    pub(crate) variation_key: u64,
    pub(crate) synthesis_embolden: bool,
    /// Faux italic/oblique skew in degrees (fontique synthesis), applied at rasterization time.
    pub(crate) synthesis_skew_degrees: i8,
    pub(crate) glyphs: u32,
    pub(crate) missing_glyphs: u32,
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
