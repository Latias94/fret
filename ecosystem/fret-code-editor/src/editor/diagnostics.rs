use std::sync::OnceLock;

use fret_code_editor_buffer::{Edit, TextBufferTx};

/// Lightweight counters for editor-local caches (bundle-friendly, no allocations).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CodeEditorCacheStats {
    pub row_text_get_calls: u64,
    pub row_text_hits: u64,
    pub row_text_misses: u64,
    pub row_text_evictions: u64,
    pub row_text_resets: u64,

    pub row_scene_get_calls: u64,
    pub row_scene_hits: u64,
    pub row_scene_misses: u64,
    pub row_scene_evictions: u64,
    pub row_scene_resets: u64,
    #[cfg(feature = "syntax")]
    pub row_scene_fast_get_calls: u64,
    #[cfg(feature = "syntax")]
    pub row_scene_fast_hits: u64,
    #[cfg(feature = "syntax")]
    pub row_scene_fast_misses: u64,

    #[cfg(feature = "syntax")]
    pub row_rich_get_calls: u64,
    #[cfg(feature = "syntax")]
    pub row_rich_hits: u64,
    #[cfg(feature = "syntax")]
    pub row_rich_misses: u64,
    #[cfg(feature = "syntax")]
    pub row_rich_evictions: u64,
    #[cfg(feature = "syntax")]
    pub row_rich_resets: u64,

    /// Number of pointer hit-tests that fell back to the monospace `cell_w` heuristic
    /// (caret stops unavailable).
    pub geom_pointer_hit_test_fallbacks: u64,
    /// Number of caret-rect queries that fell back to the monospace `cell_w` heuristic
    /// (caret stops unavailable).
    pub geom_caret_rect_fallbacks: u64,
    /// Number of vertical caret moves that fell back to the column-based display map
    /// (caret stops unavailable).
    pub geom_vertical_move_fallbacks: u64,

    pub syntax_get_calls: u64,
    pub syntax_hits: u64,
    pub syntax_misses: u64,
    pub syntax_evictions: u64,
    pub syntax_resets: u64,
}

/// Best-effort cache size estimates for diagnostics bundles.
///
/// These are intentionally approximate and are used to correlate editor-level cache growth with
/// process-level memory footprint signals (e.g. `vmmap` buckets on macOS).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CodeEditorCacheSizeSnapshotV1 {
    pub schema_version: u32,

    pub row_text_cache_entries: u64,
    pub row_text_cache_queue_len: u64,
    pub row_text_cache_text_bytes_estimate_total: u64,
    pub row_text_cache_row_spans_len_total: u64,

    pub row_geom_cache_entries: u64,
    pub row_geom_cache_queue_len: u64,
    pub row_geom_cache_caret_stops_len_total: u64,

    pub row_scene_cache_entries: u64,
    pub row_scene_cache_queue_len: u64,
    pub row_scene_cache_scene_ops_len_total: u64,

    pub syntax_row_cache_entries: u64,
    pub syntax_row_cache_queue_len: u64,
    pub syntax_row_cache_spans_len_total: u64,

    pub row_rich_cache_entries: u64,
    pub row_rich_cache_queue_len: u64,
    pub row_rich_cache_line_bytes_estimate_total: u64,
    pub row_rich_cache_row_spans_len_total: u64,
    pub row_rich_cache_syntax_spans_len_total: u64,
    pub row_rich_cache_rich_spans_len_total: u64,

    pub selection_rect_scratch_capacity: u64,
}

/// Best-effort memory attribution snapshot for editor-owned state.
///
/// This is intended to answer "what is the editor keeping alive?" rather than providing exact
/// allocator-level sizes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CodeEditorMemorySnapshotV1 {
    pub schema_version: u32,

    pub buffer_revision: u64,
    pub buffer_len_bytes: u64,
    pub buffer_line_count: u64,

    pub undo_limit: u64,
    pub undo_len: u64,
    pub redo_len: u64,

    /// Approximate UTF-8 bytes stored inside undo records (includes both forward and inverse
    /// `TextBufferTx` payloads).
    pub undo_text_bytes_estimate_total: u64,
    pub redo_text_bytes_estimate_total: u64,

    pub undo_edit_count_total: u64,
    pub redo_edit_count_total: u64,
}

fn estimate_edit_text_bytes(edit: &Edit) -> u64 {
    match edit {
        Edit::Insert { text, .. } | Edit::Replace { text, .. } => text.len() as u64,
        Edit::Delete { .. } => 0,
    }
}

pub(super) fn estimate_text_buffer_tx_text_bytes_and_edits(tx: &TextBufferTx) -> (u64, u64) {
    let mut bytes = 0u64;
    let mut edits = 0u64;

    for edit in tx.edits.iter().chain(tx.inverse_edits.iter()) {
        bytes = bytes.saturating_add(estimate_edit_text_bytes(edit));
        edits = edits.saturating_add(1);
    }

    (bytes, edits)
}

impl CodeEditorCacheStats {
    pub fn row_rich_get_calls(&self) -> u64 {
        #[cfg(feature = "syntax")]
        {
            self.row_rich_get_calls
        }
        #[cfg(not(feature = "syntax"))]
        {
            0
        }
    }

    pub fn row_rich_hits(&self) -> u64 {
        #[cfg(feature = "syntax")]
        {
            self.row_rich_hits
        }
        #[cfg(not(feature = "syntax"))]
        {
            0
        }
    }

    pub fn row_rich_misses(&self) -> u64 {
        #[cfg(feature = "syntax")]
        {
            self.row_rich_misses
        }
        #[cfg(not(feature = "syntax"))]
        {
            0
        }
    }

    pub fn row_rich_evictions(&self) -> u64 {
        #[cfg(feature = "syntax")]
        {
            self.row_rich_evictions
        }
        #[cfg(not(feature = "syntax"))]
        {
            0
        }
    }

    pub fn row_rich_resets(&self) -> u64 {
        #[cfg(feature = "syntax")]
        {
            self.row_rich_resets
        }
        #[cfg(not(feature = "syntax"))]
        {
            0
        }
    }

    pub fn row_scene_get_calls(&self) -> u64 {
        self.row_scene_get_calls
    }

    pub fn row_scene_hits(&self) -> u64 {
        self.row_scene_hits
    }

    pub fn row_scene_misses(&self) -> u64 {
        self.row_scene_misses
    }

    pub fn row_scene_evictions(&self) -> u64 {
        self.row_scene_evictions
    }

    pub fn row_scene_resets(&self) -> u64 {
        self.row_scene_resets
    }

    #[cfg(feature = "syntax")]
    pub fn row_scene_fast_get_calls(&self) -> u64 {
        self.row_scene_fast_get_calls
    }

    #[cfg(not(feature = "syntax"))]
    pub fn row_scene_fast_get_calls(&self) -> u64 {
        0
    }

    #[cfg(feature = "syntax")]
    pub fn row_scene_fast_hits(&self) -> u64 {
        self.row_scene_fast_hits
    }

    #[cfg(not(feature = "syntax"))]
    pub fn row_scene_fast_hits(&self) -> u64 {
        0
    }

    #[cfg(feature = "syntax")]
    pub fn row_scene_fast_misses(&self) -> u64 {
        self.row_scene_fast_misses
    }

    #[cfg(not(feature = "syntax"))]
    pub fn row_scene_fast_misses(&self) -> u64 {
        0
    }
}

/// Frame-local timing counters for the code editor's Canvas paint path.
///
/// This is diagnostics-only and intended for perf triage (not for strict perf gates).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CodeEditorPaintPerfFrame {
    pub frame_seq: u64,
    pub visible_start: u64,
    pub visible_end: u64,
    pub visible_rows: u64,
    pub cache_base_entries: u64,
    pub cache_frame_min_entries: u64,
    pub cache_effective_entries: u64,

    pub rows_painted: u64,
    pub rows_drew_rich: u64,
    pub rows_scene_replayed: u64,
    pub rows_scene_prepaint_planned: u64,
    pub rows_scene_prepaint_plan_used: u64,
    pub rows_scene_stored: u64,
    pub rows_scene_stored_at_visible_start: u64,
    pub rows_scene_stored_at_visible_end: u64,
    pub row_scene_ops_stored: u64,
    pub rows_scene_prepaint_edge_stored: u64,
    pub row_scene_prepaint_edge_ops_stored: u64,
    pub rows_scene_prepaint_candidates: u64,
    pub rows_scene_prepaint_skip_no_cache: u64,
    pub rows_scene_prepaint_skip_unsupported_key: u64,
    pub rows_scene_prepaint_skip_preedit: u64,
    pub rows_scene_prepaint_skip_syntax_empty: u64,
    pub rows_scene_prepaint_skip_key_mismatch: u64,
    pub rows_scene_prepaint_plan_cache_hits: u64,
    pub rows_scene_prepaint_plan_cache_rejects: u64,
    pub rows_scene_fast_miss_no_entry: u64,
    pub rows_scene_fast_miss_key_mismatch: u64,
    pub rows_scene_full_miss_no_entry: u64,
    pub rows_scene_full_miss_key_mismatch: u64,
    pub quads_selection: u64,
    pub quads_caret: u64,

    pub us_total: u64,
    pub us_row_text: u64,
    pub us_baseline_measure: u64,
    pub us_syntax_spans: u64,
    pub us_rich_materialize: u64,
    pub us_text_draw: u64,
    pub us_row_rich_cache_compare: u64,
    pub us_row_geom_key: u64,
    pub us_row_scene_key: u64,
    pub us_row_scene_fast_probe: u64,
    pub us_row_scene_full_probe: u64,
    pub us_row_scene_fast_key_compare: u64,
    pub us_row_scene_full_key_compare: u64,
    pub us_row_scene_replay_touch: u64,
    pub us_row_scene_replay_ops: u64,
    pub us_row_scene_prepaint_plan: u64,
    pub us_row_scene_prepaint_probe: u64,
    pub us_row_scene_prepaint_key_compare: u64,
    pub us_row_scene_capture_ops: u64,
    pub us_row_scene_store: u64,
    pub us_row_scene_prepaint_edge_store: u64,
    pub us_row_scene_fast_path: u64,
    pub us_row_scene_full_path: u64,
    pub syntax_rows_stored: u64,
    pub us_syntax_slice: u64,
    pub us_syntax_highlight: u64,
    pub us_syntax_distribute: u64,
    pub us_syntax_store: u64,
    pub us_selection_rects: u64,
    pub us_caret_x: u64,
    pub us_caret_stops: u64,
    pub us_caret_rect: u64,
    pub us_row_geom_cache: u64,
    pub us_row_content_resolve: u64,
    pub us_row_geom_resolve: u64,
    pub us_row_overlay: u64,
    pub us_frame_overlay_prepare: u64,
    pub surface_rows_iterated: u64,
    pub surface_rows_with_rect: u64,
    pub us_windowed_surface_paint_callback: u64,
    pub us_windowed_surface_frame_lookup: u64,
    pub us_windowed_surface_hook: u64,
    pub us_windowed_surface_row_loop: u64,
    pub us_windowed_surface_row_rect: u64,
    pub us_windowed_surface_row_paint: u64,
    pub us_windowed_surface_non_row: u64,
    pub us_windowed_surface_row_callback_gap: u64,
    pub us_torture_autoscroll: u64,
    pub us_torture_overlay: u64,

    pub ns_total: u64,
    pub ns_row_text: u64,
    pub ns_baseline_measure: u64,
    pub ns_syntax_spans: u64,
    pub ns_rich_materialize: u64,
    pub ns_text_draw: u64,
    pub ns_row_rich_cache_compare: u64,
    pub ns_row_geom_key: u64,
    pub ns_row_scene_key: u64,
    pub ns_row_scene_fast_probe: u64,
    pub ns_row_scene_full_probe: u64,
    pub ns_row_scene_fast_key_compare: u64,
    pub ns_row_scene_full_key_compare: u64,
    pub ns_row_scene_replay_touch: u64,
    pub ns_row_scene_replay_ops: u64,
    pub ns_row_scene_prepaint_plan: u64,
    pub ns_row_scene_prepaint_probe: u64,
    pub ns_row_scene_prepaint_key_compare: u64,
    pub ns_row_scene_capture_ops: u64,
    pub ns_row_scene_store: u64,
    pub ns_row_scene_prepaint_edge_store: u64,
    pub ns_row_scene_fast_path: u64,
    pub ns_row_scene_full_path: u64,
    pub ns_syntax_slice: u64,
    pub ns_syntax_highlight: u64,
    pub ns_syntax_distribute: u64,
    pub ns_syntax_store: u64,
    pub ns_selection_rects: u64,
    pub ns_caret_x: u64,
    pub ns_caret_stops: u64,
    pub ns_caret_rect: u64,
    pub ns_row_geom_cache: u64,
    pub ns_row_content_resolve: u64,
    pub ns_row_geom_resolve: u64,
    pub ns_row_overlay: u64,
    pub ns_frame_overlay_prepare: u64,
    pub ns_windowed_surface_paint_callback: u64,
    pub ns_windowed_surface_frame_lookup: u64,
    pub ns_windowed_surface_hook: u64,
    pub ns_windowed_surface_row_loop: u64,
    pub ns_windowed_surface_row_rect: u64,
    pub ns_windowed_surface_row_paint: u64,
    pub ns_windowed_surface_non_row: u64,
    pub ns_windowed_surface_row_callback_gap: u64,
    pub ns_torture_autoscroll: u64,
    pub ns_torture_overlay: u64,
}

pub(super) fn paint_perf_enabled_from_env() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("FRET_CODE_EDITOR_DIAG_PAINT_PERF")
            .is_some_and(|v| !v.is_empty() && v != "0")
    })
}

pub(super) fn normalized_paint_frame_visible_window(
    visible_start: usize,
    visible_end: usize,
) -> Option<(usize, usize)> {
    (visible_start <= visible_end).then_some((visible_start, visible_end))
}

pub(super) fn paint_frame_visible_row_count(visible_start: usize, visible_end: usize) -> usize {
    visible_end.saturating_sub(visible_start).saturating_add(1)
}

fn paint_frame_interval_union_count(a: (usize, usize), b: (usize, usize)) -> usize {
    let a_count = paint_frame_visible_row_count(a.0, a.1);
    let b_count = paint_frame_visible_row_count(b.0, b.1);
    if a.1 < b.0 || b.1 < a.0 {
        return a_count.saturating_add(b_count);
    }

    a.1.max(b.1).saturating_sub(a.0.min(b.0)).saturating_add(1)
}

pub(super) fn paint_frame_cache_min_entries(
    previous: Option<(usize, usize)>,
    current: Option<(usize, usize)>,
) -> usize {
    let Some(current) = current else {
        return 0;
    };
    let entries = previous
        .map(|previous| paint_frame_interval_union_count(previous, current))
        .unwrap_or_else(|| paint_frame_visible_row_count(current.0, current.1));
    entries.min(super::CODE_EDITOR_ROW_CACHE_MAX_ENTRIES)
}
