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
