use crate::parley_shaper::{ParleyGlyph, ParleyShaper, ShapedCluster, ShapedLineLayout};
use crate::wrapper::WrappedLayout;
use crate::wrapper_boundaries::{
    clamp_to_char_boundary, clamp_to_grapheme_boundary_down, cut_end_for_available, empty_range_at,
    trim_trailing_whitespace,
};
use crate::wrapper_ranges::{
    wrap_grapheme_range, wrap_grapheme_range_measure_only, wrap_word_break_range,
    wrap_word_break_range_measure_only, wrap_word_range, wrap_word_range_measure_only,
};
use crate::wrapper_slices::{shape_prefix, shape_slice, shape_slice_measure_only, slice_spans};
use fret_core::{TextConstraints, TextInputRef, TextOverflow, TextSpan, TextWrap};
use std::ops::Range;

const ELLIPSIS: &str = "\u{2026}";

pub(crate) fn wrap_with_newlines(
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

    WrappedLayout::new(text_len, text_len, line_ranges, lines)
}

pub(crate) fn wrap_with_newlines_measure_only(
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

    WrappedLayout::new(text_len, text_len, line_ranges, lines)
}

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
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
            shaped.set_width(max_w);
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

pub(crate) fn wrap_none_ellipsis(
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
    if full.width() <= max_width_px + 0.5 {
        return WrappedSingleLineInternal {
            kept_end: text_len,
            line: full,
        };
    }

    let mut ellipsis = shaper.shape_single_line(TextInputRef::plain(ELLIPSIS, base), scale);
    let ellipsis_w = ellipsis.width().max(0.0);
    let available = (max_width_px - ellipsis_w).max(0.0);

    let mut cut_end = cut_end_for_available(text, full.clusters(), available);
    if cut_end < text_len && cut_end > 0 {
        cut_end = trim_trailing_whitespace(text, cut_end);
        cut_end = clamp_to_char_boundary(text, cut_end);
        cut_end = clamp_to_grapheme_boundary_down(text, cut_end);
    }

    // Shape the kept prefix so truncation doesn't depend on the discarded suffix (important for
    // contextual shaping scripts).
    let mut kept = shape_prefix(shaper, text, base, spans, cut_end, scale);

    if kept.width() > available + 0.5 {
        let cut2 = cut_end_for_available(&text[..cut_end], kept.clusters(), available);
        if cut2 < cut_end {
            cut_end = clamp_to_char_boundary(text, trim_trailing_whitespace(text, cut2));
            cut_end = clamp_to_grapheme_boundary_down(text, cut_end);
            kept = shape_prefix(shaper, text, base, spans, cut_end, scale);
        }
    }

    let ellipsis_start_x = (max_width_px - ellipsis_w).max(0.0);

    let mut clusters: Vec<ShapedCluster> = kept.take_clusters();
    let mut glyphs: Vec<ParleyGlyph> = kept.take_glyphs();
    glyphs.extend(ellipsis.take_glyphs().into_iter().map(|mut g| {
        g.x += ellipsis_start_x;
        g.text_range = empty_range_at(cut_end);
        g.is_rtl = false;
        g
    }));

    clusters.push(ShapedCluster::new(
        empty_range_at(cut_end),
        ellipsis_start_x,
        ellipsis_start_x + ellipsis_w,
        false,
    ));

    WrappedSingleLineInternal {
        kept_end: cut_end,
        line: ShapedLineLayout::new(
            max_width_px,
            kept.ascent.max(ellipsis.ascent),
            kept.descent.max(ellipsis.descent),
            kept.ink_ascent.max(ellipsis.ink_ascent),
            kept.ink_descent.max(ellipsis.ink_descent),
            kept.baseline().max(ellipsis.baseline()),
            kept.line_height().max(ellipsis.line_height()),
            glyphs,
            clusters,
        ),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WrappedSingleLineInternal {
    pub(crate) kept_end: usize,
    pub(crate) line: ShapedLineLayout,
}
