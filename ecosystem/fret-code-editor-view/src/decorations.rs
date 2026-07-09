use std::ops::Range;
use std::sync::Arc;

use fret_code_editor_buffer::TextBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum RangeDecorationLayer {
    Background,
    #[default]
    Text,
    Underline,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum RangeDecorationHitTest {
    #[default]
    None,
    Text,
    Range,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeDecoration {
    pub range: Range<usize>,
    pub class: Arc<str>,
    pub layer: RangeDecorationLayer,
    pub z_index: i16,
    pub hover_id: Option<Arc<str>>,
    pub hit_test: RangeDecorationHitTest,
}

impl RangeDecoration {
    pub fn new(range: Range<usize>, class: impl Into<Arc<str>>) -> Self {
        Self {
            range,
            class: class.into(),
            layer: RangeDecorationLayer::Text,
            z_index: 0,
            hover_id: None,
            hit_test: RangeDecorationHitTest::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeDecorationError {
    RangeStartAfterEnd,
    RangeOutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    RangeNotCharBoundary {
        start: usize,
        end: usize,
    },
    EmptyClass,
}

pub fn validate_range_decorations(
    buf: &TextBuffer,
    decorations: &[RangeDecoration],
) -> Result<(), RangeDecorationError> {
    let len = buf.len_bytes();
    for decoration in decorations {
        let start = decoration.range.start;
        let end = decoration.range.end;
        if start > end {
            return Err(RangeDecorationError::RangeStartAfterEnd);
        }
        if start > len || end > len {
            return Err(RangeDecorationError::RangeOutOfBounds { start, end, len });
        }
        if !buf.is_char_boundary(start) || !buf.is_char_boundary(end) {
            return Err(RangeDecorationError::RangeNotCharBoundary { start, end });
        }
        if decoration.class.is_empty() {
            return Err(RangeDecorationError::EmptyClass);
        }
    }

    Ok(())
}

pub fn normalized_range_decorations(
    buf: &TextBuffer,
    decorations: &[RangeDecoration],
) -> Result<Vec<RangeDecoration>, RangeDecorationError> {
    validate_range_decorations(buf, decorations)?;

    let mut out = decorations.to_vec();
    out.sort_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(a.range.end.cmp(&b.range.end))
            .then(a.layer.cmp(&b.layer))
            .then(a.z_index.cmp(&b.z_index))
            .then(a.class.cmp(&b.class))
            .then_with(|| a.hover_id.as_deref().cmp(&b.hover_id.as_deref()))
            .then(a.hit_test.cmp(&b.hit_test))
    });
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_code_editor_buffer::DocId;

    fn buffer(text: &str) -> TextBuffer {
        TextBuffer::new(DocId::new(), text.to_string()).unwrap()
    }

    #[test]
    fn range_decorations_allow_empty_and_overlapping_ranges() {
        let buf = buffer("abcdef");
        let decorations = vec![
            RangeDecoration::new(1..4, "diagnostic.warning"),
            RangeDecoration::new(2..2, "cursor.match"),
            RangeDecoration::new(3..5, "search.match"),
        ];

        assert_eq!(validate_range_decorations(&buf, &decorations), Ok(()));
    }

    #[test]
    fn range_decorations_reject_invalid_ranges() {
        let buf = buffer("abc");

        assert_eq!(
            validate_range_decorations(
                &buf,
                &[RangeDecoration::new(Range { start: 2, end: 1 }, "bad")]
            ),
            Err(RangeDecorationError::RangeStartAfterEnd)
        );
        assert_eq!(
            validate_range_decorations(&buf, &[RangeDecoration::new(0..4, "bad")]),
            Err(RangeDecorationError::RangeOutOfBounds {
                start: 0,
                end: 4,
                len: 3
            })
        );
    }

    #[test]
    fn range_decorations_reject_non_char_boundaries() {
        let buf = buffer("aé");

        assert_eq!(
            validate_range_decorations(&buf, &[RangeDecoration::new(2..3, "bad")]),
            Err(RangeDecorationError::RangeNotCharBoundary { start: 2, end: 3 })
        );
    }

    #[test]
    fn range_decorations_reject_empty_classes() {
        let buf = buffer("abc");

        assert_eq!(
            validate_range_decorations(&buf, &[RangeDecoration::new(0..1, "")]),
            Err(RangeDecorationError::EmptyClass)
        );
    }

    #[test]
    fn normalized_range_decorations_sort_deterministically_without_dropping_overlaps() {
        let buf = buffer("abcdef");
        let mut overlay = RangeDecoration::new(1..4, "overlay");
        overlay.layer = RangeDecorationLayer::Overlay;
        overlay.z_index = 10;

        let mut background = RangeDecoration::new(1..4, "background");
        background.layer = RangeDecorationLayer::Background;
        background.z_index = -10;

        let mut hover = RangeDecoration::new(1..4, "hover");
        hover.layer = RangeDecorationLayer::Underline;
        hover.hover_id = Some("hover-1".into());
        hover.hit_test = RangeDecorationHitTest::Range;

        let normalized = normalized_range_decorations(
            &buf,
            &[overlay.clone(), hover.clone(), background.clone()],
        )
        .unwrap();

        assert_eq!(normalized, vec![background, hover, overlay]);
    }

    #[test]
    fn range_decoration_hover_and_hit_test_are_data_only() {
        let mut decoration = RangeDecoration::new(0..3, "diagnostic.error");
        decoration.hover_id = Some("diagnostic:0".into());
        decoration.hit_test = RangeDecorationHitTest::Text;

        assert_eq!(decoration.hover_id.as_deref(), Some("diagnostic:0"));
        assert_eq!(decoration.hit_test, RangeDecorationHitTest::Text);
    }
}
