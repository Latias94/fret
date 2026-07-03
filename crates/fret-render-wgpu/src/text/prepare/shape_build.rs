use super::super::{
    GlyphInstance, TextFontFaceUsage, TextGlyphClusterBuilder, TextLine, TextShape, TextSystem,
};
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

        PrepareShapeBuildContext {
            wrapped,
            glyphs: Vec::new(),
            clusters: Vec::new(),
            face_usage: HashMap::new(),
            lines: Vec::new(),
        }
    }

    pub(in super::super) fn finish_prepared_shape(
        &self,
        glyphs: Vec<GlyphInstance>,
        clusters: Vec<TextGlyphClusterBuilder>,
        lines: Vec<TextLine>,
        face_usage: HashMap<FontFaceKey, (u32, u32)>,
        metrics: TextMetrics,
        missing_glyphs: u32,
        first_line_caret_stops: Vec<(usize, Px)>,
    ) -> Arc<TextShape> {
        let face_usages = prepared_shape_face_usages(face_usage);
        let clusters = clusters
            .into_iter()
            .map(TextGlyphClusterBuilder::finish)
            .collect::<Vec<_>>();
        Arc::new(TextShape::new(
            Arc::from(glyphs),
            Arc::from(clusters),
            metrics,
            Arc::from(lines),
            Arc::from(first_line_caret_stops),
            missing_glyphs,
            Arc::from(face_usages),
        ))
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

fn prepared_shape_face_usages(
    face_usage: HashMap<FontFaceKey, (u32, u32)>,
) -> Vec<TextFontFaceUsage> {
    let mut face_usages: Vec<TextFontFaceUsage> = Vec::with_capacity(face_usage.len());
    for (face, (glyphs, missing)) in face_usage {
        let (font_data_id, face_index, variation_key, synthesis_embolden, synthesis_skew_degrees) =
            face.into_parts();
        face_usages.push(TextFontFaceUsage::new(
            font_data_id,
            face_index,
            variation_key,
            synthesis_embolden,
            synthesis_skew_degrees,
            glyphs,
            missing,
        ));
    }
    sort_prepared_shape_face_usages(&mut face_usages);
    face_usages
}

fn sort_prepared_shape_face_usages(face_usages: &mut [TextFontFaceUsage]) {
    face_usages.sort_by(|a, b| {
        b.glyphs()
            .cmp(&a.glyphs())
            .then_with(|| a.font_data_id().cmp(&b.font_data_id()))
            .then_with(|| a.face_index().cmp(&b.face_index()))
            .then_with(|| a.variation_key().cmp(&b.variation_key()))
            .then_with(|| a.synthesis_embolden().cmp(&b.synthesis_embolden()))
            .then_with(|| a.synthesis_skew_degrees().cmp(&b.synthesis_skew_degrees()))
    });
}
