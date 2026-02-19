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
    #[allow(dead_code)]
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
                line_ranges: vec![Range {
                    start: 0,
                    end: out.kept_end,
                }],
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
            wrap: TextWrap::WordBreak,
            ..
        } => wrap_word_break(shaper, input, text_len, max_width.0 * scale, scale),
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Grapheme,
            ..
        } => wrap_grapheme(shaper, input, text_len, max_width.0 * scale, scale),
        _ => WrappedLayout {
            text_len,
            kept_end: text_len,
            line_ranges: vec![Range {
                start: 0,
                end: text_len,
            }],
            lines: vec![shaper.shape_single_line(input, scale)],
        },
    }
}

/// Wraps text for measurement only.
///
/// The returned `lines[*].glyphs` is intentionally empty to avoid per-glyph work in layout.
pub(crate) fn wrap_with_constraints_measure_only(
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
        return wrap_with_newlines_measure_only(shaper, input, constraints, scale);
    }

    match constraints {
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            ..
        } => {
            let mut line = shaper.shape_single_line_metrics(input, scale);
            line.width = max_width.0 * scale;
            WrappedLayout {
                text_len,
                kept_end: text_len,
                line_ranges: vec![Range {
                    start: 0,
                    end: text_len,
                }],
                lines: vec![line],
            }
        }
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            ..
        } => wrap_word_measure_only(shaper, input, text_len, max_width.0 * scale, scale),
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::WordBreak,
            ..
        } => wrap_word_break_measure_only(shaper, input, text_len, max_width.0 * scale, scale),
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Grapheme,
            ..
        } => wrap_grapheme_measure_only(shaper, input, text_len, max_width.0 * scale, scale),
        _ => WrappedLayout {
            text_len,
            kept_end: text_len,
            line_ranges: vec![Range {
                start: 0,
                end: text_len,
            }],
            lines: vec![shaper.shape_single_line_metrics(input, scale)],
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

fn wrap_with_newlines_measure_only(
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
        push_paragraph_measure_only(
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
    push_paragraph_measure_only(
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
            wrap: TextWrap::WordBreak,
            ..
        } => {
            let Some(max_w) = max_width_px else {
                return;
            };
            let (ranges, lines) =
                wrap_word_break_range(shaper, text, base, spans, paragraph_range, max_w, scale);
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

fn push_paragraph_measure_only(
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
            let mut shaped = match spans.as_ref() {
                Some(s) => shaper.shape_single_line_metrics(
                    TextInputRef::Attributed {
                        text: slice,
                        base,
                        spans: s.as_slice(),
                    },
                    scale,
                ),
                None => shaper.shape_single_line_metrics(TextInputRef::plain(slice, base), scale),
            };
            shaped.width = max_w;
            out_ranges.push(paragraph_range);
            out_lines.push(shaped);
        }
        TextConstraints {
            max_width: Some(_),
            wrap: TextWrap::Word,
            ..
        } => {
            let Some(max_w) = max_width_px else {
                return;
            };
            let (ranges, lines) = wrap_word_range_measure_only(
                shaper,
                text,
                base,
                spans,
                paragraph_range,
                max_w,
                scale,
            );
            out_ranges.extend(ranges);
            out_lines.extend(lines);
        }
        TextConstraints {
            max_width: Some(_),
            wrap: TextWrap::WordBreak,
            ..
        } => {
            let Some(max_w) = max_width_px else {
                return;
            };
            let (ranges, lines) = wrap_word_break_range_measure_only(
                shaper,
                text,
                base,
                spans,
                paragraph_range,
                max_w,
                scale,
            );
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
            let (ranges, lines) = wrap_grapheme_range_measure_only(
                shaper,
                text,
                base,
                spans,
                paragraph_range,
                max_w,
                scale,
            );
            out_ranges.extend(ranges);
            out_lines.extend(lines);
        }
        _ => {
            let line =
                shape_slice_measure_only(shaper, text, base, spans, paragraph_range.clone(), scale);
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
        g.is_rtl = false;
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

fn wrap_word_break(
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
        wrap_word_break_range(shaper, text, base, spans, 0..text_len, max_width_px, scale);

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

fn wrap_word_measure_only(
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
        wrap_word_range_measure_only(shaper, text, base, spans, 0..text_len, max_width_px, scale);

    WrappedLayout {
        text_len,
        kept_end: text_len,
        line_ranges,
        lines,
    }
}

fn wrap_word_break_measure_only(
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

    let (line_ranges, lines) = wrap_word_break_range_measure_only(
        shaper,
        text,
        base,
        spans,
        0..text_len,
        max_width_px,
        scale,
    );

    WrappedLayout {
        text_len,
        kept_end: text_len,
        line_ranges,
        lines,
    }
}

fn wrap_grapheme_measure_only(
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

    let (line_ranges, lines) = wrap_grapheme_range_measure_only(
        shaper,
        text,
        base,
        spans,
        0..text_len,
        max_width_px,
        scale,
    );

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
            vec![Range { start, end: start }],
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

fn wrap_grapheme_range_measure_only(
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
            vec![Range { start, end: start }],
            vec![shape_slice_measure_only(
                shaper,
                text,
                base,
                spans,
                start..start,
                scale,
            )],
        );
    }

    let mut lines: Vec<ShapedLineLayout> = Vec::new();
    let mut line_ranges: Vec<Range<usize>> = Vec::new();

    let mut offset = start;
    while offset < end {
        let slice = &text[offset..end];
        let full = shape_slice_measure_only(shaper, text, base, spans, offset..end, scale);

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

        let mut kept =
            shape_slice_measure_only(shaper, text, base, spans, offset..(offset + cut_end), scale);
        if kept.width > max_width_px + 0.5 && cut_end > 0 {
            let cut2 = cut_end_for_available(&slice[..cut_end], &kept.clusters, max_width_px);
            if cut2 > 0 && cut2 < cut_end {
                cut_end = clamp_to_grapheme_boundary_down(slice, cut2);
                kept = shape_slice_measure_only(
                    shaper,
                    text,
                    base,
                    spans,
                    offset..(offset + cut_end),
                    scale,
                );
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
            vec![Range { start, end: start }],
            vec![shape_slice(shaper, text, base, spans, start..start, scale)],
        );
    }

    let slice = &text[start..end];
    let spans = spans.map(|spans| slice_spans(spans, start, end));
    // Allow a tiny amount of physical-pixel slack when wrapping words.
    //
    // Under fractional DPI scaling (e.g. 150%), the width returned from measuring an unwrapped
    // single line can land exactly on a device pixel boundary. Reusing that width as the wrap
    // constraint should not force an extra line break due to subpixel rounding differences inside
    // the shaper. Add a small epsilon (in physical px) to keep the behavior stable and aligned
    // with our grapheme-wrap cut logic (`+ 0.5`).
    let max_width_px = max_width_px.max(0.0) + 0.5;

    let shaped = match spans.as_ref() {
        Some(spans) => shaper.shape_paragraph_word_wrap(
            TextInputRef::Attributed {
                text: slice,
                base,
                spans: spans.as_slice(),
            },
            max_width_px,
            scale,
        ),
        None => {
            shaper.shape_paragraph_word_wrap(TextInputRef::plain(slice, base), max_width_px, scale)
        }
    };

    let mut line_ranges: Vec<Range<usize>> = Vec::with_capacity(shaped.len().max(1));
    let mut lines: Vec<ShapedLineLayout> = Vec::with_capacity(shaped.len().max(1));
    for (r, line) in shaped {
        line_ranges.push((start + r.start)..(start + r.end));
        lines.push(line);
    }
    (line_ranges, lines)
}

fn wrap_word_break_range(
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
            vec![Range { start, end: start }],
            vec![shape_slice(shaper, text, base, spans, start..start, scale)],
        );
    }

    let slice = &text[start..end];
    let spans = spans.map(|spans| slice_spans(spans, start, end));
    let max_width_px = max_width_px.max(0.0) + 0.5;

    let shaped = match spans.as_ref() {
        Some(spans) => shaper.shape_paragraph_word_break_wrap(
            TextInputRef::Attributed {
                text: slice,
                base,
                spans: spans.as_slice(),
            },
            max_width_px,
            scale,
        ),
        None => shaper.shape_paragraph_word_break_wrap(
            TextInputRef::plain(slice, base),
            max_width_px,
            scale,
        ),
    };

    let mut line_ranges: Vec<Range<usize>> = Vec::with_capacity(shaped.len().max(1));
    let mut lines: Vec<ShapedLineLayout> = Vec::with_capacity(shaped.len().max(1));
    for (r, line) in shaped {
        line_ranges.push((start + r.start)..(start + r.end));
        lines.push(line);
    }
    (line_ranges, lines)
}

fn wrap_word_range_measure_only(
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
            vec![Range { start, end: start }],
            vec![shape_slice_measure_only(
                shaper,
                text,
                base,
                spans,
                start..start,
                scale,
            )],
        );
    }

    let slice = &text[start..end];
    let spans = spans.map(|spans| slice_spans(spans, start, end));
    let max_width_px = max_width_px.max(0.0) + 0.5;

    let shaped = match spans.as_ref() {
        Some(spans) => shaper.shape_paragraph_word_wrap_metrics(
            TextInputRef::Attributed {
                text: slice,
                base,
                spans: spans.as_slice(),
            },
            max_width_px,
            scale,
        ),
        None => shaper.shape_paragraph_word_wrap_metrics(
            TextInputRef::plain(slice, base),
            max_width_px,
            scale,
        ),
    };

    let mut line_ranges: Vec<Range<usize>> = Vec::with_capacity(shaped.len().max(1));
    let mut lines: Vec<ShapedLineLayout> = Vec::with_capacity(shaped.len().max(1));
    for (r, line) in shaped {
        line_ranges.push((start + r.start)..(start + r.end));
        lines.push(line);
    }
    (line_ranges, lines)
}

fn wrap_word_break_range_measure_only(
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
            vec![Range { start, end: start }],
            vec![shape_slice_measure_only(
                shaper,
                text,
                base,
                spans,
                start..start,
                scale,
            )],
        );
    }

    let slice = &text[start..end];
    let spans = spans.map(|spans| slice_spans(spans, start, end));
    let max_width_px = max_width_px.max(0.0) + 0.5;

    let shaped = match spans.as_ref() {
        Some(spans) => shaper.shape_paragraph_word_break_wrap_metrics(
            TextInputRef::Attributed {
                text: slice,
                base,
                spans: spans.as_slice(),
            },
            max_width_px,
            scale,
        ),
        None => shaper.shape_paragraph_word_break_wrap_metrics(
            TextInputRef::plain(slice, base),
            max_width_px,
            scale,
        ),
    };

    let mut line_ranges: Vec<Range<usize>> = Vec::with_capacity(shaped.len().max(1));
    let mut lines: Vec<ShapedLineLayout> = Vec::with_capacity(shaped.len().max(1));
    for (r, line) in shaped {
        line_ranges.push((start + r.start)..(start + r.end));
        lines.push(line);
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

fn shape_slice_measure_only(
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
            shaper.shape_single_line_metrics(
                TextInputRef::Attributed {
                    text: slice,
                    base,
                    spans: &out,
                },
                scale,
            )
        }
        None => shaper.shape_single_line_metrics(TextInputRef::plain(slice, base), scale),
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

#[allow(dead_code)]
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
    use serde::Deserialize;

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

    fn is_forbidden_line_start_char(c: char) -> bool {
        // Minimal “kinsoku”-style set: avoid starting a line with closing punctuation in CJK text.
        // Keep this conservative; conformance fixtures can expand it as needed.
        matches!(
            c,
            '，' | '。'
                | '、'
                | '：'
                | '；'
                | '！'
                | '？'
                | '）'
                | '】'
                | '》'
                | '〉'
                | '」'
                | '』'
                | '〕'
                | '］'
                | '｝'
                | '’'
                | '”'
        )
    }

    fn is_forbidden_line_end_char(c: char) -> bool {
        // Avoid ending a line with opening punctuation in CJK text.
        matches!(
            c,
            '（' | '【' | '《' | '〈' | '「' | '『' | '〔' | '［' | '｛'
        )
    }

    #[test]
    fn none_ellipsis_adds_zero_len_cluster_at_cut_end() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
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
            align: fret_core::TextAlign::Start,
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
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
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
            align: fret_core::TextAlign::Start,
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
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "hello world hello world hello world";
        // Multiple spans to ensure wrapping remains correct across span boundaries.
        let spans = [
            TextSpan {
                len: 6, // "hello "
                shaping: TextShapingStyle::default(),
                paint: TextPaintStyle::default(),
            },
            TextSpan {
                len: 5, // "world"
                shaping: TextShapingStyle::default(),
                paint: TextPaintStyle::default(),
            },
            TextSpan {
                len: text.len().saturating_sub(11),
                shaping: TextShapingStyle::default(),
                paint: TextPaintStyle::default(),
            },
        ];

        let constraints = TextConstraints {
            max_width: Some(Px(60.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
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
    fn parley_word_wrap_handles_long_plain_paragraph_under_resize_jitter() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let mut text = String::new();
        for i in 0..500 {
            if i > 0 {
                text.push(' ');
            }
            text.push_str("word");
            text.push_str(&(i % 97).to_string());
        }

        let widths = [60.0, 80.0, 120.0, 90.0, 70.0, 140.0, 60.0];
        for w in widths {
            let constraints = TextConstraints {
                max_width: Some(Px(w)),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor: 1.0,
            };
            let wrapped =
                wrap_with_constraints(&mut shaper, TextInputRef::plain(&text, &base), constraints);

            assert_eq!(wrapped.text_len, text.len());
            assert!(!wrapped.line_ranges.is_empty());
            assert_eq!(wrapped.line_ranges[0].start, 0);
            assert_eq!(wrapped.line_ranges.last().unwrap().end, text.len());

            for r in &wrapped.line_ranges {
                assert!(text.is_char_boundary(r.start));
                assert!(text.is_char_boundary(r.end));
                assert!(r.start < r.end, "expected non-empty line range");
            }
            for win in wrapped.line_ranges.windows(2) {
                assert_eq!(
                    win[0].end, win[1].start,
                    "expected contiguous coverage for a single-paragraph plain text wrap"
                );
            }
        }
    }

    #[test]
    fn word_wrap_does_not_break_single_token() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(20.0),
            ..Default::default()
        };

        let text = "Demo";
        let constraints = TextConstraints {
            max_width: Some(Px(1.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let wrapped = wrap_with_constraints_measure_only(
            &mut shaper,
            TextInputRef::plain(text, &base),
            constraints,
        );

        assert_eq!(wrapped.lines.len(), 1);
        assert!(
            wrapped.lines[0].width > 1.0,
            "expected word-wrap to keep a single token unbroken and allow overflow"
        );
    }

    #[test]
    fn word_wrap_min_content_width_matches_longest_unbreakable_segment() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(20.0),
            ..Default::default()
        };

        // Under a near-zero wrap width, word-wrap should break at whitespace opportunities, but
        // must not break within tokens. The resulting wrapped lines should therefore represent
        // the "unbreakable segments" whose maximum width matches min-content semantics.
        let text = "foo barbaz qux";
        let wrapped = wrap_with_constraints_measure_only(
            &mut shaper,
            TextInputRef::plain(text, &base),
            TextConstraints {
                max_width: Some(Px(0.0)),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor: 1.0,
            },
        );

        assert!(
            wrapped.lines.len() >= 2,
            "expected near-zero word-wrap to produce multiple visual lines for spaced text"
        );
        assert_eq!(
            wrapped.lines.len(),
            wrapped.line_ranges.len(),
            "expected line_ranges to match wrapped line count"
        );

        // Validate each produced line width against an independently shaped single-line slice
        // matching the wrapped range. This avoids making assumptions about whether trailing
        // whitespace is kept at soft wrap boundaries.
        for (range, line) in wrapped.line_ranges.iter().zip(wrapped.lines.iter()) {
            let slice = &text[range.clone()];
            let expected = shaper.shape_single_line_metrics(TextInputRef::plain(slice, &base), 1.0);
            let delta = (expected.width - line.width).abs();
            assert!(
                delta <= 0.75,
                "expected wrapped line width to match shaped slice; slice={:?} expected={} actual={} delta={}",
                slice,
                expected.width,
                line.width,
                delta
            );
        }

        let max_line_w = wrapped.lines.iter().map(|l| l.width).fold(0.0f32, f32::max);
        assert!(
            max_line_w > 0.0,
            "expected non-zero min-content width for non-empty text"
        );
    }

    #[test]
    fn word_break_wrap_can_break_single_token() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(20.0),
            ..Default::default()
        };

        let text = "Demo";
        let constraints = TextConstraints {
            max_width: Some(Px(1.0)),
            wrap: TextWrap::WordBreak,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let wrapped = wrap_with_constraints_measure_only(
            &mut shaper,
            TextInputRef::plain(text, &base),
            constraints,
        );

        assert!(
            wrapped.lines.len() > 1,
            "expected word-break wrap to split a single long token under tight constraints"
        );
    }

    #[test]
    fn parley_word_wrap_handles_long_attributed_paragraph_under_resize_jitter() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let mut text = String::new();
        for i in 0..500 {
            if i > 0 {
                text.push(' ');
            }
            text.push_str("word");
            text.push_str(&(i % 97).to_string());
        }

        let text_len = text.len();
        let mut spans: Vec<TextSpan> = Vec::new();
        let mut remaining = text_len;
        let mut toggle = false;
        while remaining > 0 {
            let take = remaining.min(if toggle { 17 } else { 31 });
            spans.push(TextSpan {
                len: take,
                shaping: TextShapingStyle::default(),
                paint: if toggle {
                    TextPaintStyle {
                        fg: Some(fret_core::Color {
                            r: 0.9,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        ..Default::default()
                    }
                } else {
                    TextPaintStyle::default()
                },
            });
            remaining = remaining.saturating_sub(take);
            toggle = !toggle;
        }
        assert_eq!(
            spans.iter().map(|s| s.len).sum::<usize>(),
            text_len,
            "spans must fully cover the text"
        );

        let widths = [60.0, 80.0, 120.0, 90.0, 70.0, 140.0, 60.0];
        for w in widths {
            let constraints = TextConstraints {
                max_width: Some(Px(w)),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor: 1.0,
            };
            let wrapped = wrap_with_constraints(
                &mut shaper,
                TextInputRef::Attributed {
                    text: text.as_str(),
                    base: &base,
                    spans: spans.as_slice(),
                },
                constraints,
            );

            assert_eq!(wrapped.text_len, text_len);
            assert_eq!(wrapped.kept_end, text_len);
            assert!(!wrapped.line_ranges.is_empty());
            assert_eq!(wrapped.line_ranges[0].start, 0);
            assert_eq!(wrapped.line_ranges.last().unwrap().end, text_len);
            assert_eq!(wrapped.lines.len(), wrapped.line_ranges.len());

            for r in &wrapped.line_ranges {
                assert!(text.is_char_boundary(r.start));
                assert!(text.is_char_boundary(r.end));
                assert!(r.start < r.end, "expected non-empty line range");
            }
            for win in wrapped.line_ranges.windows(2) {
                assert_eq!(
                    win[0].end, win[1].start,
                    "expected contiguous coverage for a single-paragraph attributed text wrap"
                );
            }
        }
    }

    #[test]
    fn newlines_split_into_paragraphs_and_create_gaps_in_ranges() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
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
            align: fret_core::TextAlign::Start,
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
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "\n";
        let constraints = TextConstraints {
            max_width: Some(Px(40.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
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
    fn wrap_measure_only_matches_line_ranges_and_sizes_for_word_wrap() {
        let mut shaper_full = shaper_with_bundled_fonts();
        let mut shaper_measure = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "hello world hello world hello world hello world hello world hello world";
        let spans = [TextSpan {
            len: text.len(),
            shaping: TextShapingStyle::default(),
            paint: TextPaintStyle::default(),
        }];

        let constraints = TextConstraints {
            max_width: Some(Px(60.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let full = wrap_with_constraints(
            &mut shaper_full,
            TextInputRef::Attributed {
                text,
                base: &base,
                spans: &spans,
            },
            constraints,
        );
        let measure = wrap_with_constraints_measure_only(
            &mut shaper_measure,
            TextInputRef::Attributed {
                text,
                base: &base,
                spans: &spans,
            },
            constraints,
        );

        assert_eq!(full.line_ranges, measure.line_ranges);
        assert_eq!(full.lines.len(), measure.lines.len());
        for (a, b) in full.lines.iter().zip(measure.lines.iter()) {
            assert!((a.width - b.width).abs() < 0.01);
            assert!((a.line_height - b.line_height).abs() < 0.01);
        }
        assert!(measure.lines.iter().all(|l| l.glyphs.is_empty()));
    }

    #[test]
    fn wrap_measure_only_matches_line_ranges_and_sizes_for_grapheme_wrap() {
        let mut shaper_full = shaper_with_bundled_fonts();
        let mut shaper_measure = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let constraints = TextConstraints {
            max_width: Some(Px(40.0)),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let full = wrap_with_constraints(
            &mut shaper_full,
            TextInputRef::plain(text, &base),
            constraints,
        );
        let measure = wrap_with_constraints_measure_only(
            &mut shaper_measure,
            TextInputRef::plain(text, &base),
            constraints,
        );

        assert_eq!(full.line_ranges, measure.line_ranges);
        assert_eq!(full.lines.len(), measure.lines.len());
        for (a, b) in full.lines.iter().zip(measure.lines.iter()) {
            assert!((a.width - b.width).abs() < 0.01);
            assert!((a.line_height - b.line_height).abs() < 0.01);
        }
        assert!(measure.lines.iter().all(|l| l.glyphs.is_empty()));
    }

    #[test]
    fn grapheme_wrap_breaks_long_token_without_spaces() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Fira Mono"),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let constraints = TextConstraints {
            max_width: Some(Px(40.0)),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
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
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Noto Sans CJK SC"),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "你好世界你好世界你好世界你好世界你好世界";
        let constraints = TextConstraints {
            max_width: Some(Px(40.0)),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
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
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Noto Color Emoji"),
            size: Px(16.0),
            ..Default::default()
        };

        let emoji = "👨‍👩‍👧‍👦";
        let text = format!("{emoji}{emoji}{emoji}{emoji}{emoji}");
        let constraints = TextConstraints {
            max_width: Some(Px(60.0)),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum FixtureWrapMode {
        Word,
        WordBreak,
        Grapheme,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct WrapFixtureCase {
        id: String,
        text: String,
        font_family: String,
        wrap: FixtureWrapMode,
        max_width_px: f32,
        #[serde(default)]
        assert_no_forbidden_punct: bool,
        #[serde(default)]
        expected_line_ranges: Option<Vec<[usize; 2]>>,
    }

    #[derive(Debug, Deserialize)]
    struct WrapFixtureSuite {
        schema_version: u32,
        cases: Vec<WrapFixtureCase>,
    }

    fn wrap_mode_for_fixture(mode: FixtureWrapMode) -> TextWrap {
        match mode {
            FixtureWrapMode::Word => TextWrap::Word,
            FixtureWrapMode::WordBreak => TextWrap::WordBreak,
            FixtureWrapMode::Grapheme => TextWrap::Grapheme,
        }
    }

    fn sanitize_line_ranges_for_fixture(ranges: &[std::ops::Range<usize>]) -> Vec<[usize; 2]> {
        ranges.iter().map(|r| [r.start, r.end]).collect()
    }

    #[test]
    fn text_wrap_conformance_v1_fixtures() {
        let raw = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/text/tests/fixtures/text_wrap_conformance_v1.json"
        ));
        let suite: WrapFixtureSuite =
            serde_json::from_str(raw).expect("wrap conformance fixtures JSON");
        assert_eq!(suite.schema_version, 2);

        let mut shaper = shaper_with_bundled_fonts();

        let mut failures: Vec<String> = Vec::new();
        for case in suite.cases {
            let style = TextStyle {
                font: FontId::family(case.font_family.clone()),
                size: Px(16.0),
                ..Default::default()
            };
            let constraints = TextConstraints {
                max_width: Some(Px(case.max_width_px)),
                wrap: wrap_mode_for_fixture(case.wrap),
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor: 1.0,
            };

            let wrapped = wrap_with_constraints(
                &mut shaper,
                TextInputRef::plain(&case.text, &style),
                constraints,
            );

            let text_len = case.text.len();
            assert_eq!(
                wrapped.text_len, text_len,
                "case {}: expected wrapper text_len to match input length",
                case.id
            );
            assert!(
                !wrapped.line_ranges.is_empty(),
                "case {}: expected at least one line range",
                case.id
            );

            for r in &wrapped.line_ranges {
                assert!(
                    r.start <= r.end && r.end <= text_len,
                    "case {}: invalid line range {r:?} for len={text_len}",
                    case.id
                );
            }

            for w in wrapped.line_ranges.windows(2) {
                let prev = &w[0];
                let next = &w[1];
                assert!(
                    prev.end <= next.start,
                    "case {}: expected non-decreasing line ranges: prev={prev:?} next={next:?}",
                    case.id
                );
                if next.start > prev.end {
                    let gap = &case.text[prev.end..next.start];
                    assert!(
                        gap.chars().all(|ch| ch == '\n'),
                        "case {}: expected paragraph gaps to contain only newlines (gap={gap:?})",
                        case.id
                    );
                }
            }

            if case.assert_no_forbidden_punct {
                for r in &wrapped.line_ranges {
                    if r.start < text_len {
                        let start_ch = case.text[r.start..]
                            .chars()
                            .next()
                            .expect("expected start char");
                        assert!(
                            !is_forbidden_line_start_char(start_ch),
                            "case {}: expected line not to start with forbidden punctuation: start={:?} range={:?}",
                            case.id,
                            start_ch,
                            r
                        );
                    }

                    if r.end > r.start {
                        let line = &case.text[r.start..r.end];
                        let mut it = line.chars();
                        let Some(mut end_ch) = it.next_back() else {
                            continue;
                        };
                        while matches!(end_ch, '\n' | ' ') {
                            let Some(prev) = it.next_back() else {
                                break;
                            };
                            end_ch = prev;
                        }

                        assert!(
                            !is_forbidden_line_end_char(end_ch),
                            "case {}: expected line not to end with forbidden punctuation: end={:?} range={:?}",
                            case.id,
                            end_ch,
                            r
                        );
                    }
                }
            }

            let got = sanitize_line_ranges_for_fixture(&wrapped.line_ranges);
            match case.expected_line_ranges.as_ref() {
                None => failures.push(format!(
                    "case {}: missing expected_line_ranges; computed={got:?}",
                    case.id
                )),
                Some(expected) => {
                    if &got != expected {
                        failures.push(format!(
                            "case {}: line ranges mismatch: expected={expected:?} got={got:?}",
                            case.id
                        ));
                    }
                }
            }
        }

        assert!(
            failures.is_empty(),
            "wrap conformance fixture failures:\n{}",
            failures.join("\n")
        );
    }
}
