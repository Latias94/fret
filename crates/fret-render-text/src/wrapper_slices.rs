use crate::parley_shaper::{ParleyShaper, ShapedLineLayout};
use fret_core::{TextInputRef, TextSpan, TextStyle};
use std::ops::Range;

pub(crate) fn shape_prefix(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &TextStyle,
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

pub(crate) fn truncate_spans(spans: &[TextSpan], end: usize) -> Vec<TextSpan> {
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

pub(crate) fn shape_slice(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &TextStyle,
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

pub(crate) fn shape_slice_measure_only(
    shaper: &mut ParleyShaper,
    text: &str,
    base: &TextStyle,
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

pub(crate) fn slice_spans(spans: &[TextSpan], start: usize, end: usize) -> Vec<TextSpan> {
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
