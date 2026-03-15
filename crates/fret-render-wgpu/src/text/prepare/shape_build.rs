use super::super::{GlyphInstance, TextFontFaceUsage, TextLine, TextShape, TextSystem};
use super::PrepareShapeBuildContext;
use fret_core::{TextConstraints, TextInputRef, TextMetrics, TextSpan, TextStyle, geometry::Px};
use fret_render_text::FontFaceKey;
use std::{collections::HashMap, sync::Arc};

impl TextSystem {
    pub(in super::super) fn begin_prepare_shape_build(
        &mut self,
        text: &str,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
    ) -> PrepareShapeBuildContext {
        let input = prepare_shape_input(text, style, spans);
        let wrapped = self.wrap_for_prepare(input, constraints);
        let epoch = next_prepare_shape_epoch(self);

        PrepareShapeBuildContext {
            wrapped,
            epoch,
            glyphs: Vec::new(),
            face_usage: HashMap::new(),
            lines: Vec::new(),
        }
    }

    pub(in super::super) fn finish_prepared_shape(
        &self,
        glyphs: Vec<GlyphInstance>,
        lines: Vec<TextLine>,
        face_usage: HashMap<FontFaceKey, (u32, u32)>,
        metrics: TextMetrics,
        missing_glyphs: u32,
        first_line_caret_stops: Vec<(usize, Px)>,
    ) -> Arc<TextShape> {
        let face_usages = prepared_shape_face_usages(face_usage);
        Arc::new(TextShape {
            glyphs: Arc::from(glyphs),
            metrics,
            lines: Arc::from(lines),
            caret_stops: Arc::from(first_line_caret_stops),
            missing_glyphs,
            font_faces: Arc::from(face_usages),
        })
    }
}

fn prepare_shape_input<'a>(
    text: &'a str,
    style: &'a TextStyle,
    spans: Option<&'a [TextSpan]>,
) -> TextInputRef<'a> {
    match spans {
        Some(spans) => TextInputRef::Attributed {
            text,
            base: style,
            spans,
        },
        None => TextInputRef::Plain { text, style },
    }
}

fn next_prepare_shape_epoch(text_system: &mut TextSystem) -> u64 {
    text_system.atlas_epoch.next()
}

fn prepared_shape_face_usages(
    face_usage: HashMap<FontFaceKey, (u32, u32)>,
) -> Vec<TextFontFaceUsage> {
    let mut face_usages: Vec<TextFontFaceUsage> = Vec::with_capacity(face_usage.len());
    for (face, (glyphs, missing)) in face_usage {
        face_usages.push(TextFontFaceUsage {
            font_data_id: face.font_data_id,
            face_index: face.face_index,
            variation_key: face.variation_key,
            synthesis_embolden: face.synthesis_embolden,
            synthesis_skew_degrees: face.synthesis_skew_degrees,
            glyphs,
            missing_glyphs: missing,
        });
    }
    sort_prepared_shape_face_usages(&mut face_usages);
    face_usages
}

fn sort_prepared_shape_face_usages(face_usages: &mut [TextFontFaceUsage]) {
    face_usages.sort_by(|a, b| {
        b.glyphs
            .cmp(&a.glyphs)
            .then_with(|| a.font_data_id.cmp(&b.font_data_id))
            .then_with(|| a.face_index.cmp(&b.face_index))
            .then_with(|| a.variation_key.cmp(&b.variation_key))
            .then_with(|| a.synthesis_embolden.cmp(&b.synthesis_embolden))
            .then_with(|| a.synthesis_skew_degrees.cmp(&b.synthesis_skew_degrees))
    });
}
