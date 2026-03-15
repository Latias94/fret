use crate::parley_shaper::ShapedCluster;
use fret_core::{CaretAffinity, HitTestResult, Point, Rect, Size, TextMetrics, geometry::Px};
use std::ops::Range;

fn utf8_grapheme_boundaries(text: &str) -> Vec<usize> {
    use unicode_segmentation::UnicodeSegmentation as _;

    let mut out: Vec<usize> = Vec::with_capacity(text.chars().count().saturating_add(2));
    out.push(0);
    for (i, _) in text.grapheme_indices(true) {
        out.push(i);
    }
    out.push(text.len());
    out.sort_unstable();
    out.dedup();
    out
}

pub fn caret_stops_for_slice(
    slice: &str,
    base_offset: usize,
    clusters: &[ShapedCluster],
    line_width_px: f32,
    scale: f32,
    kept_end: usize,
) -> Vec<(usize, Px)> {
    let mut out: Vec<(usize, Px)> = Vec::new();
    let boundaries = utf8_grapheme_boundaries(slice);

    if boundaries.is_empty() {
        return vec![(base_offset, Px(0.0))];
    }

    if clusters.is_empty() {
        for &b in &boundaries {
            let idx = base_offset + b;
            if idx > kept_end {
                continue;
            }
            let x = if b >= slice.len() {
                (line_width_px / scale).max(0.0)
            } else {
                0.0
            };
            out.push((idx, Px(x)));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
        out.dedup_by(|a, b| a.0 == b.0);
        return out;
    }

    let last_cluster_end = clusters
        .iter()
        .map(|c| c.text_range.end)
        .max()
        .unwrap_or(0)
        .min(slice.len());
    let effective_line_width_px = clusters
        .iter()
        .flat_map(|c| [c.x0, c.x1])
        .fold(line_width_px, |acc, x| acc.max(x.max(0.0)));

    let mut cluster_i = 0usize;
    for &b in &boundaries {
        let idx = base_offset + b;
        if idx > kept_end {
            continue;
        }

        while cluster_i + 1 < clusters.len() && clusters[cluster_i].text_range.end < b {
            cluster_i = cluster_i.saturating_add(1);
        }

        let x = if b <= clusters[0].text_range.start {
            let first = &clusters[0];
            if first.is_rtl {
                first.x1.max(0.0)
            } else {
                first.x0.max(0.0)
            }
        } else if b > last_cluster_end {
            let last = clusters.last().unwrap_or(&clusters[0]);
            if last.is_rtl {
                0.0
            } else {
                effective_line_width_px
            }
        } else if cluster_i >= clusters.len() {
            let last = clusters.last().unwrap_or(&clusters[0]);
            if last.is_rtl {
                0.0
            } else {
                line_width_px.max(0.0)
            }
        } else {
            let c = &clusters[cluster_i];
            let start = c.text_range.start.min(slice.len());
            let end = c.text_range.end.min(slice.len());

            if start == end {
                c.x0.max(0.0)
            } else if b <= start {
                if c.is_rtl {
                    c.x1.max(0.0)
                } else {
                    c.x0.max(0.0)
                }
            } else if b >= end {
                if c.is_rtl {
                    c.x0.max(0.0)
                } else {
                    c.x1.max(0.0)
                }
            } else {
                let denom = (end - start) as f32;
                let mut t = ((b - start) as f32 / denom).clamp(0.0, 1.0);
                if c.is_rtl {
                    t = 1.0 - t;
                }
                let w = (c.x1 - c.x0).max(0.0);
                (c.x0 + w * t).max(0.0)
            }
        };

        out.push((idx, Px((x / scale).max(0.0))));
    }

    out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.0.total_cmp(&b.1.0)));
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

pub fn caret_x_from_stops(stops: &[(usize, Px)], index: usize) -> Px {
    if stops.is_empty() {
        return Px(0.0);
    }
    if let Ok(pos) = stops.binary_search_by_key(&index, |(idx, _)| *idx) {
        return stops[pos].1;
    }
    match stops.partition_point(|(idx, _)| *idx <= index) {
        0 => stops[0].1,
        n => stops[n.saturating_sub(1)].1,
    }
}

pub fn hit_test_x_from_stops(stops: &[(usize, Px)], x: Px) -> usize {
    if stops.is_empty() {
        return 0;
    }
    let mut best = stops[0].0;
    let mut best_dist = (stops[0].1.0 - x.0).abs();
    for (idx, px) in stops {
        let dist = (px.0 - x.0).abs();
        if dist < best_dist {
            best = *idx;
            best_dist = dist;
        }
    }
    best
}

pub fn shaped_line_visual_x_bounds_px(line: &crate::parley_shaper::ShapedLineLayout) -> (f32, f32) {
    let fallback_max = line.width.max(0.0);
    if line.clusters.is_empty() {
        return (0.0, fallback_max);
    }

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    for c in &line.clusters {
        let a = c.x0;
        let b = c.x1;
        min_x = min_x.min(a.min(b));
        max_x = max_x.max(a.max(b));
    }

    if !min_x.is_finite() || !max_x.is_finite() || max_x < min_x {
        return (0.0, fallback_max);
    }

    (min_x, max_x.max(min_x))
}

pub fn shaped_line_visual_width_px(line: &crate::parley_shaper::ShapedLineLayout) -> f32 {
    let (min_x, max_x) = shaped_line_visual_x_bounds_px(line);
    (max_x - min_x).max(0.0)
}

pub fn metrics_from_wrapped_lines(
    lines: &[crate::parley_shaper::ShapedLineLayout],
    scale: f32,
) -> TextMetrics {
    let snap_vertical = scale.is_finite() && scale.fract().abs() > 1e-4 && scale >= 1.0;

    let mut first_baseline_px = lines.first().map(|l| l.baseline.max(0.0)).unwrap_or(0.0);
    if snap_vertical && let Some(first) = lines.first() {
        let top_px = 0.0_f32;
        let bottom_px = (top_px + first.line_height.max(0.0)).round().max(top_px);
        let height_px = (bottom_px - top_px).max(0.0);
        first_baseline_px = (top_px + first.baseline.max(0.0))
            .round()
            .clamp(top_px, top_px + height_px);
    }

    let mut max_w_px = 0.0_f32;
    let mut total_h_px = 0.0_f32;
    if snap_vertical {
        let mut top_px = 0.0_f32;
        for line in lines {
            max_w_px = max_w_px.max(shaped_line_visual_width_px(line));
            let bottom_px = (top_px + line.line_height.max(0.0)).round().max(top_px);
            top_px = bottom_px;
        }
        total_h_px = top_px;
    } else {
        for line in lines {
            max_w_px = max_w_px.max(shaped_line_visual_width_px(line));
            total_h_px += line.line_height.max(0.0);
        }
    }

    TextMetrics {
        size: fret_core::Size::new(
            Px((max_w_px / scale).max(0.0)),
            Px((total_h_px / scale).max(0.0)),
        ),
        baseline: Px((first_baseline_px / scale).max(0.0)),
    }
}

pub fn metrics_for_uniform_lines(
    max_w_px: f32,
    line_count: usize,
    baseline_px: f32,
    line_height_px: f32,
    scale: f32,
) -> TextMetrics {
    let snap_vertical = scale.is_finite() && scale.fract().abs() > 1e-4 && scale >= 1.0;

    let mut first_baseline_px = baseline_px.max(0.0);
    if snap_vertical {
        let top_px = 0.0_f32;
        let bottom_px = (top_px + line_height_px.max(0.0)).round().max(top_px);
        let height_px = (bottom_px - top_px).max(0.0);
        first_baseline_px = (top_px + baseline_px.max(0.0))
            .round()
            .clamp(top_px, top_px + height_px);
    }

    let total_h_px = if snap_vertical {
        let mut top_px = 0.0_f32;
        for _ in 0..line_count.max(1) {
            top_px = (top_px + line_height_px.max(0.0)).round().max(top_px);
        }
        top_px
    } else {
        line_height_px.max(0.0) * (line_count.max(1) as f32)
    };

    TextMetrics {
        size: fret_core::Size::new(
            Px((max_w_px.max(0.0) / scale).max(0.0)),
            Px((total_h_px / scale).max(0.0)),
        ),
        baseline: Px((first_baseline_px / scale).max(0.0)),
    }
}

#[derive(Debug, Clone)]
pub struct TextLineCluster {
    pub text_range: Range<usize>,
    pub x0: Px,
    pub x1: Px,
    pub is_rtl: bool,
}

pub trait TextLineGeometry {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn y_top(&self) -> Px;
    fn height(&self) -> Px;
    fn caret_stops(&self) -> &[(usize, Px)];
    fn clusters(&self) -> &[TextLineCluster];
}

pub trait TextLineDecorationGeometry: TextLineGeometry {
    fn y_baseline(&self) -> Px;
}

pub fn caret_rect_from_lines<L: TextLineGeometry>(
    lines: &[L],
    index: usize,
    affinity: CaretAffinity,
) -> Option<Rect> {
    if lines.is_empty() {
        return None;
    }

    let mut candidates: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if index >= line.start() && index <= line.end() {
            candidates.push(i);
        }
    }

    let line_idx = match candidates.as_slice() {
        [] => {
            if index <= lines[0].start() {
                0
            } else {
                lines.len().saturating_sub(1)
            }
        }
        [only] => *only,
        many => match affinity {
            CaretAffinity::Upstream => many[0],
            CaretAffinity::Downstream => many[many.len().saturating_sub(1)],
        },
    };

    let line = &lines[line_idx];
    let x = caret_x_from_stops(line.caret_stops(), index);
    Some(Rect::new(
        Point::new(x, line.y_top()),
        Size::new(Px(1.0), line.height()),
    ))
}

pub fn hit_test_point_from_lines<L: TextLineGeometry>(
    lines: &[L],
    point: Point,
) -> Option<HitTestResult> {
    if lines.is_empty() {
        return None;
    }

    let mut line_idx = 0usize;
    for (i, line) in lines.iter().enumerate() {
        let y0 = line.y_top().0;
        let y1 = (line.y_top().0 + line.height().0).max(y0);
        if point.y.0 >= y0 && point.y.0 < y1 {
            line_idx = i;
            break;
        }
        if point.y.0 >= y1 {
            line_idx = i;
        }
    }

    let line = &lines[line_idx];
    let index = hit_test_x_from_stops(line.caret_stops(), point.x);

    let mut affinity = CaretAffinity::Downstream;
    if line_idx + 1 < lines.len() && index == line.end() && lines[line_idx + 1].start() == index {
        affinity = CaretAffinity::Upstream;
    }

    Some(HitTestResult { index, affinity })
}

pub fn selection_rects_from_lines<L: TextLineGeometry>(
    lines: &[L],
    range: (usize, usize),
    out: &mut Vec<Rect>,
) {
    out.clear();
    if lines.is_empty() {
        return;
    }

    let (a, b) = (range.0.min(range.1), range.0.max(range.1));
    if a == b {
        return;
    }

    for line in lines {
        let start = a.max(line.start());
        let end = b.min(line.end());
        if start >= end {
            continue;
        }

        let clusters = line.clusters();
        if !clusters.is_empty() {
            for c in clusters.iter() {
                let seg_start = start.max(c.text_range.start);
                let seg_end = end.min(c.text_range.end);
                if seg_start >= seg_end {
                    continue;
                }

                let x0 = cluster_x_from_range(c, seg_start);
                let x1 = cluster_x_from_range(c, seg_end);
                let left = x0.0.min(x1.0);
                let right = x0.0.max(x1.0);
                if right <= left {
                    continue;
                }

                out.push(Rect::new(
                    Point::new(Px(left), line.y_top()),
                    Size::new(Px((right - left).max(0.0)), line.height()),
                ));
            }
        } else {
            let x0 = caret_x_from_stops(line.caret_stops(), start);
            let x1 = caret_x_from_stops(line.caret_stops(), end);
            let left = Px(x0.0.min(x1.0));
            let right = Px(x0.0.max(x1.0));

            out.push(Rect::new(
                Point::new(left, line.y_top()),
                Size::new(Px((right.0 - left.0).max(0.0)), line.height()),
            ));
        }
    }

    coalesce_selection_rects_in_place(out);
}

pub fn selection_rects_from_lines_clipped<L: TextLineGeometry>(
    lines: &[L],
    range: (usize, usize),
    clip: Rect,
    out: &mut Vec<Rect>,
) {
    out.clear();
    if lines.is_empty() {
        return;
    }

    let clip_x0 = clip.origin.x.0;
    let clip_y0 = clip.origin.y.0;
    let clip_x1 = clip_x0 + clip.size.width.0;
    let clip_y1 = clip_y0 + clip.size.height.0;
    if clip_x1 <= clip_x0 || clip_y1 <= clip_y0 {
        return;
    }

    let (a, b) = (range.0.min(range.1), range.0.max(range.1));
    if a == b {
        return;
    }

    let start_idx = lines.partition_point(|line| {
        let y0 = line.y_top().0;
        let y1 = (line.y_top().0 + line.height().0).max(y0);
        y1 <= clip_y0
    });
    let end_idx = lines.partition_point(|line| line.y_top().0 < clip_y1);
    let start_idx = start_idx.min(end_idx);
    if start_idx >= end_idx {
        return;
    }

    for line in &lines[start_idx..end_idx] {
        let start = a.max(line.start());
        let end = b.min(line.end());
        if start >= end {
            continue;
        }

        let y0 = line.y_top().0;
        let y1 = (line.y_top().0 + line.height().0).max(y0);

        let iy0 = y0.max(clip_y0);
        let iy1 = y1.min(clip_y1);
        if iy1 <= iy0 {
            continue;
        }

        let clusters = line.clusters();
        if !clusters.is_empty() {
            for c in clusters.iter() {
                let seg_start = start.max(c.text_range.start);
                let seg_end = end.min(c.text_range.end);
                if seg_start >= seg_end {
                    continue;
                }

                let x0 = cluster_x_from_range(c, seg_start).0;
                let x1 = cluster_x_from_range(c, seg_end).0;
                let left = x0.min(x1);
                let right = x0.max(x1);

                let ix0 = left.max(clip_x0);
                let ix1 = right.min(clip_x1);
                if ix1 <= ix0 {
                    continue;
                }

                out.push(Rect::new(
                    Point::new(Px(ix0), Px(iy0)),
                    Size::new(Px((ix1 - ix0).max(0.0)), Px((iy1 - iy0).max(0.0))),
                ));
            }
        } else {
            let x0 = caret_x_from_stops(line.caret_stops(), start).0;
            let x1 = caret_x_from_stops(line.caret_stops(), end).0;
            let left = x0.min(x1);
            let right = x0.max(x1);

            let ix0 = left.max(clip_x0);
            let ix1 = right.min(clip_x1);
            if ix1 <= ix0 {
                continue;
            }

            out.push(Rect::new(
                Point::new(Px(ix0), Px(iy0)),
                Size::new(Px((ix1 - ix0).max(0.0)), Px((iy1 - iy0).max(0.0))),
            ));
        }
    }

    coalesce_selection_rects_in_place(out);
}

fn cluster_x_from_range(cluster: &TextLineCluster, boundary: usize) -> Px {
    let start = cluster.text_range.start;
    let end = cluster.text_range.end;
    if start == end {
        return cluster.x0;
    }

    if boundary <= start {
        return if cluster.is_rtl {
            cluster.x1
        } else {
            cluster.x0
        };
    }
    if boundary >= end {
        return if cluster.is_rtl {
            cluster.x0
        } else {
            cluster.x1
        };
    }

    let denom = (end - start) as f32;
    if denom <= 0.0 {
        return cluster.x0;
    }

    let mut t = ((boundary - start) as f32 / denom).clamp(0.0, 1.0);
    if cluster.is_rtl {
        t = 1.0 - t;
    }
    let w = (cluster.x1.0 - cluster.x0.0).max(0.0);
    Px((cluster.x0.0 + w * t).max(0.0))
}

fn coalesce_selection_rects_in_place(rects: &mut Vec<Rect>) {
    if rects.len() <= 1 {
        return;
    }

    rects.sort_by(|a, b| {
        a.origin
            .y
            .0
            .total_cmp(&b.origin.y.0)
            .then_with(|| a.size.height.0.total_cmp(&b.size.height.0))
            .then_with(|| a.origin.x.0.total_cmp(&b.origin.x.0))
    });

    let mut out: Vec<Rect> = Vec::with_capacity(rects.len());
    for r in rects.drain(..) {
        match out.last_mut() {
            Some(prev)
                if prev.origin.y == r.origin.y
                    && prev.size.height == r.size.height
                    && r.origin.x.0 <= prev.origin.x.0 + prev.size.width.0 =>
            {
                let x0 = prev.origin.x.0.min(r.origin.x.0);
                let x1 = (prev.origin.x.0 + prev.size.width.0).max(r.origin.x.0 + r.size.width.0);
                prev.origin.x = Px(x0);
                prev.size.width = Px((x1 - x0).max(0.0));
            }
            _ => out.push(r),
        }
    }
    *rects = out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parley_shaper::ParleyShaper, prepare_layout, wrapper};
    use fret_core::{
        FontId, Point, Px, Rect, Size, TextConstraints, TextInputRef, TextLineHeightPolicy,
        TextOverflow, TextShapingStyle, TextSpan, TextStyle, TextWrap,
    };
    use std::sync::Arc;

    fn shaper_with_bundled_fonts() -> ParleyShaper {
        let mut shaper = ParleyShaper::new_without_system_fonts();
        let added = shaper.add_fonts(
            fret_fonts::bootstrap_fonts()
                .iter()
                .copied()
                .chain(
                    fret_fonts::default_profile()
                        .font_bytes_for_role(fret_fonts::BundledFontRole::EmojiFallback),
                )
                .chain(
                    fret_fonts::default_profile()
                        .font_bytes_for_role(fret_fonts::BundledFontRole::CjkFallback),
                )
                .map(|b| b.to_vec()),
        );
        assert!(added > 0, "expected bundled fonts to load");
        shaper
    }

    fn prepare_layout_for_test(
        shaper: &mut ParleyShaper,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> prepare_layout::PreparedLayout {
        let scale = crate::effective_text_scale_factor(constraints.scale_factor);
        let snap_vertical = scale.fract().abs() > 1e-4;

        let wrapped =
            wrapper::wrap_with_constraints(shaper, TextInputRef::plain(text, style), constraints);
        prepare_layout::prepare_layout_from_wrapped(
            text,
            wrapped,
            constraints,
            scale,
            snap_vertical,
        )
    }

    fn prepare_lines(
        shaper: &mut ParleyShaper,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> Vec<crate::line_layout::TextLineLayout> {
        prepare_layout_for_test(shaper, text, style, constraints)
            .lines
            .into_iter()
            .map(|l| l.layout)
            .collect()
    }

    fn prepare_layout_for_attributed_test(
        shaper: &mut ParleyShaper,
        text: &str,
        base: &TextStyle,
        spans: &[TextSpan],
        constraints: TextConstraints,
    ) -> prepare_layout::PreparedLayout {
        let scale = crate::effective_text_scale_factor(constraints.scale_factor);
        let snap_vertical = scale.fract().abs() > 1e-4;

        let wrapped = wrapper::wrap_with_constraints(
            shaper,
            TextInputRef::attributed(text, base, spans),
            constraints,
        );
        prepare_layout::prepare_layout_from_wrapped(
            text,
            wrapped,
            constraints,
            scale,
            snap_vertical,
        )
    }

    fn prepare_lines_attributed(
        shaper: &mut ParleyShaper,
        text: &str,
        base: &TextStyle,
        spans: &[TextSpan],
        constraints: TextConstraints,
    ) -> Vec<crate::line_layout::TextLineLayout> {
        prepare_layout_for_attributed_test(shaper, text, base, spans, constraints)
            .lines
            .into_iter()
            .map(|l| l.layout)
            .collect()
    }

    fn is_synthetic_rtl_char(ch: char) -> bool {
        // A minimal heuristic for test inputs; the production shaper determines RTL runs via
        // Unicode properties.
        matches!(
            ch,
            '\u{0590}'..='\u{05FF}' // Hebrew
                | '\u{0600}'..='\u{06FF}' // Arabic
                | '\u{0750}'..='\u{077F}' // Arabic Supplement
                | '\u{08A0}'..='\u{08FF}' // Arabic Extended-A
        )
    }

    fn synthetic_clusters_for_text(
        text: &str,
        advance: f32,
    ) -> Vec<crate::parley_shaper::ShapedCluster> {
        let mut out = Vec::new();
        let mut x = 0.0_f32;
        for (start, ch) in text.char_indices() {
            let end = start + ch.len_utf8();
            out.push(crate::parley_shaper::ShapedCluster {
                text_range: start..end,
                x0: x,
                x1: x + advance,
                is_rtl: is_synthetic_rtl_char(ch),
            });
            x += advance;
        }
        out
    }

    fn line_clusters_from_shaped(
        base_offset: usize,
        clusters: &[crate::parley_shaper::ShapedCluster],
    ) -> Arc<[TextLineCluster]> {
        let mut out: Vec<TextLineCluster> = Vec::with_capacity(clusters.len());
        for c in clusters {
            out.push(TextLineCluster {
                text_range: (base_offset + c.text_range.start)..(base_offset + c.text_range.end),
                x0: Px(c.x0.max(0.0)),
                x1: Px(c.x1.max(0.0)),
                is_rtl: c.is_rtl,
            });
        }
        Arc::from(out)
    }

    fn caret_x_for_index_from_single_line(
        lines: &[crate::line_layout::TextLineLayout],
        index: usize,
    ) -> Px {
        assert_eq!(lines.len(), 1, "expected a single-line layout");
        caret_x_from_stops(lines[0].caret_stops.as_slice(), index)
    }

    fn assert_caret_rects_are_non_degenerate_at_grapheme_boundaries(text: &str, style: &TextStyle) {
        use unicode_segmentation::UnicodeSegmentation as _;

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let mut shaper = shaper_with_bundled_fonts();
        let lines = prepare_lines(&mut shaper, text, style, constraints);
        assert_eq!(lines.len(), 1, "expected a single-line layout");

        let mut boundaries: Vec<usize> = Vec::new();
        boundaries.push(0);
        let mut cursor = 0usize;
        for g in text.graphemes(true) {
            cursor = cursor.saturating_add(g.len());
            boundaries.push(cursor.min(text.len()));
        }
        boundaries.sort_unstable();
        boundaries.dedup();

        let mut last_x = 0.0_f32;
        for idx in boundaries {
            assert!(
                text.is_char_boundary(idx),
                "expected grapheme boundary to be a char boundary: idx={idx}"
            );

            let x = caret_x_from_stops(lines[0].caret_stops.as_slice(), idx);
            assert!(
                x.0.is_finite(),
                "expected caret_x to be finite for idx={idx}, got {x:?}"
            );
            assert!(
                x.0 + 0.01 >= last_x,
                "expected caret_x to be monotonic for LTR text; idx={idx} last_x={last_x} x={x:?}"
            );
            last_x = x.0;

            let rect =
                caret_rect_from_lines(&lines, idx, CaretAffinity::Downstream).expect("caret rect");
            assert!(
                rect.origin.x.0.is_finite()
                    && rect.origin.y.0.is_finite()
                    && rect.size.width.0.is_finite()
                    && rect.size.height.0.is_finite(),
                "expected caret rect to be finite; idx={idx} rect={rect:?}"
            );
            assert!(
                rect.size.height.0 > 0.1 && rect.size.width.0 > 0.0,
                "expected non-degenerate caret rect; idx={idx} rect={rect:?}"
            );
        }
    }

    #[test]
    fn fixed_line_box_baseline_is_stable_across_fallback_glyphs() {
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(14.0),
            line_height: Some(Px(18.0)),
            line_height_policy: TextLineHeightPolicy::FixedFromStyle,
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let mut shaper = shaper_with_bundled_fonts();

        let baseline_for = |shaper: &mut ParleyShaper, text: &str| -> (Px, Px) {
            let prepared = prepare_layout_for_test(shaper, text, &style, constraints);
            assert_eq!(
                prepared.metrics.size.height,
                Px(18.0),
                "expected fixed line box height to remain stable for text={text:?}"
            );
            assert!(
                !prepared.lines.is_empty(),
                "expected at least one line for text={text:?}"
            );
            assert_eq!(
                prepared.lines[0].layout.height,
                Px(18.0),
                "expected first line height to match fixed line box for text={text:?}"
            );
            (
                prepared.metrics.baseline,
                prepared.lines[0].layout.y_baseline,
            )
        };

        let (baseline_ascii, line_baseline_ascii) = baseline_for(&mut shaper, "Settings");
        let (baseline_emoji, line_baseline_emoji) = baseline_for(&mut shaper, "Settings 😄");
        let (baseline_cjk, line_baseline_cjk) = baseline_for(&mut shaper, "Settings 漢字");
        let (baseline_mixed, line_baseline_mixed) = baseline_for(&mut shaper, "😄 漢字");

        assert_eq!(baseline_ascii, baseline_emoji);
        assert_eq!(baseline_ascii, baseline_cjk);
        assert_eq!(baseline_ascii, baseline_mixed);

        assert_eq!(line_baseline_ascii, line_baseline_emoji);
        assert_eq!(line_baseline_ascii, line_baseline_cjk);
        assert_eq!(line_baseline_ascii, line_baseline_mixed);
    }

    #[test]
    fn caret_stops_for_slice_interpolates_within_cluster_ltr() {
        let clusters = vec![crate::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: false,
        }];

        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let x_at = |i: usize| stops.iter().find(|(idx, _)| *idx == i).unwrap().1.0;

        assert_eq!(x_at(0), 0.0);
        assert_eq!(x_at(1), 10.0);
        assert_eq!(x_at(2), 20.0);
        assert_eq!(x_at(3), 30.0);
        assert_eq!(x_at(4), 40.0);
    }

    #[test]
    fn caret_stops_for_slice_interpolates_within_cluster_rtl() {
        let clusters = vec![crate::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: true,
        }];

        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let x_at = |i: usize| stops.iter().find(|(idx, _)| *idx == i).unwrap().1.0;

        assert_eq!(x_at(0), 40.0);
        assert_eq!(x_at(1), 30.0);
        assert_eq!(x_at(2), 20.0);
        assert_eq!(x_at(3), 10.0);
        assert_eq!(x_at(4), 0.0);
    }

    #[test]
    fn selection_rects_for_rtl_line_has_positive_width() {
        let clusters = vec![crate::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: true,
        }];
        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let line = crate::line_layout::TextLineLayout::new(
            0,
            4,
            Px(40.0),
            Px(0.0),
            Px(0.0),
            Px(10.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            stops,
            line_clusters_from_shaped(0, &clusters),
        );

        let mut rects = Vec::new();
        selection_rects_from_lines(&[line], (0, 4), &mut rects);
        assert_eq!(rects.len(), 1);
        assert!((rects[0].origin.x.0 - 0.0).abs() < 0.001);
        assert!((rects[0].size.width.0 - 40.0).abs() < 0.001);
    }

    #[test]
    fn hit_test_point_for_rtl_line_maps_left_edge_to_logical_end() {
        let clusters = vec![crate::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: true,
        }];
        let stops = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let line = crate::line_layout::TextLineLayout::new(
            0,
            4,
            Px(40.0),
            Px(0.0),
            Px(0.0),
            Px(10.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            stops,
            line_clusters_from_shaped(0, &clusters),
        );

        let left =
            hit_test_point_from_lines(std::slice::from_ref(&line), Point::new(Px(0.0), Px(5.0)))
                .expect("hit test");
        assert_eq!(left.index, 4);

        let right =
            hit_test_point_from_lines(std::slice::from_ref(&line), Point::new(Px(40.0), Px(5.0)))
                .expect("hit test");
        assert_eq!(right.index, 0);
    }

    #[test]
    fn mixed_direction_selection_rects_are_nonempty() {
        // Mixed LTR + RTL + numbers + punctuation.
        let text = "abc אבג (123)";
        let clusters = synthetic_clusters_for_text(text, 10.0);
        let stops = super::caret_stops_for_slice(
            text,
            0,
            &clusters,
            10.0 * clusters.len() as f32,
            1.0,
            text.len(),
        );
        let line = crate::line_layout::TextLineLayout::new(
            0,
            text.len(),
            Px(10.0 * clusters.len() as f32),
            Px(0.0),
            Px(0.0),
            Px(10.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            stops,
            line_clusters_from_shaped(0, &clusters),
        );

        let rtl_start = text.find('א').expect("hebrew start");
        let rtl_end = text.find('ג').expect("hebrew end") + 'ג'.len_utf8();

        let mut rects = Vec::new();
        selection_rects_from_lines(&[line], (rtl_start, rtl_end), &mut rects);
        assert_eq!(rects.len(), 1);
        assert!(
            rects[0].size.width.0 > 0.1,
            "expected a non-empty selection rect"
        );
    }

    #[test]
    fn mixed_direction_selection_rects_split_across_visual_runs() {
        // Simulate bidi reordering by assigning cluster x positions that do not correspond to the
        // logical order of the text ranges.
        let text = "aaa אבג def";
        let clusters = vec![
            crate::parley_shaper::ShapedCluster {
                text_range: 0..4, // "aaa "
                x0: 0.0,
                x1: 40.0,
                is_rtl: false,
            },
            crate::parley_shaper::ShapedCluster {
                text_range: 4..11, // "אבג "
                x0: 70.0,
                x1: 110.0,
                is_rtl: true,
            },
            crate::parley_shaper::ShapedCluster {
                text_range: 11..14, // "def"
                x0: 40.0,
                x1: 70.0,
                is_rtl: false,
            },
        ];

        let stops = super::caret_stops_for_slice(text, 0, &clusters, 110.0, 1.0, text.len());
        let line = crate::line_layout::TextLineLayout::new(
            0,
            text.len(),
            Px(110.0),
            Px(0.0),
            Px(0.0),
            Px(10.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            stops,
            line_clusters_from_shaped(0, &clusters),
        );

        let mut rects = Vec::new();
        selection_rects_from_lines(&[line], (0, 11), &mut rects);

        assert_eq!(
            rects.len(),
            2,
            "expected two disjoint visual spans, got {rects:?}"
        );
        rects.sort_by(|a, b| a.origin.x.0.total_cmp(&b.origin.x.0));

        assert!((rects[0].origin.x.0 - 0.0).abs() < 0.001);
        assert!((rects[0].size.width.0 - 40.0).abs() < 0.001);

        assert!((rects[1].origin.x.0 - 70.0).abs() < 0.001);
        assert!((rects[1].size.width.0 - 40.0).abs() < 0.001);
    }

    #[test]
    fn caret_stops_for_slice_use_grapheme_boundaries_for_combining_marks_and_emoji_sequences() {
        use unicode_segmentation::UnicodeSegmentation as _;

        let cases = [
            ("e\u{0301}x", "combining mark (e + acute)"),
            ("1\u{FE0F}\u{20E3}", "keycap sequence"),
            ("\u{1F1FA}\u{1F1F8}", "flag sequence"),
            ("\u{1F469}\u{200D}\u{1F4BB}", "zwj emoji sequence"),
        ];

        for (text, label) in cases {
            let clusters = vec![crate::parley_shaper::ShapedCluster {
                text_range: 0..text.len(),
                x0: 0.0,
                x1: 40.0,
                is_rtl: false,
            }];

            let stops = super::caret_stops_for_slice(text, 0, &clusters, 40.0, 1.0, text.len());
            let indices: Vec<usize> = stops.iter().map(|(idx, _)| *idx).collect();

            let mut expected: Vec<usize> = text.grapheme_indices(true).map(|(i, _)| i).collect();
            expected.push(text.len());
            expected.sort_unstable();
            expected.dedup();

            assert_eq!(
                indices, expected,
                "expected caret stops to land on grapheme boundaries ({label}): text={text:?} stops={indices:?} expected={expected:?}"
            );
        }
    }

    #[test]
    fn empty_string_produces_nonzero_line_metrics_and_caret_rect() {
        let mut shaper = shaper_with_bundled_fonts();

        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let prepared = prepare_layout_for_test(&mut shaper, "", &style, constraints);
        assert!(
            prepared.metrics.size.height.0 > 0.1,
            "expected empty string to have non-zero metrics height, got {:?}",
            prepared.metrics
        );
        assert!(
            prepared.metrics.baseline.0 >= 0.0
                && prepared.metrics.baseline.0 <= prepared.metrics.size.height.0 + 0.01,
            "expected empty string baseline to be within the metrics box, got {:?}",
            prepared.metrics
        );

        let lines: Vec<_> = prepared.lines.into_iter().map(|l| l.layout).collect();
        assert!(!lines.is_empty(), "expected at least one line layout");
        assert!(
            lines[0].height.0 > 0.1,
            "expected a non-zero line height for empty string, got {:?}",
            lines[0]
        );

        let caret = caret_rect_from_lines(&lines, 0, CaretAffinity::Downstream)
            .expect("expected caret rect for empty string");
        assert!(
            caret.size.height.0 > 0.1,
            "expected a non-zero caret rect height for empty string, got {caret:?}"
        );
    }

    #[test]
    fn selection_and_caret_rects_are_nonzero_even_with_zero_line_height_override() {
        let mut shaper = shaper_with_bundled_fonts();

        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            line_height: Some(Px(0.0)),
            line_height_policy: TextLineHeightPolicy::ExpandToFit,
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let lines = prepare_lines(&mut shaper, "a", &style, constraints);

        let caret =
            caret_rect_from_lines(&lines, 0, CaretAffinity::Downstream).expect("caret rect");
        assert!(
            caret.size.height.0 > 0.1,
            "expected a non-zero caret rect height even with a zero line-height override, got {caret:?}"
        );

        let mut rects: Vec<Rect> = Vec::new();
        selection_rects_from_lines(&lines, (0, 1), &mut rects);
        assert!(
            rects.iter().any(|r| r.size.height.0 > 0.1),
            "expected selection rects to have non-zero height even with a zero line-height override, got {rects:?}"
        );
    }

    #[test]
    fn selection_rects_clipped_do_not_return_zero_height_rects() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "hello world hello world hello world";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: Some(Px(60.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let lines = prepare_lines(&mut shaper, content, &style, constraints);

        let mut rects: Vec<Rect> = Vec::new();
        selection_rects_from_lines_clipped(
            &lines,
            (0, content.len()),
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(20.0))),
            &mut rects,
        );
        assert!(
            !rects.is_empty(),
            "expected selection_rects_from_lines_clipped to produce at least one rect"
        );
        for r in &rects {
            assert!(
                r.size.height.0 > 0.1,
                "expected clipped selection rect height to be non-zero, got {r:?}"
            );
        }
    }

    #[test]
    fn selection_rects_clipped_handles_mixed_bidi_word_wrap_and_clips_to_rect() {
        let mut shaper = shaper_with_bundled_fonts();

        // Ensure mixed directionality plus enough content to reliably wrap.
        let content = "hello אבג def ghi jkl אבג mno pqr stu אבג vwx yz";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        let max_width = Px(80.0);
        let constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let lines = prepare_lines(&mut shaper, content, &style, constraints);
        assert!(lines.len() >= 2, "expected mixed bidi content to wrap");
        let line0 = &lines[0];
        let line1 = &lines[1];

        // Clip both X and Y so we exercise trimming of partially visible runs/lines.
        let clip_y0 = line0.y_top.0 + (line0.height.0 * 0.5);
        let clip_y1 = line1.y_top.0 + (line1.height.0 * 0.5);
        let clip_width = Px((max_width.0 * 0.5).max(1.0));
        let clip_height = Px((clip_y1 - clip_y0).max(1.0));
        let clip = Rect::new(
            Point::new(Px(0.0), Px(clip_y0)),
            Size::new(clip_width, clip_height),
        );

        let clip_x0 = clip.origin.x.0;
        let clip_y0 = clip.origin.y.0;
        let clip_x1 = clip_x0 + clip.size.width.0;
        let clip_y1 = clip_y0 + clip.size.height.0;

        // Sanity: unclipped selection should extend beyond our clip width on at least one line.
        let mut full_rects: Vec<Rect> = Vec::new();
        selection_rects_from_lines(&lines, (0, content.len()), &mut full_rects);
        assert!(
            full_rects
                .iter()
                .any(|r| r.origin.x.0 + r.size.width.0 > clip_x1 + 0.5),
            "expected unclipped selection to extend beyond clip_x1"
        );

        let mut rects: Vec<Rect> = Vec::new();
        selection_rects_from_lines_clipped(&lines, (0, content.len()), clip, &mut rects);

        assert!(!rects.is_empty(), "expected clipped selection rects");
        assert!(
            rects.iter().any(|r| (r.origin.y.0 - clip_y0).abs() < 0.02),
            "expected at least one rect to be trimmed at clip_y0"
        );
        assert!(
            rects
                .iter()
                .any(|r| ((r.origin.y.0 + r.size.height.0) - clip_y1).abs() < 0.02),
            "expected at least one rect to be trimmed at clip_y1"
        );

        let eps = 0.02;
        for r in &rects {
            assert!(
                r.size.width.0 > 0.1 && r.size.height.0 > 0.1,
                "expected non-degenerate clipped rect, got {r:?}"
            );
            assert!(
                r.origin.x.0 + eps >= clip_x0
                    && r.origin.y.0 + eps >= clip_y0
                    && (r.origin.x.0 + r.size.width.0) <= clip_x1 + eps
                    && (r.origin.y.0 + r.size.height.0) <= clip_y1 + eps,
                "expected rect to be inside clip; rect={r:?} clip={clip:?}"
            );
        }

        // Rects should already be coalesced; ensure no overlap within the same (clipped) line slice.
        rects.sort_by(|a, b| {
            a.origin
                .y
                .0
                .total_cmp(&b.origin.y.0)
                .then_with(|| a.size.height.0.total_cmp(&b.size.height.0))
                .then_with(|| a.origin.x.0.total_cmp(&b.origin.x.0))
        });
        for w in rects.windows(2) {
            let a = &w[0];
            let b = &w[1];
            if a.origin.y == b.origin.y && a.size.height == b.size.height {
                assert!(
                    b.origin.x.0 + 0.001 >= a.origin.x.0 + a.size.width.0,
                    "expected rects on the same line slice to not overlap; a={a:?} b={b:?}"
                );
            }
        }
    }

    #[test]
    fn mixed_direction_word_wrap_selection_rects_for_rtl_range_are_nonempty() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "abc אבג (123) def ghi jkl mno pqr";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: Some(Px(70.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let lines = prepare_lines(&mut shaper, content, &style, constraints);

        let rtl_start = content.find('א').expect("hebrew start");
        let rtl_end = content.find('ג').expect("hebrew end") + 'ג'.len_utf8();

        let mut rects = Vec::new();
        selection_rects_from_lines(&lines, (rtl_start, rtl_end), &mut rects);
        assert!(
            rects.iter().any(|r| r.size.width.0 > 0.1),
            "expected a non-empty selection rect for wrapped RTL range: rects={rects:?}"
        );
    }

    #[test]
    fn mixed_direction_word_wrap_caret_affinity_at_soft_wrap_boundary_selects_previous_or_next_line()
     {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "abc אבג def ghi jkl";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        // Pick a width between "…def" and "…ghi" so the first line includes mixed-direction text.
        let single_line_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let single_lines = prepare_lines(&mut shaper, content, &style, single_line_constraints);
        let def_end = content.find("def").expect("def") + "def".len();
        let ghi_end = content.find("ghi").expect("ghi") + "ghi".len();
        let x_def_end = caret_x_for_index_from_single_line(&single_lines, def_end);
        let x_ghi_end = caret_x_for_index_from_single_line(&single_lines, ghi_end);
        assert!(
            x_ghi_end.0 > x_def_end.0 + 0.1,
            "expected ghi to advance beyond def in single-line layout"
        );

        let constraints = TextConstraints {
            max_width: Some(Px((x_def_end.0 + x_ghi_end.0) * 0.5)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let lines = prepare_lines(&mut shaper, content, &style, constraints);
        assert!(lines.len() >= 2, "expected the text to wrap");
        let line0 = &lines[0];
        let line1 = &lines[1];
        assert_eq!(
            line0.end, line1.start,
            "expected a shared soft-wrap boundary index"
        );
        let break_index = line1.start;

        let upstream = caret_rect_from_lines(&lines, break_index, CaretAffinity::Upstream)
            .expect("caret rect upstream");
        let downstream = caret_rect_from_lines(&lines, break_index, CaretAffinity::Downstream)
            .expect("caret rect downstream");

        assert!(
            (upstream.origin.y.0 - line0.y_top.0).abs() < 0.01,
            "expected upstream caret to be on the previous line at wrap boundary"
        );
        assert!(
            (downstream.origin.y.0 - line1.y_top.0).abs() < 0.01,
            "expected downstream caret to be on the next line at wrap boundary"
        );
    }

    #[test]
    fn mixed_direction_word_wrap_hit_test_reports_expected_affinity_at_soft_wrap_boundary() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "abc אבג def ghi jkl";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: Some(Px(70.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let lines = prepare_lines(&mut shaper, content, &style, constraints);
        assert!(lines.len() >= 2, "expected wrapped lines");
        let line0 = &lines[0];
        let line1 = &lines[1];
        assert_eq!(line0.end, line1.start, "expected shared boundary");
        let break_index = line1.start;

        let x0 = caret_x_from_stops(line0.caret_stops.as_slice(), break_index);
        let x1 = caret_x_from_stops(line1.caret_stops.as_slice(), break_index);

        let y0 = Px(line0.y_top.0 + line0.height.0 * 0.5);
        let y1 = Px(line1.y_top.0 + line1.height.0 * 0.5);

        let ht0 = hit_test_point_from_lines(&lines, Point::new(x0, y0)).expect("hit test (line0)");
        assert_eq!(ht0.index, break_index);
        assert_eq!(
            ht0.affinity,
            CaretAffinity::Upstream,
            "expected wrap-boundary hit on line0 to report upstream affinity"
        );

        let ht1 = hit_test_point_from_lines(&lines, Point::new(x1, y1)).expect("hit test (line1)");
        assert_eq!(ht1.index, break_index);
        assert_eq!(
            ht1.affinity,
            CaretAffinity::Downstream,
            "expected wrap-boundary hit on line1 to report downstream affinity"
        );
    }

    #[test]
    fn mixed_direction_word_wrap_selection_rects_are_coalesced_per_visual_line() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "abc אבג def ghi jkl mno pqr";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: Some(Px(70.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let lines = prepare_lines(&mut shaper, content, &style, constraints);

        let mut rects = Vec::new();
        selection_rects_from_lines(&lines, (0, content.len()), &mut rects);
        assert!(
            !rects.is_empty(),
            "expected selection rects for full-range selection"
        );

        rects.sort_by(|a, b| {
            a.origin
                .y
                .0
                .total_cmp(&b.origin.y.0)
                .then_with(|| a.size.height.0.total_cmp(&b.size.height.0))
                .then_with(|| a.origin.x.0.total_cmp(&b.origin.x.0))
        });

        let mut prev: Option<Rect> = None;
        for r in rects.iter() {
            assert!(
                r.size.width.0 > 0.0 && r.size.height.0 > 0.0,
                "expected non-degenerate selection rects, got {r:?}"
            );
            if let Some(p) = prev {
                if (p.origin.y.0 - r.origin.y.0).abs() < 1e-3
                    && (p.size.height.0 - r.size.height.0).abs() < 1e-3
                {
                    let p_end = p.origin.x.0 + p.size.width.0;
                    assert!(
                        r.origin.x.0 + 1e-3 >= p_end,
                        "expected coalesced selection rects to be non-overlapping on the same line: prev={p:?} next={r:?}"
                    );
                }
            }
            prev = Some(*r);
        }
    }

    #[test]
    fn caret_rects_are_non_degenerate_at_grapheme_boundaries_for_zwj_emoji() {
        let content = "👩‍👩‍👧‍👦 hello";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        assert_caret_rects_are_non_degenerate_at_grapheme_boundaries(content, &style);
    }

    #[test]
    fn caret_rects_are_non_degenerate_at_grapheme_boundaries_for_keycap_emoji() {
        let content = "1️⃣ hello";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        assert_caret_rects_are_non_degenerate_at_grapheme_boundaries(content, &style);
    }

    #[test]
    fn caret_rects_are_non_degenerate_at_grapheme_boundaries_for_regional_indicator_flag() {
        let content = "🇺🇸 hello";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        assert_caret_rects_are_non_degenerate_at_grapheme_boundaries(content, &style);
    }

    #[test]
    fn caret_rects_are_non_degenerate_at_grapheme_boundaries_for_vs16_emoji() {
        let content = "✈️ hello";
        let style = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        assert_caret_rects_are_non_degenerate_at_grapheme_boundaries(content, &style);
    }

    #[test]
    fn caret_affinity_at_soft_wrap_boundary_selects_previous_or_next_line() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "hello world";
        let style = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let single_line_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let single_lines = prepare_lines(&mut shaper, content, &style, single_line_constraints);
        let x_space_end = caret_x_for_index_from_single_line(&single_lines, 6);
        let x_w_start = caret_x_for_index_from_single_line(&single_lines, 6 + "w".len());

        // Force a soft wrap between the space and the next word.
        let max_width = Px((x_space_end.0 + x_w_start.0) * 0.5);
        let wrapped_constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let lines = prepare_lines(&mut shaper, content, &style, wrapped_constraints);
        assert!(lines.len() >= 2, "expected the text to wrap");
        let line0 = &lines[0];
        let line1 = &lines[1];

        let wrap_index = line1.start;
        assert_eq!(
            line0.end, wrap_index,
            "expected wrapped lines to share the boundary index"
        );

        let upstream =
            caret_rect_from_lines(&lines, wrap_index, CaretAffinity::Upstream).expect("caret rect");
        let downstream = caret_rect_from_lines(&lines, wrap_index, CaretAffinity::Downstream)
            .expect("caret rect");

        assert!(
            (upstream.origin.y.0 - line0.y_top.0).abs() < 0.01,
            "expected upstream caret to be on the previous line; upstream={upstream:?} line0={line0:?}"
        );
        assert!(
            (downstream.origin.y.0 - line1.y_top.0).abs() < 0.01,
            "expected downstream caret to be on the next line; downstream={downstream:?} line1={line1:?}"
        );
    }

    #[test]
    fn hit_test_point_reports_upstream_affinity_at_visual_line_end() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "hello world";
        let style = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let constraints = TextConstraints {
            max_width: Some(Px(60.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let lines = prepare_lines(&mut shaper, content, &style, constraints);
        assert!(lines.len() >= 2, "expected wrapped lines");
        let line0 = &lines[0];
        let line1 = &lines[1];
        assert_eq!(line0.end, line1.start, "expected a shared break index");

        let p0 = fret_core::Point::new(Px(10_000.0), Px(line0.y_top.0 + (line0.height.0 * 0.5)));
        let ht0 = hit_test_point_from_lines(&lines, p0).expect("hit test point");
        assert_eq!(ht0.index, line0.end);
        assert_eq!(ht0.affinity, CaretAffinity::Upstream);

        let p1 = fret_core::Point::new(Px(0.0), Px(line1.y_top.0 + (line1.height.0 * 0.5)));
        let ht1 = hit_test_point_from_lines(&lines, p1).expect("hit test point");
        assert_eq!(ht1.index, line1.start);
        assert_eq!(ht1.affinity, CaretAffinity::Downstream);
    }

    #[test]
    fn explicit_newline_boundary_is_not_ambiguous_for_caret_affinity() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "hello\nworld";
        let style = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let lines = prepare_lines(&mut shaper, content, &style, constraints);
        assert!(lines.len() >= 2, "expected multiple lines");
        let line0 = &lines[0];
        let line1 = &lines[1];

        let newline_index = content.find('\n').expect("expected newline");
        let after_newline = newline_index + "\n".len();

        assert_eq!(
            line0.end, newline_index,
            "expected line0 to end at newline index"
        );
        assert_eq!(
            line1.start, after_newline,
            "expected line1 to start after newline index"
        );

        let end_line0_upstream =
            caret_rect_from_lines(&lines, newline_index, CaretAffinity::Upstream).expect("caret");
        let end_line0_downstream =
            caret_rect_from_lines(&lines, newline_index, CaretAffinity::Downstream).expect("caret");
        assert!(
            (end_line0_upstream.origin.y.0 - line0.y_top.0).abs() < 0.01,
            "expected newline_index caret to be on line0 regardless of affinity"
        );
        assert!(
            (end_line0_downstream.origin.y.0 - line0.y_top.0).abs() < 0.01,
            "expected newline_index caret to be on line0 regardless of affinity"
        );

        let start_line1_upstream =
            caret_rect_from_lines(&lines, after_newline, CaretAffinity::Upstream).expect("caret");
        let start_line1_downstream =
            caret_rect_from_lines(&lines, after_newline, CaretAffinity::Downstream).expect("caret");
        assert!(
            (start_line1_upstream.origin.y.0 - line1.y_top.0).abs() < 0.01,
            "expected after_newline caret to be on line1 regardless of affinity"
        );
        assert!(
            (start_line1_downstream.origin.y.0 - line1.y_top.0).abs() < 0.01,
            "expected after_newline caret to be on line1 regardless of affinity"
        );
    }

    #[test]
    fn selection_rects_clipped_culls_offscreen_lines() {
        let mut lines = Vec::new();
        for i in 0..1000usize {
            let start = i * 4;
            let end = start + 4;
            lines.push(crate::line_layout::TextLineLayout::new(
                start,
                end,
                Px(100.0),
                Px((i as f32) * 10.0),
                Px(0.0),
                Px(10.0),
                Px(0.0),
                Px(0.0),
                Px(0.0),
                Px(0.0),
                vec![(start, Px(0.0)), (end, Px(100.0))],
                Arc::from([]),
            ));
        }

        let clip = Rect::new(
            Point::new(Px(0.0), Px(1000.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let mut rects = Vec::new();
        selection_rects_from_lines_clipped(&lines, (0, 4000), clip, &mut rects);

        assert_eq!(rects.len(), 10);
        for r in &rects {
            assert!(r.origin.y.0 >= 1000.0 && r.origin.y.0 < 1100.0);
            assert!(r.size.height.0 > 0.0);
        }
    }

    #[test]
    fn selection_rects_clipped_trims_partially_visible_line() {
        let line = crate::line_layout::TextLineLayout::new(
            0,
            4,
            Px(100.0),
            Px(0.0),
            Px(0.0),
            Px(10.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            vec![(0, Px(0.0)), (4, Px(100.0))],
            Arc::from([]),
        );
        let clip = Rect::new(Point::new(Px(0.0), Px(5.0)), Size::new(Px(100.0), Px(10.0)));
        let mut rects = Vec::new();
        selection_rects_from_lines_clipped(&[line], (0, 4), clip, &mut rects);

        assert_eq!(rects.len(), 1);
        assert!((rects[0].origin.y.0 - 5.0).abs() < 0.001);
        assert!((rects[0].size.height.0 - 5.0).abs() < 0.001);
    }

    #[test]
    fn trailing_space_at_soft_wrap_is_selectable() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "hello world";
        let style = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let single_line_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let single_lines = prepare_lines(&mut shaper, content, &style, single_line_constraints);
        let x_space_end = caret_x_for_index_from_single_line(&single_lines, 6);
        let x_w_end = caret_x_for_index_from_single_line(&single_lines, 7);
        assert!(
            x_w_end.0 > x_space_end.0 + 0.1,
            "expected the 'w' to advance beyond the trailing space"
        );

        let max_width = Px((x_space_end.0 + x_w_end.0) * 0.5);
        let wrapped_constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let lines = prepare_lines(&mut shaper, content, &style, wrapped_constraints);
        assert!(lines.len() >= 2, "expected the text to wrap");

        let first = &lines[0];
        assert!(
            first.end >= 6,
            "expected the first visual line to include the trailing space (end={})",
            first.end
        );

        let caret_after_o =
            caret_rect_from_lines(&lines, 5, CaretAffinity::Downstream).expect("caret rect");
        let caret_after_space =
            caret_rect_from_lines(&lines, 6, CaretAffinity::Upstream).expect("caret rect");
        assert!(
            caret_after_space.origin.x.0 > caret_after_o.origin.x.0 + 0.1,
            "expected the trailing space to have positive width in caret geometry"
        );

        let mut rects = Vec::new();
        selection_rects_from_lines(&lines, (5, 6), &mut rects);
        assert_eq!(rects.len(), 1);
        assert!(
            rects[0].size.width.0 > 0.1,
            "expected a non-empty selection rect for the trailing space"
        );
    }

    #[test]
    fn trailing_whitespace_run_at_soft_wrap_is_selectable() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "foo   bar";
        let style = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let single_line_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let single_lines = prepare_lines(&mut shaper, content, &style, single_line_constraints);

        let space_run_end = 6;
        let b_end = 7;

        let x_space_end = caret_x_for_index_from_single_line(&single_lines, space_run_end);
        let x_b_end = caret_x_for_index_from_single_line(&single_lines, b_end);
        assert!(
            x_b_end.0 > x_space_end.0 + 0.1,
            "expected 'b' to advance beyond the trailing whitespace"
        );

        let max_width = Px((x_space_end.0 + x_b_end.0) * 0.5);
        let wrapped_constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let lines = prepare_lines(&mut shaper, content, &style, wrapped_constraints);
        assert!(lines.len() >= 2, "expected the text to wrap");

        let first = &lines[0];
        assert!(
            first.end >= space_run_end,
            "expected the first visual line to include the trailing whitespace run (end={})",
            first.end
        );

        let caret_after_second_space =
            caret_rect_from_lines(&lines, 5, CaretAffinity::Downstream).expect("caret rect");
        let caret_after_space_run =
            caret_rect_from_lines(&lines, space_run_end, CaretAffinity::Upstream)
                .expect("caret rect");
        assert!(
            caret_after_space_run.origin.x.0 > caret_after_second_space.origin.x.0 + 0.1,
            "expected the trailing whitespace run to have positive width in caret geometry"
        );

        let mut rects = Vec::new();
        selection_rects_from_lines(&lines, (5, 6), &mut rects);
        assert_eq!(rects.len(), 1);
        assert!(
            rects[0].size.width.0 > 0.1,
            "expected a non-empty selection rect for the trailing whitespace"
        );
    }

    #[test]
    fn trailing_whitespace_run_at_soft_wrap_is_selectable_for_attributed_text() {
        let mut shaper = shaper_with_bundled_fonts();

        let content = "foo   bar";
        let style = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let spans = vec![
            TextSpan {
                len: 4,
                shaping: TextShapingStyle::default(),
                paint: Default::default(),
            },
            TextSpan {
                len: content.len() - 4,
                shaping: TextShapingStyle::default(),
                paint: Default::default(),
            },
        ];

        let single_line_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let single_lines = prepare_lines_attributed(
            &mut shaper,
            content,
            &style,
            &spans,
            single_line_constraints,
        );

        let space_run_end = 6;
        let b_end = 7;

        let x_space_end = caret_x_for_index_from_single_line(&single_lines, space_run_end);
        let x_b_end = caret_x_for_index_from_single_line(&single_lines, b_end);
        assert!(
            x_b_end.0 > x_space_end.0 + 0.1,
            "expected 'b' to advance beyond the trailing whitespace"
        );

        let max_width = Px((x_space_end.0 + x_b_end.0) * 0.5);
        let wrapped_constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let lines =
            prepare_lines_attributed(&mut shaper, content, &style, &spans, wrapped_constraints);
        assert!(lines.len() >= 2, "expected the text to wrap");

        let first = &lines[0];
        assert!(
            first.end >= space_run_end,
            "expected the first visual line to include the trailing whitespace run (end={})",
            first.end
        );

        let caret_after_second_space =
            caret_rect_from_lines(&lines, 5, CaretAffinity::Downstream).expect("caret rect");
        let caret_after_space_run =
            caret_rect_from_lines(&lines, space_run_end, CaretAffinity::Upstream)
                .expect("caret rect");
        assert!(
            caret_after_space_run.origin.x.0 > caret_after_second_space.origin.x.0 + 0.1,
            "expected the trailing whitespace run to have positive width in caret geometry"
        );

        let mut rects = Vec::new();
        selection_rects_from_lines(&lines, (5, 6), &mut rects);
        assert_eq!(rects.len(), 1);
        assert!(
            rects[0].size.width.0 > 0.1,
            "expected a non-empty selection rect for the trailing whitespace"
        );
    }

    #[test]
    fn rtl_multiline_hit_test_maps_line_edges_to_logical_ends() {
        let clusters = vec![crate::parley_shaper::ShapedCluster {
            text_range: 0..4,
            x0: 0.0,
            x1: 40.0,
            is_rtl: true,
        }];

        let stops0 = super::caret_stops_for_slice("abcd", 0, &clusters, 40.0, 1.0, 4);
        let line0 = crate::line_layout::TextLineLayout::new(
            0,
            4,
            Px(40.0),
            Px(0.0),
            Px(0.0),
            Px(10.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            stops0,
            line_clusters_from_shaped(0, &clusters),
        );

        let stops1 = super::caret_stops_for_slice("efgh", 4, &clusters, 40.0, 1.0, 8);
        let line1 = crate::line_layout::TextLineLayout::new(
            4,
            8,
            Px(40.0),
            Px(10.0),
            Px(0.0),
            Px(10.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            stops1,
            line_clusters_from_shaped(4, &clusters),
        );

        let lines = [line0, line1];

        let left0 = hit_test_point_from_lines(&lines, Point::new(Px(0.0), Px(5.0)))
            .expect("hit test line0");
        let right0 = hit_test_point_from_lines(&lines, Point::new(Px(40.0), Px(5.0)))
            .expect("hit test line0");
        assert_eq!(left0.index, 4);
        assert_eq!(right0.index, 0);

        let left1 = hit_test_point_from_lines(&lines, Point::new(Px(0.0), Px(15.0)))
            .expect("hit test line1");
        let right1 = hit_test_point_from_lines(&lines, Point::new(Px(40.0), Px(15.0)))
            .expect("hit test line1");
        assert_eq!(left1.index, 8);
        assert_eq!(right1.index, 4);
    }

    #[test]
    fn ellipsis_truncation_hit_test_maps_ellipsis_region_to_kept_end() {
        let text = "This is a long line that should truncate";
        let constraints = TextConstraints {
            max_width: Some(Px(80.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };
        let wrapped = wrapper::wrap_with_constraints(
            &mut shaper,
            TextInputRef::plain(text, &base),
            constraints,
        );

        assert_eq!(wrapped.lines.len(), 1);
        let kept_end = wrapped.kept_end;
        assert!(kept_end < text.len());

        let line_layout = &wrapped.lines[0];
        assert!(
            line_layout
                .clusters
                .iter()
                .any(|c| c.text_range == (kept_end..kept_end)),
            "expected a synthetic zero-length cluster at kept_end for ellipsis mapping"
        );

        let slice = &text[..kept_end];
        let caret_stops = super::caret_stops_for_slice(
            slice,
            0,
            &line_layout.clusters,
            line_layout.width,
            1.0,
            kept_end,
        );
        let line = crate::line_layout::TextLineLayout::new(
            0,
            kept_end,
            Px(line_layout.width),
            Px(0.0),
            Px(0.0),
            Px(10.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            Px(0.0),
            caret_stops,
            line_clusters_from_shaped(0, &line_layout.clusters),
        );

        let x = Px((line_layout.width - 1.0).max(0.0));
        let hit = hit_test_point_from_lines(&[line], Point::new(x, Px(5.0))).expect("hit test");
        assert_eq!(hit.index, kept_end);
    }
}
