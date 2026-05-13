//! Row geometry cache ownership for paint-produced caret/hit-test geometry.

use super::*;

pub(super) fn store_row_geom_cache(
    st: &mut CodeEditorState,
    row: usize,
    fresh_geom: Option<RowGeom>,
    text_cache_max_entries: usize,
    perf_enabled: bool,
) {
    let row_geom_cache_started = perf_enabled.then(Instant::now);

    // Cache row geometry for pointer hit-testing / IME cursor-area anchoring in event handlers.
    let rev = st.buffer.revision();
    let wrap_cols = st.display_wrap_cols;
    let folds_epoch = st.folds_epoch;
    let inlays_epoch = st.inlays_epoch;
    let display_map_epoch = st.display_map_epoch;
    if st.row_geom_cache_rev != rev
        || st.row_geom_cache_wrap_cols != wrap_cols
        || st.row_geom_cache_folds_epoch != folds_epoch
        || st.row_geom_cache_inlays_epoch != inlays_epoch
        || st.row_geom_cache_display_map_epoch != display_map_epoch
    {
        st.row_geom_cache_rev = rev;
        st.row_geom_cache_wrap_cols = wrap_cols;
        st.row_geom_cache_folds_epoch = folds_epoch;
        st.row_geom_cache_inlays_epoch = inlays_epoch;
        st.row_geom_cache_display_map_epoch = display_map_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
        st.row_geom_cache_caret_stops_len_total = 0;
    }

    st.row_geom_cache_tick = st.row_geom_cache_tick.saturating_add(1);
    let tick = st.row_geom_cache_tick;
    let has_row_geom = fresh_geom.is_some() || st.row_geom_cache.contains_key(&row);
    if has_row_geom {
        if let Some(geom) = fresh_geom {
            let caret_stops_len = geom.caret_stops.len() as u64;
            if let Some((old, _)) = st.row_geom_cache.insert(row, (geom, tick)) {
                st.row_geom_cache_caret_stops_len_total = st
                    .row_geom_cache_caret_stops_len_total
                    .saturating_sub(old.caret_stops.len() as u64);
            }
            st.row_geom_cache_caret_stops_len_total = st
                .row_geom_cache_caret_stops_len_total
                .saturating_add(caret_stops_len);
        } else if let Some((_, last_used)) = st.row_geom_cache.get_mut(&row) {
            *last_used = tick;
        }

        st.row_geom_cache_queue.push_back((row, tick));
        compact_row_lru_queue_if_needed(
            &st.row_geom_cache,
            &mut st.row_geom_cache_queue,
            text_cache_max_entries,
        );
        while st.row_geom_cache.len() > text_cache_max_entries {
            let Some((victim, victim_tick)) = st.row_geom_cache_queue.pop_front() else {
                break;
            };
            let remove = st
                .row_geom_cache
                .get(&victim)
                .is_some_and(|(_, last_used)| *last_used == victim_tick);
            if remove && let Some((old, _)) = st.row_geom_cache.remove(&victim) {
                st.row_geom_cache_caret_stops_len_total = st
                    .row_geom_cache_caret_stops_len_total
                    .saturating_sub(old.caret_stops.len() as u64);
            }
        }
    }

    if let Some(started) = row_geom_cache_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_row_geom_cache,
            &mut st.paint_perf_frame.ns_row_geom_cache,
            started,
        );
    }
}
