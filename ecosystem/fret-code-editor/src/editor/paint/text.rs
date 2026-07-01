use std::ops::Range;
use std::sync::Arc;

use super::*;

#[cfg(test)]
pub(in crate::editor) fn cached_row_text(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> Arc<str> {
    cached_row_text_with_range(st, row, max_entries).1
}

pub(in crate::editor) fn cached_row_text_with_range(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> (
    Range<usize>,
    Arc<str>,
    Option<super::geom::RowFoldMap>,
    Option<Range<usize>>,
    Arc<[fret_code_editor_view::DisplayRowSpan]>,
) {
    cached_row_content_snapshot(st, row, max_entries).cloned_parts()
}

pub(in crate::editor) fn cached_row_content_snapshot(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> Arc<RowContentSnapshot> {
    st.cache_stats.row_text_get_calls = st.cache_stats.row_text_get_calls.saturating_add(1);
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    let folds_epoch = st.folds_epoch;
    let inlays_epoch = st.inlays_epoch;
    let display_map_epoch = st.display_map_epoch;
    if st.row_text_cache_rev != rev
        || st.row_text_cache_wrap_cols != wrap_cols
        || st.row_text_cache_folds_epoch != folds_epoch
        || st.row_text_cache_inlays_epoch != inlays_epoch
        || st.row_text_cache_display_map_epoch != display_map_epoch
    {
        st.row_text_cache_rev = rev;
        st.row_text_cache_wrap_cols = wrap_cols;
        st.row_text_cache_folds_epoch = folds_epoch;
        st.row_text_cache_inlays_epoch = inlays_epoch;
        st.row_text_cache_display_map_epoch = display_map_epoch;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.row_text_cache_text_bytes_estimate_total = 0;
        st.row_text_cache_row_spans_len_total = 0;
        st.cache_stats.row_text_resets = st.cache_stats.row_text_resets.saturating_add(1);
        #[cfg(feature = "syntax")]
        {
            st.clear_row_rich_prefetch_runtime();
            st.row_rich_cache_tick = 0;
            st.row_rich_cache.clear();
            st.row_rich_cache_queue.clear();
            st.row_rich_cache_line_bytes_estimate_total = 0;
            st.row_rich_cache_row_spans_len_total = 0;
            st.row_rich_cache_syntax_spans_len_total = 0;
            st.row_rich_cache_rich_spans_len_total = 0;
            st.cache_stats.row_rich_resets = st.cache_stats.row_rich_resets.saturating_add(1);
        }
    }

    st.row_text_cache_tick = st.row_text_cache_tick.saturating_add(1);
    let tick = st.row_text_cache_tick;

    if let Some((text, last_used)) = st.row_text_cache.get_mut(&row) {
        *last_used = tick;
        let out = Arc::clone(text);
        st.row_text_cache_queue.push_back((row, tick));
        compact_row_lru_queue_if_needed(
            &st.row_text_cache,
            &mut st.row_text_cache_queue,
            max_entries,
        );
        st.cache_stats.row_text_hits = st.cache_stats.row_text_hits.saturating_add(1);
        return out;
    }
    st.cache_stats.row_text_misses = st.cache_stats.row_text_misses.saturating_add(1);

    let materialized = st.display_map.materialize_display_row_text(&st.buffer, row);
    let range = materialized.row_range.clone();
    let range_for_return = range.clone();
    let preedit_range = materialized.preedit_range.clone();

    let row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]> = Arc::from(materialized.spans);
    let spans: Vec<super::geom::RowFoldSpan> = row_spans
        .iter()
        .map(|span| super::geom::RowFoldSpan {
            buffer_range: span.buffer_range.clone(),
            display_range: span.display_range.clone(),
        })
        .collect();
    let fold_map = (!spans.is_empty()).then_some(super::geom::RowFoldMap::new(spans));
    let text = materialized.text;

    let entry_text_bytes = text.len() as u64;
    let entry_row_spans_len = row_spans.len() as u64;

    let snapshot = Arc::new(RowContentSnapshot {
        text: Arc::clone(&text),
        range,
        fold_map: fold_map.clone(),
        preedit_range: preedit_range.clone(),
        row_spans: Arc::clone(&row_spans),
    });

    if let Some((old, _)) = st.row_text_cache.insert(row, (Arc::clone(&snapshot), tick)) {
        st.row_text_cache_text_bytes_estimate_total = st
            .row_text_cache_text_bytes_estimate_total
            .saturating_sub(old.text.len() as u64);
        st.row_text_cache_row_spans_len_total = st
            .row_text_cache_row_spans_len_total
            .saturating_sub(old.row_spans.len() as u64);
    }
    st.row_text_cache_text_bytes_estimate_total = st
        .row_text_cache_text_bytes_estimate_total
        .saturating_add(entry_text_bytes);
    st.row_text_cache_row_spans_len_total = st
        .row_text_cache_row_spans_len_total
        .saturating_add(entry_row_spans_len);
    st.row_text_cache_queue.push_back((row, tick));
    compact_row_lru_queue_if_needed(
        &st.row_text_cache,
        &mut st.row_text_cache_queue,
        max_entries,
    );

    while st.row_text_cache.len() > max_entries {
        let Some((victim, victim_tick)) = st.row_text_cache_queue.pop_front() else {
            break;
        };
        let remove = st
            .row_text_cache
            .get(&victim)
            .is_some_and(|(_, last_used)| *last_used == victim_tick);
        if remove {
            if let Some((old, _)) = st.row_text_cache.remove(&victim) {
                st.row_text_cache_text_bytes_estimate_total = st
                    .row_text_cache_text_bytes_estimate_total
                    .saturating_sub(old.text.len() as u64);
                st.row_text_cache_row_spans_len_total = st
                    .row_text_cache_row_spans_len_total
                    .saturating_sub(old.row_spans.len() as u64);
            }
            st.cache_stats.row_text_evictions = st.cache_stats.row_text_evictions.saturating_add(1);
        }
    }

    debug_assert_eq!(snapshot.range, range_for_return);
    debug_assert!(Arc::ptr_eq(&snapshot.text, &text));
    debug_assert_eq!(snapshot.fold_map, fold_map);
    debug_assert_eq!(snapshot.preedit_range, preedit_range);
    debug_assert!(Arc::ptr_eq(&snapshot.row_spans, &row_spans));
    snapshot
}

pub(in crate::editor) fn shift_row_text_cache_for_single_line_edit(
    st: &mut CodeEditorState,
    before_line_rows: Range<usize>,
    after_line_rows: Range<usize>,
    edit_old_end: usize,
    edit_byte_delta: isize,
) {
    st.row_text_cache_rev = st.buffer.revision();
    st.row_text_cache_wrap_cols = st.display_wrap_cols;
    st.row_text_cache_folds_epoch = st.folds_epoch;
    st.row_text_cache_inlays_epoch = st.inlays_epoch;
    st.row_text_cache_display_map_epoch = st.display_map_epoch;

    #[cfg(feature = "syntax")]
    {
        st.clear_row_rich_prefetch_runtime();
        st.row_rich_cache_tick = 0;
        st.row_rich_cache.clear();
        st.row_rich_cache_queue.clear();
        st.row_rich_cache_line_bytes_estimate_total = 0;
        st.row_rich_cache_row_spans_len_total = 0;
        st.row_rich_cache_syntax_spans_len_total = 0;
        st.row_rich_cache_rich_spans_len_total = 0;
        st.cache_stats.row_rich_resets = st.cache_stats.row_rich_resets.saturating_add(1);
    }

    if st.row_text_cache.is_empty() {
        return;
    }

    let row_delta = after_line_rows.len() as isize - before_line_rows.len() as isize;
    let old_cache = std::mem::take(&mut st.row_text_cache);
    let mut new_cache = std::collections::HashMap::with_capacity(old_cache.len());

    st.row_text_cache_text_bytes_estimate_total = 0;
    st.row_text_cache_row_spans_len_total = 0;

    for (row, (snapshot, tick)) in old_cache {
        if before_line_rows.contains(&row) {
            continue;
        }

        let shifted_snapshot = if row >= before_line_rows.end {
            shift_row_content_snapshot_for_single_line_edit(snapshot, edit_old_end, edit_byte_delta)
        } else {
            Some(snapshot)
        };
        let Some(snapshot) = shifted_snapshot else {
            continue;
        };

        let new_row = if row >= before_line_rows.end {
            shift_usize(row, row_delta)
        } else {
            row
        };

        st.row_text_cache_text_bytes_estimate_total = st
            .row_text_cache_text_bytes_estimate_total
            .saturating_add(snapshot.text.len() as u64);
        st.row_text_cache_row_spans_len_total = st
            .row_text_cache_row_spans_len_total
            .saturating_add(snapshot.row_spans.len() as u64);
        new_cache.insert(new_row, (snapshot, tick));
    }

    st.row_text_cache = new_cache;
    rebuild_row_text_cache_queue(st);
}

fn shift_row_content_snapshot_for_single_line_edit(
    snapshot: Arc<RowContentSnapshot>,
    edit_old_end: usize,
    edit_byte_delta: isize,
) -> Option<Arc<RowContentSnapshot>> {
    if snapshot.preedit_range.is_some()
        || snapshot.fold_map.is_some()
        || !snapshot.row_spans.is_empty()
    {
        return None;
    }

    let range =
        shift_range_for_single_line_edit(snapshot.range.clone(), edit_old_end, edit_byte_delta);
    Some(Arc::new(RowContentSnapshot {
        text: Arc::clone(&snapshot.text),
        range,
        fold_map: None,
        preedit_range: None,
        row_spans: Arc::from([]),
    }))
}

fn rebuild_row_text_cache_queue(st: &mut CodeEditorState) {
    let mut entries = st
        .row_text_cache
        .iter()
        .map(|(row, (_, tick))| (*row, *tick))
        .collect::<Vec<_>>();
    entries.sort_by_key(|(_, tick)| *tick);
    st.row_text_cache_queue = entries.into();
}

fn shift_range_for_single_line_edit(
    range: Range<usize>,
    edit_old_end: usize,
    delta: isize,
) -> Range<usize> {
    if range.end <= edit_old_end || delta == 0 {
        return range;
    }
    let start = shift_usize(range.start, delta);
    let end = shift_usize(range.end, delta);
    start..end.max(start)
}

fn shift_usize(value: usize, delta: isize) -> usize {
    if delta >= 0 {
        value.saturating_add(delta as usize)
    } else {
        value.saturating_sub((-delta) as usize)
    }
}
