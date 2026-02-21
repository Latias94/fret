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
    use fret_core::{FontId, Px, TextConstraints, TextInputRef, TextOverflow, TextStyle, TextWrap};

    fn shaper_with_bundled_fonts() -> ParleyShaper {
        let mut shaper = ParleyShaper::new_without_system_fonts();
        let added = shaper.add_fonts(
            fret_fonts::bootstrap_fonts()
                .iter()
                .chain(fret_fonts::emoji_fonts().iter())
                .chain(fret_fonts::cjk_lite_fonts().iter())
                .map(|b| b.to_vec()),
        );
        assert!(added > 0, "expected bundled fonts to load");
        shaper
    }

    fn prepare_lines(
        shaper: &mut ParleyShaper,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> Vec<crate::line_layout::TextLineLayout> {
        let scale = crate::effective_text_scale_factor(constraints.scale_factor);
        let snap_vertical = scale.fract().abs() > 1e-4;

        let wrapped =
            wrapper::wrap_with_constraints(shaper, TextInputRef::plain(text, style), constraints);
        let prepared = prepare_layout::prepare_layout_from_wrapped(
            text,
            wrapped,
            constraints,
            scale,
            snap_vertical,
        );
        prepared.lines.into_iter().map(|l| l.layout).collect()
    }

    fn caret_x_for_index_from_single_line(
        lines: &[crate::line_layout::TextLineLayout],
        index: usize,
    ) -> Px {
        assert_eq!(lines.len(), 1, "expected a single-line layout");
        caret_x_from_stops(lines[0].caret_stops.as_slice(), index)
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
}
