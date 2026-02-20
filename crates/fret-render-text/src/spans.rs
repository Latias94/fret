use fret_core::TextSpan;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ResolvedSpan {
    pub start: usize,
    pub end: usize,
    pub slot: u16,
    pub fg: Option<fret_core::Color>,
    pub underline: Option<ResolvedDecoration>,
    pub strikethrough: Option<ResolvedDecoration>,
}

#[derive(Clone, Debug)]
pub struct ResolvedDecoration {
    pub color: Option<fret_core::Color>,
}

pub fn resolve_spans_for_text(text: &str, spans: &[TextSpan]) -> Option<Vec<ResolvedSpan>> {
    if spans.is_empty() {
        return None;
    }

    let mut out: Vec<ResolvedSpan> = Vec::with_capacity(spans.len());
    let mut offset: usize = 0;
    for span in spans {
        let end = offset.saturating_add(span.len);
        if end > text.len() {
            return None;
        }
        if !text.is_char_boundary(offset) || !text.is_char_boundary(end) {
            return None;
        }
        if span.len != 0 {
            let slot = u16::try_from(out.len()).ok()?;
            out.push(ResolvedSpan {
                start: offset,
                end,
                slot,
                fg: span.paint.fg,
                underline: span
                    .paint
                    .underline
                    .as_ref()
                    .map(|u| ResolvedDecoration { color: u.color }),
                strikethrough: span
                    .paint
                    .strikethrough
                    .as_ref()
                    .map(|s| ResolvedDecoration { color: s.color }),
            });
        }
        offset = end;
    }
    if offset != text.len() {
        return None;
    }

    Some(out)
}

fn span_has_any_overrides(span: &TextSpan) -> bool {
    span.shaping.font.is_some()
        || span.shaping.weight.is_some()
        || span.shaping.slant.is_some()
        || span.shaping.letter_spacing_em.is_some()
        || !span.shaping.features.is_empty()
        || !span.shaping.axes.is_empty()
        || span.paint.fg.is_some()
        || span.paint.bg.is_some()
        || span.paint.underline.is_some()
        || span.paint.strikethrough.is_some()
}

fn clamp_down_to_char_boundary(text: &str, idx: usize) -> usize {
    let mut i = idx.min(text.len());
    while i > 0 && !text.is_char_boundary(i) {
        i = i.saturating_sub(1);
    }
    i
}

fn next_char_boundary(text: &str, idx: usize) -> usize {
    if idx >= text.len() {
        return text.len();
    }
    let idx = clamp_down_to_char_boundary(text, idx);
    if idx >= text.len() {
        return text.len();
    }
    let ch = text[idx..].chars().next().unwrap();
    idx + ch.len_utf8()
}

fn clamp_span_end_to_char_boundary(text: &str, start: usize, desired_end: usize) -> usize {
    let raw_end = desired_end.min(text.len());
    if text.is_char_boundary(raw_end) {
        return raw_end;
    }

    let down = clamp_down_to_char_boundary(text, raw_end);
    if down > start {
        return down;
    }

    let up = next_char_boundary(text, raw_end);
    up.max(start).min(text.len())
}

pub fn sanitize_spans_for_text(text: &str, spans: &[TextSpan]) -> Option<Arc<[TextSpan]>> {
    if spans.is_empty() || text.is_empty() {
        return None;
    }

    let text_len = text.len();
    let mut out: Vec<TextSpan> = Vec::with_capacity(spans.len().saturating_add(1));

    let mut offset: usize = 0;
    for span in spans {
        if offset >= text_len {
            break;
        }

        let desired_end = offset.saturating_add(span.len);
        let mut end = clamp_span_end_to_char_boundary(text, offset, desired_end);

        if end == offset && desired_end > offset {
            end = next_char_boundary(text, offset);
        }

        let mut s = span.clone();
        s.len = end.saturating_sub(offset);
        out.push(s);
        offset = end;
    }

    if offset < text_len {
        out.push(TextSpan::new(text_len - offset));
    }

    // Avoid forcing "attributed" shaping when spans carry no effective overrides.
    if out.len() == 1 && out[0].len == text_len && !span_has_any_overrides(&out[0]) {
        return None;
    }

    Some(Arc::<[TextSpan]>::from(out))
}

pub fn paint_span_for_text_range(
    spans: &[ResolvedSpan],
    range: &std::ops::Range<usize>,
    is_rtl: bool,
) -> Option<u16> {
    let idx = if range.start == range.end {
        range.start.saturating_sub(1)
    } else if is_rtl {
        range.end.saturating_sub(1)
    } else {
        range.start
    };
    spans
        .iter()
        .find(|s| idx >= s.start && idx < s.end)
        .map(|s| s.slot)
}
