use super::{TextShape, TextSystem};
use fret_core::{
    AttributedText, TextBlobId, TextConstraints, TextInputRef, TextMetrics, TextStyle,
};
use fret_render_text::decorations::TextDecorationMetricsPx;
use fret_render_text::font_instance_key::FontFaceKey;
use fret_render_text::font_trace::FontTraceFamilyResolved;
use std::sync::Arc;

impl TextSystem {
    #[allow(dead_code)]
    pub fn prepare_input(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        match input {
            TextInputRef::Plain { text, style } => self.prepare(text, style, constraints),
            TextInputRef::Attributed { text, base, spans } => {
                let spans = fret_render_text::spans::sanitize_spans_for_text(text, spans);
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
        let key = fret_render_text::cache_keys::TextBlobKey::new(
            text,
            style,
            constraints,
            self.font_stack_key,
        );
        self.prepare_with_key(key, style, None, constraints)
    }

    pub fn prepare_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let spans = fret_render_text::spans::sanitize_spans_for_text(
            rich.text.as_ref(),
            rich.spans.as_ref(),
        );
        if spans.is_none() {
            return self.prepare(rich.text.as_ref(), base_style, constraints);
        }
        let rich = AttributedText {
            text: rich.text.clone(),
            spans: spans.expect("non-empty spans"),
        };
        let key = fret_render_text::cache_keys::TextBlobKey::new_attributed(
            &rich,
            base_style,
            constraints,
            self.font_stack_key,
        );
        self.prepare_with_key(key, base_style, Some(rich.spans.as_ref()), constraints)
    }

    pub(super) fn maybe_record_font_trace_entry(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
        shape: &Arc<TextShape>,
    ) {
        let mut families: Vec<FontTraceFamilyResolved> =
            Vec::with_capacity(shape.font_faces.len().max(1));
        for usage in shape.font_faces.iter() {
            let family = self
                .family_name_for_face(usage.font_data_id, usage.face_index)
                .unwrap_or_else(|| {
                    format!(
                        "font_data_id={} face_index={}",
                        usage.font_data_id, usage.face_index
                    )
                });
            families.push(FontTraceFamilyResolved {
                family,
                glyphs: usage.glyphs,
                missing_glyphs: usage.missing_glyphs,
            });
        }
        self.font_trace.maybe_record(
            text,
            style,
            constraints,
            &self.fallback_policy,
            shape.missing_glyphs,
            families,
        );
    }

    pub(super) fn decoration_metrics_for_shape(
        &self,
        style: &TextStyle,
        scale: f32,
        shape: &Arc<TextShape>,
    ) -> Option<TextDecorationMetricsPx> {
        let usage = shape.font_faces.first()?;

        let face_key = FontFaceKey {
            font_data_id: usage.font_data_id,
            face_index: usage.face_index,
            variation_key: usage.variation_key,
            synthesis_embolden: usage.synthesis_embolden,
            synthesis_skew_degrees: usage.synthesis_skew_degrees,
        };

        let font_data = self
            .font_data_by_face
            .get(&(usage.font_data_id, usage.face_index))?;
        let coords: &[i16] = self
            .font_instance_coords_by_face
            .get(&face_key)
            .map(|v| v.as_ref())
            .unwrap_or(&[]);

        let ppem = style.size.0 * scale;
        fret_render_text::decorations::decoration_metrics_px_for_font_bytes(
            font_data.data.data(),
            usage.face_index,
            coords,
            ppem,
        )
    }

    fn family_name_for_face(&mut self, font_data_id: u64, face_index: u32) -> Option<String> {
        if let Some(name) = self
            .font_face_family_name_cache
            .get(&(font_data_id, face_index))
            .cloned()
        {
            return Some(name);
        }

        let font_data = self.font_data_by_face.get(&(font_data_id, face_index))?;
        let name = fret_render_text::font_names::best_family_name_from_font_bytes(
            font_data.data.data(),
            face_index,
        )?;
        self.font_face_family_name_cache
            .insert((font_data_id, face_index), name.clone());
        Some(name)
    }

    pub(super) fn wrap_for_prepare(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> crate::text::wrapper::WrappedLayout {
        crate::text::wrapper::wrap_with_constraints(&mut self.parley_shaper, input, constraints)
    }
}
