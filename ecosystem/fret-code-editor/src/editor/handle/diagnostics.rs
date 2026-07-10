use super::*;

impl CodeEditorHandle {
    pub fn cache_stats(&self) -> CodeEditorCacheStats {
        self.state.borrow().cache_stats
    }

    pub fn cache_size_snapshot(&self) -> CodeEditorCacheSizeSnapshot {
        let st = self.state.borrow();
        let mut out = CodeEditorCacheSizeSnapshot {
            schema_version: 2,
            row_text_cache_entries: st.row_text_cache.len() as u64,
            row_text_cache_queue_len: st.row_text_cache_queue.len() as u64,
            row_text_cache_text_bytes_estimate_total: st.row_text_cache_text_bytes_estimate_total,
            row_text_cache_row_spans_len_total: st.row_text_cache_row_spans_len_total,
            row_geom_cache_entries: st.row_geom_cache.len() as u64,
            row_geom_cache_queue_len: st.row_geom_cache_queue.len() as u64,
            row_geom_cache_caret_stops_len_total: st.row_geom_cache_caret_stops_len_total,
            row_scene_cache_entries: st.row_scene_cache.len() as u64,
            row_scene_cache_queue_len: st.row_scene_cache_queue.len() as u64,
            row_scene_cache_scene_ops_len_total: st.row_scene_cache_scene_ops_len_total,
            selection_rect_scratch_capacity: st.selection_rect_scratch.capacity() as u64,
            ..Default::default()
        };

        #[cfg(feature = "syntax")]
        {
            out.syntax_row_cache_entries = st.syntax_row_cache.len() as u64;
            out.syntax_row_cache_queue_len = st.syntax_row_cache_queue.len() as u64;
            out.syntax_row_cache_spans_len_total = st.syntax_row_cache_spans_len_total;
            out.row_rich_cache_entries = st.row_rich_cache.len() as u64;
            out.row_rich_cache_queue_len = st.row_rich_cache_queue.len() as u64;
            out.row_rich_cache_line_bytes_estimate_total =
                st.row_rich_cache_line_bytes_estimate_total;
            out.row_rich_cache_row_spans_len_total = st.row_rich_cache_row_spans_len_total;
            out.row_rich_cache_syntax_spans_len_total = st.row_rich_cache_syntax_spans_len_total;
            out.row_rich_cache_rich_spans_len_total = st.row_rich_cache_rich_spans_len_total;
        }

        out
    }

    pub fn memory_snapshot(&self) -> CodeEditorMemorySnapshot {
        let st = self.state.borrow();

        let mut undo_text_bytes = 0u64;
        let mut undo_edit_count = 0u64;
        for record in st.undo.undo_records() {
            let (bytes, edits) = estimate_text_buffer_tx_text_bytes_and_edits(&record.tx.buffer_tx);
            undo_text_bytes = undo_text_bytes.saturating_add(bytes);
            undo_edit_count = undo_edit_count.saturating_add(edits);
        }

        let mut redo_text_bytes = 0u64;
        let mut redo_edit_count = 0u64;
        for record in st.undo.redo_records() {
            let (bytes, edits) = estimate_text_buffer_tx_text_bytes_and_edits(&record.tx.buffer_tx);
            redo_text_bytes = redo_text_bytes.saturating_add(bytes);
            redo_edit_count = redo_edit_count.saturating_add(edits);
        }

        CodeEditorMemorySnapshot {
            schema_version: 1,
            buffer_revision: st.buffer.revision().0,
            buffer_len_bytes: st.buffer.len_bytes() as u64,
            buffer_line_count: st.buffer.line_count() as u64,
            undo_limit: st.undo.limit() as u64,
            undo_len: st.undo.undo_len() as u64,
            redo_len: st.undo.redo_len() as u64,
            undo_text_bytes_estimate_total: undo_text_bytes,
            redo_text_bytes_estimate_total: redo_text_bytes,
            undo_edit_count_total: undo_edit_count,
            redo_edit_count_total: redo_edit_count,
        }
    }

    pub fn paint_perf_frame(&self) -> Option<CodeEditorPaintPerfFrame> {
        let st = self.state.borrow();
        st.paint_perf_enabled.then_some(st.paint_perf_frame)
    }

    pub fn reset_cache_stats(&self) {
        self.state.borrow_mut().cache_stats = CodeEditorCacheStats::default();
    }

    /// Diagnostics helper: best-effort substring check cached by buffer revision.
    ///
    /// This avoids repeated full-document scans during `UiDiagnosticsService` snapshot recording.
    pub fn diag_buffer_contains_str_cached(&self, needle: &'static str) -> bool {
        let needle_hash = diag_hash_bytes(needle.as_bytes());

        let mut st = self.state.borrow_mut();
        let rev = st.buffer.revision();
        if let Some(cache) = st.diag_contains_str_cache
            && cache.rev == rev
            && cache.needle_hash == needle_hash
        {
            return cache.value;
        }

        // NOTE: Prefer determinism over cleverness here.
        //
        // We cache this value by `TextBuffer` revision, so `to_string()` is paid at most once per
        // edit revision (acceptable for diagnostic/script use). This avoids pathological slowdowns
        // when searching directly over fragmented rope chunks.
        let value = st.buffer.text_string().contains(needle);
        st.diag_contains_str_cache = Some(DiagContainsStrCacheEntry {
            rev,
            needle_hash,
            value,
        });
        value
    }
}
