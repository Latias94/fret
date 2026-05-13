#[cfg(feature = "syntax")]
use super::*;
#[cfg(feature = "syntax")]
use std::collections::HashMap;
#[cfg(feature = "syntax")]
use std::collections::{HashSet, VecDeque};
#[cfg(feature = "syntax")]
use std::ops::Range;
#[cfg(feature = "syntax")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "syntax")]
use std::time::Instant;

#[cfg(feature = "syntax")]
use super::paint::{
    add_paint_perf_elapsed, compact_row_lru_queue_if_needed, frame_cache_max_entries,
};

#[cfg(feature = "syntax")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::editor) struct SyntaxSpan {
    /// Range within the row text (UTF-8 byte indices).
    pub(in crate::editor) range: Range<usize>,
    pub(in crate::editor) highlight: &'static str,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(in crate::editor) struct SyntaxPrefetchKey {
    pub(in crate::editor) doc: DocId,
    pub(in crate::editor) rev: fret_code_editor_buffer::Revision,
    pub(in crate::editor) language: Arc<str>,
    pub(in crate::editor) chunk_start: usize,
    pub(in crate::editor) chunk_end: usize,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone)]
pub(in crate::editor) struct SyntaxPrefetchChunk {
    pub(in crate::editor) key: SyntaxPrefetchKey,
    pub(in crate::editor) rows: Arc<[(usize, Arc<[SyntaxSpan]>)]>,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Default)]
pub(in crate::editor) struct SyntaxPrefetchRuntimeState {
    pub(in crate::editor) pending: HashSet<SyntaxPrefetchKey>,
    pub(in crate::editor) ready: VecDeque<SyntaxPrefetchChunk>,
    last_visible_start: Option<usize>,
}

#[cfg(feature = "syntax")]
#[derive(Clone)]
pub(in crate::editor) struct SyntaxPrefetchRuntime {
    pub(in crate::editor) shared: Arc<Mutex<SyntaxPrefetchRuntimeState>>,
    pub(in crate::editor) dispatcher: DispatcherHandle,
}

#[cfg(feature = "syntax")]
impl std::fmt::Debug for SyntaxPrefetchRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntaxPrefetchRuntime")
            .field("shared", &self.shared)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "syntax")]
impl SyntaxPrefetchRuntime {
    pub(in crate::editor) fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            shared: Arc::new(Mutex::new(SyntaxPrefetchRuntimeState::default())),
            dispatcher,
        }
    }

    pub(in crate::editor) fn clear(&self) {
        let mut state = self.lock_state();
        state.pending.clear();
        state.ready.clear();
        state.last_visible_start = None;
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, SyntaxPrefetchRuntimeState> {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::editor) fn note_visible_start(&self, visible_start: usize) -> i8 {
        let mut state = self.lock_state();
        let direction = match state.last_visible_start {
            Some(prev) if visible_start < prev => -1,
            Some(prev) if visible_start > prev => 1,
            _ => 1,
        };
        state.last_visible_start = Some(visible_start);
        direction
    }

    pub(in crate::editor) fn drain_ready(&self) -> Vec<SyntaxPrefetchChunk> {
        let mut state = self.lock_state();
        state.ready.drain(..).collect()
    }

    pub(in crate::editor) fn try_mark_pending(&self, key: SyntaxPrefetchKey) -> bool {
        const MAX_PENDING: usize = 12;

        let mut state = self.lock_state();
        if state.pending.contains(&key) || state.ready.iter().any(|chunk| chunk.key == key) {
            return false;
        }
        if state.pending.len() >= MAX_PENDING {
            return false;
        }
        state.pending.insert(key)
    }
}

#[cfg(feature = "syntax")]
pub(in crate::editor) enum SyntaxRowCacheLookup {
    Hit(Arc<[SyntaxSpan]>),
    Miss { tick: u64 },
}

#[cfg(feature = "syntax")]
const SYNTAX_CACHE_LOOKBACK_ROWS: usize = 64;

#[cfg(feature = "syntax")]
const SYNTAX_CACHE_LOOKAHEAD_ROWS: usize = 64;

#[cfg(feature = "syntax")]
const SYNTAX_PREFETCH_CHUNK_ROWS: usize =
    SYNTAX_CACHE_LOOKBACK_ROWS + SYNTAX_CACHE_LOOKAHEAD_ROWS + 1;

#[cfg(feature = "syntax")]
pub(in crate::editor) const SYNTAX_PREFETCH_AHEAD_ROWS: usize = SYNTAX_PREFETCH_CHUNK_ROWS / 2;

#[cfg(feature = "syntax")]
pub(in crate::editor) fn syntax_prefetch_chunk_for_row(
    row: usize,
    line_count: usize,
) -> Option<(usize, usize)> {
    if line_count == 0 {
        return None;
    }

    let row = row.min(line_count.saturating_sub(1));
    let chunk_start = (row / SYNTAX_PREFETCH_CHUNK_ROWS) * SYNTAX_PREFETCH_CHUNK_ROWS;
    let chunk_end = chunk_start
        .saturating_add(SYNTAX_PREFETCH_CHUNK_ROWS.saturating_sub(1))
        .min(line_count.saturating_sub(1));
    Some((chunk_start, chunk_end))
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn syntax_row_cache_chunk_is_ready(
    st: &CodeEditorState,
    chunk_start: usize,
    chunk_end: usize,
) -> bool {
    (chunk_start..=chunk_end).all(|row| st.syntax_row_cache.contains_key(&row))
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn syntax_prefetch_visible_line_window(
    st: &CodeEditorState,
    frame: WindowedRowsPaintFrame,
) -> Option<(usize, usize)> {
    let line_count = st.buffer.line_count();
    let row_count = st.display_map.row_count();
    if line_count == 0 || row_count == 0 {
        return None;
    }

    let last_display_row = row_count.saturating_sub(1);
    let visible_start = frame.visible_start.min(last_display_row);
    let visible_end = frame.visible_end.min(last_display_row);
    let start_line = st
        .display_map
        .display_row_line(visible_start)
        .min(line_count.saturating_sub(1));
    let end_line = st
        .display_map
        .display_row_line(visible_end)
        .min(line_count.saturating_sub(1));

    Some((start_line.min(end_line), start_line.max(end_line)))
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn syntax_row_cache_store_rows<I>(
    st: &mut CodeEditorState,
    rows: I,
    max_entries: usize,
    tick: u64,
) -> usize
where
    I: IntoIterator<Item = (usize, Arc<[SyntaxSpan]>)>,
{
    let mut stored_rows = 0usize;
    for (row, spans) in rows {
        let spans_len = spans.len() as u64;
        if let Some((old, _)) = st.syntax_row_cache.insert(row, (Arc::clone(&spans), tick)) {
            st.syntax_row_cache_spans_len_total = st
                .syntax_row_cache_spans_len_total
                .saturating_sub(old.len() as u64);
        }
        st.syntax_row_cache_spans_len_total = st
            .syntax_row_cache_spans_len_total
            .saturating_add(spans_len);
        st.syntax_row_cache_queue.push_back((row, tick));
        compact_row_lru_queue_if_needed(
            &st.syntax_row_cache,
            &mut st.syntax_row_cache_queue,
            max_entries,
        );
        stored_rows = stored_rows.saturating_add(1);

        while st.syntax_row_cache.len() > max_entries {
            let Some((victim, victim_tick)) = st.syntax_row_cache_queue.pop_front() else {
                break;
            };
            let remove = st
                .syntax_row_cache
                .get(&victim)
                .is_some_and(|(_, last_used)| *last_used == victim_tick);
            if remove {
                if let Some((old, _)) = st.syntax_row_cache.remove(&victim) {
                    st.syntax_row_cache_spans_len_total = st
                        .syntax_row_cache_spans_len_total
                        .saturating_sub(old.len() as u64);
                }
                st.cache_stats.syntax_evictions = st.cache_stats.syntax_evictions.saturating_add(1);
            }
        }
    }

    stored_rows
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn syntax_rows_from_highlight_spans(
    start_byte: usize,
    row_start: usize,
    row_ranges: &[Range<usize>],
    spans: Vec<fret_syntax::HighlightSpan>,
) -> Arc<[(usize, Arc<[SyntaxSpan]>)]> {
    let mut per_row = vec![Vec::<SyntaxSpan>::new(); row_ranges.len()];

    for span in spans {
        let Some(highlight) = span.highlight else {
            continue;
        };

        let global_start = start_byte.saturating_add(span.range.start);
        let global_end = start_byte.saturating_add(span.range.end);
        if global_start >= global_end {
            continue;
        }

        let mut row_idx = row_ranges
            .iter()
            .position(|row_range| row_range.end > global_start)
            .unwrap_or(row_ranges.len());
        while row_idx < row_ranges.len() {
            let row_range = &row_ranges[row_idx];
            if row_range.start >= global_end {
                break;
            }

            let inter_start = global_start.max(row_range.start);
            let inter_end = global_end.min(row_range.end);
            if inter_start < inter_end {
                per_row[row_idx].push(SyntaxSpan {
                    range: (inter_start - row_range.start)..(inter_end - row_range.start),
                    highlight,
                });
            }

            row_idx = row_idx.saturating_add(1);
        }
    }

    let mut rows = Vec::with_capacity(row_ranges.len());
    for (idx, mut spans) in per_row.into_iter().enumerate() {
        spans.sort_by(|a, b| {
            a.range
                .start
                .cmp(&b.range.start)
                .then(a.range.end.cmp(&b.range.end))
                .then(a.highlight.cmp(&b.highlight))
        });
        spans.dedup_by(|a, b| a.range == b.range && a.highlight == b.highlight);

        let mut merged: Vec<SyntaxSpan> = Vec::new();
        for span in spans {
            if let Some(last) = merged.last_mut()
                && last.highlight == span.highlight
                && last.range.end == span.range.start
            {
                last.range.end = span.range.end;
                continue;
            }
            merged.push(span);
        }

        rows.push((row_start + idx, Arc::from(merged)));
    }

    Arc::from(rows)
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn ensure_syntax_row_cache_fresh(st: &mut CodeEditorState) {
    let rev = st.buffer.revision();
    if st.syntax_row_cache_rev != rev || st.syntax_row_cache_language != st.language {
        st.syntax_row_cache_rev = rev;
        st.syntax_row_cache_language = st.language.clone();
        st.syntax_row_cache_tick = 0;
        st.syntax_row_cache.clear();
        st.syntax_row_cache_queue.clear();
        st.syntax_row_cache_spans_len_total = 0;
        st.cache_stats.syntax_resets = st.cache_stats.syntax_resets.saturating_add(1);
        st.row_rich_cache_tick = 0;
        st.row_rich_cache.clear();
        st.row_rich_cache_queue.clear();
        st.row_rich_cache_line_bytes_estimate_total = 0;
        st.row_rich_cache_row_spans_len_total = 0;
        st.row_rich_cache_syntax_spans_len_total = 0;
        st.row_rich_cache_rich_spans_len_total = 0;
        st.invalidate_row_scene_cache();
        st.cache_stats.row_rich_resets = st.cache_stats.row_rich_resets.saturating_add(1);
        st.clear_row_rich_prefetch_runtime();
        if let Some(runtime) = st.syntax_prefetch_runtime.as_ref() {
            runtime.clear();
        }
    }
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn lookup_row_syntax_spans(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> SyntaxRowCacheLookup {
    st.cache_stats.syntax_get_calls = st.cache_stats.syntax_get_calls.saturating_add(1);
    ensure_syntax_row_cache_fresh(st);

    st.syntax_row_cache_tick = st.syntax_row_cache_tick.saturating_add(1);
    let tick = st.syntax_row_cache_tick;

    if let Some((spans, last_used)) = st.syntax_row_cache.get_mut(&row) {
        *last_used = tick;
        let spans = Arc::clone(spans);
        st.syntax_row_cache_queue.push_back((row, tick));
        compact_row_lru_queue_if_needed(
            &st.syntax_row_cache,
            &mut st.syntax_row_cache_queue,
            max_entries,
        );
        st.cache_stats.syntax_hits = st.cache_stats.syntax_hits.saturating_add(1);
        return SyntaxRowCacheLookup::Hit(spans);
    }

    st.cache_stats.syntax_misses = st.cache_stats.syntax_misses.saturating_add(1);
    SyntaxRowCacheLookup::Miss { tick }
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn populate_row_syntax_spans_after_miss(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
    tick: u64,
) -> Arc<[SyntaxSpan]> {
    let language = st.language.clone();
    let Some(language) = language.as_deref() else {
        return Arc::<[SyntaxSpan]>::from([]);
    };

    let line_count = st.buffer.line_count();
    if line_count == 0 {
        return Arc::<[SyntaxSpan]>::from([]);
    }

    if st.syntax_prefetch_runtime.is_some() {
        let _ = tick;
        let _ = language;
        return Arc::<[SyntaxSpan]>::from([]);
    }

    let chunk_start = row.saturating_sub(SYNTAX_CACHE_LOOKBACK_ROWS);
    let chunk_end = row
        .saturating_add(SYNTAX_CACHE_LOOKAHEAD_ROWS)
        .min(line_count.saturating_sub(1));
    populate_syntax_row_cache_for_chunk(st, chunk_start, chunk_end, language, max_entries, tick);

    st.syntax_row_cache
        .get(&row)
        .map(|(spans, _)| Arc::clone(spans))
        .unwrap_or_else(|| Arc::<[SyntaxSpan]>::from([]))
}

#[cfg(feature = "syntax")]
#[allow(dead_code)]
pub(in crate::editor) fn cached_row_syntax_spans(
    st: &mut CodeEditorState,
    row: usize,
    max_entries: usize,
) -> Arc<[SyntaxSpan]> {
    let lookup = lookup_row_syntax_spans(st, row, max_entries);
    match lookup {
        SyntaxRowCacheLookup::Hit(spans) => spans,
        SyntaxRowCacheLookup::Miss { tick } => {
            populate_row_syntax_spans_after_miss(st, row, max_entries, tick)
        }
    }
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn populate_syntax_row_cache_for_chunk(
    st: &mut CodeEditorState,
    chunk_start: usize,
    chunk_end: usize,
    language: &str,
    max_entries: usize,
    tick: u64,
) {
    let line_count = st.buffer.line_count();
    if line_count == 0 || chunk_start > chunk_end {
        return;
    }

    let start_byte = st
        .buffer
        .line_start(chunk_start)
        .unwrap_or(0)
        .min(st.buffer.len_bytes());
    let end_byte = if chunk_end.saturating_add(1) < line_count {
        st.buffer
            .line_start(chunk_end.saturating_add(1))
            .unwrap_or(st.buffer.len_bytes())
            .min(st.buffer.len_bytes())
    } else {
        st.buffer.len_bytes()
    };

    if start_byte >= end_byte {
        return;
    }

    let slice_started = st.paint_perf_enabled.then(Instant::now);
    let Some(slice) = st.buffer.slice_to_string(start_byte..end_byte) else {
        return;
    };
    if let Some(started) = slice_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_syntax_slice,
            &mut st.paint_perf_frame.ns_syntax_slice,
            started,
        );
    }

    let highlight_started = st.paint_perf_enabled.then(Instant::now);
    let spans = fret_syntax::highlight(slice.as_str(), language).unwrap_or_default();
    if let Some(started) = highlight_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_syntax_highlight,
            &mut st.paint_perf_frame.ns_syntax_highlight,
            started,
        );
    }

    let distribute_started = st.paint_perf_enabled.then(Instant::now);
    let mut row_ranges = Vec::with_capacity(chunk_end - chunk_start + 1);
    for row in chunk_start..=chunk_end {
        row_ranges.push(st.buffer.line_byte_range(row).unwrap_or(0..0));
    }

    let rows = syntax_rows_from_highlight_spans(start_byte, chunk_start, &row_ranges, spans);
    if let Some(started) = distribute_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_syntax_distribute,
            &mut st.paint_perf_frame.ns_syntax_distribute,
            started,
        );
    }

    let store_started = st.paint_perf_enabled.then(Instant::now);
    let stored_rows = syntax_row_cache_store_rows(st, rows.iter().cloned(), max_entries, tick);
    st.paint_perf_frame.syntax_rows_stored = st
        .paint_perf_frame
        .syntax_rows_stored
        .saturating_add(stored_rows as u64);
    if let Some(started) = store_started {
        add_paint_perf_elapsed(
            &mut st.paint_perf_frame.us_syntax_store,
            &mut st.paint_perf_frame.ns_syntax_store,
            started,
        );
    }
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn invalidate_syntax_row_cache_for_delta(
    st: &mut CodeEditorState,
    delta: fret_code_editor_buffer::BufferDelta,
) {
    // Keep the revision in sync so cached-row requests don't force a full cache clear.
    st.syntax_row_cache_rev = delta.after;
    if st.syntax_row_cache.is_empty() {
        return;
    }

    let line_count = st.buffer.line_count().max(1);
    let max_line = line_count.saturating_sub(1);

    let old_edit_start = delta.lines.start;
    let new_edit_start = delta.lines.start.min(max_line);
    let old_count = delta.lines.old_count.max(1);
    let new_count = delta.lines.new_count.max(1);
    let old_end_excl = old_edit_start.saturating_add(old_count);

    let invalidation_start = new_edit_start.saturating_sub(SYNTAX_CACHE_LOOKBACK_ROWS);
    let new_span_end = new_edit_start
        .saturating_add(new_count.saturating_sub(1))
        .min(max_line);
    let invalidation_end = new_span_end
        .saturating_add(SYNTAX_CACHE_LOOKAHEAD_ROWS)
        .min(max_line);

    let shift: isize = new_count as isize - old_count as isize;
    let shift_row = |row: usize| -> usize {
        if shift >= 0 {
            row.saturating_add(shift as usize)
        } else {
            row.saturating_sub(shift.unsigned_abs())
        }
    };

    let before_len = st.syntax_row_cache.len();
    let prev = std::mem::take(&mut st.syntax_row_cache);
    let mut next = HashMap::with_capacity(prev.len());

    for (row, (spans, tick)) in prev {
        // Always invalidate the edited line span in the old coordinate space.
        if row >= old_edit_start && row < old_end_excl {
            continue;
        }

        let mapped = if row >= old_end_excl {
            shift_row(row)
        } else {
            row
        };
        if mapped >= line_count {
            continue;
        }

        // Invalidate a bounded lookback/lookahead window in the new coordinate space.
        if mapped >= invalidation_start && mapped <= invalidation_end {
            continue;
        }

        next.insert(mapped, (spans, tick));
    }

    st.syntax_row_cache = next;
    let after_len = st.syntax_row_cache.len();
    let removed = before_len.saturating_sub(after_len);
    if removed > 0 {
        st.cache_stats.syntax_evictions = st
            .cache_stats
            .syntax_evictions
            .saturating_add(removed as u64);
    }
    rebuild_syntax_row_cache_queue(st);
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn rebuild_syntax_row_cache_queue(st: &mut CodeEditorState) {
    let mut entries: Vec<(usize, u64)> = st
        .syntax_row_cache
        .iter()
        .map(|(row, (_, tick))| (*row, *tick))
        .collect();
    entries.sort_by_key(|(_, tick)| *tick);
    st.syntax_row_cache_queue = entries.into();
}

#[cfg(feature = "syntax")]
fn drain_syntax_prefetch_ready(st: &mut CodeEditorState, max_entries: usize) {
    let Some(runtime) = st.syntax_prefetch_runtime.as_ref().cloned() else {
        return;
    };

    let mut drained = runtime.drain_ready();
    if drained.is_empty() {
        return;
    }

    ensure_syntax_row_cache_fresh(st);
    let Some(language) = st.language.as_ref().cloned() else {
        return;
    };
    let doc = st.buffer.doc();
    let rev = st.buffer.revision();
    let line_count = st.buffer.line_count();

    for chunk in drained.drain(..) {
        if chunk.key.doc != doc
            || chunk.key.rev != rev
            || chunk.key.language.as_ref() != language.as_ref()
            || chunk.key.chunk_start >= line_count
            || chunk.key.chunk_end >= line_count
        {
            continue;
        }

        st.syntax_row_cache_tick = st.syntax_row_cache_tick.saturating_add(1);
        let tick = st.syntax_row_cache_tick;
        let store_started = st.paint_perf_enabled.then(Instant::now);
        let stored_rows =
            syntax_row_cache_store_rows(st, chunk.rows.iter().cloned(), max_entries, tick);
        if let Some(started) = store_started {
            add_paint_perf_elapsed(
                &mut st.paint_perf_frame.us_syntax_store,
                &mut st.paint_perf_frame.ns_syntax_store,
                started,
            );
        }
        st.paint_perf_frame.syntax_rows_stored = st
            .paint_perf_frame
            .syntax_rows_stored
            .saturating_add(stored_rows as u64);
    }
}

#[cfg(feature = "syntax")]
pub(in crate::editor) fn schedule_syntax_prefetch_for_frame(
    st: &mut CodeEditorState,
    frame: WindowedRowsPaintFrame,
    max_entries: usize,
    window: fret_core::AppWindowId,
) {
    let max_entries = frame_cache_max_entries(st, max_entries);
    drain_syntax_prefetch_ready(st, max_entries);

    let Some(runtime) = st.syntax_prefetch_runtime.as_ref().cloned() else {
        return;
    };
    let Some(language) = st.language.as_ref().cloned() else {
        return;
    };

    ensure_syntax_row_cache_fresh(st);

    let line_count = st.buffer.line_count();
    let Some((visible_start_line, visible_end_line)) =
        syntax_prefetch_visible_line_window(st, frame)
    else {
        return;
    };

    let direction = runtime.note_visible_start(frame.visible_start);
    let mut candidate_rows = vec![visible_start_line, visible_end_line];
    let lookahead_row = if direction < 0 {
        visible_start_line.saturating_sub(SYNTAX_PREFETCH_AHEAD_ROWS)
    } else {
        visible_end_line
            .saturating_add(SYNTAX_PREFETCH_AHEAD_ROWS)
            .min(line_count.saturating_sub(1))
    };
    candidate_rows.push(lookahead_row);

    let doc = st.buffer.doc();
    let rev = st.buffer.revision();
    let mut seen = std::collections::HashSet::<(usize, usize)>::new();

    for row in candidate_rows {
        let Some((chunk_start, chunk_end)) = syntax_prefetch_chunk_for_row(row, line_count) else {
            continue;
        };
        if !seen.insert((chunk_start, chunk_end)) {
            continue;
        }
        if syntax_row_cache_chunk_is_ready(st, chunk_start, chunk_end) {
            continue;
        }

        let key = SyntaxPrefetchKey {
            doc,
            rev,
            language: language.clone(),
            chunk_start,
            chunk_end,
        };
        if !runtime.try_mark_pending(key.clone()) {
            continue;
        }

        let start_byte = st
            .buffer
            .line_start(chunk_start)
            .unwrap_or(0)
            .min(st.buffer.len_bytes());
        let end_byte = if chunk_end.saturating_add(1) < line_count {
            st.buffer
                .line_start(chunk_end.saturating_add(1))
                .unwrap_or(st.buffer.len_bytes())
                .min(st.buffer.len_bytes())
        } else {
            st.buffer.len_bytes()
        };
        let row_ranges = (chunk_start..=chunk_end)
            .map(|row| st.buffer.line_byte_range(row).unwrap_or(0..0))
            .collect::<Vec<_>>();

        let shared = runtime.shared.clone();
        let dispatcher = runtime.dispatcher.clone();
        let wake_dispatcher = dispatcher.clone();
        let job_language = language.clone();
        let slice = if start_byte < end_byte {
            st.buffer
                .slice_to_string(start_byte..end_byte)
                .unwrap_or_default()
        } else {
            String::new()
        };

        dispatcher.dispatch_background(
            Box::new(move || {
                let spans = fret_syntax::highlight(slice.as_str(), job_language.as_ref())
                    .unwrap_or_default();
                let rows =
                    syntax_rows_from_highlight_spans(start_byte, chunk_start, &row_ranges, spans);
                let mut state = shared
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let _ = state.pending.remove(&key);
                state.ready.push_back(SyntaxPrefetchChunk { key, rows });
                while state.ready.len() > 8 {
                    let _ = state.ready.pop_front();
                }
                drop(state);
                wake_dispatcher.wake(Some(window));
            }),
            fret_runtime::DispatchPriority::Low,
        );
    }
}
