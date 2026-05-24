use super::*;

impl CodeEditorHandle {
    pub fn interaction(&self) -> CodeEditorInteractionOptions {
        self.state.borrow().interaction
    }

    pub fn set_interaction(&self, interaction: CodeEditorInteractionOptions) {
        self.state.borrow_mut().set_interaction(interaction);
    }

    pub fn buffer_revision(&self) -> fret_code_editor_buffer::Revision {
        self.state.borrow().buffer.revision()
    }

    pub fn selection(&self) -> Selection {
        self.state.borrow().selection
    }

    pub fn set_selection(&self, selection: Selection) {
        let mut st = self.state.borrow_mut();
        let max = st.buffer.len_bytes();
        let anchor = selection.anchor.min(max);
        let focus = selection.focus.min(max);
        st.selection = Selection { anchor, focus };
        st.set_preedit(None);
        st.caret_preferred_x = None;
        st.undo_group = None;
        st.dragging = false;
        st.drag_pointer = None;
    }

    pub fn set_caret(&self, caret: usize) {
        let caret = caret.min(self.state.borrow().buffer.len_bytes());
        self.set_selection(Selection {
            anchor: caret,
            focus: caret,
        });
    }

    pub fn preedit_active(&self) -> bool {
        self.state.borrow().preedit.is_some()
    }

    pub fn region_id(&self) -> Option<fret_ui::GlobalElementId> {
        self.state.borrow().region_id
    }

    pub fn replace_buffer(&self, buffer: TextBuffer) {
        let mut st = self.state.borrow_mut();
        st.buffer = buffer;
        #[cfg(feature = "syntax")]
        if let Some(runtime) = st.syntax_prefetch_runtime.as_ref() {
            runtime.clear();
        }
        st.selection = Selection::default();
        st.preedit = None;
        st.caret_preferred_x = None;
        st.undo = UndoHistory::with_limit(512);
        st.undo_group = None;
        st.dragging = false;
        st.drag_pointer = None;
        st.drag_autoscroll_timer = None;
        st.drag_autoscroll_viewport_pos = None;
        st.last_bounds = None;
        st.cache_stats = CodeEditorCacheStats::default();
        st.paint_perf_frame_seq = 0;
        st.paint_perf_frame = CodeEditorPaintPerfFrame::default();
        st.line_folds.clear();
        st.folds_epoch = st.folds_epoch.saturating_add(1);
        st.line_inlays.clear();
        st.inlays_epoch = st.inlays_epoch.saturating_add(1);
        st.refresh_display_map();
        st.clear_feature_payloads_for_buffer_change();
        st.row_text_cache_rev = st.buffer.revision();
        st.row_text_cache_folds_epoch = st.folds_epoch;
        st.row_text_cache_inlays_epoch = st.inlays_epoch;
        st.row_text_cache_display_map_epoch = st.display_map_epoch;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.row_text_cache_text_bytes_estimate_total = 0;
        st.row_text_cache_row_spans_len_total = 0;
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
        st.row_geom_cache_folds_epoch = st.folds_epoch;
        st.row_geom_cache_inlays_epoch = st.inlays_epoch;
        st.row_geom_cache_display_map_epoch = st.display_map_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
        st.row_geom_cache_caret_stops_len_total = 0;
        st.sync_row_scene_cache_epoch();
        st.clear_row_scene_cache_storage();
        #[cfg(feature = "syntax")]
        {
            st.syntax_row_cache_rev = st.buffer.revision();
            st.syntax_row_cache_language = st.language.clone();
            st.syntax_row_cache_tick = 0;
            st.syntax_row_cache.clear();
            st.syntax_row_cache_queue.clear();
            st.syntax_row_cache_spans_len_total = 0;
            st.clear_row_rich_prefetch_runtime();
            st.row_rich_cache_tick = 0;
            st.row_rich_cache.clear();
            st.row_rich_cache_queue.clear();
            st.row_rich_cache_line_bytes_estimate_total = 0;
            st.row_rich_cache_row_spans_len_total = 0;
            st.row_rich_cache_syntax_spans_len_total = 0;
            st.row_rich_cache_rich_spans_len_total = 0;
        }
    }

    pub fn set_text(&self, text: impl Into<String>) {
        let text = text.into();
        if self.state.borrow().buffer.text_eq(text.as_str()) {
            return;
        }
        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, text).unwrap_or_else(|_| {
            TextBuffer::new(doc, String::new()).expect("empty buffer must be valid")
        });
        self.replace_buffer(buffer);
    }

    pub fn with_buffer<R>(&self, f: impl FnOnce(&TextBuffer) -> R) -> R {
        let st = self.state.borrow();
        f(&st.buffer)
    }

    pub fn can_undo(&self) -> bool {
        self.state.borrow().undo.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.state.borrow().undo.can_redo()
    }
}
