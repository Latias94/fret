use std::ops::Range;
use std::sync::Arc;

use crate::clamp_to_char_boundary;

/// A line-local fold span expressed in UTF-8 byte indices.
///
/// v1 constraints (enforced by `validate_fold_spans`):
/// - Ranges must be within the line text and on UTF-8 char boundaries.
/// - Ranges must be non-empty, sorted, and non-overlapping.
/// - This is a view-layer contract only (no policy): selection/interaction rules are owned by the
///   surface layer (ADR 0200).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldSpan {
    pub range: Range<usize>,
    pub placeholder: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoldSpanError {
    RangeOutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    RangeNotCharBoundary {
        start: usize,
        end: usize,
    },
    EmptyRange,
    NotSortedOrOverlapping,
}

pub fn validate_fold_spans(text: &str, spans: &[FoldSpan]) -> Result<(), FoldSpanError> {
    let len = text.len();
    let mut prev_end = 0usize;
    for span in spans {
        let start = span.range.start;
        let end = span.range.end;
        if start > len || end > len {
            return Err(FoldSpanError::RangeOutOfBounds { start, end, len });
        }
        if start >= end {
            return Err(FoldSpanError::EmptyRange);
        }
        let start_clamped = clamp_to_char_boundary(text, start);
        let end_clamped = clamp_to_char_boundary(text, end);
        if start != start_clamped || end != end_clamped {
            return Err(FoldSpanError::RangeNotCharBoundary { start, end });
        }
        if start < prev_end {
            return Err(FoldSpanError::NotSortedOrOverlapping);
        }
        prev_end = end;
    }
    Ok(())
}

pub fn folded_col_count(text: &str, spans: &[FoldSpan]) -> usize {
    let mut col = 0usize;
    let mut cursor = 0usize;
    for span in spans {
        let start = span.range.start.min(text.len());
        let end = span.range.end.min(text.len()).max(start);
        col = col.saturating_add(text[cursor..start].chars().count());
        col = col.saturating_add(span.placeholder.chars().count());
        cursor = end;
    }
    col.saturating_add(text[cursor..].chars().count())
}

/// Map a line-local buffer byte offset into a "folded" display column.
///
/// If `byte` lands inside a folded region, this maps to the fold start (i.e. the column before
/// the placeholder). This keeps caret/selection clamping deterministic without making a policy
/// decision about whether the placeholder itself is navigable.
pub fn folded_byte_to_col(text: &str, spans: &[FoldSpan], byte: usize) -> usize {
    let byte = clamp_to_char_boundary(text, byte.min(text.len()));

    let mut col = 0usize;
    let mut cursor = 0usize;
    for span in spans {
        let start = span.range.start.min(text.len());
        let end = span.range.end.min(text.len()).max(start);
        let prefix = &text[cursor..start];
        let prefix_cols = prefix.chars().count();
        if byte < start {
            return col.saturating_add(text[cursor..byte].chars().count());
        }

        // Inside the folded region: clamp to the fold start (before placeholder).
        let col_before_placeholder = col.saturating_add(prefix_cols);
        if byte < end {
            return col_before_placeholder;
        }

        col = col_before_placeholder.saturating_add(span.placeholder.chars().count());
        cursor = end;
    }

    col.saturating_add(text[cursor..byte].chars().count())
}

fn byte_offset_for_col(slice: &str, col: usize) -> usize {
    if col == 0 {
        return 0;
    }
    let mut remaining = col;
    for (i, _) in slice.char_indices() {
        if remaining == 0 {
            return i;
        }
        remaining = remaining.saturating_sub(1);
    }
    slice.len()
}

/// Map a "folded" display column back to a line-local buffer byte offset.
///
/// Columns that land inside a placeholder map to the fold start.
pub fn folded_col_to_byte(text: &str, spans: &[FoldSpan], col: usize) -> usize {
    let mut cursor = 0usize;
    let mut cursor_col = 0usize;

    for span in spans {
        let start = span.range.start.min(text.len());
        let end = span.range.end.min(text.len()).max(start);

        let prefix = &text[cursor..start];
        let prefix_cols = prefix.chars().count();
        if col < cursor_col.saturating_add(prefix_cols) {
            let local_col = col.saturating_sub(cursor_col);
            let offset = byte_offset_for_col(prefix, local_col);
            return cursor.saturating_add(offset).min(text.len());
        }

        cursor_col = cursor_col.saturating_add(prefix_cols);
        let placeholder_cols = span.placeholder.chars().count();
        if col < cursor_col.saturating_add(placeholder_cols) {
            return start;
        }

        cursor_col = cursor_col.saturating_add(placeholder_cols);
        cursor = end;
    }

    let suffix = &text[cursor..];
    let local_col = col.saturating_sub(cursor_col);
    let offset = byte_offset_for_col(suffix, local_col);
    cursor.saturating_add(offset).min(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folded_mapping_basic_single_span() {
        let text = "abcdef";
        let spans = vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("…"),
        }];
        assert_eq!(folded_col_count(text, &spans), "a…ef".chars().count());

        // Inside folded bytes clamps to fold start (col=1).
        assert_eq!(folded_byte_to_col(text, &spans, 1), 1);
        assert_eq!(folded_byte_to_col(text, &spans, 2), 1);
        assert_eq!(folded_byte_to_col(text, &spans, 3), 1);

        // After fold accounts for placeholder width (1 char).
        assert_eq!(folded_byte_to_col(text, &spans, 4), 2);
        assert_eq!(folded_byte_to_col(text, &spans, 6), 4);

        // Placeholder columns map back to fold start.
        assert_eq!(folded_col_to_byte(text, &spans, 1), 1); // before placeholder
        assert_eq!(folded_col_to_byte(text, &spans, 2), 4); // after placeholder (col 2 in a…ef)
    }

    #[test]
    fn folded_mapping_placeholder_is_atomic() {
        let text = "abcdef";
        let spans = vec![FoldSpan {
            range: 1..4,
            placeholder: Arc::<str>::from("[...]"),
        }];

        // Any column within the placeholder maps to the fold start.
        // Visible text: a[...]ef
        assert_eq!(folded_col_to_byte(text, &spans, 1), 1);
        assert_eq!(folded_col_to_byte(text, &spans, 2), 1);
        assert_eq!(folded_col_to_byte(text, &spans, 5), 1);
        assert_eq!(folded_col_to_byte(text, &spans, 6), 4);
    }

    #[test]
    fn folded_mapping_handles_multibyte_chars() {
        let text = "a好b";
        let spans = vec![FoldSpan {
            range: 1..("a好".len()),
            placeholder: Arc::<str>::from("…"),
        }];

        // Visible: a…b
        assert_eq!(folded_col_count(text, &spans), 3);
        assert_eq!(folded_byte_to_col(text, &spans, 1), 1);
        assert_eq!(folded_col_to_byte(text, &spans, 1), 1);
        assert_eq!(folded_col_to_byte(text, &spans, 2), "a好".len());
    }

    #[test]
    fn validate_fold_spans_rejects_overlaps() {
        let text = "abcdef";
        let spans = vec![
            FoldSpan {
                range: 1..3,
                placeholder: Arc::<str>::from("…"),
            },
            FoldSpan {
                range: 2..4,
                placeholder: Arc::<str>::from("…"),
            },
        ];
        assert_eq!(
            validate_fold_spans(text, &spans),
            Err(FoldSpanError::NotSortedOrOverlapping)
        );
    }
}
