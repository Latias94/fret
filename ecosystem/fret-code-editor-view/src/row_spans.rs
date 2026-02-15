use crate::DisplayRowSpan;
use std::ops::Range;

/// Map a row-local buffer byte offset to a row-local composed display byte offset.
///
/// Semantics match the editor-side `RowFoldMap`:
/// - insertions at `buffer_range.start` affect offsets strictly greater than `start` (not `==`),
/// - offsets inside a removed/replaced buffer range map to the start of the inserted display range.
pub fn buffer_local_to_display_local(spans: &[DisplayRowSpan], buffer_local: usize) -> usize {
    let mut removed = 0i64;
    let mut added = 0i64;

    for span in spans {
        let start = span.buffer_range.start;
        let end = span.buffer_range.end.max(start);
        if buffer_local <= start {
            break;
        }
        if buffer_local < end {
            return span.display_range.start;
        }
        removed += end.saturating_sub(start) as i64;
        added += span.display_range.len() as i64;
    }

    let base = buffer_local as i64 + added - removed;
    base.max(0) as usize
}

fn inserted_display_len_at(spans: &[DisplayRowSpan], buffer_local: usize) -> usize {
    spans
        .iter()
        .filter(|span| span.buffer_range.start == buffer_local && span.buffer_range.is_empty())
        .map(|span| span.display_range.len())
        .sum()
}

/// Map a row-local buffer byte range to one or more row-local composed display byte ranges.
///
/// Notes:
/// - Inserted display-only spans (inlays/placeholders/preedit text) are intentionally *not*
///   highlighted by this mapping; only ranges originating from the base buffer are returned.
/// - Removed/replaced buffer ranges are dropped from the output.
pub fn map_buffer_range_to_display_ranges(
    spans: &[DisplayRowSpan],
    buffer_range: Range<usize>,
    base_len: usize,
    display_len: usize,
) -> Vec<Range<usize>> {
    let start = buffer_range.start.min(base_len);
    let end = buffer_range.end.min(base_len).max(start);
    if start >= end {
        return Vec::new();
    }

    let mut out: Vec<Range<usize>> = Vec::new();

    // Assume `spans` are already row-local and in ascending buffer order (as produced by the
    // display-row materialization loop). Be defensive about out-of-order spans anyway.
    let mut i = 0usize;
    let mut pos = start;
    while pos < end {
        while i < spans.len() {
            let s_start = spans[i].buffer_range.start.min(base_len);
            let s_end = spans[i].buffer_range.end.min(base_len).max(s_start);

            if s_end <= pos && s_start < pos {
                i = i.saturating_add(1);
                continue;
            }
            if s_start < pos && s_start == s_end {
                // Insertion strictly before `pos`.
                i = i.saturating_add(1);
                continue;
            }
            break;
        }

        let next_boundary = spans
            .get(i)
            .map(|s| s.buffer_range.start.min(base_len))
            .unwrap_or(end)
            .min(end);

        if pos < next_boundary {
            // For buffer content starting exactly at an insertion boundary, the content appears
            // *after* the inserted display-only bytes.
            let ds = buffer_local_to_display_local(spans, pos)
                .saturating_add(inserted_display_len_at(spans, pos))
                .min(display_len);
            let de = buffer_local_to_display_local(spans, next_boundary)
                .min(display_len)
                .max(ds);
            if ds < de {
                out.push(ds..de);
            }
            pos = next_boundary;
            continue;
        }

        let Some(span) = spans.get(i) else {
            break;
        };

        let s_start = span.buffer_range.start.min(base_len);
        let s_end = span.buffer_range.end.min(base_len).max(s_start);
        if s_start < s_end && pos < s_end {
            pos = s_end.min(end);
        }

        // Always advance to avoid getting stuck on insertion spans at `pos`.
        i = i.saturating_add(1);
    }

    // Merge adjacent segments (common when multiple events sit exactly at the highlight boundary).
    let mut merged: Vec<Range<usize>> = Vec::new();
    for r in out {
        if let Some(last) = merged.last_mut()
            && last.end == r.start
        {
            last.end = r.end;
            continue;
        }
        merged.push(r);
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DisplayMap, FoldSpan, InlaySpan, InlinePreedit};
    use fret_code_editor_buffer::{DocId, TextBuffer};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn insertion_splits_mapped_range_and_drops_inserted_bytes() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcd".to_string()).unwrap();

        let mut inlays: HashMap<usize, Arc<[InlaySpan]>> = HashMap::new();
        inlays.insert(
            0,
            Arc::from([InlaySpan {
                byte: 2,
                text: Arc::<str>::from("X"),
            }]),
        );
        let map = DisplayMap::new_with_decorations_and_preedit(
            &buf,
            None,
            &HashMap::new(),
            &inlays,
            None,
        );
        let row = map.materialize_display_row_text(&buf, 0);
        assert_eq!(row.text.as_ref(), "abXcd");

        let mapped = map_buffer_range_to_display_ranges(&row.spans, 0..4, 4, row.text.len());
        assert_eq!(mapped, vec![0..2, 3..5]);
    }

    #[test]
    fn removal_drops_removed_bytes_and_accounts_for_placeholder_in_display_shift() {
        // base: "abcdef"
        // display: "ab..ef" (fold placeholder replaces "cd" at 2..4 with 2 bytes)
        let spans = vec![DisplayRowSpan {
            buffer_range: 2..4,
            display_range: 2..4,
        }];
        let mapped = map_buffer_range_to_display_ranges(&spans, 0..6, 6, 6);
        assert_eq!(mapped, vec![0..2, 4..6]);
    }

    #[test]
    fn range_inside_removed_region_maps_to_empty() {
        let spans = vec![DisplayRowSpan {
            buffer_range: 2..4,
            display_range: 2..3,
        }];
        let mapped = map_buffer_range_to_display_ranges(&spans, 2..4, 6, 5);
        assert_eq!(mapped, Vec::<Range<usize>>::new());
    }

    #[test]
    fn end_to_end_fold_placeholder_and_inlay_at_fold_start_skip_both_insertions() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "abcdef".to_string()).unwrap();

        let mut folds: HashMap<usize, Arc<[FoldSpan]>> = HashMap::new();
        folds.insert(
            0,
            Arc::from([FoldSpan {
                range: 2..4,
                placeholder: Arc::<str>::from(".."),
            }]),
        );
        let mut inlays: HashMap<usize, Arc<[InlaySpan]>> = HashMap::new();
        inlays.insert(
            0,
            Arc::from([InlaySpan {
                byte: 2,
                text: Arc::<str>::from("X"),
            }]),
        );

        let map = DisplayMap::new_with_decorations_and_preedit(&buf, None, &folds, &inlays, None);
        let row = map.materialize_display_row_text(&buf, 0);
        assert_eq!(row.text.as_ref(), "abX..ef");

        // Highlight the full base range; inserted X and placeholder must not be included.
        let mapped = map_buffer_range_to_display_ranges(&row.spans, 0..6, 6, row.text.len());
        assert_eq!(mapped, vec![0..2, 5..7]);
    }

    #[test]
    fn end_to_end_preedit_replacement_does_not_highlight_inserted_preedit_bytes() {
        let doc = DocId::new();
        let buf = TextBuffer::new(doc, "hello".to_string()).unwrap();

        let preedit = InlinePreedit {
            anchor: 1,
            replace_range: Some(1..4),
            text: Arc::<str>::from("XY"),
        };
        let map = DisplayMap::new_with_decorations_and_preedit(
            &buf,
            None,
            &HashMap::new(),
            &HashMap::new(),
            Some(preedit),
        );
        let row = map.materialize_display_row_text(&buf, 0);
        assert_eq!(row.text.as_ref(), "hXYo");

        // Base "hello" (0..5) becomes "h" + "o" in the composed view. The inserted "XY" is
        // display-only and is intentionally not highlighted by this mapping.
        let mapped = map_buffer_range_to_display_ranges(&row.spans, 0..5, 5, row.text.len());
        assert_eq!(mapped, vec![0..1, 3..4]);
    }
}
