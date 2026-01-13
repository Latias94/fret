use super::parley_shaper::{ParleyShaper, ShapedCluster, ShapedLineLayout};
use fret_core::{CaretAffinity, TextConstraints, TextInput, TextOverflow, TextSpan, TextWrap};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WrappedSingleLine {
    pub display_text: String,
    pub display: ShapedLineLayout,
    pub kept_end: usize,
}

impl WrappedSingleLine {
    pub fn hit_test_x_source(&self, x: f32) -> (usize, CaretAffinity) {
        let (display_index, affinity) =
            hit_test_x(&self.display.clusters, x, self.display_text.len());
        (display_index.min(self.kept_end), affinity)
    }
}

pub(crate) fn wrap_single_line_with_constraints(
    shaper: &mut ParleyShaper,
    input: TextInput<'_>,
    constraints: TextConstraints,
) -> WrappedSingleLine {
    let scale = constraints.scale_factor.max(1.0);

    match constraints {
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            ..
        } => wrap_none_ellipsis(shaper, input, max_width.0 * scale, scale),
        _ => shape_as_is(shaper, input, scale),
    }
}

fn shape_as_is(shaper: &mut ParleyShaper, input: TextInput<'_>, scale: f32) -> WrappedSingleLine {
    let text = match input {
        TextInput::Plain { text, .. } => text,
        TextInput::Attributed { text, .. } => text,
    };

    WrappedSingleLine {
        display_text: text.to_string(),
        display: shaper.shape_single_line(input, scale),
        kept_end: text.len(),
    }
}

fn wrap_none_ellipsis(
    shaper: &mut ParleyShaper,
    input: TextInput<'_>,
    max_width_px: f32,
    scale: f32,
) -> WrappedSingleLine {
    let (text, base, spans) = match input {
        TextInput::Plain { text, style } => (text, style, None),
        TextInput::Attributed { text, base, spans } => (text, base, Some(spans)),
    };

    let full = shaper.shape_single_line(input, scale);
    if full.width <= max_width_px + 0.5 {
        return WrappedSingleLine {
            display_text: text.to_string(),
            display: full,
            kept_end: text.len(),
        };
    }

    let ellipsis = shaper.shape_single_line(TextInput::plain("…", base), scale);
    let ellipsis_w = ellipsis.width.max(0.0);
    let available = (max_width_px - ellipsis_w).max(0.0);

    let mut cut_end = 0usize;
    for c in &full.clusters {
        if c.x1 <= available + 0.5 {
            cut_end = cut_end.max(c.text_range.end.min(text.len()));
        }
    }

    while cut_end > 0
        && text
            .as_bytes()
            .get(cut_end.saturating_sub(1))
            .is_some_and(|b| b.is_ascii_whitespace())
    {
        cut_end = cut_end.saturating_sub(1);
    }
    while cut_end > 0 && !text.is_char_boundary(cut_end) {
        cut_end = cut_end.saturating_sub(1);
    }

    let mut display_text = String::from(&text[..cut_end]);
    display_text.push('…');

    let display = if let Some(src_spans) = spans {
        let mut out = truncate_spans(src_spans, cut_end);
        out.push(TextSpan {
            len: '…'.len_utf8(),
            shaping: Default::default(),
            paint: Default::default(),
        });
        shaper.shape_single_line(
            TextInput::Attributed {
                text: &display_text,
                base,
                spans: &out,
            },
            scale,
        )
    } else {
        shaper.shape_single_line(
            TextInput::Plain {
                text: &display_text,
                style: base,
            },
            scale,
        )
    };
    WrappedSingleLine {
        display_text,
        display,
        kept_end: cut_end,
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
    fn none_ellipsis_hits_inside_ellipsis_maps_to_cut_end() {
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

        let wrapped = wrap_single_line_with_constraints(
            &mut shaper,
            TextInput::Attributed {
                text,
                base: &base,
                spans: &spans,
            },
            constraints,
        );

        assert!(wrapped.kept_end < text.len());
        assert!(wrapped.display_text.ends_with('…'));

        let (hit, _affinity) = wrapped.hit_test_x_source(79.0);
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

        let wrapped = wrap_single_line_with_constraints(
            &mut shaper,
            TextInput::Attributed {
                text,
                base: &base,
                spans: &spans,
            },
            constraints,
        );

        assert_eq!(wrapped.display_text, text);
        assert_eq!(wrapped.kept_end, text.len());
    }
}
