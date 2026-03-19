use crate::parley_shaper::{ParleyShaper, ShapedLineLayout};
use crate::wrapper_boundaries::{
    clamp_to_grapheme_boundary_down, clamp_to_grapheme_boundary_up, cut_end_for_available,
    first_cluster_end, first_grapheme_end, wrap_grapheme_cut_end,
};
use crate::wrapper_slices::{shape_slice, shape_slice_measure_only, slice_spans};
use fret_core::{TextInputRef, TextSpan, TextStyle};
use std::ops::Range;

pub(crate) fn wrap_grapheme_range(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &TextStyle,
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

        if full.width() <= max_width_px + 0.5 {
            lines.push(full);
            line_ranges.push(offset..end);
            break;
        }

        let mut cut_end = wrap_grapheme_cut_end(slice, full.clusters(), max_width_px);
        cut_end = clamp_to_grapheme_boundary_down(slice, cut_end);

        if cut_end == 0 {
            cut_end = first_cluster_end(slice, full.clusters());
            cut_end = clamp_to_grapheme_boundary_up(slice, cut_end);
        }
        if cut_end == 0 {
            cut_end = first_grapheme_end(slice);
        }

        let mut kept = shape_slice(shaper, text, base, spans, offset..(offset + cut_end), scale);
        if kept.width() > max_width_px + 0.5 && cut_end > 0 {
            let cut2 = cut_end_for_available(&slice[..cut_end], kept.clusters(), max_width_px);
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

pub(crate) fn wrap_grapheme_range_measure_only(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &TextStyle,
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

        if full.width() <= max_width_px + 0.5 {
            lines.push(full);
            line_ranges.push(offset..end);
            break;
        }

        let mut cut_end = wrap_grapheme_cut_end(slice, full.clusters(), max_width_px);
        cut_end = clamp_to_grapheme_boundary_down(slice, cut_end);

        if cut_end == 0 {
            cut_end = first_cluster_end(slice, full.clusters());
            cut_end = clamp_to_grapheme_boundary_up(slice, cut_end);
        }
        if cut_end == 0 {
            cut_end = first_grapheme_end(slice);
        }

        let mut kept =
            shape_slice_measure_only(shaper, text, base, spans, offset..(offset + cut_end), scale);
        if kept.width() > max_width_px + 0.5 && cut_end > 0 {
            let cut2 = cut_end_for_available(&slice[..cut_end], kept.clusters(), max_width_px);
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

pub(crate) fn wrap_word_range(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &TextStyle,
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

pub(crate) fn wrap_word_break_range(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &TextStyle,
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

pub(crate) fn wrap_word_range_measure_only(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &TextStyle,
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

pub(crate) fn wrap_word_break_range_measure_only(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &TextStyle,
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
