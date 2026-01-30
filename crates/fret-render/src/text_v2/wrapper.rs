use super::parley_shaper::{ParleyGlyph, ParleyShaper, ShapedCluster, ShapedLineLayout};
use fret_core::{CaretAffinity, TextConstraints, TextInputRef, TextOverflow, TextSpan, TextWrap};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

const ELLIPSIS: &str = "\u{2026}";

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WrappedLayout {
    pub text_len: usize,
    pub kept_end: usize,
    pub line_ranges: Vec<Range<usize>>,
    pub lines: Vec<ShapedLineLayout>,
}

impl WrappedLayout {
    pub fn hit_test_x(&self, line_index: usize, x: f32) -> (usize, CaretAffinity) {
        let Some(line) = self.lines.get(line_index) else {
            return (0, CaretAffinity::Downstream);
        };
        let Some(range) = self.line_ranges.get(line_index) else {
            return (0, CaretAffinity::Downstream);
        };

        let (idx_local, affinity) = hit_test_x(&line.clusters, x, range.len());
        let mut idx = range.start.saturating_add(idx_local);
        if idx > self.kept_end {
            idx = self.kept_end;
        }
        (idx, affinity)
    }
}

pub(crate) fn wrap_with_constraints(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    constraints: TextConstraints,
) -> WrappedLayout {
    let scale = constraints.scale_factor.max(1.0);
    let text_len = match input {
        TextInputRef::Plain { text, .. } => text.len(),
        TextInputRef::Attributed { text, .. } => text.len(),
    };

    let has_newlines = match input {
        TextInputRef::Plain { text, .. } => text.contains('\n'),
        TextInputRef::Attributed { text, .. } => text.contains('\n'),
    };
    if has_newlines {
        return wrap_with_newlines(shaper, input, constraints, scale);
    }

    match constraints {
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            ..
        } => {
            let out = wrap_none_ellipsis(shaper, input, text_len, max_width.0 * scale, scale);
            WrappedLayout {
                text_len,
                kept_end: out.kept_end,
                line_ranges: vec![0..out.kept_end],
                lines: vec![out.line],
            }
        }
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            ..
        } => wrap_word(shaper, input, text_len, max_width.0 * scale, scale),
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Grapheme,
            ..
        } => wrap_grapheme(shaper, input, text_len, max_width.0 * scale, scale),
        _ => WrappedLayout {
            text_len,
            kept_end: text_len,
            line_ranges: vec![0..text_len],
            lines: vec![shaper.shape_single_line(input, scale)],
        },
    }
}

fn wrap_with_newlines(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    constraints: TextConstraints,
    scale: f32,
) -> WrappedLayout {
    let (text, base, spans) = match input {
        TextInputRef::Plain { text, style } => (text, style, None),
        TextInputRef::Attributed { text, base, spans } => (text, base, Some(spans)),
    };

    let text_len = text.len();
    let max_width_px = constraints.max_width.map(|w| w.0 * scale);

    let mut line_ranges: Vec<Range<usize>> = Vec::new();
    let mut lines: Vec<ShapedLineLayout> = Vec::new();

    let mut p_start = 0usize;
    for (i, ch) in text.char_indices() {
        if ch != '\n' {
            continue;
        }
        push_paragraph(
            shaper,
            text,
            base,
            spans,
            p_start..i,
            constraints,
            max_width_px,
            scale,
            &mut line_ranges,
            &mut lines,
        );
        p_start = i + 1;
    }
    push_paragraph(
        shaper,
        text,
        base,
        spans,
        p_start..text_len,
        constraints,
        max_width_px,
        scale,
        &mut line_ranges,
        &mut lines,
    );

    WrappedLayout {
        text_len,
        kept_end: text_len,
        line_ranges,
        lines,
    }
}

fn push_paragraph(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &fret_core::TextStyle,
    spans: Option<&[TextSpan]>,
    range: Range<usize>,
    constraints: TextConstraints,
    max_width_px: Option<f32>,
    scale: f32,
    out_ranges: &mut Vec<Range<usize>>,
    out_lines: &mut Vec<ShapedLineLayout>,
) {
    let start = range.start.min(text.len());
    let end = range.end.min(text.len());
    let paragraph_range = start..end;

    match constraints {
        TextConstraints {
            max_width: Some(_),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            ..
        } => {
            let Some(max_w) = max_width_px else {
                return;
            };
            let slice = &text[paragraph_range.clone()];
            let spans =
                spans.map(|spans| slice_spans(spans, paragraph_range.start, paragraph_range.end));
            let shaped = match spans.as_ref() {
                Some(s) => wrap_none_ellipsis(
                    shaper,
                    TextInputRef::Attributed {
                        text: slice,
                        base,
                        spans: s.as_slice(),
                    },
                    slice.len(),
                    max_w,
                    scale,
                ),
                None => wrap_none_ellipsis(
                    shaper,
                    TextInputRef::plain(slice, base),
                    slice.len(),
                    max_w,
                    scale,
                ),
            };

            out_ranges.push(paragraph_range.start..(paragraph_range.start + shaped.kept_end));
            out_lines.push(shaped.line);
        }
        TextConstraints {
            max_width: Some(_),
            wrap: TextWrap::Word,
            ..
        } => {
            let Some(max_w) = max_width_px else {
                return;
            };
            let (ranges, lines) =
                wrap_word_range(shaper, text, base, spans, paragraph_range, max_w, scale);
            out_ranges.extend(ranges);
            out_lines.extend(lines);
        }
        TextConstraints {
            max_width: Some(_),
            wrap: TextWrap::Grapheme,
            ..
        } => {
            let Some(max_w) = max_width_px else {
                return;
            };
            let (ranges, lines) =
                wrap_grapheme_range(shaper, text, base, spans, paragraph_range, max_w, scale);
            out_ranges.extend(ranges);
            out_lines.extend(lines);
        }
        _ => {
            let line = shape_slice(shaper, text, base, spans, paragraph_range.clone(), scale);
            out_ranges.push(paragraph_range);
            out_lines.push(line);
        }
    }
}

fn wrap_none_ellipsis(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    text_len: usize,
    max_width_px: f32,
    scale: f32,
) -> WrappedSingleLineInternal {
    let (text, base, spans) = match input {
        TextInputRef::Plain { text, style } => (text, style, None),
        TextInputRef::Attributed { text, base, spans } => (text, base, Some(spans)),
    };

    let full = shaper.shape_single_line(input, scale);
    if full.width <= max_width_px + 0.5 {
        return WrappedSingleLineInternal {
            kept_end: text_len,
            line: full,
        };
    }

    let ellipsis = shaper.shape_single_line(TextInputRef::plain(ELLIPSIS, base), scale);
    let ellipsis_w = ellipsis.width.max(0.0);
    let available = (max_width_px - ellipsis_w).max(0.0);

    let mut cut_end = cut_end_for_available(text, &full.clusters, available);
    if cut_end < text_len && cut_end > 0 {
        cut_end = trim_trailing_whitespace(text, cut_end);
        cut_end = clamp_to_char_boundary(text, cut_end);
    }

    // Shape the kept prefix so truncation doesn't depend on the discarded suffix (important for
    // contextual shaping scripts).
    let mut kept = shape_prefix(shaper, text, base, spans, cut_end, scale);

    if kept.width > available + 0.5 {
        let cut2 = cut_end_for_available(&text[..cut_end], &kept.clusters, available);
        if cut2 < cut_end {
            cut_end = clamp_to_char_boundary(text, trim_trailing_whitespace(text, cut2));
            kept = shape_prefix(shaper, text, base, spans, cut_end, scale);
        }
    }

    let ellipsis_start_x = (max_width_px - ellipsis_w).max(0.0);

    let mut glyphs: Vec<ParleyGlyph> = kept.glyphs;
    glyphs.extend(ellipsis.glyphs.into_iter().map(|mut g| {
        g.x += ellipsis_start_x;
        g.text_range = empty_range_at(cut_end);
        g
    }));

    let mut clusters: Vec<ShapedCluster> = kept.clusters;
    clusters.push(ShapedCluster {
        text_range: empty_range_at(cut_end),
        x0: ellipsis_start_x,
        x1: ellipsis_start_x + ellipsis_w,
        is_rtl: false,
    });

    WrappedSingleLineInternal {
        kept_end: cut_end,
        line: ShapedLineLayout {
            width: max_width_px,
            ascent: kept.ascent.max(ellipsis.ascent),
            descent: kept.descent.max(ellipsis.descent),
            baseline: kept.baseline.max(ellipsis.baseline),
            line_height: kept.line_height.max(ellipsis.line_height),
            glyphs,
            clusters,
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
struct WrappedSingleLineInternal {
    kept_end: usize,
    line: ShapedLineLayout,
}

fn wrap_word(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    text_len: usize,
    max_width_px: f32,
    scale: f32,
) -> WrappedLayout {
    let (text, base, spans) = match input {
        TextInputRef::Plain { text, style } => (text, style, None),
        TextInputRef::Attributed { text, base, spans } => (text, base, Some(spans)),
    };

    let (line_ranges, lines) =
        wrap_word_range(shaper, text, base, spans, 0..text_len, max_width_px, scale);

    WrappedLayout {
        text_len,
        kept_end: text_len,
        line_ranges,
        lines,
    }
}

fn wrap_grapheme(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    text_len: usize,
    max_width_px: f32,
    scale: f32,
) -> WrappedLayout {
    let (text, base, spans) = match input {
        TextInputRef::Plain { text, style } => (text, style, None),
        TextInputRef::Attributed { text, base, spans } => (text, base, Some(spans)),
    };

    let (line_ranges, lines) =
        wrap_grapheme_range(shaper, text, base, spans, 0..text_len, max_width_px, scale);

    WrappedLayout {
        text_len,
        kept_end: text_len,
        line_ranges,
        lines,
    }
}

fn wrap_grapheme_range(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &fret_core::TextStyle,
    spans: Option<&[TextSpan]>,
    range: Range<usize>,
    max_width_px: f32,
    scale: f32,
) -> (Vec<Range<usize>>, Vec<ShapedLineLayout>) {
    let start = range.start.min(text.len());
    let end = range.end.min(text.len());

    if start >= end {
        return (
            vec![start..start],
            vec![shape_slice(shaper, text, base, spans, start..start, scale)],
        );
    }

    let mut lines: Vec<ShapedLineLayout> = Vec::new();
    let mut line_ranges: Vec<Range<usize>> = Vec::new();

    let mut offset = start;
    while offset < end {
        let slice = &text[offset..end];
        let full = shape_slice(shaper, text, base, spans, offset..end, scale);

        if full.width <= max_width_px + 0.5 {
            lines.push(full);
            line_ranges.push(offset..end);
            break;
        }

        let mut cut_end = wrap_grapheme_cut_end(slice, &full.clusters, max_width_px);
        cut_end = clamp_to_grapheme_boundary_down(slice, cut_end);

        if cut_end == 0 {
            cut_end = first_cluster_end(slice, &full.clusters);
            cut_end = clamp_to_grapheme_boundary_up(slice, cut_end);
        }
        if cut_end == 0 {
            cut_end = first_grapheme_end(slice);
        }

        let mut kept = shape_slice(shaper, text, base, spans, offset..(offset + cut_end), scale);
        if kept.width > max_width_px + 0.5 && cut_end > 0 {
            let cut2 = cut_end_for_available(&slice[..cut_end], &kept.clusters, max_width_px);
            if cut2 > 0 && cut2 < cut_end {
                cut_end = clamp_to_grapheme_boundary_down(slice, cut2);
                kept = shape_slice(shaper, text, base, spans, offset..(offset + cut_end), scale);
            }
        }

        if cut_end == 0 {
            break;
        }

        lines.push(kept);
        line_ranges.push(offset..(offset + cut_end));
        offset = offset.saturating_add(cut_end);
    }

    (line_ranges, lines)
}

fn wrap_word_range(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &fret_core::TextStyle,
    spans: Option<&[TextSpan]>,
    range: Range<usize>,
    max_width_px: f32,
    scale: f32,
) -> (Vec<Range<usize>>, Vec<ShapedLineLayout>) {
    let start = range.start.min(text.len());
    let end = range.end.min(text.len());

    if start >= end {
        return (
            vec![start..start],
            vec![shape_slice(shaper, text, base, spans, start..start, scale)],
        );
    }

    let mut lines: Vec<ShapedLineLayout> = Vec::new();
    let mut line_ranges: Vec<Range<usize>> = Vec::new();

    let mut offset = start;
    while offset < end {
        let slice = &text[offset..end];
        let full = shape_slice(shaper, text, base, spans, offset..end, scale);

        if full.width <= max_width_px + 0.5 {
            lines.push(full);
            line_ranges.push(offset..end);
            break;
        }

        let mut cut_end = wrap_word_cut_end(slice, &full.clusters, max_width_px);
        cut_end = clamp_to_char_boundary(slice, cut_end);

        if cut_end == 0 {
            cut_end = first_cluster_end(slice, &full.clusters);
            cut_end = clamp_to_char_boundary(slice, cut_end);
        }
        if cut_end == 0 {
            let first = slice.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            cut_end = first.max(1).min(slice.len());
            cut_end = clamp_to_char_boundary(slice, cut_end);
        }

        let mut kept = shape_slice(shaper, text, base, spans, offset..(offset + cut_end), scale);
        if kept.width > max_width_px + 0.5 && cut_end > 0 {
            let cut2 = cut_end_for_available(&slice[..cut_end], &kept.clusters, max_width_px);
            if cut2 > 0 && cut2 < cut_end {
                cut_end = clamp_to_char_boundary(slice, cut2);
                kept = shape_slice(shaper, text, base, spans, offset..(offset + cut_end), scale);
            }
        }

        if cut_end == 0 {
            break;
        }

        lines.push(kept);
        line_ranges.push(offset..(offset + cut_end));
        offset = offset.saturating_add(cut_end);
    }

    (line_ranges, lines)
}

fn shape_prefix(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &fret_core::TextStyle,
    spans: Option<&[TextSpan]>,
    end: usize,
    scale: f32,
) -> ShapedLineLayout {
    let slice = &text[..end.min(text.len())];
    match spans {
        Some(spans) => {
            let out = truncate_spans(spans, slice.len());
            shaper.shape_single_line(
                TextInputRef::Attributed {
                    text: slice,
                    base,
                    spans: &out,
                },
                scale,
            )
        }
        None => shaper.shape_single_line(TextInputRef::plain(slice, base), scale),
    }
}

fn truncate_spans(spans: &[TextSpan], end: usize) -> Vec<TextSpan> {
    let mut out: Vec<TextSpan> = Vec::new();
    let mut offset: usize = 0;
    for span in spans {
        if offset >= end {
            break;
        }
        let span_end = offset.saturating_add(span.len);
        if span_end <= end {
            out.push(span.clone());
        } else {
            let mut s = span.clone();
            s.len = end.saturating_sub(offset);
            out.push(s);
            break;
        }
        offset = span_end;
    }
    out
}

fn shape_slice(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &fret_core::TextStyle,
    spans: Option<&[TextSpan]>,
    range: Range<usize>,
    scale: f32,
) -> ShapedLineLayout {
    let start = range.start.min(text.len());
    let end = range.end.min(text.len());
    let slice = &text[start..end];
    match spans {
        Some(spans) => {
            let out = slice_spans(spans, start, end);
            shaper.shape_single_line(
                TextInputRef::Attributed {
                    text: slice,
                    base,
                    spans: &out,
                },
                scale,
            )
        }
        None => shaper.shape_single_line(TextInputRef::plain(slice, base), scale),
    }
}

fn slice_spans(spans: &[TextSpan], start: usize, end: usize) -> Vec<TextSpan> {
    let mut out: Vec<TextSpan> = Vec::new();
    let mut offset: usize = 0;

    for span in spans {
        let span_end = offset.saturating_add(span.len);
        if span_end <= start {
            offset = span_end;
            continue;
        }
        if offset >= end {
            break;
        }

        let take_start = start.max(offset);
        let take_end = end.min(span_end);
        if take_end > take_start {
            let mut s = span.clone();
            s.len = take_end - take_start;
            out.push(s);
        }

        offset = span_end;
    }

    out
}

fn wrap_word_cut_end(text: &str, clusters: &[ShapedCluster], max_width_px: f32) -> usize {
    let mut last_candidate: usize = 0;
    let mut first_non_whitespace: Option<usize> = None;
    let mut prev_ch: char = '\0';

    for c in clusters {
        if c.text_range.start >= text.len() {
            continue;
        }
        if c.x1 > max_width_px + 0.5 {
            break;
        }

        let Some(ch) = text[c.text_range.start..].chars().next() else {
            continue;
        };

        if ch != ' ' && first_non_whitespace.is_none() {
            first_non_whitespace = Some(c.text_range.start);
        }

        if first_non_whitespace.is_some() {
            if is_word_char(ch) {
                if prev_ch == ' ' && ch != ' ' {
                    last_candidate = c.text_range.start;
                }
            } else if ch != ' ' {
                last_candidate = c.text_range.start;
            }
        }

        prev_ch = ch;
    }

    if last_candidate > 0 {
        return last_candidate;
    }

    cut_end_for_available(text, clusters, max_width_px)
}

fn wrap_grapheme_cut_end(text: &str, clusters: &[ShapedCluster], max_width_px: f32) -> usize {
    let mut cut_end = 0usize;
    for c in clusters {
        if c.text_range.start >= text.len() {
            continue;
        }
        if c.x1 > max_width_px + 0.5 {
            break;
        }
        cut_end = cut_end.max(c.text_range.end.min(text.len()));
    }
    cut_end
}

fn first_cluster_end(text: &str, clusters: &[ShapedCluster]) -> usize {
    for c in clusters {
        let end = c.text_range.end.min(text.len());
        if end > 0 {
            return end;
        }
    }
    0
}

fn first_grapheme_end(text: &str) -> usize {
    text.grapheme_indices(true)
        .next()
        .map(|(start, g)| start + g.len())
        .unwrap_or(0)
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(c, '\u{00C0}'..='\u{00FF}')
        || matches!(c, '\u{0100}'..='\u{017F}')
        || matches!(c, '\u{0180}'..='\u{024F}')
        || matches!(c, '\u{0400}'..='\u{04FF}')
        || matches!(c, '\u{1E00}'..='\u{1EFF}')
        || matches!(c, '\u{0300}'..='\u{036F}')
        || matches!(
            c,
            '-' | '_' | '.' | '\'' | '$' | '%' | '@' | '#' | '^' | '~' | ',' | '=' | ':' | '?'
        )
}

fn cut_end_for_available(text: &str, clusters: &[ShapedCluster], available: f32) -> usize {
    let mut cut_end = 0usize;
    for c in clusters {
        if c.x1 <= available + 0.5 {
            cut_end = cut_end.max(c.text_range.end.min(text.len()));
        }
    }
    cut_end
}

fn trim_trailing_whitespace(text: &str, mut end: usize) -> usize {
    while end > 0
        && text
            .as_bytes()
            .get(end.saturating_sub(1))
            .is_some_and(|b| b.is_ascii_whitespace())
    {
        end = end.saturating_sub(1);
    }
    end
}

fn clamp_to_char_boundary(text: &str, mut end: usize) -> usize {
    while end > 0 && !text.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    end
}

fn is_grapheme_boundary(text: &str, idx: usize) -> bool {
    let idx = idx.min(text.len());
    if idx == 0 || idx == text.len() {
        return true;
    }
    text.grapheme_indices(true).any(|(start, _)| start == idx)
}

fn clamp_to_grapheme_boundary_down(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    if is_grapheme_boundary(text, idx) {
        return idx;
    }

    let mut prev = 0usize;
    for (start, _) in text.grapheme_indices(true) {
        if start >= idx {
            break;
        }
        prev = start;
    }
    prev
}

fn clamp_to_grapheme_boundary_up(text: &str, idx: usize) -> usize {
    let idx = idx.min(text.len());
    if is_grapheme_boundary(text, idx) {
        return idx;
    }
    for (start, g) in text.grapheme_indices(true) {
        let end = start + g.len();
        if idx < end {
            return end;
        }
    }
    text.len()
}

fn empty_range_at(idx: usize) -> Range<usize> {
    idx..idx
}

fn hit_test_x(clusters: &[ShapedCluster], x: f32, text_len: usize) -> (usize, CaretAffinity) {
    if clusters.is_empty() {
        return (0, CaretAffinity::Downstream);
    }

    if x.is_nan() || x <= clusters[0].x0 {
        return (0, CaretAffinity::Downstream);
    }

    for c in clusters {
        if x > c.x1 {
            continue;
        }
        if c.text_range.start == c.text_range.end {
            return (c.text_range.start, CaretAffinity::Downstream);
        }
        let mid = c.x0 + (c.x1 - c.x0) * 0.5;
        let left_half = x <= mid;
        if c.is_rtl {
            if left_half {
                return (c.text_range.end, CaretAffinity::Upstream);
            }
            return (c.text_range.start, CaretAffinity::Downstream);
        }
        if left_half {
            return (c.text_range.start, CaretAffinity::Downstream);
        }
        return (c.text_range.end, CaretAffinity::Upstream);
    }

    (text_len, CaretAffinity::Downstream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{FontId, Px, TextPaintStyle, TextShapingStyle, TextStyle};

    #[test]
    fn none_ellipsis_adds_zero_len_cluster_at_cut_end() {
        let mut shaper = ParleyShaper::new();
        let base = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "This is a long line that should truncate";
        let spans = [TextSpan {
            len: text.len(),
            shaping: TextShapingStyle::default(),
            paint: TextPaintStyle::default(),
        }];

        let constraints = TextConstraints {
            max_width: Some(Px(80.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            scale_factor: 1.0,
        };

        let wrapped = wrap_with_constraints(
            &mut shaper,
            TextInputRef::Attributed {
                text,
                base: &base,
                spans: &spans,
            },
            constraints,
        );

        assert!(wrapped.kept_end < text.len());
        assert!(
            wrapped.lines[0]
                .clusters
                .iter()
                .any(|c| c.text_range == (wrapped.kept_end..wrapped.kept_end)),
            "expected a synthetic zero-length cluster for ellipsis mapping"
        );

        let (hit, _affinity) = wrapped.hit_test_x(0, 79.0);
        assert_eq!(hit, wrapped.kept_end);
    }

    #[test]
    fn no_ellipsis_keeps_full_text() {
        let mut shaper = ParleyShaper::new();
        let base = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "short";
        let spans = [TextSpan {
            len: text.len(),
            shaping: TextShapingStyle::default(),
            paint: TextPaintStyle::default(),
        }];

        let constraints = TextConstraints {
            max_width: Some(Px(800.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            scale_factor: 1.0,
        };

        let wrapped = wrap_with_constraints(
            &mut shaper,
            TextInputRef::Attributed {
                text,
                base: &base,
                spans: &spans,
            },
            constraints,
        );

        assert_eq!(wrapped.kept_end, text.len());
    }

    #[test]
    fn word_wrap_produces_multiple_lines_and_full_coverage() {
        let mut shaper = ParleyShaper::new();
        let base = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "hello world hello world hello world";
        let spans = [TextSpan {
            len: text.len(),
            shaping: TextShapingStyle::default(),
            paint: TextPaintStyle::default(),
        }];

        let constraints = TextConstraints {
            max_width: Some(Px(60.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let wrapped = wrap_with_constraints(
            &mut shaper,
            TextInputRef::Attributed {
                text,
                base: &base,
                spans: &spans,
            },
            constraints,
        );

        assert!(wrapped.lines.len() > 1);
        assert_eq!(wrapped.line_ranges.first().unwrap().start, 0);
        assert_eq!(wrapped.line_ranges.last().unwrap().end, text.len());
        for w in wrapped.line_ranges.windows(2) {
            assert_eq!(w[0].end, w[1].start);
        }
    }

    #[test]
    fn newlines_split_into_paragraphs_and_create_gaps_in_ranges() {
        let mut shaper = ParleyShaper::new();
        let base = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "hello\nworld";
        let spans = [TextSpan {
            len: text.len(),
            shaping: TextShapingStyle::default(),
            paint: TextPaintStyle::default(),
        }];

        let constraints = TextConstraints {
            max_width: Some(Px(40.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let wrapped = wrap_with_constraints(
            &mut shaper,
            TextInputRef::Attributed {
                text,
                base: &base,
                spans: &spans,
            },
            constraints,
        );

        assert!(wrapped.lines.len() >= 2);
        assert_eq!(wrapped.line_ranges.first().unwrap().start, 0);
        assert_eq!(
            wrapped.line_ranges.last().unwrap().end,
            text.len(),
            "last line should end at the full text length"
        );

        assert!(
            wrapped
                .line_ranges
                .windows(2)
                .any(|w| w[0].end + 1 == w[1].start),
            "expected at least one paragraph boundary gap caused by a newline"
        );
    }

    #[test]
    fn empty_lines_produce_lines_for_consecutive_newlines() {
        let mut shaper = ParleyShaper::new();
        let base = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "\n";
        let constraints = TextConstraints {
            max_width: Some(Px(40.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let wrapped =
            wrap_with_constraints(&mut shaper, TextInputRef::plain(text, &base), constraints);
        assert_eq!(wrapped.lines.len(), 2, "expected two empty paragraphs");
        assert_eq!(wrapped.line_ranges.len(), 2);
        assert_eq!(wrapped.line_ranges[0], 0..0);
        assert_eq!(wrapped.line_ranges[1], 1..1);
    }

    #[test]
    fn grapheme_wrap_breaks_long_token_without_spaces() {
        let mut shaper = ParleyShaper::new();
        let base = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let constraints = TextConstraints {
            max_width: Some(Px(40.0)),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let wrapped =
            wrap_with_constraints(&mut shaper, TextInputRef::plain(text, &base), constraints);
        assert!(wrapped.lines.len() > 1);
        assert_eq!(wrapped.line_ranges.first().unwrap().start, 0);
        assert_eq!(wrapped.line_ranges.last().unwrap().end, text.len());
        for w in wrapped.line_ranges.windows(2) {
            assert_eq!(w[0].end, w[1].start);
        }
    }

    #[test]
    fn grapheme_wrap_handles_cjk_string() {
        let mut shaper = ParleyShaper::new();
        let base = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "你好世界你好世界你好世界你好世界你好世界";
        let constraints = TextConstraints {
            max_width: Some(Px(40.0)),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let wrapped =
            wrap_with_constraints(&mut shaper, TextInputRef::plain(text, &base), constraints);
        assert!(wrapped.lines.len() > 1);
        assert_eq!(wrapped.line_ranges.first().unwrap().start, 0);
        assert_eq!(wrapped.line_ranges.last().unwrap().end, text.len());
        for w in wrapped.line_ranges.windows(2) {
            assert_eq!(w[0].end, w[1].start);
        }
    }

    #[test]
    fn grapheme_wrap_does_not_split_zwj_clusters() {
        let mut shaper = ParleyShaper::new();
        let base = TextStyle {
            font: FontId::default(),
            size: Px(16.0),
            ..Default::default()
        };

        let emoji = "👨‍👩‍👧‍👦";
        let text = format!("{emoji}{emoji}{emoji}{emoji}{emoji}");
        let constraints = TextConstraints {
            max_width: Some(Px(60.0)),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            scale_factor: 1.0,
        };

        let wrapped =
            wrap_with_constraints(&mut shaper, TextInputRef::plain(&text, &base), constraints);
        assert!(wrapped.lines.len() > 1);
        for r in &wrapped.line_ranges {
            assert!(
                is_grapheme_boundary(&text, r.start),
                "expected line start to be a grapheme boundary: {:?}",
                r
            );
            assert!(
                is_grapheme_boundary(&text, r.end),
                "expected line end to be a grapheme boundary: {:?}",
                r
            );
        }
    }
}
