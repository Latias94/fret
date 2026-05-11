use super::*;

#[derive(Clone)]
pub struct CodeEditorHandle {
    pub(super) state: Rc<RefCell<CodeEditorState>>,
}

impl CodeEditorHandle {
    pub fn new(text: impl Into<String>) -> Self {
        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, text.into()).unwrap_or_else(|_| {
            TextBuffer::new(doc, String::new()).expect("empty buffer must be valid")
        });
        let state = CodeEditorState::new(buffer);
        Self {
            state: Rc::new(RefCell::new(state)),
        }
    }

    /// v1 font feature seam for code surfaces.
    ///
    /// This controls OpenType feature overrides (e.g. `liga`/`calt`) applied to code text shaping.
    /// The policy is ecosystem-owned and best-effort: if the resolved font face does not support a
    /// tag, it will be ignored by the shaping backend.
    pub fn set_code_font_feature_policy(&self, policy: CodeFontFeaturePolicy) {
        let mut st = self.state.borrow_mut();
        if st.code_font_feature_policy == policy {
            return;
        }
        st.code_font_feature_policy = policy;
        st.code_font_feature_policy_rev = st.code_font_feature_policy_rev.saturating_add(1);
        st.code_font_shaping_style = st.code_font_feature_policy.shaping_style();
        st.invalidate_row_caches();
    }

    pub fn set_language(&self, language: Option<impl Into<Arc<str>>>) {
        #[cfg(feature = "syntax")]
        {
            let mut st = self.state.borrow_mut();
            let next: Option<Arc<str>> = language.map(Into::into);
            if st.language == next {
                return;
            }
            st.language = next;
            st.cache_stats.syntax_resets = st.cache_stats.syntax_resets.saturating_add(1);
            st.syntax_row_cache_language = None;
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
            st.invalidate_row_scene_cache();
            st.cache_stats.row_rich_resets = st.cache_stats.row_rich_resets.saturating_add(1);
        }
        #[cfg(not(feature = "syntax"))]
        {
            let _ = language;
        }
    }

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

    pub fn set_preedit_debug(&self, text: impl Into<String>, cursor: Option<(usize, usize)>) {
        let text = text.into();
        let mut st = self.state.borrow_mut();
        let preedit = (!text.is_empty()).then_some(PreeditState { text, cursor });
        st.set_preedit(preedit);
        st.caret_preferred_x = None;
    }

    /// Debug-only IME composition helper: start/advance composition by replacing the current
    /// selection in the platform-facing composed view (UTF-16) without mutating the base buffer.
    ///
    /// This exists to support diag scripts that need to exercise the same path as the
    /// `TextInputRegion` platform replace hooks (selection-replacing composition + cancel).
    pub fn debug_platform_set_marked_text_for_selection(&self, text: &str) {
        let mut st = self.state.borrow_mut();

        let (value, selection, _composition) = a11y::a11y_composed_text_window(&mut st, 0);
        let Some((anchor, focus)) = selection else {
            return;
        };
        let (sel_lo, sel_hi) = if anchor <= focus {
            (anchor, focus)
        } else {
            (focus, anchor)
        };

        let start_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value.as_str(),
            sel_lo as usize,
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let end_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value.as_str(),
            sel_hi as usize,
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let range = fret_runtime::Utf16Range::new(start_utf16, end_utf16);

        let text_len_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            text,
            text.len(),
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let marked =
            fret_runtime::Utf16Range::new(start_utf16, start_utf16.saturating_add(text_len_utf16));

        let _ = platform_replace_and_mark_text_in_range_utf16(
            &mut st,
            0,
            value.as_str(),
            range,
            text,
            Some(marked),
            None,
        );
    }

    /// Debug-only IME composition helper: cancel/unmark composition best-effort.
    ///
    /// This sends an empty composition update (`text=""` with `marked=Some(_)`), which our editor
    /// treats as cancel/unmark and restores the selection captured at composition start.
    pub fn debug_platform_cancel_marked_text(&self) {
        let mut st = self.state.borrow_mut();

        let (value, selection, _composition) = a11y::a11y_composed_text_window(&mut st, 0);
        let Some((anchor, focus)) = selection else {
            return;
        };
        let (sel_lo, sel_hi) = if anchor <= focus {
            (anchor, focus)
        } else {
            (focus, anchor)
        };

        let start_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value.as_str(),
            sel_lo as usize,
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let end_utf16 = fret_core::utf::utf8_byte_offset_to_utf16_offset(
            value.as_str(),
            sel_hi as usize,
            fret_core::utf::UtfIndexClamp::Down,
        ) as u32;
        let range = fret_runtime::Utf16Range::new(start_utf16, end_utf16);

        let marked = fret_runtime::Utf16Range::new(start_utf16, start_utf16);
        let _ = platform_replace_and_mark_text_in_range_utf16(
            &mut st,
            0,
            value.as_str(),
            range,
            "",
            Some(marked),
            None,
        );
    }

    pub fn preedit_active(&self) -> bool {
        self.state.borrow().preedit.is_some()
    }

    pub fn allow_decorations_under_inline_preedit(&self) -> bool {
        self.state.borrow().allow_decorations_under_inline_preedit
    }

    pub fn set_allow_decorations_under_inline_preedit(&self, allowed: bool) {
        self.state
            .borrow_mut()
            .set_allow_decorations_under_inline_preedit(allowed);
    }

    pub fn compose_inline_preedit(&self) -> bool {
        self.state.borrow().compose_inline_preedit
    }

    pub fn set_compose_inline_preedit(&self, enabled: bool) {
        self.state.borrow_mut().set_compose_inline_preedit(enabled);
    }

    pub fn region_id(&self) -> Option<fret_ui::GlobalElementId> {
        self.state.borrow().region_id
    }

    pub fn text_boundary_mode(&self) -> TextBoundaryMode {
        self.state.borrow().active_text_boundary_mode
    }

    pub fn text_boundary_mode_override(&self) -> Option<TextBoundaryMode> {
        self.state.borrow().text_boundary_mode_override
    }

    pub fn cache_stats(&self) -> CodeEditorCacheStats {
        self.state.borrow().cache_stats
    }

    pub fn cache_size_snapshot(&self) -> CodeEditorCacheSizeSnapshotV1 {
        let st = self.state.borrow();
        let mut out = CodeEditorCacheSizeSnapshotV1 {
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

    pub fn memory_snapshot(&self) -> CodeEditorMemorySnapshotV1 {
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

        CodeEditorMemorySnapshotV1 {
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

    pub fn feature_payload_snapshot(&self) -> CodeEditorFeaturePayloadSnapshotV1 {
        self.state.borrow().feature_payloads.snapshot()
    }

    pub fn diagnostic_line_summaries(&self) -> Vec<DiagnosticLineSummary> {
        self.state
            .borrow()
            .feature_payloads
            .diagnostic_line_summaries()
            .to_vec()
    }

    pub fn set_diagnostic_spans(
        &self,
        spans: Vec<DiagnosticSpan>,
    ) -> Result<(), DiagnosticSpanError> {
        let mut st = self.state.borrow_mut();
        let normalized = normalized_diagnostic_spans(&st.buffer, &spans)?;
        let summaries = diagnostic_line_summaries(&st.buffer, &normalized)?;
        let buffer_revision = st.buffer.revision();
        if st
            .feature_payloads
            .set_diagnostic_spans(buffer_revision, normalized, summaries)
        {
            st.invalidate_feature_payload_paint_caches();
        }
        Ok(())
    }

    pub fn clear_diagnostic_spans(&self) {
        let _ = self.set_diagnostic_spans(Vec::new());
    }

    pub fn set_range_decorations(
        &self,
        decorations: Vec<RangeDecoration>,
    ) -> Result<(), RangeDecorationError> {
        let mut st = self.state.borrow_mut();
        let normalized = normalized_range_decorations(&st.buffer, &decorations)?;
        let buffer_revision = st.buffer.revision();
        if st
            .feature_payloads
            .set_range_decorations(buffer_revision, normalized)
        {
            st.invalidate_feature_payload_paint_caches();
        }
        Ok(())
    }

    pub fn clear_range_decorations(&self) {
        let _ = self.set_range_decorations(Vec::new());
    }

    pub fn set_gutter_markers(&self, markers: Vec<GutterMarker>) -> Result<(), GutterMarkerError> {
        let mut st = self.state.borrow_mut();
        validate_gutter_markers(&st.buffer, Some(&st.display_map), &markers)?;
        let normalized = normalized_gutter_markers(&markers);
        let buffer_revision = st.buffer.revision();
        let display_map_epoch = st.display_map_epoch;
        if st
            .feature_payloads
            .set_gutter_markers(buffer_revision, display_map_epoch, normalized)
        {
            st.invalidate_feature_payload_paint_caches();
        }
        Ok(())
    }

    pub fn clear_gutter_markers(&self) {
        let _ = self.set_gutter_markers(Vec::new());
    }

    pub fn set_semantic_tokens(
        &self,
        tokens: Vec<SemanticToken>,
    ) -> Result<(), SemanticTokenError> {
        let mut st = self.state.borrow_mut();
        let normalized = normalized_semantic_tokens(&st.buffer, &tokens)?;
        let buffer_revision = st.buffer.revision();
        if st
            .feature_payloads
            .set_semantic_tokens(buffer_revision, normalized)
        {
            st.invalidate_feature_payload_paint_caches();
        }
        Ok(())
    }

    pub fn clear_semantic_tokens(&self) {
        let _ = self.set_semantic_tokens(Vec::new());
    }

    pub fn set_text_boundary_mode(&self, mode: TextBoundaryMode) {
        self.set_text_boundary_mode_override(Some(mode));
    }

    pub fn set_text_boundary_mode_override(&self, mode: Option<TextBoundaryMode>) {
        let mut st = self.state.borrow_mut();
        if st.text_boundary_mode_override == mode {
            return;
        }
        st.text_boundary_mode_override = mode;
        if let Some(mode) = mode {
            st.active_text_boundary_mode = mode;
        }
        st.undo_group = None;
    }

    pub fn set_line_folds(&self, line: usize, spans: Vec<FoldSpan>) {
        let mut st = self.state.borrow_mut();
        if spans.is_empty() {
            if !st.line_folds.contains_key(&line) {
                return;
            }
            st.line_folds.remove(&line);
        } else {
            if st
                .line_folds
                .get(&line)
                .is_some_and(|existing| existing.as_ref() == spans.as_slice())
            {
                return;
            }
            st.line_folds.insert(line, Arc::from(spans));
        }
        st.folds_epoch = st.folds_epoch.saturating_add(1);
        input::clamp_selection_out_of_folds(&mut st);
        st.refresh_display_map();

        st.row_text_cache_folds_epoch = st.folds_epoch;
        st.row_text_cache_display_map_epoch = st.display_map_epoch;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.row_text_cache_text_bytes_estimate_total = 0;
        st.row_text_cache_row_spans_len_total = 0;
        st.cache_stats.row_text_resets = st.cache_stats.row_text_resets.saturating_add(1);

        st.row_geom_cache_folds_epoch = st.folds_epoch;
        st.row_geom_cache_display_map_epoch = st.display_map_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
        st.row_geom_cache_caret_stops_len_total = 0;
        st.invalidate_row_scene_cache();
    }

    pub fn clear_all_folds(&self) {
        let mut st = self.state.borrow_mut();
        if st.line_folds.is_empty() {
            return;
        }
        st.line_folds.clear();
        st.folds_epoch = st.folds_epoch.saturating_add(1);
        input::clamp_selection_out_of_folds(&mut st);
        st.refresh_display_map();

        st.row_text_cache_folds_epoch = st.folds_epoch;
        st.row_text_cache_display_map_epoch = st.display_map_epoch;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.row_text_cache_text_bytes_estimate_total = 0;
        st.row_text_cache_row_spans_len_total = 0;
        st.cache_stats.row_text_resets = st.cache_stats.row_text_resets.saturating_add(1);

        st.row_geom_cache_folds_epoch = st.folds_epoch;
        st.row_geom_cache_display_map_epoch = st.display_map_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
        st.row_geom_cache_caret_stops_len_total = 0;
        st.invalidate_row_scene_cache();
    }

    pub fn set_line_inlays(&self, line: usize, spans: Vec<InlaySpan>) {
        let mut st = self.state.borrow_mut();
        if spans.is_empty() {
            if !st.line_inlays.contains_key(&line) {
                return;
            }
            st.line_inlays.remove(&line);
        } else {
            if st
                .line_inlays
                .get(&line)
                .is_some_and(|existing| existing.as_ref() == spans.as_slice())
            {
                return;
            }
            st.line_inlays.insert(line, Arc::from(spans));
        }
        st.inlays_epoch = st.inlays_epoch.saturating_add(1);
        input::clamp_selection_out_of_folds(&mut st);
        st.refresh_display_map();

        st.row_text_cache_inlays_epoch = st.inlays_epoch;
        st.row_text_cache_display_map_epoch = st.display_map_epoch;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.row_text_cache_text_bytes_estimate_total = 0;
        st.row_text_cache_row_spans_len_total = 0;
        st.cache_stats.row_text_resets = st.cache_stats.row_text_resets.saturating_add(1);

        st.row_geom_cache_inlays_epoch = st.inlays_epoch;
        st.row_geom_cache_display_map_epoch = st.display_map_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
        st.row_geom_cache_caret_stops_len_total = 0;
        st.invalidate_row_scene_cache();
    }

    pub fn clear_all_inlays(&self) {
        let mut st = self.state.borrow_mut();
        if st.line_inlays.is_empty() {
            return;
        }
        st.line_inlays.clear();
        st.inlays_epoch = st.inlays_epoch.saturating_add(1);
        input::clamp_selection_out_of_folds(&mut st);
        st.refresh_display_map();

        st.row_text_cache_inlays_epoch = st.inlays_epoch;
        st.row_text_cache_display_map_epoch = st.display_map_epoch;
        st.row_text_cache_tick = 0;
        st.row_text_cache.clear();
        st.row_text_cache_queue.clear();
        st.row_text_cache_text_bytes_estimate_total = 0;
        st.row_text_cache_row_spans_len_total = 0;
        st.cache_stats.row_text_resets = st.cache_stats.row_text_resets.saturating_add(1);

        st.row_geom_cache_inlays_epoch = st.inlays_epoch;
        st.row_geom_cache_display_map_epoch = st.display_map_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
        st.row_geom_cache_caret_stops_len_total = 0;
        st.invalidate_row_scene_cache();
    }

    pub fn debug_decorated_line_text(&self, line: usize) -> Option<String> {
        let mut st = self.state.borrow_mut();
        if st.preedit.is_some() {
            return None;
        }
        let row = st.display_map.line_first_display_row(line);
        let (_, text, _, _, _) = paint::cached_row_text_with_range(&mut st, row, 64);
        Some(text.as_ref().to_string())
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
        st.row_scene_cache_tick = 0;
        st.row_scene_cache.clear();
        st.row_scene_cache_queue.clear();
        st.row_scene_cache_scene_ops_len_total = 0;
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
        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, text.into()).unwrap_or_else(|_| {
            TextBuffer::new(doc, String::new()).expect("empty buffer must be valid")
        });
        self.replace_buffer(buffer);
    }

    pub fn with_buffer<R>(&self, f: impl FnOnce(&TextBuffer) -> R) -> R {
        let st = self.state.borrow();
        f(&st.buffer)
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

    pub fn can_undo(&self) -> bool {
        self.state.borrow().undo.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.state.borrow().undo.can_redo()
    }

    /// v1 soft-wrap seam.
    ///
    /// This controls the view-layer `DisplayMap` and therefore affects:
    /// - rendered row splitting (logical lines -> display rows),
    /// - caret/selection geometry (byte ↔ display point).
    pub fn set_soft_wrap_cols(&self, cols: Option<usize>) {
        let mut st = self.state.borrow_mut();
        let cols = cols.filter(|v| *v > 0);
        if st.display_wrap_cols == cols {
            return;
        }
        st.display_wrap_cols = cols;
        st.refresh_display_map();
        input::clamp_selection_out_of_folds(&mut st);
        st.caret_preferred_x = None;
        st.row_text_cache_display_map_epoch = st.display_map_epoch;
        st.row_geom_cache_rev = st.buffer.revision();
        st.row_geom_cache_wrap_cols = st.display_wrap_cols;
        st.row_geom_cache_folds_epoch = st.folds_epoch;
        st.row_geom_cache_inlays_epoch = st.inlays_epoch;
        st.row_geom_cache_display_map_epoch = st.display_map_epoch;
        st.row_geom_cache_tick = 0;
        st.row_geom_cache.clear();
        st.row_geom_cache_queue.clear();
        st.row_geom_cache_caret_stops_len_total = 0;
        st.invalidate_row_scene_cache();
    }

    /// v1 code-wrap seam (ecosystem policy).
    ///
    /// This policy is only applied when soft wrap is enabled (`set_soft_wrap_cols(Some(_))`).
    pub fn set_code_wrap_policy(&self, policy: Option<CodeWrapPolicy>) {
        let mut st = self.state.borrow_mut();
        if st.code_wrap_policy == policy {
            return;
        }
        st.code_wrap_policy = policy;
        if st.display_wrap_cols.is_some() {
            st.refresh_display_map();
            input::clamp_selection_out_of_folds(&mut st);
            st.caret_preferred_x = None;
            st.row_text_cache_display_map_epoch = st.display_map_epoch;
            st.row_geom_cache_rev = st.buffer.revision();
            st.row_geom_cache_wrap_cols = st.display_wrap_cols;
            st.row_geom_cache_folds_epoch = st.folds_epoch;
            st.row_geom_cache_inlays_epoch = st.inlays_epoch;
            st.row_geom_cache_display_map_epoch = st.display_map_epoch;
            st.row_geom_cache_tick = 0;
            st.row_geom_cache.clear();
            st.row_geom_cache_queue.clear();
            st.row_geom_cache_caret_stops_len_total = 0;
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
            st.invalidate_row_scene_cache();
        }
    }
}
