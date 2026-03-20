use super::super::{TextShape, TextSystem};
use fret_core::{TextConstraints, TextStyle};
use fret_render_text::{FontFaceKey, FontTraceFamilyResolved, TextDecorationMetricsPx};
use std::sync::Arc;

impl TextSystem {
    pub(super) fn maybe_record_font_trace_entry(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
        shape: &Arc<TextShape>,
    ) {
        let mut families: Vec<FontTraceFamilyResolved> =
            Vec::with_capacity(shape.font_faces().len().max(1));
        for usage in shape.font_faces() {
            let family = self
                .family_name_for_face(usage.font_data_id, usage.face_index)
                .unwrap_or_else(|| {
                    format!(
                        "font_data_id={} face_index={}",
                        usage.font_data_id, usage.face_index
                    )
                });
            families.push(FontTraceFamilyResolved::new(
                family,
                usage.glyphs,
                usage.missing_glyphs,
            ));
        }
        self.font_runtime.font_trace.maybe_record(
            text,
            style,
            constraints,
            &self.font_runtime.fallback_policy,
            &self.parley_shaper,
            shape.missing_glyphs(),
            families,
        );
    }

    pub(super) fn decoration_metrics_for_shape(
        &self,
        style: &TextStyle,
        scale: f32,
        shape: &Arc<TextShape>,
    ) -> Option<TextDecorationMetricsPx> {
        let usage = shape.font_faces().first()?;

        let face_key = FontFaceKey::new(
            usage.font_data_id,
            usage.face_index,
            usage.variation_key,
            usage.synthesis_embolden,
            usage.synthesis_skew_degrees,
        );

        let font_data = self
            .face_cache
            .font_data_by_face
            .get(&(usage.font_data_id, usage.face_index))?;
        let coords: &[i16] = self
            .face_cache
            .font_instance_coords_by_face
            .get(&face_key)
            .map(|v| v.as_ref())
            .unwrap_or(&[]);

        let ppem = style.size.0 * scale;
        fret_render_text::decoration_metrics_px_for_font_bytes(
            font_data.bytes(),
            usage.face_index,
            coords,
            ppem,
        )
    }

    fn family_name_for_face(&mut self, font_data_id: u64, face_index: u32) -> Option<String> {
        if let Some(name) = self
            .face_cache
            .font_face_family_name_cache
            .get(&(font_data_id, face_index))
            .cloned()
        {
            return Some(name);
        }

        let font_data = self
            .face_cache
            .font_data_by_face
            .get(&(font_data_id, face_index))?;
        let name =
            fret_render_text::best_family_name_from_font_bytes(font_data.bytes(), face_index)?;
        self.face_cache
            .font_face_family_name_cache
            .insert((font_data_id, face_index), name.clone());
        Some(name)
    }
}
