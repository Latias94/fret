use super::super::{TextBlob, TextShape, TextSystem};
use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextStyle};
use fret_render_text::{ResolvedSpan, TextBlobKey, TextShapeKey};
use std::sync::Arc;

impl TextSystem {
    pub(in super::super) fn try_reuse_cached_blob(
        &mut self,
        key: &TextBlobKey,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> Option<(TextBlobId, TextMetrics)> {
        let id = self.blob_state.blob_cache.get(key).copied()?;

        let mut hit: Option<(TextMetrics, u32, Arc<TextShape>, bool)> = None;
        if let Some(blob) = self.blob_state.blobs.get_mut(id) {
            self.frame_perf.blob_cache_hits = self.frame_perf.blob_cache_hits.saturating_add(1);
            let was_released = blob.ref_count == 0;
            blob.ref_count = blob.ref_count.saturating_add(1);
            hit = Some((
                blob.shape.metrics(),
                blob.shape.missing_glyphs(),
                blob.shape.clone(),
                was_released,
            ));
        }

        if let Some((metrics, missing_glyphs, shape, was_released)) = hit {
            if was_released {
                self.remove_released_blob(id);
            }
            self.record_shape_prepare_metrics(missing_glyphs);
            self.maybe_record_font_trace_entry(text, style, constraints, &shape);
            return Some((id, metrics));
        }

        // Stale cache entry (shouldn't happen, but keep it robust).
        self.blob_state.blob_cache.remove(key);
        self.blob_state.blob_key_by_id.remove(&id);
        None
    }

    pub(in super::super) fn finalize_prepared_blob(
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
        let metrics = shape.metrics();
        let decorations =
            self.prepare_blob_decorations(style, scale, &shape, resolved_spans, snap_vertical);
        self.record_shape_prepare_metrics(shape.missing_glyphs());
        self.maybe_record_font_trace_entry(text, style, constraints, &shape);
        let id = self.insert_prepared_blob(shape, paint_palette, decorations);
        self.blob_state.blob_cache.insert(key.clone(), id);
        self.blob_state.blob_key_by_id.insert(id, key);
        (id, metrics)
    }

    pub(in super::super) fn try_reuse_cached_shape(
        &mut self,
        shape_key: &TextShapeKey,
    ) -> Option<Arc<TextShape>> {
        let shape = self.layout_cache.shape_cache.get(shape_key)?.clone();
        self.frame_perf.shape_cache_hits = self.frame_perf.shape_cache_hits.saturating_add(1);
        Some(shape)
    }

    pub(in super::super) fn cache_prepared_shape(
        &mut self,
        shape_key: TextShapeKey,
        shape: Arc<TextShape>,
    ) -> Arc<TextShape> {
        self.frame_perf.shapes_created = self.frame_perf.shapes_created.saturating_add(1);
        self.layout_cache
            .shape_cache
            .insert(shape_key, shape.clone());
        shape
    }

    fn prepare_blob_decorations(
        &self,
        style: &TextStyle,
        scale: f32,
        shape: &Arc<TextShape>,
        resolved_spans: Option<&[ResolvedSpan]>,
        snap_vertical: bool,
    ) -> Vec<super::super::TextDecoration> {
        let decoration_metrics = self.decoration_metrics_for_shape(style, scale, shape);
        resolved_spans
            .map(|spans| {
                fret_render_text::decorations_for_lines(
                    shape.lines(),
                    spans,
                    decoration_metrics,
                    scale,
                    snap_vertical,
                )
            })
            .unwrap_or_default()
    }

    fn record_shape_prepare_metrics(&mut self, missing_glyphs: u32) {
        self.frame_perf.record_missing_glyphs(missing_glyphs);
    }

    fn insert_prepared_blob(
        &mut self,
        shape: Arc<TextShape>,
        paint_palette: Option<Arc<[Option<fret_core::Color>]>>,
        decorations: Vec<super::super::TextDecoration>,
    ) -> TextBlobId {
        let id = self.blob_state.blobs.insert(TextBlob {
            shape,
            paint_palette,
            decorations: Arc::from(decorations),
            ref_count: 1,
        });
        self.frame_perf.blobs_created = self.frame_perf.blobs_created.saturating_add(1);
        id
    }
}
