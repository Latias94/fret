use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PaintFrameCaretOverlay {
    pub(super) byte: usize,
    pub(super) row: usize,
    pub(super) col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct PaintFrameOverlayState {
    pub(super) selection_start: usize,
    pub(super) selection_end: usize,
    pub(super) selection_start_point: DisplayPoint,
    pub(super) selection_end_point: DisplayPoint,
    pub(super) caret: Option<PaintFrameCaretOverlay>,
}

impl PaintFrameOverlayState {
    pub(super) fn selection_range(self) -> Range<usize> {
        self.selection_start..self.selection_end
    }

    pub(super) fn touches_row(self, row: usize, row_range: &Range<usize>) -> bool {
        if self.caret.is_some_and(|caret| caret.row == row) {
            return true;
        }

        if self.selection_start >= self.selection_end {
            return false;
        }

        row >= self.selection_start_point.row
            && row <= self.selection_end_point.row
            && self.selection_start.max(row_range.start) < self.selection_end.min(row_range.end)
    }
}

#[derive(Debug, Clone)]
pub(super) struct CodeEditorState {
    pub(super) buffer: TextBuffer,
    pub(super) selection: Selection,
    pub(super) preedit: Option<PreeditState>,
    pub(super) preedit_replace_range: Option<Range<usize>>,
    pub(super) preedit_saved_selection: Option<Selection>,
    pub(super) code_font_feature_policy: CodeFontFeaturePolicy,
    pub(super) code_font_feature_policy_rev: u64,
    pub(super) code_font_shaping_style: TextShapingStyle,
    pub(super) font_stack_key: fret_runtime::TextFontStackKey,
    pub(super) allow_decorations_under_inline_preedit: bool,
    pub(super) compose_inline_preedit: bool,
    pub(super) interaction: CodeEditorInteractionOptions,
    pub(super) region_id: Option<fret_ui::GlobalElementId>,
    pub(super) text_boundary_mode_override: Option<TextBoundaryMode>,
    pub(super) active_text_boundary_mode: TextBoundaryMode,
    pub(super) display_wrap_cols: Option<usize>,
    pub(super) code_wrap_policy: Option<CodeWrapPolicy>,
    pub(super) display_map: DisplayMap,
    pub(super) display_map_epoch: u64,
    pub(super) feature_payloads: CodeEditorFeaturePayloadStore,
    pub(super) caret_preferred_x: Option<Px>,
    pub(super) undo: UndoHistory<CodeEditorTx>,
    pub(super) undo_group: Option<UndoGroup>,
    pub(super) dragging: bool,
    pub(super) drag_pointer: Option<fret_core::PointerId>,
    pub(super) drag_autoscroll_timer: Option<TimerToken>,
    pub(super) drag_autoscroll_viewport_pos: Option<fret_core::Point>,
    pub(super) last_bounds: Option<Rect>,
    pub(super) cache_stats: CodeEditorCacheStats,
    pub(super) line_folds: HashMap<usize, Arc<[FoldSpan]>>,
    pub(super) folds_epoch: u64,
    pub(super) line_inlays: HashMap<usize, Arc<[InlaySpan]>>,
    pub(super) inlays_epoch: u64,
    pub(super) row_text_cache_rev: fret_code_editor_buffer::Revision,
    pub(super) row_text_cache_wrap_cols: Option<usize>,
    pub(super) row_text_cache_folds_epoch: u64,
    pub(super) row_text_cache_inlays_epoch: u64,
    pub(super) row_text_cache_display_map_epoch: u64,
    pub(super) row_text_cache_tick: u64,
    pub(super) row_text_cache: HashMap<usize, (Arc<RowContentSnapshot>, u64)>,
    pub(super) row_text_cache_queue: VecDeque<(usize, u64)>,
    pub(super) row_text_cache_text_bytes_estimate_total: u64,
    pub(super) row_text_cache_row_spans_len_total: u64,
    pub(super) row_geom_cache_rev: fret_code_editor_buffer::Revision,
    pub(super) row_geom_cache_wrap_cols: Option<usize>,
    pub(super) row_geom_cache_folds_epoch: u64,
    pub(super) row_geom_cache_inlays_epoch: u64,
    pub(super) row_geom_cache_display_map_epoch: u64,
    pub(super) row_geom_cache_tick: u64,
    pub(super) row_geom_cache: HashMap<usize, (RowGeom, u64)>,
    pub(super) row_geom_cache_queue: VecDeque<(usize, u64)>,
    pub(super) row_geom_cache_caret_stops_len_total: u64,
    pub(super) row_scene_cache_rev: fret_code_editor_buffer::Revision,
    pub(super) row_scene_cache_wrap_cols: Option<usize>,
    pub(super) row_scene_cache_folds_epoch: u64,
    pub(super) row_scene_cache_inlays_epoch: u64,
    pub(super) row_scene_cache_display_map_epoch: u64,
    pub(super) row_scene_cache_feature_payload_epoch: u64,
    pub(super) row_scene_cache_tick: u64,
    pub(super) row_scene_cache: HashMap<usize, (RowSceneCacheEntry, u64)>,
    pub(super) row_scene_cache_queue: VecDeque<(usize, u64)>,
    pub(super) row_scene_cache_scene_ops_len_total: u64,
    #[cfg(feature = "syntax")]
    pub(super) row_scene_replay_plan_cache: Option<RowSceneReplayPlanCache>,
    pub(super) paint_frame_visible_window: Option<(usize, usize)>,
    pub(super) paint_frame_cache_min_entries: usize,
    pub(super) ime_surrounding_text_cache: Option<ImeSurroundingTextCache>,
    pub(super) selection_rect_scratch: Vec<Rect>,
    pub(super) baseline_measure_cache: Option<BaselineMeasureCache>,
    pub(super) paint_perf_enabled: bool,
    pub(super) paint_perf_frame_seq: u64,
    pub(super) paint_perf_frame: CodeEditorPaintPerfFrame,
    pub(super) paint_frame_overlay: PaintFrameOverlayState,
    #[cfg(feature = "syntax")]
    pub(super) language: Option<Arc<str>>,
    #[cfg(feature = "syntax")]
    pub(super) syntax_row_cache_rev: fret_code_editor_buffer::Revision,
    #[cfg(feature = "syntax")]
    pub(super) syntax_row_cache_tick: u64,
    #[cfg(feature = "syntax")]
    pub(super) syntax_row_cache_language: Option<Arc<str>>,
    #[cfg(feature = "syntax")]
    pub(super) syntax_row_cache: HashMap<usize, (Arc<[SyntaxSpan]>, u64)>,
    #[cfg(feature = "syntax")]
    pub(super) syntax_row_cache_queue: VecDeque<(usize, u64)>,
    #[cfg(feature = "syntax")]
    pub(super) syntax_row_cache_spans_len_total: u64,
    #[cfg(feature = "syntax")]
    pub(super) syntax_prefetch_runtime: Option<SyntaxPrefetchRuntime>,
    #[cfg(feature = "syntax")]
    pub(super) row_rich_prefetch_runtime: Option<RowRichPrefetchRuntime>,
    #[cfg(feature = "syntax")]
    pub(super) row_rich_cache_tick: u64,
    #[cfg(feature = "syntax")]
    pub(super) row_rich_cache: HashMap<usize, (RowRichCacheEntry, u64)>,
    #[cfg(feature = "syntax")]
    pub(super) row_rich_cache_queue: VecDeque<(usize, u64)>,
    #[cfg(feature = "syntax")]
    pub(super) row_rich_cache_line_bytes_estimate_total: u64,
    #[cfg(feature = "syntax")]
    pub(super) row_rich_cache_row_spans_len_total: u64,
    #[cfg(feature = "syntax")]
    pub(super) row_rich_cache_syntax_spans_len_total: u64,
    #[cfg(feature = "syntax")]
    pub(super) row_rich_cache_rich_spans_len_total: u64,

    pub(super) diag_contains_str_cache: Option<DiagContainsStrCacheEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DiagContainsStrCacheEntry {
    pub(super) rev: fret_code_editor_buffer::Revision,
    pub(super) needle_hash: u64,
    pub(super) value: bool,
}

pub(super) fn diag_hash_bytes(bytes: &[u8]) -> u64 {
    use std::hash::Hasher as _;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(bytes);
    hasher.finish()
}

#[derive(Debug, Clone)]
pub(super) struct ImeSurroundingTextCache {
    pub(super) revision: fret_code_editor_buffer::Revision,
    pub(super) selection: Selection,
    pub(super) surrounding: fret_runtime::WindowImeSurroundingText,
}

#[derive(Debug, Clone)]
pub(super) struct RowContentSnapshot {
    pub(super) text: Arc<str>,
    pub(super) range: Range<usize>,
    pub(super) fold_map: Option<geom::RowFoldMap>,
    pub(super) preedit_range: Option<Range<usize>>,
    pub(super) row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]>,
}

impl RowContentSnapshot {
    pub(super) fn cloned_parts(
        &self,
    ) -> (
        Range<usize>,
        Arc<str>,
        Option<geom::RowFoldMap>,
        Option<Range<usize>>,
        Arc<[fret_code_editor_view::DisplayRowSpan]>,
    ) {
        (
            self.range.clone(),
            Arc::clone(&self.text),
            self.fold_map.clone(),
            self.preedit_range.clone(),
            Arc::clone(&self.row_spans),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct ColorKey {
    pub(super) r: u32,
    pub(super) g: u32,
    pub(super) b: u32,
    pub(super) a: u32,
}

impl From<Color> for ColorKey {
    fn from(value: Color) -> Self {
        Self {
            r: value.r.to_bits(),
            g: value.g.to_bits(),
            b: value.b.to_bits(),
            a: value.a.to_bits(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum RowScenePaintKey {
    Plain {
        fg: ColorKey,
    },
    #[cfg(feature = "syntax")]
    Syntax {
        fg: ColorKey,
        theme_revision: u64,
    },
    Preedit {
        fg: ColorKey,
        selection_bg: ColorKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct RowSceneKey {
    pub(super) row_geom_key: geom::RowGeomKey,
    pub(super) paint_key: RowScenePaintKey,
}

impl RowSceneKey {
    pub(super) fn plain(row_geom_key: geom::RowGeomKey, fg: Color) -> Self {
        Self {
            row_geom_key,
            paint_key: RowScenePaintKey::Plain { fg: fg.into() },
        }
    }

    #[cfg(feature = "syntax")]
    pub(super) fn syntax(row_geom_key: geom::RowGeomKey, fg: Color, theme_revision: u64) -> Self {
        Self {
            row_geom_key,
            paint_key: RowScenePaintKey::Syntax {
                fg: fg.into(),
                theme_revision,
            },
        }
    }

    pub(super) fn preedit(row_geom_key: geom::RowGeomKey, fg: Color, selection_bg: Color) -> Self {
        Self {
            row_geom_key,
            paint_key: RowScenePaintKey::Preedit {
                fg: fg.into(),
                selection_bg: selection_bg.into(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct RowSceneRetainedFragment {
    pub(super) content: Arc<RowContentSnapshot>,
    pub(super) origin: Point,
    pub(super) geom: geom::RowGeom,
    pub(super) is_rich: bool,
    pub(super) ops: Arc<[SceneOp]>,
    pub(super) hosted_resources: fret_ui::canvas::CanvasHostedResources,
}

#[derive(Debug, Clone)]
pub(super) struct RowSceneCacheEntry {
    pub(super) key: RowSceneKey,
    pub(super) retained: Arc<RowSceneRetainedFragment>,
    #[cfg(feature = "syntax")]
    pub(super) syntax_replay_key: Option<RowSceneSyntaxReplayKey>,
}

#[derive(Debug, Clone)]
pub(super) struct RowSceneReplayPlanEntry {
    pub(super) row: usize,
    pub(super) retained: Arc<RowSceneRetainedFragment>,
    pub(super) local_bounds: Rect,
}

#[derive(Debug, Clone)]
pub(super) struct RowSceneReplayPlanCacheEntry {
    pub(super) row: usize,
    pub(super) retained: Arc<RowSceneRetainedFragment>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct RowSceneReplayPlan {
    pub(super) frame_seq: u64,
    pub(super) entries: VecDeque<RowSceneReplayPlanEntry>,
    pub(super) hosted_resources: fret_ui::canvas::CanvasHostedResources,
    pub(super) hosted_resources_touched: bool,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RowSceneReplayPlanCacheKey {
    pub(super) buffer_revision: fret_code_editor_buffer::Revision,
    pub(super) display_wrap_cols: Option<usize>,
    pub(super) folds_epoch: u64,
    pub(super) inlays_epoch: u64,
    pub(super) display_map_epoch: u64,
    pub(super) feature_payload_epoch: u64,
    pub(super) max_entries: usize,
    pub(super) row_count: usize,
    pub(super) row_height_bits: u32,
    pub(super) row_stride_bits: u32,
    pub(super) gap_bits: u32,
    pub(super) scroll_margin_bits: u32,
    pub(super) content_origin_x_bits: u32,
    pub(super) content_width_bits: u32,
    pub(super) content_height_bits: u32,
    pub(super) text_style: RowSceneTextStyleKey,
    pub(super) constraints: RowSceneTextConstraintsKey,
    pub(super) font_stack_key: u64,
    pub(super) scale_bits: u32,
    pub(super) theme_revision: u64,
    pub(super) code_font_feature_policy_rev: u64,
    pub(super) fg: ColorKey,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone)]
pub(super) struct RowSceneReplayPlanCache {
    pub(super) key: RowSceneReplayPlanCacheKey,
    pub(super) entries: Vec<RowSceneReplayPlanCacheEntry>,
}

impl fret_ui::tree::BoundarySceneFragmentDebug for RowSceneReplayPlan {
    fn boundary_scene_fragment_entry_count(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct BaselineMeasureCache {
    pub(super) max_width: Px,
    pub(super) row_h: Px,
    pub(super) scale_bits: u32,
    pub(super) text_style: TextStyle,
    pub(super) metrics: fret_core::TextMetrics,
    pub(super) measured_h: Px,
}

#[cfg(feature = "syntax")]
#[derive(Debug, Clone)]
pub(super) struct RowRichCacheEntry {
    pub(super) row_range: Range<usize>,
    pub(super) line: Arc<str>,
    pub(super) syntax_spans: Arc<[SyntaxSpan]>,
    pub(super) row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]>,
    pub(super) theme_revision: u64,
    pub(super) code_font_feature_policy_rev: u64,
    pub(super) rich: fret_core::AttributedText,
}

impl CodeEditorState {
    pub(super) fn new(buffer: TextBuffer) -> Self {
        let display_map = DisplayMap::new(&buffer, None);
        let buffer_revision = buffer.revision();
        Self {
            buffer,
            selection: Selection::default(),
            preedit: None,
            preedit_replace_range: None,
            preedit_saved_selection: None,
            code_font_feature_policy: CodeFontFeaturePolicy::default(),
            code_font_feature_policy_rev: 0,
            code_font_shaping_style: CodeFontFeaturePolicy::default().shaping_style(),
            font_stack_key: fret_runtime::TextFontStackKey::default(),
            allow_decorations_under_inline_preedit: false,
            compose_inline_preedit: false,
            interaction: CodeEditorInteractionOptions::default(),
            region_id: None,
            text_boundary_mode_override: Some(TextBoundaryMode::Identifier),
            active_text_boundary_mode: TextBoundaryMode::Identifier,
            display_wrap_cols: None,
            code_wrap_policy: Some(CodeWrapPolicy::preset(CodeWrapPreset::Balanced)),
            display_map,
            display_map_epoch: 0,
            feature_payloads: CodeEditorFeaturePayloadStore::new(buffer_revision, 0),
            caret_preferred_x: None,
            undo: UndoHistory::with_limit(512),
            undo_group: None,
            dragging: false,
            drag_pointer: None,
            drag_autoscroll_timer: None,
            drag_autoscroll_viewport_pos: None,
            last_bounds: None,
            cache_stats: CodeEditorCacheStats::default(),
            line_folds: HashMap::new(),
            folds_epoch: 0,
            line_inlays: HashMap::new(),
            inlays_epoch: 0,
            row_text_cache_rev: fret_code_editor_buffer::Revision(0),
            row_text_cache_wrap_cols: None,
            row_text_cache_folds_epoch: 0,
            row_text_cache_inlays_epoch: 0,
            row_text_cache_display_map_epoch: 0,
            row_text_cache_tick: 0,
            row_text_cache: HashMap::new(),
            row_text_cache_queue: VecDeque::new(),
            row_text_cache_text_bytes_estimate_total: 0,
            row_text_cache_row_spans_len_total: 0,
            row_geom_cache_rev: fret_code_editor_buffer::Revision(0),
            row_geom_cache_wrap_cols: None,
            row_geom_cache_folds_epoch: 0,
            row_geom_cache_inlays_epoch: 0,
            row_geom_cache_display_map_epoch: 0,
            row_geom_cache_tick: 0,
            row_geom_cache: HashMap::new(),
            row_geom_cache_queue: VecDeque::new(),
            row_geom_cache_caret_stops_len_total: 0,
            row_scene_cache_rev: fret_code_editor_buffer::Revision(0),
            row_scene_cache_wrap_cols: None,
            row_scene_cache_folds_epoch: 0,
            row_scene_cache_inlays_epoch: 0,
            row_scene_cache_display_map_epoch: 0,
            row_scene_cache_feature_payload_epoch: 0,
            row_scene_cache_tick: 0,
            row_scene_cache: HashMap::new(),
            row_scene_cache_queue: VecDeque::new(),
            row_scene_cache_scene_ops_len_total: 0,
            #[cfg(feature = "syntax")]
            row_scene_replay_plan_cache: None,
            paint_frame_visible_window: None,
            paint_frame_cache_min_entries: 0,
            ime_surrounding_text_cache: None,
            selection_rect_scratch: Vec::new(),
            baseline_measure_cache: None,
            paint_perf_enabled: paint_perf_enabled_from_env(),
            paint_perf_frame_seq: 0,
            paint_perf_frame: CodeEditorPaintPerfFrame::default(),
            paint_frame_overlay: PaintFrameOverlayState::default(),
            #[cfg(feature = "syntax")]
            language: None,
            #[cfg(feature = "syntax")]
            syntax_row_cache_rev: fret_code_editor_buffer::Revision(0),
            #[cfg(feature = "syntax")]
            syntax_row_cache_tick: 0,
            #[cfg(feature = "syntax")]
            syntax_row_cache_language: None,
            #[cfg(feature = "syntax")]
            syntax_row_cache: HashMap::new(),
            #[cfg(feature = "syntax")]
            syntax_row_cache_queue: VecDeque::new(),
            #[cfg(feature = "syntax")]
            syntax_row_cache_spans_len_total: 0,
            #[cfg(feature = "syntax")]
            syntax_prefetch_runtime: None,
            #[cfg(feature = "syntax")]
            row_rich_prefetch_runtime: None,
            #[cfg(feature = "syntax")]
            row_rich_cache_tick: 0,
            #[cfg(feature = "syntax")]
            row_rich_cache: HashMap::new(),
            #[cfg(feature = "syntax")]
            row_rich_cache_queue: VecDeque::new(),
            #[cfg(feature = "syntax")]
            row_rich_cache_line_bytes_estimate_total: 0,
            #[cfg(feature = "syntax")]
            row_rich_cache_row_spans_len_total: 0,
            #[cfg(feature = "syntax")]
            row_rich_cache_syntax_spans_len_total: 0,
            #[cfg(feature = "syntax")]
            row_rich_cache_rich_spans_len_total: 0,

            diag_contains_str_cache: None,
        }
    }

    pub(super) fn update_font_stack_key(&mut self, next: fret_runtime::TextFontStackKey) {
        if self.font_stack_key == next {
            return;
        }
        self.font_stack_key = next;

        // Font stack changes can affect shaping and therefore caret/selection geometry. Ensure we
        // never answer platform geometry queries from stale cached row geometry.
        self.row_geom_cache_tick = 0;
        self.row_geom_cache.clear();
        self.row_geom_cache_queue.clear();
        self.row_geom_cache_caret_stops_len_total = 0;
        self.invalidate_row_scene_cache();
        self.baseline_measure_cache = None;
    }

    pub(super) fn clear_row_scene_cache_storage(&mut self) {
        self.row_scene_cache_tick = 0;
        self.row_scene_cache.clear();
        self.row_scene_cache_queue.clear();
        self.row_scene_cache_scene_ops_len_total = 0;
        #[cfg(feature = "syntax")]
        {
            self.row_scene_replay_plan_cache = None;
        }
    }

    pub(super) fn clear_row_scene_cache(&mut self) {
        self.clear_row_scene_cache_storage();
        self.cache_stats.row_scene_resets = self.cache_stats.row_scene_resets.saturating_add(1);
    }

    #[cfg(feature = "syntax")]
    pub(super) fn clear_row_rich_prefetch_runtime(&mut self) {
        if let Some(runtime) = self.row_rich_prefetch_runtime.as_ref() {
            runtime.clear();
        }
    }

    pub(super) fn sync_row_scene_cache_epoch(&mut self) {
        self.row_scene_cache_rev = self.buffer.revision();
        self.row_scene_cache_wrap_cols = self.display_wrap_cols;
        self.row_scene_cache_folds_epoch = self.folds_epoch;
        self.row_scene_cache_inlays_epoch = self.inlays_epoch;
        self.row_scene_cache_display_map_epoch = self.display_map_epoch;
        self.row_scene_cache_feature_payload_epoch = self.feature_payloads.epoch();
    }

    pub(super) fn invalidate_row_scene_cache(&mut self) {
        self.sync_row_scene_cache_epoch();
        self.clear_row_scene_cache();
    }

    pub(super) fn invalidate_feature_payload_paint_caches(&mut self) {
        self.invalidate_row_scene_cache();

        #[cfg(feature = "syntax")]
        {
            self.clear_row_rich_prefetch_runtime();
            self.row_rich_cache_tick = 0;
            self.row_rich_cache.clear();
            self.row_rich_cache_queue.clear();
            self.row_rich_cache_line_bytes_estimate_total = 0;
            self.row_rich_cache_row_spans_len_total = 0;
            self.row_rich_cache_syntax_spans_len_total = 0;
            self.row_rich_cache_rich_spans_len_total = 0;
            self.cache_stats.row_rich_resets = self.cache_stats.row_rich_resets.saturating_add(1);
        }
    }

    pub(super) fn clear_feature_payloads_for_buffer_change(&mut self) {
        if self
            .feature_payloads
            .clear_all_for_buffer_change(self.buffer.revision(), self.display_map_epoch)
        {
            self.invalidate_feature_payload_paint_caches();
        }
    }

    pub(super) fn invalidate_row_caches(&mut self) {
        self.row_text_cache_display_map_epoch = self.display_map_epoch;
        self.row_text_cache_tick = 0;
        self.row_text_cache.clear();
        self.row_text_cache_queue.clear();
        self.row_text_cache_text_bytes_estimate_total = 0;
        self.row_text_cache_row_spans_len_total = 0;
        self.cache_stats.row_text_resets = self.cache_stats.row_text_resets.saturating_add(1);

        self.row_geom_cache_display_map_epoch = self.display_map_epoch;
        self.row_geom_cache_tick = 0;
        self.row_geom_cache.clear();
        self.row_geom_cache_queue.clear();
        self.row_geom_cache_caret_stops_len_total = 0;
        self.invalidate_row_scene_cache();

        #[cfg(feature = "syntax")]
        {
            self.clear_row_rich_prefetch_runtime();
            self.row_rich_cache_tick = 0;
            self.row_rich_cache.clear();
            self.row_rich_cache_queue.clear();
            self.row_rich_cache_line_bytes_estimate_total = 0;
            self.row_rich_cache_row_spans_len_total = 0;
            self.row_rich_cache_syntax_spans_len_total = 0;
            self.row_rich_cache_rich_spans_len_total = 0;
            self.cache_stats.row_rich_resets = self.cache_stats.row_rich_resets.saturating_add(1);
        }
    }

    pub(super) fn refresh_display_map(&mut self) {
        // ADR 0185 / ADR 0188:
        //
        // v1 baseline: inline IME preedit is modeled as a paint-time injection. This means we
        // cannot allow wrap-driven row breaking to depend on the preedit string, so by default we
        // suppress fold placeholders / inlays while preedit is active in wrapped mode.
        //
        // Staging: downstream consumers (and the UI Gallery harness) can opt into keeping
        // decorations enabled under inline preedit even when wrapped. This keeps row-breaking
        // stable (still based on fold/inlay composition only) while we migrate toward a fragment-
        // composed DisplayMap (ADR 0188).
        let force_inline_preedit = self
            .preedit_replace_range
            .as_ref()
            .is_some_and(|r| !r.is_empty());
        let compose_inline_preedit = self.compose_inline_preedit || force_inline_preedit;

        let suppress_decorations = !compose_inline_preedit
            && self.preedit.is_some()
            && self.display_wrap_cols.is_some()
            && !self.allow_decorations_under_inline_preedit;

        let code_wrap_policy = self
            .display_wrap_cols
            .is_some()
            .then_some(self.code_wrap_policy)
            .flatten();

        let preedit = compose_inline_preedit
            .then_some(())
            .and_then(|_| self.preedit.as_ref())
            .map(|p| InlinePreedit {
                anchor: self.selection.caret().min(self.buffer.len_bytes()),
                replace_range: self.preedit_replace_range.clone(),
                text: Arc::<str>::from(p.text.as_str()),
            });

        self.display_map = if suppress_decorations {
            DisplayMap::new_with_code_wrap_policy(
                &self.buffer,
                self.display_wrap_cols,
                code_wrap_policy,
            )
        } else if compose_inline_preedit {
            DisplayMap::new_with_decorations_and_preedit_and_code_wrap_policy(
                &self.buffer,
                self.display_wrap_cols,
                &self.line_folds,
                &self.line_inlays,
                preedit,
                code_wrap_policy,
            )
        } else {
            DisplayMap::new_with_decorations_and_preedit_and_code_wrap_policy(
                &self.buffer,
                self.display_wrap_cols,
                &self.line_folds,
                &self.line_inlays,
                None,
                code_wrap_policy,
            )
        };
        self.display_map_epoch = self.display_map_epoch.saturating_add(1);
        self.feature_payloads
            .retain_gutter_markers_valid_for_display_map(
                &self.buffer,
                &self.display_map,
                self.display_map_epoch,
            );
        #[cfg(feature = "syntax")]
        self.clear_row_rich_prefetch_runtime();
    }

    pub(super) fn begin_paint_frame(&mut self, frame: WindowedRowsPaintFrame) {
        let visible_window =
            normalized_paint_frame_visible_window(frame.visible_start, frame.visible_end);
        self.paint_frame_cache_min_entries =
            paint_frame_cache_min_entries(self.paint_frame_visible_window, visible_window);
        self.paint_frame_visible_window = visible_window;

        if self.paint_perf_enabled {
            self.paint_perf_frame_seq = self.paint_perf_frame_seq.saturating_add(1);
            let visible_rows = visible_window
                .map(|(start, end)| paint_frame_visible_row_count(start, end) as u64)
                .unwrap_or(0);
            self.paint_perf_frame = CodeEditorPaintPerfFrame {
                frame_seq: self.paint_perf_frame_seq,
                visible_start: frame.visible_start as u64,
                visible_end: frame.visible_end as u64,
                visible_rows,
                cache_frame_min_entries: self.paint_frame_cache_min_entries as u64,
                ..CodeEditorPaintPerfFrame::default()
            };
        }

        let started = self.paint_perf_enabled.then(Instant::now);
        let len = self.buffer.len_bytes();
        let selection = self.selection.normalized();
        let selection_start = selection.start.min(len);
        let selection_end = selection.end.min(len);
        let (selection_start_point, selection_end_point) = if selection_start < selection_end {
            (
                self.display_map
                    .byte_to_display_point(&self.buffer, selection_start),
                self.display_map
                    .byte_to_display_point(&self.buffer, selection_end),
            )
        } else {
            (DisplayPoint::default(), DisplayPoint::default())
        };
        let caret = if self.selection.is_caret() {
            let byte = self.selection.caret().min(len);
            let point = self.display_map.byte_to_display_point(&self.buffer, byte);
            Some(PaintFrameCaretOverlay {
                byte,
                row: point.row,
                col: point.col,
            })
        } else {
            None
        };
        self.paint_frame_overlay = PaintFrameOverlayState {
            selection_start,
            selection_end,
            selection_start_point,
            selection_end_point,
            caret,
        };
        if let Some(started) = started {
            let elapsed = started.elapsed();
            self.paint_perf_frame.us_frame_overlay_prepare =
                u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
            self.paint_perf_frame.ns_frame_overlay_prepare =
                u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
        }
    }

    pub(super) fn record_windowed_rows_paint_diagnostics(
        &mut self,
        diagnostics: WindowedRowsPaintDiagnostics,
    ) {
        if !self.paint_perf_enabled {
            return;
        }

        self.paint_perf_frame.surface_rows_iterated = diagnostics.rows_iterated;
        self.paint_perf_frame.surface_rows_with_rect = diagnostics.rows_with_rect;
        self.paint_perf_frame.us_windowed_surface_paint_callback = diagnostics.us_paint_callback;
        self.paint_perf_frame.us_windowed_surface_frame_lookup = diagnostics.us_frame_lookup;
        self.paint_perf_frame.us_windowed_surface_hook = diagnostics.us_on_paint_frame;
        self.paint_perf_frame.us_windowed_surface_row_loop = diagnostics.us_row_loop;
        self.paint_perf_frame.us_windowed_surface_row_rect = diagnostics.us_row_rect;
        self.paint_perf_frame.us_windowed_surface_row_paint = diagnostics.us_row_paint;
        self.paint_perf_frame.us_windowed_surface_non_row = diagnostics.us_non_row;
        self.paint_perf_frame.us_windowed_surface_row_callback_gap = diagnostics
            .us_row_paint
            .saturating_sub(self.paint_perf_frame.us_total);
        self.paint_perf_frame.ns_windowed_surface_paint_callback = diagnostics.ns_paint_callback;
        self.paint_perf_frame.ns_windowed_surface_frame_lookup = diagnostics.ns_frame_lookup;
        self.paint_perf_frame.ns_windowed_surface_hook = diagnostics.ns_on_paint_frame;
        self.paint_perf_frame.ns_windowed_surface_row_loop = diagnostics.ns_row_loop;
        self.paint_perf_frame.ns_windowed_surface_row_rect = diagnostics.ns_row_rect;
        self.paint_perf_frame.ns_windowed_surface_row_paint = diagnostics.ns_row_paint;
        self.paint_perf_frame.ns_windowed_surface_non_row = diagnostics.ns_non_row;
        self.paint_perf_frame.ns_windowed_surface_row_callback_gap = diagnostics
            .ns_row_paint
            .saturating_sub(self.paint_perf_frame.ns_total);
    }

    pub(super) fn record_torture_autoscroll_paint_elapsed(&mut self, started: Instant) {
        if !self.paint_perf_enabled {
            return;
        }

        let elapsed = started.elapsed();
        self.paint_perf_frame.us_torture_autoscroll =
            u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
        self.paint_perf_frame.ns_torture_autoscroll =
            u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
    }

    pub(super) fn record_torture_overlay_paint_elapsed(&mut self, started: Instant) {
        if !self.paint_perf_enabled {
            return;
        }

        let elapsed = started.elapsed();
        self.paint_perf_frame.us_torture_overlay =
            u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
        self.paint_perf_frame.ns_torture_overlay =
            u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
    }

    pub(super) fn set_preedit(&mut self, preedit: Option<PreeditState>) {
        let same = self.preedit == preedit;
        let mut cleared = false;
        if preedit.is_none() {
            if self.preedit_replace_range.take().is_some() {
                cleared = true;
            }
            if self.preedit_saved_selection.take().is_some() {
                cleared = true;
            }
        }
        if same && !cleared {
            return;
        }
        self.preedit = preedit;
        self.refresh_display_map();
        self.invalidate_row_caches();
    }

    pub(super) fn set_allow_decorations_under_inline_preedit(&mut self, allowed: bool) {
        if self.allow_decorations_under_inline_preedit == allowed {
            return;
        }
        self.allow_decorations_under_inline_preedit = allowed;
        self.refresh_display_map();
        self.invalidate_row_caches();
    }

    pub(super) fn set_compose_inline_preedit(&mut self, enabled: bool) {
        if self.compose_inline_preedit == enabled {
            return;
        }
        self.compose_inline_preedit = enabled;
        self.refresh_display_map();
        self.invalidate_row_caches();
    }

    pub(super) fn set_interaction(&mut self, interaction: CodeEditorInteractionOptions) {
        if self.interaction == interaction {
            return;
        }
        self.interaction = interaction;

        if !interaction.editable {
            self.undo_group = None;
            self.set_preedit(None);
        }

        if !interaction.enabled || !interaction.selectable {
            self.dragging = false;
            self.drag_pointer = None;
            self.drag_autoscroll_viewport_pos = None;
            // Keep any timer token so the next timer tick can self-cancel.
        }
    }

    pub(super) fn ime_surrounding_text_best_effort_cached(
        &mut self,
    ) -> fret_runtime::WindowImeSurroundingText {
        let revision = self.buffer.revision();
        let selection = self.selection;
        if let Some(cache) = self.ime_surrounding_text_cache.as_ref()
            && cache.revision == revision
            && cache.selection == selection
        {
            return cache.surrounding.clone();
        }

        let surrounding = best_effort_ime_surrounding_text(&self.buffer, selection);
        self.ime_surrounding_text_cache = Some(ImeSurroundingTextCache {
            revision,
            selection,
            surrounding: surrounding.clone(),
        });
        surrounding
    }
}
