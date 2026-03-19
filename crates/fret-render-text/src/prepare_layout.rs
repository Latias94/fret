use crate::geometry::{
    TextLineCluster, caret_stops_for_slice, metrics_from_wrapped_lines,
    shaped_line_visual_x_bounds_px,
};
use crate::line_layout::TextLineLayout;
use crate::parley_shaper::{ParleyGlyph, ShapedCluster};
use crate::wrapper::WrappedLayout;
use fret_core::{TextAlign, TextConstraints, TextMetrics, geometry::Px};
use std::ops::Range;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PreparedLine {
    layout: TextLineLayout,
    glyphs: Vec<ParleyGlyph>,
}

#[derive(Debug, Clone)]
pub struct PreparedLayout {
    kept_end: usize,
    metrics: TextMetrics,
    lines: Vec<PreparedLine>,
    first_line_caret_stops: Vec<(usize, Px)>,
    missing_glyphs: u32,
}

impl PreparedLine {
    fn new(layout: TextLineLayout, glyphs: Vec<ParleyGlyph>) -> Self {
        Self { layout, glyphs }
    }

    pub fn layout(&self) -> &TextLineLayout {
        &self.layout
    }

    pub fn glyphs(&self) -> &[ParleyGlyph] {
        &self.glyphs
    }

    pub fn into_parts(self) -> (TextLineLayout, Vec<ParleyGlyph>) {
        (self.layout, self.glyphs)
    }
}

impl PreparedLayout {
    fn new(
        kept_end: usize,
        metrics: TextMetrics,
        lines: Vec<PreparedLine>,
        first_line_caret_stops: Vec<(usize, Px)>,
        missing_glyphs: u32,
    ) -> Self {
        Self {
            kept_end,
            metrics,
            lines,
            first_line_caret_stops,
            missing_glyphs,
        }
    }

    pub fn kept_end(&self) -> usize {
        self.kept_end
    }

    pub fn metrics(&self) -> TextMetrics {
        self.metrics
    }

    pub fn lines(&self) -> &[PreparedLine] {
        &self.lines
    }

    pub fn first_line_caret_stops(&self) -> &[(usize, Px)] {
        &self.first_line_caret_stops
    }

    pub fn missing_glyphs(&self) -> u32 {
        self.missing_glyphs
    }

    pub fn into_parts(self) -> (usize, TextMetrics, Vec<PreparedLine>, Vec<(usize, Px)>, u32) {
        (
            self.kept_end,
            self.metrics,
            self.lines,
            self.first_line_caret_stops,
            self.missing_glyphs,
        )
    }
}

fn align_offset_px_for_line(
    constraints: TextConstraints,
    scale: f32,
    line_min_x_px: f32,
    line_visual_width_px: f32,
) -> f32 {
    let container_width_px = constraints
        .max_width
        .map(|w| w.0 * scale)
        .unwrap_or_else(|| line_visual_width_px.max(0.0));
    let slack_px = (container_width_px - line_visual_width_px.max(0.0)).max(0.0);
    let target_left_px = match constraints.align {
        TextAlign::Start => 0.0,
        TextAlign::Center => slack_px * 0.5,
        TextAlign::End => slack_px,
    };
    target_left_px - line_min_x_px
}

fn clusters_for_line(
    line_clusters: &[ShapedCluster],
    line_range: Range<usize>,
    kept_end: usize,
    line_align_offset_px: f32,
    scale: f32,
) -> Arc<[TextLineCluster]> {
    if line_clusters.is_empty() {
        return Arc::from([]);
    }

    let mut out: Vec<TextLineCluster> = Vec::with_capacity(line_clusters.len());
    for c in line_clusters {
        let start = (line_range.start + c.text_range.start).min(kept_end);
        let end = (line_range.start + c.text_range.end).min(kept_end);
        if start >= end {
            continue;
        }

        let x0 = ((c.x0 + line_align_offset_px) / scale).max(0.0);
        let x1 = ((c.x1 + line_align_offset_px) / scale).max(0.0);
        let x0 = if x0.is_finite() { Px(x0) } else { Px(0.0) };
        let x1 = if x1.is_finite() { Px(x1) } else { Px(0.0) };

        out.push(TextLineCluster {
            text_range: start..end,
            x0,
            x1,
            is_rtl: c.is_rtl,
        });
    }

    Arc::from(out)
}

pub fn prepare_layout_from_wrapped(
    text: &str,
    wrapped: WrappedLayout,
    constraints: TextConstraints,
    scale: f32,
    snap_vertical: bool,
) -> PreparedLayout {
    let (_, kept_end, line_ranges, mut wrapped_lines) = wrapped.into_parts();

    let first_baseline_px = wrapped_lines
        .first()
        .map(|l| l.baseline.max(0.0))
        .unwrap_or(0.0);
    let first_baseline_px = if snap_vertical && let Some(first) = wrapped_lines.first() {
        let top_px = 0.0_f32;
        let bottom_px = (top_px + first.line_height.max(0.0)).round().max(top_px);
        let height_px = (bottom_px - top_px).max(0.0);
        (top_px + first.baseline.max(0.0))
            .round()
            .clamp(top_px, top_px + height_px)
    } else {
        first_baseline_px
    };

    let metrics = metrics_from_wrapped_lines(&wrapped_lines, scale);

    let mut out_lines: Vec<PreparedLine> = Vec::with_capacity(wrapped_lines.len().max(1));
    let mut first_line_caret_stops: Vec<(usize, Px)> = Vec::new();
    let mut missing_glyphs: u32 = 0;

    let mut line_top_px = 0.0_f32;

    for (i, (range, mut line)) in line_ranges
        .into_iter()
        .zip(wrapped_lines.drain(..))
        .enumerate()
    {
        if snap_vertical {
            line_top_px = line_top_px.round();
        }

        let line_height_px_raw = line.line_height.max(0.0);
        let line_baseline_px_raw = line.baseline.max(0.0);

        let (line_height_px, baseline_pos_px) = if snap_vertical {
            let bottom_px = (line_top_px + line_height_px_raw).round().max(line_top_px);
            let height_px = (bottom_px - line_top_px).max(0.0);
            let baseline_pos_px = (line_top_px + line_baseline_px_raw)
                .round()
                .clamp(line_top_px, line_top_px + height_px);
            (height_px, baseline_pos_px)
        } else {
            (line_height_px_raw, line_top_px + line_baseline_px_raw)
        };

        let line_offset_px = baseline_pos_px - first_baseline_px;

        let slice = &text[range.clone()];
        let (line_min_x_px, line_max_x_px) = shaped_line_visual_x_bounds_px(&line);
        let line_visual_width_px = (line_max_x_px - line_min_x_px).max(0.0);
        let line_align_offset_px =
            align_offset_px_for_line(constraints, scale, line_min_x_px, line_visual_width_px);
        let line_align_offset = Px(line_align_offset_px / scale);

        let clusters = clusters_for_line(
            &line.clusters,
            range.clone(),
            kept_end,
            line_align_offset_px,
            scale,
        );

        let mut caret_stops = caret_stops_for_slice(
            slice,
            range.start,
            &line.clusters,
            line_visual_width_px.max(0.0),
            scale,
            kept_end,
        );
        if line_align_offset.0 != 0.0 {
            for (_, x) in caret_stops.iter_mut() {
                *x = Px(x.0 + line_align_offset.0);
            }
        }
        if i == 0 {
            first_line_caret_stops = caret_stops.clone();
        }

        for g in line.glyphs.iter_mut() {
            if g.id == 0 {
                missing_glyphs = missing_glyphs.saturating_add(1);
            }
            g.x += line_align_offset_px;
            g.y += line_offset_px;
            g.text_range = (range.start + g.text_range.start)..(range.start + g.text_range.end);
        }

        let layout = TextLineLayout::new(
            range.start,
            range.end.min(kept_end),
            Px((line_visual_width_px / scale).max(0.0)),
            Px((line_top_px / scale).max(0.0)),
            Px((baseline_pos_px / scale).max(0.0)),
            Px(((line_height_px / scale).max(0.0)).max(1.0)),
            Px((line.ascent.abs().max(0.0) / scale).max(0.0)),
            Px((line.descent.abs().max(0.0) / scale).max(0.0)),
            Px((line.ink_ascent.abs().max(0.0) / scale).max(0.0)),
            Px((line.ink_descent.abs().max(0.0) / scale).max(0.0)),
            caret_stops,
            clusters,
        );

        out_lines.push(PreparedLine::new(layout, line.glyphs));

        line_top_px += line_height_px;
    }

    // Safety: ensure we always return at least one line layout even for empty text.
    if out_lines.is_empty() {
        let caret_stops = vec![(0, Px(0.0))];
        first_line_caret_stops = caret_stops.clone();
        out_lines.push(PreparedLine::new(
            TextLineLayout::new(
                0,
                0,
                Px(0.0),
                Px(0.0),
                Px(0.0),
                Px(1.0),
                Px(0.0),
                Px(0.0),
                Px(0.0),
                Px(0.0),
                caret_stops,
                Arc::from([]),
            ),
            Vec::new(),
        ));
    }

    PreparedLayout::new(
        kept_end,
        metrics,
        out_lines,
        first_line_caret_stops,
        missing_glyphs,
    )
}
