use super::atlas::{GlyphKey, GlyphPinKeys};
use fret_core::{TextMetrics, geometry::Px};
use fret_render_text::{FontFaceKey, TextDecoration};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Range,
    sync::{Arc, RwLock},
};

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
    cluster_index: u32,
    pub(super) key: GlyphKey,
}

impl GlyphInstance {
    pub(super) fn new(
        rect: [f32; 4],
        paint_span: Option<u16>,
        cluster_index: u32,
        key: GlyphKey,
    ) -> Self {
        Self {
            rect,
            paint_span,
            cluster_index,
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

    pub(crate) fn cluster_index(&self) -> u32 {
        self.cluster_index
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
    clusters: Arc<[TextGlyphCluster]>,
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
        clusters: Arc<[TextGlyphCluster]>,
        metrics: TextMetrics,
        lines: Arc<[TextLine]>,
        caret_stops: Arc<[(usize, Px)]>,
        missing_glyphs: u32,
        font_faces: Arc<[TextFontFaceUsage]>,
    ) -> Self {
        let pin_keys = GlyphPinKeys::from_keys(glyphs.iter().map(|glyph| glyph.key));
        Self {
            glyphs,
            clusters,
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

    pub(crate) fn clusters(&self) -> &[TextGlyphCluster] {
        self.clusters.as_ref()
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

#[derive(Debug, Clone)]
pub(crate) struct TextGlyphCluster {
    line_index: u32,
    text_range: Range<usize>,
    glyph_range: Range<usize>,
    visual_bounds: [f32; 4],
    is_rtl: bool,
    font_fingerprint: u64,
    glyph_fingerprint: u64,
    paint_span: Option<u16>,
    mixed_paint_spans: bool,
}

impl TextGlyphCluster {
    #[allow(clippy::too_many_arguments)]
    fn new(
        line_index: u32,
        text_range: Range<usize>,
        glyph_range: Range<usize>,
        visual_bounds: [f32; 4],
        is_rtl: bool,
        font_fingerprint: u64,
        glyph_fingerprint: u64,
        paint_span: Option<u16>,
        mixed_paint_spans: bool,
    ) -> Self {
        Self {
            line_index,
            text_range,
            glyph_range,
            visual_bounds,
            is_rtl,
            font_fingerprint,
            glyph_fingerprint,
            paint_span,
            mixed_paint_spans,
        }
    }

    #[cfg(test)]
    pub(crate) fn line_index(&self) -> u32 {
        self.line_index
    }

    #[cfg(test)]
    pub(crate) fn text_range(&self) -> Range<usize> {
        self.text_range.clone()
    }

    pub(crate) fn glyph_range(&self) -> Range<usize> {
        self.glyph_range.clone()
    }

    pub(crate) fn visual_bounds(&self) -> [f32; 4] {
        self.visual_bounds
    }

    #[cfg(test)]
    pub(crate) fn is_rtl(&self) -> bool {
        self.is_rtl
    }

    #[cfg(test)]
    pub(crate) fn font_fingerprint(&self) -> u64 {
        self.font_fingerprint
    }

    #[cfg(test)]
    pub(crate) fn glyph_fingerprint(&self) -> u64 {
        self.glyph_fingerprint
    }

    #[cfg(test)]
    pub(crate) fn paint_span(&self) -> Option<u16> {
        self.paint_span
    }

    #[cfg(test)]
    pub(crate) fn mixed_paint_spans(&self) -> bool {
        self.mixed_paint_spans
    }

    pub(super) fn hash_residency_key(&self, state: &mut impl Hasher) {
        self.line_index.hash(state);
        self.text_range.start.hash(state);
        self.text_range.end.hash(state);
        self.glyph_range.start.hash(state);
        self.glyph_range.end.hash(state);
        for value in self.visual_bounds {
            value.to_bits().hash(state);
        }
        self.is_rtl.hash(state);
        self.font_fingerprint.hash(state);
        self.glyph_fingerprint.hash(state);
        self.paint_span.hash(state);
        self.mixed_paint_spans.hash(state);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TextGlyphClusterBuilder {
    line_index: u32,
    text_range: Range<usize>,
    visual_bounds: [f32; 4],
    is_rtl: bool,
    glyph_start: usize,
    glyph_end: usize,
    font_fingerprint: u64,
    glyph_fingerprint: u64,
    paint_span: Option<u16>,
    has_paint_span: bool,
    mixed_paint_spans: bool,
}

impl TextGlyphClusterBuilder {
    pub(super) fn new(
        line_index: u32,
        text_range: Range<usize>,
        visual_bounds: [f32; 4],
        is_rtl: bool,
    ) -> Self {
        Self {
            line_index,
            text_range,
            visual_bounds: sanitized_rect(visual_bounds),
            is_rtl,
            glyph_start: usize::MAX,
            glyph_end: 0,
            font_fingerprint: 0,
            glyph_fingerprint: 0,
            paint_span: None,
            has_paint_span: false,
            mixed_paint_spans: false,
        }
    }

    pub(super) fn text_range(&self) -> Range<usize> {
        self.text_range.clone()
    }

    pub(super) fn record_glyph(&mut self, glyph_index: usize, glyph: &GlyphInstance) {
        self.glyph_start = self.glyph_start.min(glyph_index);
        self.glyph_end = self.glyph_end.max(glyph_index.saturating_add(1));
        self.visual_bounds = union_rects(self.visual_bounds, glyph.rect());
        fold_hash(&mut self.font_fingerprint, &glyph.key.font);
        fold_hash(&mut self.glyph_fingerprint, &glyph.key);
        match (self.has_paint_span, self.paint_span == glyph.paint_span()) {
            (false, _) => {
                self.paint_span = glyph.paint_span();
                self.has_paint_span = true;
            }
            (true, false) => {
                self.mixed_paint_spans = true;
            }
            (true, true) => {}
        }
    }

    pub(super) fn finish(self) -> TextGlyphCluster {
        let glyph_range = if self.glyph_start == usize::MAX {
            0..0
        } else {
            self.glyph_start..self.glyph_end
        };
        TextGlyphCluster::new(
            self.line_index,
            self.text_range,
            glyph_range,
            self.visual_bounds,
            self.is_rtl,
            self.font_fingerprint,
            self.glyph_fingerprint,
            self.paint_span,
            self.mixed_paint_spans,
        )
    }
}

fn fold_hash<T: Hash>(fingerprint: &mut u64, value: &T) {
    let mut hasher = DefaultHasher::new();
    fingerprint.hash(&mut hasher);
    value.hash(&mut hasher);
    *fingerprint = hasher.finish();
}

fn sanitized_rect(rect: [f32; 4]) -> [f32; 4] {
    let [x, y, w, h] = rect;
    [
        finite_or_zero(x),
        finite_or_zero(y),
        finite_or_zero(w).max(0.0),
        finite_or_zero(h).max(0.0),
    ]
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn union_rects(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let a = sanitized_rect(a);
    let b = sanitized_rect(b);
    let ax1 = a[0] + a[2];
    let ay1 = a[1] + a[3];
    let bx1 = b[0] + b[2];
    let by1 = b[1] + b[3];
    let x0 = a[0].min(b[0]);
    let y0 = a[1].min(b[1]);
    let x1 = ax1.max(bx1);
    let y1 = ay1.max(by1);
    [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)]
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
