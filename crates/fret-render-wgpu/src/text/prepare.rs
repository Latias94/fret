use super::atlas::subpixel_bin_as_float;
use super::{TextLine, TextSystem};
use fret_core::{
    AttributedText, TextBlobId, TextConstraints, TextInputRef, TextMetrics, TextStyle,
};
use fret_render_text::{
    FontFaceKey, TextBlobKey, WrappedLayout, sanitize_spans_for_text, wrap_with_constraints,
};
use std::{collections::HashMap, sync::Arc};

mod cache_flow;
mod driver;
mod face_metadata;
mod glyph_bounds;
mod glyph_face;
mod glyph_materialize;
mod glyph_raster;
mod glyph_render;
mod shape_build;

pub(super) struct PrepareShapeBuildContext {
    pub(super) wrapped: WrappedLayout,
    pub(super) epoch: u64,
    pub(super) glyphs: Vec<super::GlyphInstance>,
    pub(super) face_usage: HashMap<FontFaceKey, (u32, u32)>,
    pub(super) lines: Vec<TextLine>,
}

impl TextSystem {
    fn prepare_with_key(
        &mut self,
        key: TextBlobKey,
        style: &TextStyle,
        spans: Option<&[fret_core::TextSpan]>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        self.prepare_with_key_driver(key, style, spans, constraints)
    }

    #[allow(dead_code)]
    pub fn prepare_input(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        match input {
            TextInputRef::Plain { text, style } => self.prepare(text, style, constraints),
            TextInputRef::Attributed { text, base, spans } => {
                let spans = sanitize_spans_for_text(text, spans);
                if spans.is_none() {
                    return self.prepare(text, base, constraints);
                }
                let rich = AttributedText {
                    text: Arc::<str>::from(text),
                    spans: spans.expect("non-empty spans"),
                };
                self.prepare_attributed(&rich, base, constraints)
            }
        }
    }

    pub fn prepare(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let key = TextBlobKey::new(text, style, constraints, self.font_runtime.font_stack_key);
        self.prepare_with_key(key, style, None, constraints)
    }

    pub fn prepare_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let spans = sanitize_spans_for_text(rich.text.as_ref(), rich.spans.as_ref());
        if spans.is_none() {
            return self.prepare(rich.text.as_ref(), base_style, constraints);
        }
        let rich = AttributedText {
            text: rich.text.clone(),
            spans: spans.expect("non-empty spans"),
        };
        let key = TextBlobKey::new_attributed(
            &rich,
            base_style,
            constraints,
            self.font_runtime.font_stack_key,
        );
        self.prepare_with_key(key, base_style, Some(rich.spans.as_ref()), constraints)
    }

    pub(super) fn wrap_for_prepare(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> WrappedLayout {
        wrap_with_constraints(&mut self.parley_shaper, input, constraints)
    }
}

pub(super) fn glyph_offset_px(x_bin: u8, y_bin: u8) -> parley::swash::zeno::Vector {
    parley::swash::zeno::Vector::new(subpixel_bin_as_float(x_bin), subpixel_bin_as_float(y_bin))
}

const GLYPH_RENDER_SOURCES: [parley::swash::scale::Source; 3] = [
    parley::swash::scale::Source::ColorOutline(0),
    parley::swash::scale::Source::ColorBitmap(parley::swash::scale::StrikeWith::BestFit),
    parley::swash::scale::Source::Outline,
];

pub(super) fn glyph_render_sources() -> &'static [parley::swash::scale::Source; 3] {
    &GLYPH_RENDER_SOURCES
}

pub(super) fn glyph_render_at_bins(x_bin: u8, y_bin: u8) -> parley::swash::scale::Render<'static> {
    let mut render = parley::swash::scale::Render::new(glyph_render_sources());
    render.offset(glyph_offset_px(x_bin, y_bin));
    render
}

pub(super) fn font_ref_from_face_bytes<'a>(
    font_bytes: &'a [u8],
    face_index: u32,
) -> Option<parley::swash::FontRef<'a>> {
    parley::swash::FontRef::from_index(font_bytes, face_index as usize)
}

pub(super) fn build_glyph_scaler<'a>(
    parley_scale: &'a mut parley::swash::scale::ScaleContext,
    font_ref: parley::swash::FontRef<'a>,
    font_size: f32,
    normalized_coords: Option<&'a [i16]>,
) -> parley::swash::scale::Scaler<'a> {
    let mut scaler_builder = parley_scale
        .builder(font_ref)
        .size(font_size.max(1.0))
        .hint(false);
    if let Some(coords) = normalized_coords.filter(|coords| !coords.is_empty()) {
        scaler_builder = scaler_builder.normalized_coords(coords.iter());
    }
    scaler_builder.build()
}

pub(super) fn build_glyph_scaler_from_face_bytes<'a>(
    parley_scale: &'a mut parley::swash::scale::ScaleContext,
    font_bytes: &'a [u8],
    face_index: u32,
    font_size: f32,
    normalized_coords: Option<&'a [i16]>,
) -> Option<parley::swash::scale::Scaler<'a>> {
    let font_ref = font_ref_from_face_bytes(font_bytes, face_index)?;
    Some(build_glyph_scaler(
        parley_scale,
        font_ref,
        font_size,
        normalized_coords,
    ))
}
