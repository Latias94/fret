use super::{PrepareShapeBuildContext, TextSystem};
use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextSpan, TextStyle};
use fret_render_text::cache_keys::{TextBlobKey, TextShapeKey};
use fret_render_text::spans::{ResolvedSpan, resolve_spans_for_text};
use std::sync::Arc;

impl TextSystem {
    pub(in super::super) fn prepare_with_key_driver(
        &mut self,
        mut key: TextBlobKey,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let text = key.text.clone();
        key.backend = 1;

        let scale = super::super::effective_text_scale_factor(constraints.scale_factor);
        let snap_vertical = scale.fract().abs() > 1e-4;

        if let Some(hit) = self.try_reuse_cached_blob(&key, text.as_ref(), style, constraints) {
            return hit;
        }
        self.frame_perf.blob_cache_misses = self.frame_perf.blob_cache_misses.saturating_add(1);

        let resolved_spans = spans.and_then(|spans| resolve_spans_for_text(text.as_ref(), spans));
        let paint_palette = prepare_paint_palette(resolved_spans.as_deref());
        let shape = self.prepare_or_reuse_shape(
            &TextShapeKey::from_blob_key(&key),
            text.as_ref(),
            style,
            spans,
            constraints,
            scale,
            snap_vertical,
            resolved_spans.as_deref(),
        );

        self.finalize_prepared_blob(
            key,
            text.as_ref(),
            style,
            constraints,
            shape,
            resolved_spans.as_deref(),
            paint_palette,
            scale,
            snap_vertical,
        )
    }

    fn prepare_or_reuse_shape(
        &mut self,
        shape_key: &TextShapeKey,
        text: &str,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
        scale: f32,
        snap_vertical: bool,
        resolved_spans: Option<&[ResolvedSpan]>,
    ) -> Arc<super::super::TextShape> {
        if let Some(shape) = self.try_reuse_cached_shape(shape_key) {
            return shape;
        }
        self.frame_perf.shape_cache_misses = self.frame_perf.shape_cache_misses.saturating_add(1);
        let shape = self.build_prepared_shape(
            text,
            style,
            spans,
            constraints,
            scale,
            snap_vertical,
            resolved_spans,
        );
        self.cache_prepared_shape(shape_key.clone(), shape)
    }

    fn build_prepared_shape(
        &mut self,
        text: &str,
        style: &TextStyle,
        spans: Option<&[TextSpan]>,
        constraints: TextConstraints,
        scale: f32,
        snap_vertical: bool,
        resolved_spans: Option<&[ResolvedSpan]>,
    ) -> Arc<super::super::TextShape> {
        let PrepareShapeBuildContext {
            wrapped,
            epoch,
            mut glyphs,
            mut face_usage,
            mut lines,
        } = self.begin_prepare_shape_build(text, style, spans, constraints);

        let prepared = fret_render_text::prepare_layout::prepare_layout_from_wrapped(
            text,
            wrapped,
            constraints,
            scale,
            snap_vertical,
        );
        let metrics = prepared.metrics;
        let missing_glyphs = prepared.missing_glyphs;
        let first_line_caret_stops = prepared.first_line_caret_stops;

        lines.reserve(prepared.lines.len().max(1));
        for prepared_line in prepared.lines {
            self.materialize_prepared_line(
                prepared_line,
                resolved_spans,
                scale,
                epoch,
                &mut glyphs,
                &mut face_usage,
                &mut lines,
            );
        }

        self.finish_prepared_shape(
            glyphs,
            lines,
            face_usage,
            metrics,
            missing_glyphs,
            first_line_caret_stops,
        )
    }
}

fn prepare_paint_palette(
    resolved_spans: Option<&[ResolvedSpan]>,
) -> Option<Arc<[Option<fret_core::Color>]>> {
    resolved_spans.map(|spans| {
        let mut palette: Vec<Option<fret_core::Color>> = Vec::with_capacity(spans.len());
        palette.extend(spans.iter().map(|span| span.fg));
        Arc::<[Option<fret_core::Color>]>::from(palette)
    })
}
