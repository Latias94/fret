use super::atlas::subpixel_bin_as_float;
use super::{GlyphInstance, TextBlob, TextFontFaceUsage, TextLine, TextShape, TextSystem};
use fret_core::{
    AttributedText, TextBlobId, TextConstraints, TextInputRef, TextMetrics, TextSpan, TextStyle,
    geometry::Px,
};
use fret_render_text::cache_keys::{TextBlobKey, TextShapeKey};
use fret_render_text::font_instance_key::FontFaceKey;
use fret_render_text::spans::ResolvedSpan;
use std::{collections::HashMap, sync::Arc};

mod face_metadata;
mod glyph_bounds;
mod glyph_face;
mod glyph_materialize;
mod glyph_raster;
mod glyph_render;

pub(super) struct PrepareShapeBuildContext {
    pub(super) wrapped: crate::text::wrapper::WrappedLayout,
    pub(super) epoch: u64,
    pub(super) glyphs: Vec<super::GlyphInstance>,
    pub(super) face_usage: HashMap<FontFaceKey, (u32, u32)>,
    pub(super) lines: Vec<TextLine>,
}

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

    pub(super) fn try_reuse_cached_blob(
        &mut self,
        key: &TextBlobKey,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> Option<(TextBlobId, TextMetrics)> {
        let id = self.blob_cache.get(key).copied()?;

        let mut hit: Option<(TextMetrics, u32, Arc<TextShape>, bool)> = None;
        if let Some(blob) = self.blobs.get_mut(id) {
            self.perf_frame_blob_cache_hits = self.perf_frame_blob_cache_hits.saturating_add(1);
            let was_released = blob.ref_count == 0;
            blob.ref_count = blob.ref_count.saturating_add(1);
            hit = Some((
                blob.shape.metrics,
                blob.shape.missing_glyphs,
                blob.shape.clone(),
                was_released,
            ));
        }

        if let Some((metrics, missing_glyphs, shape, was_released)) = hit {
            if was_released {
                self.remove_released_blob(id);
            }
            if missing_glyphs > 0 {
                self.perf_frame_missing_glyphs = self
                    .perf_frame_missing_glyphs
                    .saturating_add(u64::from(missing_glyphs));
                self.perf_frame_texts_with_missing_glyphs =
                    self.perf_frame_texts_with_missing_glyphs.saturating_add(1);
            }
            self.maybe_record_font_trace_entry(text, style, constraints, &shape);
            return Some((id, metrics));
        }

        // Stale cache entry (shouldn't happen, but keep it robust).
        self.blob_cache.remove(key);
        self.blob_key_by_id.remove(&id);
        None
    }

    pub(super) fn finalize_prepared_blob(
        &mut self,
        key: TextBlobKey,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
        shape: Arc<TextShape>,
        resolved_spans: Option<&[ResolvedSpan]>,
        paint_palette: Option<Arc<[Option<fret_core::Color>]>>,
        scale: f32,
        snap_vertical: bool,
    ) -> (TextBlobId, TextMetrics) {
        let decoration_metrics = self.decoration_metrics_for_shape(style, scale, &shape);
        let decorations: Vec<super::TextDecoration> = resolved_spans
            .map(|spans| {
                fret_render_text::decorations::decorations_for_lines(
                    shape.lines.as_ref(),
                    spans,
                    decoration_metrics,
                    scale,
                    snap_vertical,
                )
            })
            .unwrap_or_default();

        let metrics = shape.metrics;
        if shape.missing_glyphs > 0 {
            self.perf_frame_missing_glyphs = self
                .perf_frame_missing_glyphs
                .saturating_add(u64::from(shape.missing_glyphs));
            self.perf_frame_texts_with_missing_glyphs =
                self.perf_frame_texts_with_missing_glyphs.saturating_add(1);
        }
        self.maybe_record_font_trace_entry(text, style, constraints, &shape);
        let id = self.blobs.insert(TextBlob {
            shape,
            paint_palette,
            decorations: Arc::from(decorations),
            ref_count: 1,
        });
        self.perf_frame_blobs_created = self.perf_frame_blobs_created.saturating_add(1);
        self.blob_cache.insert(key.clone(), id);
        self.blob_key_by_id.insert(id, key);
        (id, metrics)
    }

    pub(super) fn try_reuse_cached_shape(
        &mut self,
        shape_key: &TextShapeKey,
    ) -> Option<Arc<TextShape>> {
        let shape = self.shape_cache.get(shape_key)?.clone();
        self.perf_frame_shape_cache_hits = self.perf_frame_shape_cache_hits.saturating_add(1);
        Some(shape)
    }

    pub(super) fn cache_prepared_shape(
        &mut self,
        shape_key: TextShapeKey,
        shape: Arc<TextShape>,
    ) -> Arc<TextShape> {
        self.perf_frame_shapes_created = self.perf_frame_shapes_created.saturating_add(1);
        self.shape_cache.insert(shape_key, shape.clone());
        shape
    }

    pub(super) fn begin_prepare_shape_build(
        &mut self,
        text: &str,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
    ) -> PrepareShapeBuildContext {
        let input = match spans {
            Some(spans) => TextInputRef::Attributed {
                text,
                base: style,
                spans,
            },
            None => TextInputRef::Plain { text, style },
        };
        let wrapped = self.wrap_for_prepare(input, constraints);
        let epoch = {
            let e = self.glyph_atlas_epoch;
            self.glyph_atlas_epoch = self.glyph_atlas_epoch.saturating_add(1);
            e
        };

        PrepareShapeBuildContext {
            wrapped,
            epoch,
            glyphs: Vec::new(),
            face_usage: HashMap::new(),
            lines: Vec::new(),
        }
    }

    pub(super) fn finish_prepared_shape(
        &self,
        glyphs: Vec<GlyphInstance>,
        lines: Vec<TextLine>,
        face_usage: HashMap<FontFaceKey, (u32, u32)>,
        metrics: TextMetrics,
        missing_glyphs: u32,
        first_line_caret_stops: Vec<(usize, Px)>,
    ) -> Arc<TextShape> {
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
        face_usages.sort_by(|a, b| {
            b.glyphs
                .cmp(&a.glyphs)
                .then_with(|| a.font_data_id.cmp(&b.font_data_id))
                .then_with(|| a.face_index.cmp(&b.face_index))
                .then_with(|| a.variation_key.cmp(&b.variation_key))
                .then_with(|| a.synthesis_embolden.cmp(&b.synthesis_embolden))
                .then_with(|| a.synthesis_skew_degrees.cmp(&b.synthesis_skew_degrees))
        });

        Arc::new(TextShape {
            glyphs: Arc::from(glyphs),
            metrics,
            lines: Arc::from(lines),
            caret_stops: Arc::from(first_line_caret_stops),
            missing_glyphs,
            font_faces: Arc::from(face_usages),
        })
    }

    pub(super) fn wrap_for_prepare(
        &mut self,
        input: TextInputRef<'_>,
        constraints: TextConstraints,
    ) -> crate::text::wrapper::WrappedLayout {
        crate::text::wrapper::wrap_with_constraints(&mut self.parley_shaper, input, constraints)
    }
}

fn prepared_glyph_offset_px(x_bin: u8, y_bin: u8) -> parley::swash::zeno::Vector {
    parley::swash::zeno::Vector::new(subpixel_bin_as_float(x_bin), subpixel_bin_as_float(y_bin))
}
