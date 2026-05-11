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
    pub(super) row_text_cache: HashMap<usize, (RowTextCacheEntry, u64)>,
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
pub(super) struct RowTextCacheEntry {
    pub(super) text: Arc<str>,
    pub(super) range: Range<usize>,
    pub(super) fold_map: Option<geom::RowFoldMap>,
    pub(super) preedit_range: Option<Range<usize>>,
    pub(super) row_spans: Arc<[fret_code_editor_view::DisplayRowSpan]>,
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
pub(super) struct RowSceneCacheEntry {
    pub(super) key: RowSceneKey,
    pub(super) origin: Point,
    pub(super) geom: geom::RowGeom,
    pub(super) is_rich: bool,
    pub(super) ops: Vec<SceneOp>,
    pub(super) hosted_resources: fret_ui::canvas::CanvasHostedResources,
    #[cfg(feature = "syntax")]
    pub(super) syntax_replay_key: Option<RowSceneSyntaxReplayKey>,
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
