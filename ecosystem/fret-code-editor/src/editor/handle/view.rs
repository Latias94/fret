use super::*;

impl CodeEditorHandle {
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

    pub fn text_boundary_mode(&self) -> TextBoundaryMode {
        self.state.borrow().active_text_boundary_mode
    }

    pub fn text_boundary_mode_override(&self) -> Option<TextBoundaryMode> {
        self.state.borrow().text_boundary_mode_override
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

    /// v1 soft-wrap seam.
    ///
    /// This controls the view-layer `DisplayMap` and therefore affects:
    /// - rendered row splitting (logical lines -> display rows),
    /// - caret/selection geometry (byte <-> display point).
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
