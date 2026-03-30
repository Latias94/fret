use crate::parley_shaper::{ParleyShaper, ShapedLineLayout};
use crate::wrapper_balance::balanced_word_wrap_width_px;
use crate::wrapper_boundaries::hit_test_x;
#[cfg(test)]
use crate::wrapper_boundaries::is_grapheme_boundary;
use crate::wrapper_paragraphs::{
    wrap_none_ellipsis, wrap_with_newlines, wrap_with_newlines_measure_only,
};
use crate::wrapper_ranges::{
    wrap_grapheme_range, wrap_grapheme_range_measure_only, wrap_word_break_range,
    wrap_word_break_range_measure_only, wrap_word_range, wrap_word_range_measure_only,
};
use fret_core::{CaretAffinity, TextConstraints, TextInputRef, TextOverflow, TextWrap};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct WrappedLayout {
    text_len: usize,
    kept_end: usize,
    line_ranges: Vec<Range<usize>>,
    lines: Vec<ShapedLineLayout>,
}

impl WrappedLayout {
    pub(crate) fn new(
        text_len: usize,
        kept_end: usize,
        line_ranges: Vec<Range<usize>>,
        lines: Vec<ShapedLineLayout>,
    ) -> Self {
        Self {
            text_len,
            kept_end,
            line_ranges,
            lines,
        }
    }

    pub fn text_len(&self) -> usize {
        self.text_len
    }

    pub fn kept_end(&self) -> usize {
        self.kept_end
    }

    pub fn line_ranges(&self) -> &[Range<usize>] {
        &self.line_ranges
    }

    pub fn lines(&self) -> &[ShapedLineLayout] {
        &self.lines
    }

    pub fn into_parts(self) -> (usize, usize, Vec<Range<usize>>, Vec<ShapedLineLayout>) {
        (self.text_len, self.kept_end, self.line_ranges, self.lines)
    }

    #[allow(dead_code)]
    pub fn hit_test_x(&self, line_index: usize, x: f32) -> (usize, CaretAffinity) {
        let Some(line) = self.lines.get(line_index) else {
            return (0, CaretAffinity::Downstream);
        };
        let Some(range) = self.line_ranges.get(line_index) else {
            return (0, CaretAffinity::Downstream);
        };

        let (idx_local, affinity) = hit_test_x(line.clusters(), x, range.len());
        let mut idx = range.start.saturating_add(idx_local);
        if idx > self.kept_end {
            idx = self.kept_end;
        }
        (idx, affinity)
    }
}

pub fn wrap_with_constraints(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    constraints: TextConstraints,
) -> WrappedLayout {
    let scale = crate::effective_text_scale_factor(constraints.scale_factor);
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
            WrappedLayout::new(
                text_len,
                out.kept_end,
                vec![Range {
                    start: 0,
                    end: out.kept_end,
                }],
                vec![out.line],
            )
        }
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            ..
        } => wrap_word(shaper, input, text_len, max_width.0 * scale, scale),
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Balance,
            ..
        } => wrap_word_balance(shaper, input, text_len, max_width.0 * scale, scale),
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
        _ => WrappedLayout::new(
            text_len,
            text_len,
            vec![Range {
                start: 0,
                end: text_len,
            }],
            vec![shaper.shape_single_line(input, scale)],
        ),
    }
}

/// Wraps text for measurement only.
///
/// The returned `lines[*].glyphs()` is intentionally empty to avoid per-glyph work in layout.
pub fn wrap_with_constraints_measure_only(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    constraints: TextConstraints,
) -> WrappedLayout {
    let scale = crate::effective_text_scale_factor(constraints.scale_factor);
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
            line.set_width(max_width.0 * scale);
            WrappedLayout::new(
                text_len,
                text_len,
                vec![Range {
                    start: 0,
                    end: text_len,
                }],
                vec![line],
            )
        }
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            ..
        } => wrap_word_measure_only(shaper, input, text_len, max_width.0 * scale, scale),
        TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Balance,
            ..
        } => wrap_word_balance_measure_only(shaper, input, text_len, max_width.0 * scale, scale),
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
        _ => WrappedLayout::new(
            text_len,
            text_len,
            vec![Range {
                start: 0,
                end: text_len,
            }],
            vec![shaper.shape_single_line_metrics(input, scale)],
        ),
    }
}

fn wrap_word_balance(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    text_len: usize,
    max_width_px: f32,
    scale: f32,
) -> WrappedLayout {
    let width_px = balanced_word_wrap_width_px(shaper, input, text_len, max_width_px, scale);
    wrap_word(shaper, input, text_len, width_px, scale)
}

fn wrap_word_balance_measure_only(
    shaper: &mut ParleyShaper,
    input: TextInputRef<'_>,
    text_len: usize,
    max_width_px: f32,
    scale: f32,
) -> WrappedLayout {
    let width_px = balanced_word_wrap_width_px(shaper, input, text_len, max_width_px, scale);
    wrap_word_measure_only(shaper, input, text_len, width_px, scale)
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

    WrappedLayout::new(text_len, text_len, line_ranges, lines)
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

    WrappedLayout::new(text_len, text_len, line_ranges, lines)
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

    WrappedLayout::new(text_len, text_len, line_ranges, lines)
}

pub(crate) fn wrap_word_measure_only(
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

    WrappedLayout::new(text_len, text_len, line_ranges, lines)
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

    WrappedLayout::new(text_len, text_len, line_ranges, lines)
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

    WrappedLayout::new(text_len, text_len, line_ranges, lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{FontId, Px, TextPaintStyle, TextShapingStyle, TextSpan, TextStyle};
    use serde::Deserialize;

    fn shaper_with_bundled_fonts() -> ParleyShaper {
        let mut shaper = ParleyShaper::new_without_system_fonts();
        let added = shaper.add_fonts(fret_fonts::test_support::face_blobs(
            fret_fonts::bootstrap_profile()
                .faces
                .iter()
                .chain(
                    fret_fonts::default_profile()
                        .faces_for_role(fret_fonts::BundledFontRole::EmojiFallback),
                )
                .chain(
                    fret_fonts::default_profile()
                        .faces_for_role(fret_fonts::BundledFontRole::CjkFallback),
                ),
        ));
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
                .clusters()
                .iter()
                .any(|c| c.text_range() == (wrapped.kept_end..wrapped.kept_end)),
            "expected a synthetic zero-length cluster for ellipsis mapping"
        );

        let (hit, _affinity) = wrapped.hit_test_x(0, 79.0);
        assert_eq!(hit, wrapped.kept_end);
    }

    #[test]
    fn none_ellipsis_truncates_single_line_and_respects_max_width() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "This is a long line that should truncate";
        let constraints = TextConstraints {
            max_width: Some(Px(80.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let wrapped =
            wrap_with_constraints(&mut shaper, TextInputRef::plain(text, &base), constraints);

        assert_eq!(wrapped.lines.len(), 1);
        assert!(wrapped.kept_end < text.len());
        assert!(
            wrapped.lines[0].width() <= 80.0 + 0.5,
            "expected truncated line width to fit within constraints, got {}",
            wrapped.lines[0].width()
        );
    }

    #[test]
    fn none_ellipsis_does_not_split_zwj_emoji_grapheme_cluster() {
        use std::collections::HashSet;
        use unicode_segmentation::UnicodeSegmentation as _;

        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        let emoji = "👩‍👩‍👧‍👦";
        let text = format!("{emoji}{emoji}{emoji}{emoji}{emoji} hello");

        let constraints = TextConstraints {
            max_width: Some(Px(80.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let wrapped = wrap_with_constraints(
            &mut shaper,
            TextInputRef::plain(text.as_str(), &base),
            constraints,
        );
        assert!(
            wrapped.kept_end < text.len(),
            "expected ellipsis to truncate the text"
        );

        let mut boundaries: HashSet<usize> = HashSet::new();
        boundaries.insert(0);
        let mut cursor = 0usize;
        for g in text.graphemes(true) {
            cursor = cursor.saturating_add(g.len());
            boundaries.insert(cursor.min(text.len()));
        }

        assert!(
            boundaries.contains(&wrapped.kept_end),
            "expected ellipsis cut point to land on a grapheme boundary; kept_end={} text={text:?}",
            wrapped.kept_end
        );
    }

    #[test]
    fn balance_keeps_line_count_and_avoids_shorter_last_line() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        let text =
            "You haven't created any projects yet. Get started by creating your first project.";
        let max_width = Px(240.0);

        let word = wrap_with_constraints_measure_only(
            &mut shaper,
            TextInputRef::plain(text, &base),
            TextConstraints {
                max_width: Some(max_width),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor: 1.0,
            },
        );
        assert!(
            word.lines.len() >= 2,
            "expected the fixture text to wrap under the chosen width"
        );
        let word_last = word.lines.last().unwrap().width();

        let balanced = wrap_with_constraints_measure_only(
            &mut shaper,
            TextInputRef::plain(text, &base),
            TextConstraints {
                max_width: Some(max_width),
                wrap: TextWrap::Balance,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                scale_factor: 1.0,
            },
        );

        assert_eq!(balanced.lines.len(), word.lines.len());
        let balanced_last = balanced.lines.last().unwrap().width();
        assert!(
            balanced_last + 0.5 >= word_last,
            "expected balanced wrap to avoid a shorter last line; word_last={word_last} balanced_last={balanced_last}"
        );
        assert!(
            balanced
                .lines
                .iter()
                .all(|l| l.width() <= max_width.0 + 0.5),
            "expected balanced lines to respect max_width"
        );
    }

    #[test]
    fn none_ellipsis_does_not_split_keycap_grapheme_cluster() {
        use std::collections::HashSet;
        use unicode_segmentation::UnicodeSegmentation as _;

        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        let keycap = "1️⃣";
        let text = format!("{keycap}{keycap}{keycap}{keycap}{keycap}{keycap}{keycap} hello");

        let constraints = TextConstraints {
            max_width: Some(Px(70.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let wrapped = wrap_with_constraints(
            &mut shaper,
            TextInputRef::plain(text.as_str(), &base),
            constraints,
        );
        assert!(
            wrapped.kept_end < text.len(),
            "expected ellipsis to truncate the text"
        );

        let mut boundaries: HashSet<usize> = HashSet::new();
        boundaries.insert(0);
        let mut cursor = 0usize;
        for g in text.graphemes(true) {
            cursor = cursor.saturating_add(g.len());
            boundaries.insert(cursor.min(text.len()));
        }

        assert!(
            boundaries.contains(&wrapped.kept_end),
            "expected ellipsis cut point to land on a grapheme boundary; kept_end={} text={text:?}",
            wrapped.kept_end
        );
    }

    #[test]
    fn none_ellipsis_does_not_split_regional_indicator_flag_grapheme_cluster() {
        use std::collections::HashSet;
        use unicode_segmentation::UnicodeSegmentation as _;

        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        let flag = "🇺🇸";
        let text = format!("{flag}{flag}{flag}{flag}{flag}{flag}{flag}{flag} hello");

        let constraints = TextConstraints {
            max_width: Some(Px(70.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let wrapped = wrap_with_constraints(
            &mut shaper,
            TextInputRef::plain(text.as_str(), &base),
            constraints,
        );
        assert!(
            wrapped.kept_end < text.len(),
            "expected ellipsis to truncate the text"
        );

        let mut boundaries: HashSet<usize> = HashSet::new();
        boundaries.insert(0);
        let mut cursor = 0usize;
        for g in text.graphemes(true) {
            cursor = cursor.saturating_add(g.len());
            boundaries.insert(cursor.min(text.len()));
        }

        assert!(
            boundaries.contains(&wrapped.kept_end),
            "expected ellipsis cut point to land on a grapheme boundary; kept_end={} text={text:?}",
            wrapped.kept_end
        );
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
    fn wrap_uses_scale_factor_below_one() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            ..Default::default()
        };

        let text = "hello world";
        let constraints_1x = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };
        let constraints_half = TextConstraints {
            scale_factor: 0.5,
            ..constraints_1x
        };

        let a = wrap_with_constraints(
            &mut shaper,
            TextInputRef::plain(text, &base),
            constraints_1x,
        );
        let b = wrap_with_constraints(
            &mut shaper,
            TextInputRef::plain(text, &base),
            constraints_half,
        );

        let Some(font_a) = a
            .lines
            .first()
            .and_then(|l| l.glyphs().first())
            .map(|g| g.font_size())
        else {
            panic!("expected shaped glyphs for scale=1.0");
        };
        let Some(font_b) = b
            .lines
            .first()
            .and_then(|l| l.glyphs().first())
            .map(|g| g.font_size())
        else {
            panic!("expected shaped glyphs for scale=0.5");
        };

        let ratio = font_b / font_a.max(1.0);
        assert!(
            (ratio - 0.5).abs() <= 0.15,
            "expected shaped glyph font_size to scale with constraints.scale_factor; font_a={font_a} font_b={font_b} ratio={ratio}",
        );
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
            wrapped.lines[0].width() > 1.0,
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
            let delta = (expected.width() - line.width()).abs();
            assert!(
                delta <= 0.75,
                "expected wrapped line width to match shaped slice; slice={:?} expected={} actual={} delta={}",
                slice,
                expected.width(),
                line.width(),
                delta
            );
        }

        let max_line_w = wrapped
            .lines
            .iter()
            .map(|l| l.width())
            .fold(0.0f32, f32::max);
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
    fn strut_force_keeps_multiline_baseline_stable_across_fallback_glyphs() {
        let mut shaper = shaper_with_bundled_fonts();
        let base = TextStyle {
            font: FontId::family("Inter"),
            size: Px(16.0),
            strut_style: Some(fret_core::TextStrutStyle {
                force: true,
                line_height: Some(Px(18.0)),
                ..Default::default()
            }),
            ..Default::default()
        };

        let text = "Settings\nSettings 😄\nSettings 漢字\n😀 你好";
        let constraints = TextConstraints {
            max_width: Some(Px(1000.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: 1.0,
        };

        let wrapped =
            wrap_with_constraints(&mut shaper, TextInputRef::plain(text, &base), constraints);
        assert_eq!(wrapped.lines.len(), 4, "expected one line per paragraph");

        let first = &wrapped.lines[0];
        for (i, line) in wrapped.lines.iter().enumerate() {
            assert!(
                (line.line_height() - 18.0).abs() < 0.01,
                "expected fixed strut line_height=18px; line[{i}] line_height={}",
                line.line_height()
            );
            assert!(
                (line.baseline() - first.baseline()).abs() < 0.01,
                "expected strut baseline to be stable across fallback glyphs; line[{i}] baseline={} first={}",
                line.baseline(),
                first.baseline()
            );
        }
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
            assert!((a.width() - b.width()).abs() < 0.01);
            assert!((a.line_height() - b.line_height()).abs() < 0.01);
        }
        assert!(measure.lines.iter().all(|l| l.glyphs().is_empty()));
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
            assert!((a.width() - b.width()).abs() < 0.01);
            assert!((a.line_height() - b.line_height()).abs() < 0.01);
        }
        assert!(measure.lines.iter().all(|l| l.glyphs().is_empty()));
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

    fn run_text_wrap_conformance_v1_fixtures() {
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

    #[test]
    fn text_wrap_conformance_v1_fixtures() {
        run_text_wrap_conformance_v1_fixtures();
    }

    #[cfg(target_arch = "wasm32")]
    mod wasm_wrap_conformance {
        use super::*;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        fn text_wrap_conformance_v1_fixtures_wasm() {
            run_text_wrap_conformance_v1_fixtures();
        }
    }
}
