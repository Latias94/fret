use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::util::{
    now_unix_ms, read_pick_result, read_pick_result_run_id, read_script_result,
    read_script_result_run_id, touch, write_json_value, write_script,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum BundleStatsSort {
    #[default]
    Invalidation,
    Time,
    Dispatch,
    HitTest,
    RendererEncodeScene,
    RendererPrepareText,
    RendererDrawCalls,
    RendererPipelineSwitches,
    RendererBindGroupSwitches,
    RendererTextAtlasUploadBytes,
    RendererTextAtlasEvictedPages,
    RendererSvgUploadBytes,
    RendererImageUploadBytes,
    RendererSvgRasterCacheMisses,
    RendererSvgRasterBudgetEvictions,
    RendererIntermediateBudgetBytes,
    RendererIntermediateInUseBytes,
    RendererIntermediatePeakInUseBytes,
    RendererIntermediateReleaseTargets,
    RendererIntermediatePoolAllocations,
    RendererIntermediatePoolReuses,
    RendererIntermediatePoolReleases,
    RendererIntermediatePoolEvictions,
    RendererIntermediatePoolFreeBytes,
    RendererIntermediatePoolFreeTextures,
}

impl BundleStatsSort {
    pub(super) fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "invalidation" => Ok(Self::Invalidation),
            "time" => Ok(Self::Time),
            "dispatch" => Ok(Self::Dispatch),
            "hit_test" => Ok(Self::HitTest),
            "encode_scene" | "encode" | "renderer_encode_scene" => Ok(Self::RendererEncodeScene),
            "prepare_text" | "renderer_prepare_text" => Ok(Self::RendererPrepareText),
            "draw_calls" | "draws" | "renderer_draw_calls" => Ok(Self::RendererDrawCalls),
            "pipeline_switches" | "pipelines" | "renderer_pipeline_switches" => {
                Ok(Self::RendererPipelineSwitches)
            }
            "bind_group_switches" | "binds" | "renderer_bind_group_switches" => {
                Ok(Self::RendererBindGroupSwitches)
            }
            "atlas_upload_bytes"
            | "text_atlas_upload_bytes"
            | "renderer_text_atlas_upload_bytes" => Ok(Self::RendererTextAtlasUploadBytes),
            "atlas_evicted_pages"
            | "text_atlas_evicted_pages"
            | "renderer_text_atlas_evicted_pages" => Ok(Self::RendererTextAtlasEvictedPages),
            "svg_upload_bytes" | "renderer_svg_upload_bytes" => Ok(Self::RendererSvgUploadBytes),
            "image_upload_bytes" | "renderer_image_upload_bytes" => {
                Ok(Self::RendererImageUploadBytes)
            }
            "svg_cache_misses" | "svg_raster_cache_misses" | "renderer_svg_raster_cache_misses" => {
                Ok(Self::RendererSvgRasterCacheMisses)
            }
            "svg_evictions"
            | "svg_raster_budget_evictions"
            | "renderer_svg_raster_budget_evictions" => Ok(Self::RendererSvgRasterBudgetEvictions),
            "intermediate_budget_bytes"
            | "intermediate_budget"
            | "renderer_intermediate_budget_bytes" => Ok(Self::RendererIntermediateBudgetBytes),
            "intermediate_in_use_bytes"
            | "intermediate_in_use"
            | "renderer_intermediate_in_use_bytes" => Ok(Self::RendererIntermediateInUseBytes),
            "intermediate_peak_bytes"
            | "intermediate_peak"
            | "renderer_intermediate_peak_in_use_bytes" => {
                Ok(Self::RendererIntermediatePeakInUseBytes)
            }
            "intermediate_release_targets" | "renderer_intermediate_release_targets" => {
                Ok(Self::RendererIntermediateReleaseTargets)
            }
            "intermediate_allocations"
            | "intermediate_pool_allocations"
            | "renderer_intermediate_pool_allocations" => {
                Ok(Self::RendererIntermediatePoolAllocations)
            }
            "intermediate_reuses"
            | "intermediate_pool_reuses"
            | "renderer_intermediate_pool_reuses" => Ok(Self::RendererIntermediatePoolReuses),
            "intermediate_releases"
            | "intermediate_pool_releases"
            | "renderer_intermediate_pool_releases" => Ok(Self::RendererIntermediatePoolReleases),
            "pool_evictions"
            | "intermediate_pool_evictions"
            | "renderer_intermediate_pool_evictions" => Ok(Self::RendererIntermediatePoolEvictions),
            "intermediate_free_bytes"
            | "intermediate_pool_free_bytes"
            | "renderer_intermediate_pool_free_bytes" => {
                Ok(Self::RendererIntermediatePoolFreeBytes)
            }
            "intermediate_free_textures"
            | "intermediate_pool_free_textures"
            | "renderer_intermediate_pool_free_textures" => {
                Ok(Self::RendererIntermediatePoolFreeTextures)
            }
            other => Err(format!(
                "invalid --sort value: {other} (expected: invalidation|time|dispatch|hit_test|encode_scene|prepare_text|draw_calls|pipeline_switches|bind_group_switches|atlas_upload_bytes|atlas_evicted_pages|svg_upload_bytes|image_upload_bytes|svg_cache_misses|svg_evictions|intermediate_budget_bytes|intermediate_in_use_bytes|intermediate_peak_bytes|intermediate_release_targets|intermediate_allocations|intermediate_reuses|intermediate_releases|pool_evictions|intermediate_free_bytes|intermediate_free_textures)"
            )),
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Invalidation => "invalidation",
            Self::Time => "time",
            Self::Dispatch => "dispatch",
            Self::HitTest => "hit_test",
            Self::RendererEncodeScene => "encode_scene",
            Self::RendererPrepareText => "prepare_text",
            Self::RendererDrawCalls => "draw_calls",
            Self::RendererPipelineSwitches => "pipeline_switches",
            Self::RendererBindGroupSwitches => "bind_group_switches",
            Self::RendererTextAtlasUploadBytes => "atlas_upload_bytes",
            Self::RendererTextAtlasEvictedPages => "atlas_evicted_pages",
            Self::RendererSvgUploadBytes => "svg_upload_bytes",
            Self::RendererImageUploadBytes => "image_upload_bytes",
            Self::RendererSvgRasterCacheMisses => "svg_cache_misses",
            Self::RendererSvgRasterBudgetEvictions => "svg_evictions",
            Self::RendererIntermediateBudgetBytes => "intermediate_budget_bytes",
            Self::RendererIntermediateInUseBytes => "intermediate_in_use_bytes",
            Self::RendererIntermediatePeakInUseBytes => "intermediate_peak_bytes",
            Self::RendererIntermediateReleaseTargets => "intermediate_release_targets",
            Self::RendererIntermediatePoolAllocations => "intermediate_allocations",
            Self::RendererIntermediatePoolReuses => "intermediate_reuses",
            Self::RendererIntermediatePoolReleases => "intermediate_releases",
            Self::RendererIntermediatePoolEvictions => "pool_evictions",
            Self::RendererIntermediatePoolFreeBytes => "intermediate_free_bytes",
            Self::RendererIntermediatePoolFreeTextures => "intermediate_free_textures",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsReport {
    sort: BundleStatsSort,
    warmup_frames: u64,
    pub(super) windows: u32,
    pub(super) snapshots: u32,
    snapshots_considered: u32,
    snapshots_skipped_warmup: u32,
    pub(super) snapshots_with_model_changes: u32,
    pub(super) snapshots_with_global_changes: u32,
    snapshots_with_propagated_model_changes: u32,
    snapshots_with_propagated_global_changes: u32,
    pub(super) snapshots_with_hover_layout_invalidations: u32,
    /// Whether the bundle includes `pointer.move` events (so the derived "pointer move" frame set
    /// can be identified from the event log rather than inferred from dispatch-only frames).
    pub(super) pointer_move_frames_present: bool,
    /// Count of snapshots in the derived "pointer move" (or fallback) frame set.
    pub(super) pointer_move_frames_considered: u32,
    /// Max dispatch time (us) across the derived "pointer move" (or fallback) frame set.
    pub(super) pointer_move_max_dispatch_time_us: u64,
    /// Snapshot identity for `pointer_move_max_dispatch_time_us`.
    pub(super) pointer_move_max_dispatch_window: u64,
    pub(super) pointer_move_max_dispatch_tick_id: u64,
    pub(super) pointer_move_max_dispatch_frame_id: u64,
    /// Max hit-test time (us) across the derived "pointer move" (or fallback) frame set.
    pub(super) pointer_move_max_hit_test_time_us: u64,
    /// Snapshot identity for `pointer_move_max_hit_test_time_us`.
    pub(super) pointer_move_max_hit_test_window: u64,
    pub(super) pointer_move_max_hit_test_tick_id: u64,
    pub(super) pointer_move_max_hit_test_frame_id: u64,
    /// Number of snapshots within the derived "pointer move" (or fallback) frame set that had
    /// propagated global changes (`debug.stats.global_change_globals > 0`).
    pub(super) pointer_move_snapshots_with_global_changes: u32,
    sum_layout_time_us: u64,
    sum_prepaint_time_us: u64,
    sum_paint_time_us: u64,
    sum_total_time_us: u64,
    sum_cache_roots: u64,
    sum_cache_roots_reused: u64,
    sum_cache_replayed_ops: u64,
    pub(super) sum_invalidation_walk_calls: u64,
    pub(super) sum_invalidation_walk_nodes: u64,
    sum_model_change_invalidation_roots: u64,
    sum_global_change_invalidation_roots: u64,
    pub(super) sum_hover_layout_invalidations: u64,
    max_layout_time_us: u64,
    max_prepaint_time_us: u64,
    max_paint_time_us: u64,
    max_total_time_us: u64,
    pub(super) max_invalidation_walk_calls: u32,
    pub(super) max_invalidation_walk_nodes: u32,
    max_model_change_invalidation_roots: u32,
    max_global_change_invalidation_roots: u32,
    pub(super) max_hover_layout_invalidations: u32,
    worst_hover_layout: Option<BundleStatsWorstHoverLayout>,
    global_type_hotspots: Vec<BundleStatsGlobalTypeHotspot>,
    model_source_hotspots: Vec<BundleStatsModelSourceHotspot>,
    pub(super) top: Vec<BundleStatsSnapshotRow>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsSnapshotRow {
    pub(super) window: u64,
    pub(super) tick_id: u64,
    pub(super) frame_id: u64,
    pub(super) timestamp_unix_ms: Option<u64>,
    pub(super) frame_arena_capacity_estimate_bytes: u64,
    pub(super) frame_arena_grow_events: u32,
    pub(super) element_children_vec_pool_reuses: u32,
    pub(super) element_children_vec_pool_misses: u32,
    pub(super) layout_time_us: u64,
    pub(super) layout_collect_roots_time_us: u64,
    pub(super) layout_invalidate_scroll_handle_bindings_time_us: u64,
    pub(super) layout_expand_view_cache_invalidations_time_us: u64,
    pub(super) layout_request_build_roots_time_us: u64,
    pub(super) layout_pending_barrier_relayouts_time_us: u64,
    pub(super) layout_repair_view_cache_bounds_time_us: u64,
    pub(super) layout_contained_view_cache_roots_time_us: u64,
    pub(super) layout_collapse_layout_observations_time_us: u64,
    pub(super) layout_prepaint_after_layout_time_us: u64,
    pub(super) layout_skipped_engine_frame: bool,
    pub(super) prepaint_time_us: u64,
    pub(super) paint_time_us: u64,
    pub(super) paint_record_visual_bounds_time_us: u64,
    pub(super) paint_record_visual_bounds_calls: u32,
    pub(super) paint_cache_key_time_us: u64,
    pub(super) paint_cache_hit_check_time_us: u64,
    pub(super) paint_widget_time_us: u64,
    pub(super) paint_observation_record_time_us: u64,
    pub(super) paint_host_widget_observed_models_time_us: u64,
    pub(super) paint_host_widget_observed_models_items: u32,
    pub(super) paint_host_widget_observed_globals_time_us: u64,
    pub(super) paint_host_widget_observed_globals_items: u32,
    pub(super) paint_host_widget_instance_lookup_time_us: u64,
    pub(super) paint_host_widget_instance_lookup_calls: u32,
    pub(super) paint_text_prepare_time_us: u64,
    pub(super) paint_text_prepare_calls: u32,
    pub(super) paint_text_prepare_reason_blob_missing: u32,
    pub(super) paint_text_prepare_reason_scale_changed: u32,
    pub(super) paint_text_prepare_reason_text_changed: u32,
    pub(super) paint_text_prepare_reason_rich_changed: u32,
    pub(super) paint_text_prepare_reason_style_changed: u32,
    pub(super) paint_text_prepare_reason_wrap_changed: u32,
    pub(super) paint_text_prepare_reason_overflow_changed: u32,
    pub(super) paint_text_prepare_reason_width_changed: u32,
    pub(super) paint_text_prepare_reason_font_stack_changed: u32,
    pub(super) paint_input_context_time_us: u64,
    pub(super) paint_scroll_handle_invalidation_time_us: u64,
    pub(super) paint_collect_roots_time_us: u64,
    pub(super) paint_publish_text_input_snapshot_time_us: u64,
    pub(super) paint_collapse_observations_time_us: u64,
    pub(super) dispatch_time_us: u64,
    pub(super) dispatch_pointer_events: u32,
    pub(super) dispatch_pointer_event_time_us: u64,
    pub(super) dispatch_timer_events: u32,
    pub(super) dispatch_timer_event_time_us: u64,
    pub(super) dispatch_timer_targeted_events: u32,
    pub(super) dispatch_timer_targeted_time_us: u64,
    pub(super) dispatch_timer_broadcast_events: u32,
    pub(super) dispatch_timer_broadcast_time_us: u64,
    pub(super) dispatch_timer_broadcast_layers_visited: u32,
    pub(super) dispatch_timer_broadcast_rebuild_visible_layers_time_us: u64,
    pub(super) dispatch_timer_broadcast_loop_time_us: u64,
    pub(super) dispatch_timer_slowest_event_time_us: u64,
    pub(super) dispatch_timer_slowest_token: Option<u64>,
    pub(super) dispatch_timer_slowest_was_broadcast: bool,
    pub(super) dispatch_other_events: u32,
    pub(super) dispatch_other_event_time_us: u64,
    pub(super) hit_test_time_us: u64,
    pub(super) dispatch_hover_update_time_us: u64,
    pub(super) dispatch_scroll_handle_invalidation_time_us: u64,
    pub(super) dispatch_active_layers_time_us: u64,
    pub(super) dispatch_input_context_time_us: u64,
    pub(super) dispatch_event_chain_build_time_us: u64,
    pub(super) dispatch_widget_capture_time_us: u64,
    pub(super) dispatch_widget_bubble_time_us: u64,
    pub(super) dispatch_cursor_query_time_us: u64,
    pub(super) dispatch_pointer_move_layer_observers_time_us: u64,
    pub(super) dispatch_synth_hover_observer_time_us: u64,
    pub(super) dispatch_cursor_effect_time_us: u64,
    pub(super) dispatch_post_dispatch_snapshot_time_us: u64,
    pub(super) dispatch_events: u32,
    pub(super) hit_test_queries: u32,
    pub(super) hit_test_bounds_tree_queries: u32,
    pub(super) hit_test_bounds_tree_disabled: u32,
    pub(super) hit_test_bounds_tree_misses: u32,
    pub(super) hit_test_bounds_tree_hits: u32,
    pub(super) hit_test_bounds_tree_candidate_rejected: u32,
    pub(super) hit_test_cached_path_time_us: u64,
    pub(super) hit_test_bounds_tree_query_time_us: u64,
    pub(super) hit_test_candidate_self_only_time_us: u64,
    pub(super) hit_test_fallback_traversal_time_us: u64,
    pub(super) total_time_us: u64,
    pub(super) layout_nodes_performed: u32,
    pub(super) paint_nodes_performed: u32,
    pub(super) paint_cache_misses: u32,
    pub(super) paint_cache_replay_time_us: u64,
    pub(super) paint_cache_bounds_translate_time_us: u64,
    pub(super) paint_cache_bounds_translated_nodes: u32,
    pub(super) renderer_tick_id: u64,
    pub(super) renderer_frame_id: u64,
    pub(super) renderer_encode_scene_us: u64,
    pub(super) renderer_prepare_text_us: u64,
    pub(super) renderer_prepare_svg_us: u64,
    pub(super) renderer_svg_upload_bytes: u64,
    pub(super) renderer_image_upload_bytes: u64,
    pub(super) renderer_svg_raster_budget_bytes: u64,
    pub(super) renderer_svg_rasters_live: u64,
    pub(super) renderer_svg_standalone_bytes_live: u64,
    pub(super) renderer_svg_mask_atlas_pages_live: u64,
    pub(super) renderer_svg_mask_atlas_bytes_live: u64,
    pub(super) renderer_svg_mask_atlas_used_px: u64,
    pub(super) renderer_svg_mask_atlas_capacity_px: u64,
    pub(super) renderer_svg_raster_cache_hits: u64,
    pub(super) renderer_svg_raster_cache_misses: u64,
    pub(super) renderer_svg_raster_budget_evictions: u64,
    pub(super) renderer_svg_mask_atlas_page_evictions: u64,
    pub(super) renderer_svg_mask_atlas_entries_evicted: u64,
    pub(super) renderer_text_atlas_upload_bytes: u64,
    pub(super) renderer_text_atlas_evicted_pages: u64,
    pub(super) renderer_intermediate_budget_bytes: u64,
    pub(super) renderer_intermediate_in_use_bytes: u64,
    pub(super) renderer_intermediate_peak_in_use_bytes: u64,
    pub(super) renderer_intermediate_release_targets: u64,
    pub(super) renderer_intermediate_pool_allocations: u64,
    pub(super) renderer_intermediate_pool_reuses: u64,
    pub(super) renderer_intermediate_pool_releases: u64,
    pub(super) renderer_intermediate_pool_evictions: u64,
    pub(super) renderer_intermediate_pool_free_bytes: u64,
    pub(super) renderer_intermediate_pool_free_textures: u64,
    pub(super) renderer_draw_calls: u64,
    pub(super) renderer_pipeline_switches: u64,
    pub(super) renderer_bind_group_switches: u64,
    pub(super) renderer_scissor_sets: u64,
    pub(super) renderer_scene_encoding_cache_misses: u64,
    pub(super) layout_engine_solves: u64,
    pub(super) layout_engine_solve_time_us: u64,
    pub(super) changed_models: u32,
    pub(super) changed_globals: u32,
    pub(super) changed_global_types_sample: Vec<String>,
    pub(super) propagated_model_change_models: u32,
    pub(super) propagated_model_change_observation_edges: u32,
    pub(super) propagated_model_change_unobserved_models: u32,
    pub(super) propagated_global_change_globals: u32,
    pub(super) propagated_global_change_observation_edges: u32,
    pub(super) propagated_global_change_unobserved_globals: u32,
    pub(super) invalidation_walk_calls: u32,
    pub(super) invalidation_walk_nodes: u32,
    pub(super) model_change_invalidation_roots: u32,
    pub(super) global_change_invalidation_roots: u32,
    pub(super) invalidation_walk_calls_model_change: u32,
    pub(super) invalidation_walk_nodes_model_change: u32,
    pub(super) invalidation_walk_calls_global_change: u32,
    pub(super) invalidation_walk_nodes_global_change: u32,
    pub(super) invalidation_walk_calls_hover: u32,
    pub(super) invalidation_walk_nodes_hover: u32,
    pub(super) invalidation_walk_calls_focus: u32,
    pub(super) invalidation_walk_nodes_focus: u32,
    pub(super) invalidation_walk_calls_other: u32,
    pub(super) invalidation_walk_nodes_other: u32,
    pub(super) top_invalidation_walks: Vec<BundleStatsInvalidationWalk>,
    pub(super) hover_pressable_target_changes: u32,
    pub(super) hover_hover_region_target_changes: u32,
    pub(super) hover_declarative_instance_changes: u32,
    pub(super) hover_declarative_hit_test_invalidations: u32,
    pub(super) hover_declarative_layout_invalidations: u32,
    pub(super) hover_declarative_paint_invalidations: u32,
    pub(super) top_hover_declarative_invalidations:
        Vec<BundleStatsHoverDeclarativeInvalidationHotspot>,
    pub(super) cache_roots: u32,
    pub(super) cache_roots_reused: u32,
    pub(super) cache_roots_contained_relayout: u32,
    pub(super) cache_replayed_ops: u64,
    pub(super) view_cache_contained_relayouts: u32,
    pub(super) view_cache_roots_total: u32,
    pub(super) view_cache_roots_reused: u32,
    pub(super) view_cache_roots_first_mount: u32,
    pub(super) view_cache_roots_node_recreated: u32,
    pub(super) view_cache_roots_cache_key_mismatch: u32,
    pub(super) view_cache_roots_not_marked_reuse_root: u32,
    pub(super) view_cache_roots_needs_rerender: u32,
    pub(super) view_cache_roots_layout_invalidated: u32,
    pub(super) view_cache_roots_manual: u32,
    pub(super) set_children_barrier_writes: u32,
    pub(super) barrier_relayouts_scheduled: u32,
    pub(super) barrier_relayouts_performed: u32,
    pub(super) virtual_list_visible_range_checks: u32,
    pub(super) virtual_list_visible_range_refreshes: u32,
    pub(super) top_cache_roots: Vec<BundleStatsCacheRoot>,
    pub(super) top_contained_relayout_cache_roots: Vec<BundleStatsCacheRoot>,
    pub(super) top_layout_engine_solves: Vec<BundleStatsLayoutEngineSolve>,
    pub(super) paint_widget_hotspots: Vec<BundleStatsPaintWidgetHotspot>,
    pub(super) paint_text_prepare_hotspots: Vec<BundleStatsPaintTextPrepareHotspot>,
    pub(super) model_change_hotspots: Vec<BundleStatsModelChangeHotspot>,
    pub(super) model_change_unobserved: Vec<BundleStatsModelChangeUnobserved>,
    pub(super) global_change_hotspots: Vec<BundleStatsGlobalChangeHotspot>,
    pub(super) global_change_unobserved: Vec<BundleStatsGlobalChangeUnobserved>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsPaintWidgetHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) widget_type: Option<String>,
    pub(super) paint_time_us: u64,
    pub(super) inclusive_time_us: u64,
    pub(super) inclusive_scene_ops_delta: u32,
    pub(super) exclusive_scene_ops_delta: u32,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsPaintTextPrepareHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) prepare_time_us: u64,
    pub(super) text_len: u32,
    pub(super) max_width: Option<f32>,
    pub(super) wrap: Option<String>,
    pub(super) overflow: Option<String>,
    pub(super) scale_factor: Option<f32>,
    pub(super) reasons_mask: u16,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsHoverDeclarativeInvalidationHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) hit_test: u32,
    pub(super) layout: u32,
    pub(super) paint: u32,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsWorstHoverLayout {
    window: u64,
    tick_id: u64,
    frame_id: u64,
    hover_declarative_layout_invalidations: u32,
    hotspots: Vec<BundleStatsHoverDeclarativeInvalidationHotspot>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsInvalidationWalk {
    pub(super) root_node: u64,
    pub(super) root_element: Option<u64>,
    pub(super) kind: Option<String>,
    pub(super) source: Option<String>,
    pub(super) detail: Option<String>,
    pub(super) walked_nodes: u32,
    pub(super) truncated_at: Option<u64>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsCacheRoot {
    pub(super) root_node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_path: Option<String>,
    pub(super) reused: bool,
    pub(super) contained_layout: bool,
    pub(super) contained_relayout_in_frame: bool,
    pub(super) paint_replayed_ops: u32,
    pub(super) reuse_reason: Option<String>,
    pub(super) root_in_semantics: Option<bool>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineSolve {
    pub(super) root_node: u64,
    pub(super) solve_time_us: u64,
    pub(super) measure_calls: u64,
    pub(super) measure_cache_hits: u64,
    pub(super) measure_time_us: u64,
    pub(super) top_measures: Vec<BundleStatsLayoutEngineMeasureHotspot>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineMeasureHotspot {
    pub(super) node: u64,
    pub(super) measure_time_us: u64,
    pub(super) calls: u64,
    pub(super) cache_hits: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) top_children: Vec<BundleStatsLayoutEngineMeasureChildHotspot>,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineMeasureChildHotspot {
    pub(super) child: u64,
    pub(super) measure_time_us: u64,
    pub(super) calls: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsModelChangeHotspot {
    model: u64,
    observation_edges: u32,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsModelChangeUnobserved {
    model: u64,
    created_type: Option<String>,
    created_at: Option<String>,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsGlobalChangeHotspot {
    type_name: String,
    observation_edges: u32,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsGlobalChangeUnobserved {
    type_name: String,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsGlobalTypeHotspot {
    type_name: String,
    count: u64,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsModelSourceHotspot {
    source: String,
    count: u64,
}

impl BundleStatsReport {
    pub(super) fn print_human(&self, bundle_path: &Path) {
        println!("bundle: {}", bundle_path.display());
        println!(
            "windows={} snapshots={} considered={} warmup_skipped={} model_changes={} global_changes={} propagated_model_changes={} propagated_global_changes={}",
            self.windows,
            self.snapshots,
            self.snapshots_considered,
            self.snapshots_skipped_warmup,
            self.snapshots_with_model_changes,
            self.snapshots_with_global_changes,
            self.snapshots_with_propagated_model_changes,
            self.snapshots_with_propagated_global_changes
        );
        if self.warmup_frames > 0 {
            println!("warmup_frames={}", self.warmup_frames);
        }
        println!("sort={}", self.sort.as_str());
        println!(
            "time sum (us): total={} layout={} prepaint={} paint={}",
            self.sum_total_time_us,
            self.sum_layout_time_us,
            self.sum_prepaint_time_us,
            self.sum_paint_time_us
        );
        println!(
            "time max (us): total={} layout={} prepaint={} paint={}",
            self.max_total_time_us,
            self.max_layout_time_us,
            self.max_prepaint_time_us,
            self.max_paint_time_us
        );
        println!(
            "cache roots sum: roots={} reused={} replayed_ops={}",
            self.sum_cache_roots, self.sum_cache_roots_reused, self.sum_cache_replayed_ops
        );
        println!(
            "invalidation sum: calls={} nodes={}",
            self.sum_invalidation_walk_calls, self.sum_invalidation_walk_nodes
        );
        println!(
            "invalidation max: calls={} nodes={}",
            self.max_invalidation_walk_calls, self.max_invalidation_walk_nodes
        );
        println!(
            "roots sum: model={} global={}",
            self.sum_model_change_invalidation_roots, self.sum_global_change_invalidation_roots
        );
        println!(
            "roots max: model={} global={}",
            self.max_model_change_invalidation_roots, self.max_global_change_invalidation_roots
        );
        if self.sum_hover_layout_invalidations > 0 || self.max_hover_layout_invalidations > 0 {
            println!(
                "hover decl layout invalidations: sum={} max_per_frame={} frames_with_hover_layout={}",
                self.sum_hover_layout_invalidations,
                self.max_hover_layout_invalidations,
                self.snapshots_with_hover_layout_invalidations
            );
        }

        if !self.global_type_hotspots.is_empty() {
            let items: Vec<String> = self
                .global_type_hotspots
                .iter()
                .map(|h| format!("{}={}", h.type_name, h.count))
                .collect();
            println!("changed_globals_top: {}", items.join(" | "));
        }
        if !self.model_source_hotspots.is_empty() {
            let items: Vec<String> = self
                .model_source_hotspots
                .iter()
                .map(|h| format!("{}={}", h.source, h.count))
                .collect();
            println!("changed_models_top: {}", items.join(" | "));
        }

        if self.pointer_move_frames_present || self.pointer_move_frames_considered > 0 {
            let mode = if self.pointer_move_frames_present {
                "pointer_move"
            } else {
                "dispatch_frames_fallback"
            };
            println!(
                "derived({mode}) frames_considered={} max.us(dispatch/hit_test)={}/{} dispatch_at=window:{}/tick:{}/frame:{} hit_test_at=window:{}/tick:{}/frame:{} snapshots_with_global_changes={}",
                self.pointer_move_frames_considered,
                self.pointer_move_max_dispatch_time_us,
                self.pointer_move_max_hit_test_time_us,
                self.pointer_move_max_dispatch_window,
                self.pointer_move_max_dispatch_tick_id,
                self.pointer_move_max_dispatch_frame_id,
                self.pointer_move_max_hit_test_window,
                self.pointer_move_max_hit_test_tick_id,
                self.pointer_move_max_hit_test_frame_id,
                self.pointer_move_snapshots_with_global_changes
            );
        }

        if self.top.is_empty() {
            return;
        }

        println!("top (sort={}):", self.sort.as_str());
        for row in &self.top {
            let ts = row
                .timestamp_unix_ms
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string());
            println!(
                "  window={} tick={} frame={} ts={} time.us(total/layout/prepaint/paint)={}/{}/{}/{} layout.solve_us={} paint.cache_misses={} layout.nodes={} paint.nodes={} paint.elem_bounds_us={} paint.elem_bounds_calls={} cache_roots={} cache.reused={} cache.replayed_ops={} cache.replay_us={} cache.translate_us={} cache.translate_nodes={} contained_relayouts={} cache.contained_relayout_roots={} barrier(set_children/scheduled/performed)={}/{}/{} vlist(range_checks/refreshes)={}/{} inv.calls={} inv.nodes={} by_src.calls(hover/focus/other)={}/{}/{} by_src.nodes(hover/focus/other)={}/{}/{} hover.decl_inv(layout/hit/paint)={}/{}/{} roots.model={} roots.global={} changed.models={} changed.globals={} propagated.models={} propagated.edges={} unobs.models={} propagated.globals={} propagated.global_edges={} unobs.globals={}",
                row.window,
                row.tick_id,
                row.frame_id,
                ts,
                row.total_time_us,
                row.layout_time_us,
                row.prepaint_time_us,
                row.paint_time_us,
                row.layout_engine_solve_time_us,
                row.paint_cache_misses,
                row.layout_nodes_performed,
                row.paint_nodes_performed,
                row.paint_record_visual_bounds_time_us,
                row.paint_record_visual_bounds_calls,
                row.cache_roots,
                row.cache_roots_reused,
                row.cache_replayed_ops,
                row.paint_cache_replay_time_us,
                row.paint_cache_bounds_translate_time_us,
                row.paint_cache_bounds_translated_nodes,
                row.view_cache_contained_relayouts,
                row.cache_roots_contained_relayout,
                row.set_children_barrier_writes,
                row.barrier_relayouts_scheduled,
                row.barrier_relayouts_performed,
                row.virtual_list_visible_range_checks,
                row.virtual_list_visible_range_refreshes,
                row.invalidation_walk_calls,
                row.invalidation_walk_nodes,
                row.invalidation_walk_calls_hover,
                row.invalidation_walk_calls_focus,
                row.invalidation_walk_calls_other,
                row.invalidation_walk_nodes_hover,
                row.invalidation_walk_nodes_focus,
                row.invalidation_walk_nodes_other,
                row.hover_declarative_layout_invalidations,
                row.hover_declarative_hit_test_invalidations,
                row.hover_declarative_paint_invalidations,
                row.model_change_invalidation_roots,
                row.global_change_invalidation_roots,
                row.changed_models,
                row.changed_globals,
                row.propagated_model_change_models,
                row.propagated_model_change_observation_edges,
                row.propagated_model_change_unobserved_models,
                row.propagated_global_change_globals,
                row.propagated_global_change_observation_edges,
                row.propagated_global_change_unobserved_globals
            );
            if row.paint_input_context_time_us > 0
                || row.paint_scroll_handle_invalidation_time_us > 0
                || row.paint_collect_roots_time_us > 0
                || row.paint_publish_text_input_snapshot_time_us > 0
                || row.paint_collapse_observations_time_us > 0
            {
                println!(
                    "    paint_breakdown.us(input_ctx/scroll_inv/collect_roots/text_snapshot/collapse)={}/{}/{}/{}/{}",
                    row.paint_input_context_time_us,
                    row.paint_scroll_handle_invalidation_time_us,
                    row.paint_collect_roots_time_us,
                    row.paint_publish_text_input_snapshot_time_us,
                    row.paint_collapse_observations_time_us
                );
            }
            if row.paint_cache_key_time_us > 0
                || row.paint_cache_hit_check_time_us > 0
                || row.paint_widget_time_us > 0
                || row.paint_observation_record_time_us > 0
            {
                println!(
                    "    paint_node.us(cache_key/hit_check/widget/obs_record)={}/{}/{}/{}",
                    row.paint_cache_key_time_us,
                    row.paint_cache_hit_check_time_us,
                    row.paint_widget_time_us,
                    row.paint_observation_record_time_us
                );
            }
            if row.paint_host_widget_observed_models_time_us > 0
                || row.paint_host_widget_observed_globals_time_us > 0
                || row.paint_host_widget_instance_lookup_time_us > 0
            {
                println!(
                    "    paint_host_widget.us(models/globals/instance)={}/{}/{} items={}/{} calls={}",
                    row.paint_host_widget_observed_models_time_us,
                    row.paint_host_widget_observed_globals_time_us,
                    row.paint_host_widget_instance_lookup_time_us,
                    row.paint_host_widget_observed_models_items,
                    row.paint_host_widget_observed_globals_items,
                    row.paint_host_widget_instance_lookup_calls,
                );
            }
            if row.paint_text_prepare_time_us > 0 || row.paint_text_prepare_calls > 0 {
                println!(
                    "    paint_text_prepare.us(time/calls)={}/{}",
                    row.paint_text_prepare_time_us, row.paint_text_prepare_calls
                );
                let reasons = [
                    row.paint_text_prepare_reason_blob_missing,
                    row.paint_text_prepare_reason_scale_changed,
                    row.paint_text_prepare_reason_text_changed,
                    row.paint_text_prepare_reason_rich_changed,
                    row.paint_text_prepare_reason_style_changed,
                    row.paint_text_prepare_reason_wrap_changed,
                    row.paint_text_prepare_reason_overflow_changed,
                    row.paint_text_prepare_reason_width_changed,
                    row.paint_text_prepare_reason_font_stack_changed,
                ];
                if reasons.iter().any(|&v| v > 0) {
                    println!(
                        "    paint_text_prepare.reasons(blob/scale/text/rich/style/wrap/overflow/width/font)={}/{}/{}/{}/{}/{}/{}/{}/{}",
                        row.paint_text_prepare_reason_blob_missing,
                        row.paint_text_prepare_reason_scale_changed,
                        row.paint_text_prepare_reason_text_changed,
                        row.paint_text_prepare_reason_rich_changed,
                        row.paint_text_prepare_reason_style_changed,
                        row.paint_text_prepare_reason_wrap_changed,
                        row.paint_text_prepare_reason_overflow_changed,
                        row.paint_text_prepare_reason_width_changed,
                        row.paint_text_prepare_reason_font_stack_changed,
                    );
                }
            }
            if !row.paint_text_prepare_hotspots.is_empty() {
                let items: Vec<String> = row
                    .paint_text_prepare_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!(
                            "us={} node={} kind={} len={} max_width={} wrap={} overflow={} reasons={}",
                            h.prepare_time_us,
                            h.node,
                            h.element_kind.as_deref().unwrap_or("?"),
                            h.text_len,
                            h.max_width
                                .map(|v| format!("{:.1}", v))
                                .unwrap_or_else(|| "?".to_string()),
                            h.wrap.as_deref().unwrap_or("?"),
                            h.overflow.as_deref().unwrap_or("?"),
                            format_text_prepare_reasons(h.reasons_mask),
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = h.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        s
                    })
                    .collect();
                println!("    paint_text_prepare_hotspots: {}", items.join(" | "));
            }
            if !row.paint_widget_hotspots.is_empty() {
                let items: Vec<String> = row
                    .paint_widget_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!(
                            "us={} ops={}/{} node={} kind={} type={}",
                            h.paint_time_us,
                            h.exclusive_scene_ops_delta,
                            h.inclusive_scene_ops_delta,
                            h.node,
                            h.element_kind.as_deref().unwrap_or("?"),
                            h.widget_type.as_deref().unwrap_or("?"),
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = h.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        s
                    })
                    .collect();
                println!("    paint_widget_hotspots: {}", items.join(" | "));
            }
            if !row.top_invalidation_walks.is_empty() {
                let items: Vec<String> = row
                    .top_invalidation_walks
                    .iter()
                    .take(3)
                    .map(|w| {
                        let mut s = format!(
                            "nodes={} src={} kind={} root={}",
                            w.walked_nodes,
                            w.source.as_deref().unwrap_or("?"),
                            w.kind.as_deref().unwrap_or("?"),
                            w.root_node
                        );
                        if let Some(detail) = w.detail.as_deref()
                            && !detail.is_empty()
                        {
                            s.push_str(&format!(" detail={detail}"));
                        }
                        if let Some(test_id) = w.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={}", test_id));
                        }
                        if let Some(role) = w.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={}", role));
                        }
                        if let Some(el) = w.root_element {
                            s.push_str(&format!(" element={}", el));
                        }
                        if let Some(trunc) = w.truncated_at {
                            s.push_str(&format!(" trunc_at={}", trunc));
                        }
                        s
                    })
                    .collect();
                println!("    top_walks: {}", items.join(" | "));
            }
            if !row.top_cache_roots.is_empty() {
                let items: Vec<String> = row
                    .top_cache_roots
                    .iter()
                    .take(3)
                    .map(|c| {
                        let mut s = format!(
                            "ops={} reused={} root={} reason={}",
                            c.paint_replayed_ops,
                            c.reused,
                            c.root_node,
                            c.reuse_reason.as_deref().unwrap_or("?")
                        );
                        if let Some(test_id) = c.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = c.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = c.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = c.element_path.as_deref()
                            && !path.is_empty()
                        {
                            s.push_str(&format!(" path={path}"));
                        }
                        if let Some(in_sem) = c.root_in_semantics {
                            s.push_str(&format!(" root_in_semantics={in_sem}"));
                        }
                        s
                    })
                    .collect();
                println!("    top_cache_roots: {}", items.join(" | "));
            }
            if !row.top_contained_relayout_cache_roots.is_empty() {
                let items: Vec<String> = row
                    .top_contained_relayout_cache_roots
                    .iter()
                    .take(3)
                    .map(|c| {
                        let mut s = format!(
                            "ops={} reused={} root={} reason={}",
                            c.paint_replayed_ops,
                            c.reused,
                            c.root_node,
                            c.reuse_reason.as_deref().unwrap_or("?")
                        );
                        if let Some(test_id) = c.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = c.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = c.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = c.element_path.as_deref()
                            && !path.is_empty()
                        {
                            s.push_str(&format!(" path={path}"));
                        }
                        if let Some(in_sem) = c.root_in_semantics {
                            s.push_str(&format!(" root_in_semantics={in_sem}"));
                        }
                        s
                    })
                    .collect();
                println!(
                    "    top_contained_relayout_cache_roots: {}",
                    items.join(" | ")
                );
            }
            if row.hover_declarative_layout_invalidations > 0
                && !row.top_hover_declarative_invalidations.is_empty()
            {
                let items: Vec<String> = row
                    .top_hover_declarative_invalidations
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!(
                            "layout={} hit={} paint={} node={}",
                            h.layout, h.hit_test, h.paint, h.node
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = h.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        s
                    })
                    .collect();
                println!("    hover_layout_hotspots: {}", items.join(" | "));
            }
            if !row.top_layout_engine_solves.is_empty() {
                let items: Vec<String> = row
                    .top_layout_engine_solves
                    .iter()
                    .take(3)
                    .map(|s| {
                        let mut out = format!(
                            "us={} measure.us={} measure.calls={} hits={} root={}",
                            s.solve_time_us,
                            s.measure_time_us,
                            s.measure_calls,
                            s.measure_cache_hits,
                            s.root_node
                        );
                        if let Some(test_id) = s.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = s.root_role.as_deref()
                            && !role.is_empty()
                        {
                            out.push_str(&format!(" role={role}"));
                        }
                        if let Some(m) = s.top_measures.first() {
                            if m.measure_time_us > 0 && m.node != 0 {
                                out.push_str(&format!(
                                    " top_measure.us={} node={}",
                                    m.measure_time_us, m.node
                                ));
                                if let Some(kind) = m.element_kind.as_deref()
                                    && !kind.is_empty()
                                {
                                    out.push_str(&format!(" kind={kind}"));
                                }
                                if let Some(el) = m.element {
                                    out.push_str(&format!(" element={el}"));
                                }
                                if let Some(test_id) = m.test_id.as_deref()
                                    && !test_id.is_empty()
                                {
                                    out.push_str(&format!(" test_id={test_id}"));
                                }
                                if let Some(role) = m.role.as_deref()
                                    && !role.is_empty()
                                {
                                    out.push_str(&format!(" role={role}"));
                                }
                                if let Some(c) = m.top_children.first() {
                                    if c.measure_time_us > 0 && c.child != 0 {
                                        out.push_str(&format!(
                                            " child.us={} child={}",
                                            c.measure_time_us, c.child
                                        ));
                                        if let Some(kind) = c.element_kind.as_deref()
                                            && !kind.is_empty()
                                        {
                                            out.push_str(&format!(" child.kind={kind}"));
                                        }
                                        if let Some(el) = c.element {
                                            out.push_str(&format!(" child.element={el}"));
                                        }
                                        if let Some(test_id) = c.test_id.as_deref()
                                            && !test_id.is_empty()
                                        {
                                            out.push_str(&format!(" child.test_id={test_id}"));
                                        }
                                        if let Some(role) = c.role.as_deref()
                                            && !role.is_empty()
                                        {
                                            out.push_str(&format!(" child.role={role}"));
                                        }
                                    }
                                }
                            }
                        }
                        out
                    })
                    .collect();
                println!("    top_layout_engine_solves: {}", items.join(" | "));
            }
            if !row.model_change_hotspots.is_empty() {
                let items: Vec<String> = row
                    .model_change_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!("{}={}", h.model, h.observation_edges);
                        if let Some(at) = h.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    hot_models: {}", items.join(" | "));
            }
            if !row.model_change_unobserved.is_empty() {
                let items: Vec<String> = row
                    .model_change_unobserved
                    .iter()
                    .take(3)
                    .map(|u| {
                        let mut s = format!("{}", u.model);
                        if let Some(ty) = u.created_type.as_deref() {
                            s.push_str(&format!("={}", ty));
                        }
                        if let Some(at) = u.created_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        if let Some(at) = u.changed_at.as_deref() {
                            s.push_str(&format!(" changed@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    unobs_models: {}", items.join(" | "));
            }
            if !row.global_change_hotspots.is_empty() {
                let items: Vec<String> = row
                    .global_change_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!("{}={}", h.type_name, h.observation_edges);
                        if let Some(at) = h.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    hot_globals: {}", items.join(" | "));
            }
            if !row.global_change_unobserved.is_empty() {
                let items: Vec<String> = row
                    .global_change_unobserved
                    .iter()
                    .take(3)
                    .map(|u| {
                        let mut s = u.type_name.clone();
                        if let Some(at) = u.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    unobs_globals: {}", items.join(" | "));
            }
            if !row.changed_global_types_sample.is_empty() {
                println!(
                    "    changed_globals: {}",
                    row.changed_global_types_sample.join(" | ")
                );
            }
        }
    }

    pub(super) fn to_json(&self) -> serde_json::Value {
        use serde_json::{Map, Value};

        let mut root = Map::new();
        root.insert("schema_version".to_string(), Value::from(1));
        root.insert("sort".to_string(), Value::from(self.sort.as_str()));
        root.insert("warmup_frames".to_string(), Value::from(self.warmup_frames));
        root.insert("windows".to_string(), Value::from(self.windows));
        root.insert("snapshots".to_string(), Value::from(self.snapshots));
        root.insert(
            "snapshots_considered".to_string(),
            Value::from(self.snapshots_considered),
        );
        root.insert(
            "snapshots_skipped_warmup".to_string(),
            Value::from(self.snapshots_skipped_warmup),
        );
        root.insert(
            "snapshots_with_model_changes".to_string(),
            Value::from(self.snapshots_with_model_changes),
        );
        root.insert(
            "snapshots_with_global_changes".to_string(),
            Value::from(self.snapshots_with_global_changes),
        );
        root.insert(
            "snapshots_with_propagated_model_changes".to_string(),
            Value::from(self.snapshots_with_propagated_model_changes),
        );
        root.insert(
            "snapshots_with_propagated_global_changes".to_string(),
            Value::from(self.snapshots_with_propagated_global_changes),
        );
        root.insert(
            "snapshots_with_hover_layout_invalidations".to_string(),
            Value::from(self.snapshots_with_hover_layout_invalidations),
        );

        root.insert(
            "pointer_move".to_string(),
            serde_json::json!({
                "frames_present": self.pointer_move_frames_present,
                "frames_considered": self.pointer_move_frames_considered,
                "max_dispatch_time_us": self.pointer_move_max_dispatch_time_us,
                "max_dispatch_at": {
                    "window": self.pointer_move_max_dispatch_window,
                    "tick_id": self.pointer_move_max_dispatch_tick_id,
                    "frame_id": self.pointer_move_max_dispatch_frame_id,
                },
                "max_hit_test_time_us": self.pointer_move_max_hit_test_time_us,
                "max_hit_test_at": {
                    "window": self.pointer_move_max_hit_test_window,
                    "tick_id": self.pointer_move_max_hit_test_tick_id,
                    "frame_id": self.pointer_move_max_hit_test_frame_id,
                },
                "snapshots_with_global_changes": self.pointer_move_snapshots_with_global_changes,
            }),
        );

        let mut sum = Map::new();
        sum.insert(
            "layout_time_us".to_string(),
            Value::from(self.sum_layout_time_us),
        );
        sum.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.sum_prepaint_time_us),
        );
        sum.insert(
            "paint_time_us".to_string(),
            Value::from(self.sum_paint_time_us),
        );
        sum.insert(
            "total_time_us".to_string(),
            Value::from(self.sum_total_time_us),
        );
        sum.insert("cache_roots".to_string(), Value::from(self.sum_cache_roots));
        sum.insert(
            "cache_roots_reused".to_string(),
            Value::from(self.sum_cache_roots_reused),
        );
        sum.insert(
            "cache_replayed_ops".to_string(),
            Value::from(self.sum_cache_replayed_ops),
        );
        sum.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(self.sum_invalidation_walk_calls),
        );
        sum.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(self.sum_invalidation_walk_nodes),
        );
        sum.insert(
            "model_change_invalidation_roots".to_string(),
            Value::from(self.sum_model_change_invalidation_roots),
        );
        sum.insert(
            "global_change_invalidation_roots".to_string(),
            Value::from(self.sum_global_change_invalidation_roots),
        );
        sum.insert(
            "hover_layout_invalidations".to_string(),
            Value::from(self.sum_hover_layout_invalidations),
        );
        root.insert("sum".to_string(), Value::Object(sum));

        let mut max = Map::new();
        max.insert(
            "layout_time_us".to_string(),
            Value::from(self.max_layout_time_us),
        );
        max.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.max_prepaint_time_us),
        );
        max.insert(
            "paint_time_us".to_string(),
            Value::from(self.max_paint_time_us),
        );
        max.insert(
            "total_time_us".to_string(),
            Value::from(self.max_total_time_us),
        );
        max.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(self.max_invalidation_walk_calls),
        );
        max.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(self.max_invalidation_walk_nodes),
        );
        max.insert(
            "model_change_invalidation_roots".to_string(),
            Value::from(self.max_model_change_invalidation_roots),
        );
        max.insert(
            "global_change_invalidation_roots".to_string(),
            Value::from(self.max_global_change_invalidation_roots),
        );
        max.insert(
            "hover_layout_invalidations".to_string(),
            Value::from(self.max_hover_layout_invalidations),
        );
        root.insert("max".to_string(), Value::Object(max));

        let global_type_hotspots = self
            .global_type_hotspots
            .iter()
            .map(|h| {
                let mut obj = Map::new();
                obj.insert("type_name".to_string(), Value::from(h.type_name.clone()));
                obj.insert("count".to_string(), Value::from(h.count));
                Value::Object(obj)
            })
            .collect::<Vec<_>>();
        root.insert(
            "global_type_hotspots".to_string(),
            Value::Array(global_type_hotspots),
        );
        let model_source_hotspots = self
            .model_source_hotspots
            .iter()
            .map(|h| {
                let mut obj = Map::new();
                obj.insert("source".to_string(), Value::from(h.source.clone()));
                obj.insert("count".to_string(), Value::from(h.count));
                Value::Object(obj)
            })
            .collect::<Vec<_>>();
        root.insert(
            "model_source_hotspots".to_string(),
            Value::Array(model_source_hotspots),
        );

        let top = self
            .top
            .iter()
            .map(|row| {
                let mut obj = Map::new();
                obj.insert("window".to_string(), Value::from(row.window));
                obj.insert("tick_id".to_string(), Value::from(row.tick_id));
                obj.insert("frame_id".to_string(), Value::from(row.frame_id));
                obj.insert(
                    "timestamp_unix_ms".to_string(),
                    row.timestamp_unix_ms
                        .map(Value::from)
                        .unwrap_or(Value::Null),
                );
                obj.insert(
                    "layout_time_us".to_string(),
                    Value::from(row.layout_time_us),
                );
                obj.insert(
                    "prepaint_time_us".to_string(),
                    Value::from(row.prepaint_time_us),
                );
                obj.insert("paint_time_us".to_string(), Value::from(row.paint_time_us));
                obj.insert(
                    "dispatch_time_us".to_string(),
                    Value::from(row.dispatch_time_us),
                );
                obj.insert(
                    "dispatch_pointer_events".to_string(),
                    Value::from(row.dispatch_pointer_events),
                );
                obj.insert(
                    "dispatch_pointer_event_time_us".to_string(),
                    Value::from(row.dispatch_pointer_event_time_us),
                );
                obj.insert(
                    "dispatch_timer_events".to_string(),
                    Value::from(row.dispatch_timer_events),
                );
                obj.insert(
                    "dispatch_timer_event_time_us".to_string(),
                    Value::from(row.dispatch_timer_event_time_us),
                );
                obj.insert(
                    "dispatch_timer_targeted_events".to_string(),
                    Value::from(row.dispatch_timer_targeted_events),
                );
                obj.insert(
                    "dispatch_timer_targeted_time_us".to_string(),
                    Value::from(row.dispatch_timer_targeted_time_us),
                );
                obj.insert(
                    "dispatch_timer_broadcast_events".to_string(),
                    Value::from(row.dispatch_timer_broadcast_events),
                );
                obj.insert(
                    "dispatch_timer_broadcast_time_us".to_string(),
                    Value::from(row.dispatch_timer_broadcast_time_us),
                );
                obj.insert(
                    "dispatch_timer_broadcast_layers_visited".to_string(),
                    Value::from(row.dispatch_timer_broadcast_layers_visited),
                );
                obj.insert(
                    "dispatch_timer_broadcast_rebuild_visible_layers_time_us".to_string(),
                    Value::from(row.dispatch_timer_broadcast_rebuild_visible_layers_time_us),
                );
                obj.insert(
                    "dispatch_timer_broadcast_loop_time_us".to_string(),
                    Value::from(row.dispatch_timer_broadcast_loop_time_us),
                );
                obj.insert(
                    "dispatch_timer_slowest_event_time_us".to_string(),
                    Value::from(row.dispatch_timer_slowest_event_time_us),
                );
                obj.insert(
                    "dispatch_timer_slowest_token".to_string(),
                    row.dispatch_timer_slowest_token
                        .map(Value::from)
                        .unwrap_or(Value::Null),
                );
                obj.insert(
                    "dispatch_timer_slowest_was_broadcast".to_string(),
                    Value::from(row.dispatch_timer_slowest_was_broadcast),
                );
                obj.insert(
                    "dispatch_other_events".to_string(),
                    Value::from(row.dispatch_other_events),
                );
                obj.insert(
                    "dispatch_other_event_time_us".to_string(),
                    Value::from(row.dispatch_other_event_time_us),
                );
                obj.insert(
                    "hit_test_time_us".to_string(),
                    Value::from(row.hit_test_time_us),
                );
                obj.insert(
                    "dispatch_hover_update_time_us".to_string(),
                    Value::from(row.dispatch_hover_update_time_us),
                );
                obj.insert(
                    "dispatch_scroll_handle_invalidation_time_us".to_string(),
                    Value::from(row.dispatch_scroll_handle_invalidation_time_us),
                );
                obj.insert(
                    "dispatch_active_layers_time_us".to_string(),
                    Value::from(row.dispatch_active_layers_time_us),
                );
                obj.insert(
                    "dispatch_input_context_time_us".to_string(),
                    Value::from(row.dispatch_input_context_time_us),
                );
                obj.insert(
                    "dispatch_event_chain_build_time_us".to_string(),
                    Value::from(row.dispatch_event_chain_build_time_us),
                );
                obj.insert(
                    "dispatch_widget_capture_time_us".to_string(),
                    Value::from(row.dispatch_widget_capture_time_us),
                );
                obj.insert(
                    "dispatch_widget_bubble_time_us".to_string(),
                    Value::from(row.dispatch_widget_bubble_time_us),
                );
                obj.insert(
                    "dispatch_cursor_query_time_us".to_string(),
                    Value::from(row.dispatch_cursor_query_time_us),
                );
                obj.insert(
                    "dispatch_pointer_move_layer_observers_time_us".to_string(),
                    Value::from(row.dispatch_pointer_move_layer_observers_time_us),
                );
                obj.insert(
                    "dispatch_synth_hover_observer_time_us".to_string(),
                    Value::from(row.dispatch_synth_hover_observer_time_us),
                );
                obj.insert(
                    "dispatch_cursor_effect_time_us".to_string(),
                    Value::from(row.dispatch_cursor_effect_time_us),
                );
                obj.insert(
                    "dispatch_post_dispatch_snapshot_time_us".to_string(),
                    Value::from(row.dispatch_post_dispatch_snapshot_time_us),
                );
                obj.insert(
                    "dispatch_events".to_string(),
                    Value::from(row.dispatch_events),
                );
                obj.insert(
                    "hit_test_queries".to_string(),
                    Value::from(row.hit_test_queries),
                );
                obj.insert(
                    "hit_test_bounds_tree_queries".to_string(),
                    Value::from(row.hit_test_bounds_tree_queries),
                );
                obj.insert(
                    "hit_test_bounds_tree_disabled".to_string(),
                    Value::from(row.hit_test_bounds_tree_disabled),
                );
                obj.insert(
                    "hit_test_bounds_tree_misses".to_string(),
                    Value::from(row.hit_test_bounds_tree_misses),
                );
                obj.insert(
                    "hit_test_bounds_tree_hits".to_string(),
                    Value::from(row.hit_test_bounds_tree_hits),
                );
                obj.insert(
                    "hit_test_bounds_tree_candidate_rejected".to_string(),
                    Value::from(row.hit_test_bounds_tree_candidate_rejected),
                );
                obj.insert(
                    "hit_test_cached_path_time_us".to_string(),
                    Value::from(row.hit_test_cached_path_time_us),
                );
                obj.insert(
                    "hit_test_bounds_tree_query_time_us".to_string(),
                    Value::from(row.hit_test_bounds_tree_query_time_us),
                );
                obj.insert(
                    "hit_test_candidate_self_only_time_us".to_string(),
                    Value::from(row.hit_test_candidate_self_only_time_us),
                );
                obj.insert(
                    "hit_test_fallback_traversal_time_us".to_string(),
                    Value::from(row.hit_test_fallback_traversal_time_us),
                );
                obj.insert("total_time_us".to_string(), Value::from(row.total_time_us));
                obj.insert(
                    "layout_nodes_performed".to_string(),
                    Value::from(row.layout_nodes_performed),
                );
                obj.insert(
                    "paint_nodes_performed".to_string(),
                    Value::from(row.paint_nodes_performed),
                );
                obj.insert(
                    "paint_cache_misses".to_string(),
                    Value::from(row.paint_cache_misses),
                );
                obj.insert(
                    "layout_engine_solves".to_string(),
                    Value::from(row.layout_engine_solves),
                );
                obj.insert(
                    "layout_engine_solve_time_us".to_string(),
                    Value::from(row.layout_engine_solve_time_us),
                );
                obj.insert(
                    "layout_collect_roots_time_us".to_string(),
                    Value::from(row.layout_collect_roots_time_us),
                );
                obj.insert(
                    "layout_invalidate_scroll_handle_bindings_time_us".to_string(),
                    Value::from(row.layout_invalidate_scroll_handle_bindings_time_us),
                );
                obj.insert(
                    "layout_expand_view_cache_invalidations_time_us".to_string(),
                    Value::from(row.layout_expand_view_cache_invalidations_time_us),
                );
                obj.insert(
                    "layout_request_build_roots_time_us".to_string(),
                    Value::from(row.layout_request_build_roots_time_us),
                );
                obj.insert(
                    "layout_pending_barrier_relayouts_time_us".to_string(),
                    Value::from(row.layout_pending_barrier_relayouts_time_us),
                );
                obj.insert(
                    "layout_repair_view_cache_bounds_time_us".to_string(),
                    Value::from(row.layout_repair_view_cache_bounds_time_us),
                );
                obj.insert(
                    "layout_contained_view_cache_roots_time_us".to_string(),
                    Value::from(row.layout_contained_view_cache_roots_time_us),
                );
                obj.insert(
                    "layout_collapse_layout_observations_time_us".to_string(),
                    Value::from(row.layout_collapse_layout_observations_time_us),
                );
                obj.insert(
                    "layout_prepaint_after_layout_time_us".to_string(),
                    Value::from(row.layout_prepaint_after_layout_time_us),
                );
                obj.insert(
                    "layout_skipped_engine_frame".to_string(),
                    Value::from(row.layout_skipped_engine_frame),
                );
                obj.insert("cache_roots".to_string(), Value::from(row.cache_roots));
                obj.insert(
                    "cache_roots_reused".to_string(),
                    Value::from(row.cache_roots_reused),
                );
                obj.insert(
                    "cache_roots_contained_relayout".to_string(),
                    Value::from(row.cache_roots_contained_relayout),
                );
                obj.insert(
                    "cache_replayed_ops".to_string(),
                    Value::from(row.cache_replayed_ops),
                );
                obj.insert(
                    "paint_record_visual_bounds_time_us".to_string(),
                    Value::from(row.paint_record_visual_bounds_time_us),
                );
                obj.insert(
                    "paint_record_visual_bounds_calls".to_string(),
                    Value::from(row.paint_record_visual_bounds_calls),
                );
                obj.insert(
                    "paint_cache_key_time_us".to_string(),
                    Value::from(row.paint_cache_key_time_us),
                );
                obj.insert(
                    "paint_cache_hit_check_time_us".to_string(),
                    Value::from(row.paint_cache_hit_check_time_us),
                );
                obj.insert(
                    "paint_widget_time_us".to_string(),
                    Value::from(row.paint_widget_time_us),
                );
                obj.insert(
                    "paint_observation_record_time_us".to_string(),
                    Value::from(row.paint_observation_record_time_us),
                );
                obj.insert(
                    "paint_host_widget_observed_models_time_us".to_string(),
                    Value::from(row.paint_host_widget_observed_models_time_us),
                );
                obj.insert(
                    "paint_host_widget_observed_models_items".to_string(),
                    Value::from(row.paint_host_widget_observed_models_items),
                );
                obj.insert(
                    "paint_host_widget_observed_globals_time_us".to_string(),
                    Value::from(row.paint_host_widget_observed_globals_time_us),
                );
                obj.insert(
                    "paint_host_widget_observed_globals_items".to_string(),
                    Value::from(row.paint_host_widget_observed_globals_items),
                );
                obj.insert(
                    "paint_host_widget_instance_lookup_time_us".to_string(),
                    Value::from(row.paint_host_widget_instance_lookup_time_us),
                );
                obj.insert(
                    "paint_host_widget_instance_lookup_calls".to_string(),
                    Value::from(row.paint_host_widget_instance_lookup_calls),
                );
                obj.insert(
                    "paint_text_prepare_time_us".to_string(),
                    Value::from(row.paint_text_prepare_time_us),
                );
                obj.insert(
                    "paint_text_prepare_calls".to_string(),
                    Value::from(row.paint_text_prepare_calls),
                );
                obj.insert(
                    "paint_text_prepare_reason_blob_missing".to_string(),
                    Value::from(row.paint_text_prepare_reason_blob_missing),
                );
                obj.insert(
                    "paint_text_prepare_reason_scale_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_scale_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_text_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_text_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_rich_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_rich_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_style_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_style_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_wrap_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_wrap_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_overflow_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_overflow_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_width_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_width_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_font_stack_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_font_stack_changed),
                );
                obj.insert(
                    "paint_input_context_time_us".to_string(),
                    Value::from(row.paint_input_context_time_us),
                );
                obj.insert(
                    "paint_scroll_handle_invalidation_time_us".to_string(),
                    Value::from(row.paint_scroll_handle_invalidation_time_us),
                );
                obj.insert(
                    "paint_collect_roots_time_us".to_string(),
                    Value::from(row.paint_collect_roots_time_us),
                );
                obj.insert(
                    "paint_publish_text_input_snapshot_time_us".to_string(),
                    Value::from(row.paint_publish_text_input_snapshot_time_us),
                );
                obj.insert(
                    "paint_collapse_observations_time_us".to_string(),
                    Value::from(row.paint_collapse_observations_time_us),
                );
                obj.insert(
                    "paint_cache_replay_time_us".to_string(),
                    Value::from(row.paint_cache_replay_time_us),
                );
                obj.insert(
                    "paint_cache_bounds_translate_time_us".to_string(),
                    Value::from(row.paint_cache_bounds_translate_time_us),
                );
                obj.insert(
                    "paint_cache_bounds_translated_nodes".to_string(),
                    Value::from(row.paint_cache_bounds_translated_nodes),
                );
                obj.insert(
                    "changed_models".to_string(),
                    Value::from(row.changed_models),
                );
                obj.insert(
                    "changed_globals".to_string(),
                    Value::from(row.changed_globals),
                );
                obj.insert(
                    "changed_global_types_sample".to_string(),
                    Value::Array(
                        row.changed_global_types_sample
                            .iter()
                            .cloned()
                            .map(Value::from)
                            .collect(),
                    ),
                );
                obj.insert(
                    "propagated_model_change_models".to_string(),
                    Value::from(row.propagated_model_change_models),
                );
                obj.insert(
                    "propagated_model_change_observation_edges".to_string(),
                    Value::from(row.propagated_model_change_observation_edges),
                );
                obj.insert(
                    "propagated_model_change_unobserved_models".to_string(),
                    Value::from(row.propagated_model_change_unobserved_models),
                );
                obj.insert(
                    "propagated_global_change_globals".to_string(),
                    Value::from(row.propagated_global_change_globals),
                );
                obj.insert(
                    "propagated_global_change_observation_edges".to_string(),
                    Value::from(row.propagated_global_change_observation_edges),
                );
                obj.insert(
                    "propagated_global_change_unobserved_globals".to_string(),
                    Value::from(row.propagated_global_change_unobserved_globals),
                );
                obj.insert(
                    "invalidation_walk_calls".to_string(),
                    Value::from(row.invalidation_walk_calls),
                );
                obj.insert(
                    "invalidation_walk_nodes".to_string(),
                    Value::from(row.invalidation_walk_nodes),
                );
                obj.insert(
                    "model_change_invalidation_roots".to_string(),
                    Value::from(row.model_change_invalidation_roots),
                );
                obj.insert(
                    "global_change_invalidation_roots".to_string(),
                    Value::from(row.global_change_invalidation_roots),
                );
                obj.insert(
                    "invalidation_walk_calls_model_change".to_string(),
                    Value::from(row.invalidation_walk_calls_model_change),
                );
                obj.insert(
                    "invalidation_walk_nodes_model_change".to_string(),
                    Value::from(row.invalidation_walk_nodes_model_change),
                );
                obj.insert(
                    "invalidation_walk_calls_global_change".to_string(),
                    Value::from(row.invalidation_walk_calls_global_change),
                );
                obj.insert(
                    "invalidation_walk_nodes_global_change".to_string(),
                    Value::from(row.invalidation_walk_nodes_global_change),
                );
                obj.insert(
                    "invalidation_walk_calls_hover".to_string(),
                    Value::from(row.invalidation_walk_calls_hover),
                );
                obj.insert(
                    "invalidation_walk_nodes_hover".to_string(),
                    Value::from(row.invalidation_walk_nodes_hover),
                );
                obj.insert(
                    "invalidation_walk_calls_focus".to_string(),
                    Value::from(row.invalidation_walk_calls_focus),
                );
                obj.insert(
                    "invalidation_walk_nodes_focus".to_string(),
                    Value::from(row.invalidation_walk_nodes_focus),
                );
                obj.insert(
                    "invalidation_walk_calls_other".to_string(),
                    Value::from(row.invalidation_walk_calls_other),
                );
                obj.insert(
                    "invalidation_walk_nodes_other".to_string(),
                    Value::from(row.invalidation_walk_nodes_other),
                );
                obj.insert(
                    "hover_pressable_target_changes".to_string(),
                    Value::from(row.hover_pressable_target_changes),
                );
                obj.insert(
                    "hover_hover_region_target_changes".to_string(),
                    Value::from(row.hover_hover_region_target_changes),
                );
                obj.insert(
                    "hover_declarative_instance_changes".to_string(),
                    Value::from(row.hover_declarative_instance_changes),
                );
                obj.insert(
                    "hover_declarative_hit_test_invalidations".to_string(),
                    Value::from(row.hover_declarative_hit_test_invalidations),
                );
                obj.insert(
                    "hover_declarative_layout_invalidations".to_string(),
                    Value::from(row.hover_declarative_layout_invalidations),
                );
                obj.insert(
                    "hover_declarative_paint_invalidations".to_string(),
                    Value::from(row.hover_declarative_paint_invalidations),
                );

                let top_invalidation_walks = row
                    .top_invalidation_walks
                    .iter()
                    .map(|w| {
                        let mut w_obj = Map::new();
                        w_obj.insert("root_node".to_string(), Value::from(w.root_node));
                        w_obj.insert(
                            "root_element".to_string(),
                            w.root_element.map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "kind".to_string(),
                            w.kind.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "source".to_string(),
                            w.source.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "detail".to_string(),
                            w.detail.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert("walked_nodes".to_string(), Value::from(w.walked_nodes));
                        w_obj.insert(
                            "truncated_at".to_string(),
                            w.truncated_at.map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "root_role".to_string(),
                            w.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "root_test_id".to_string(),
                            w.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(w_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_invalidation_walks".to_string(),
                    Value::Array(top_invalidation_walks),
                );

                let top_hover_declarative_invalidations = row
                    .top_hover_declarative_invalidations
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert("hit_test".to_string(), Value::from(h.hit_test));
                        h_obj.insert("layout".to_string(), Value::from(h.layout));
                        h_obj.insert("paint".to_string(), Value::from(h.paint));
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_hover_declarative_invalidations".to_string(),
                    Value::Array(top_hover_declarative_invalidations),
                );

                let top_cache_roots = row
                    .top_cache_roots
                    .iter()
                    .map(|c| {
                        let mut c_obj = Map::new();
                        c_obj.insert("root_node".to_string(), Value::from(c.root_node));
                        c_obj.insert(
                            "element".to_string(),
                            c.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "element_path".to_string(),
                            c.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert("reused".to_string(), Value::from(c.reused));
                        c_obj.insert(
                            "contained_layout".to_string(),
                            Value::from(c.contained_layout),
                        );
                        c_obj.insert(
                            "contained_relayout_in_frame".to_string(),
                            Value::from(c.contained_relayout_in_frame),
                        );
                        c_obj.insert(
                            "paint_replayed_ops".to_string(),
                            Value::from(c.paint_replayed_ops),
                        );
                        c_obj.insert(
                            "reuse_reason".to_string(),
                            c.reuse_reason
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_in_semantics".to_string(),
                            c.root_in_semantics.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_role".to_string(),
                            c.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_test_id".to_string(),
                            c.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(c_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert("top_cache_roots".to_string(), Value::Array(top_cache_roots));

                let top_contained_relayout_cache_roots = row
                    .top_contained_relayout_cache_roots
                    .iter()
                    .map(|c| {
                        let mut c_obj = Map::new();
                        c_obj.insert("root_node".to_string(), Value::from(c.root_node));
                        c_obj.insert(
                            "element".to_string(),
                            c.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "element_path".to_string(),
                            c.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert("reused".to_string(), Value::from(c.reused));
                        c_obj.insert(
                            "contained_layout".to_string(),
                            Value::from(c.contained_layout),
                        );
                        c_obj.insert(
                            "contained_relayout_in_frame".to_string(),
                            Value::from(c.contained_relayout_in_frame),
                        );
                        c_obj.insert(
                            "paint_replayed_ops".to_string(),
                            Value::from(c.paint_replayed_ops),
                        );
                        c_obj.insert(
                            "reuse_reason".to_string(),
                            c.reuse_reason
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_in_semantics".to_string(),
                            c.root_in_semantics.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_role".to_string(),
                            c.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_test_id".to_string(),
                            c.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(c_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_contained_relayout_cache_roots".to_string(),
                    Value::Array(top_contained_relayout_cache_roots),
                );

                let top_layout_engine_solves = row
                    .top_layout_engine_solves
                    .iter()
                    .map(|s| {
                        let mut s_obj = Map::new();
                        s_obj.insert("root_node".to_string(), Value::from(s.root_node));
                        s_obj.insert("solve_time_us".to_string(), Value::from(s.solve_time_us));
                        s_obj.insert("measure_calls".to_string(), Value::from(s.measure_calls));
                        s_obj.insert(
                            "measure_cache_hits".to_string(),
                            Value::from(s.measure_cache_hits),
                        );
                        s_obj.insert(
                            "measure_time_us".to_string(),
                            Value::from(s.measure_time_us),
                        );
                        let top_measures = s
                            .top_measures
                            .iter()
                            .map(|m| {
                                let mut m_obj = Map::new();
                                m_obj.insert("node".to_string(), Value::from(m.node));
                                m_obj.insert(
                                    "measure_time_us".to_string(),
                                    Value::from(m.measure_time_us),
                                );
                                m_obj.insert("calls".to_string(), Value::from(m.calls));
                                m_obj.insert("cache_hits".to_string(), Value::from(m.cache_hits));
                                m_obj.insert(
                                    "element".to_string(),
                                    m.element.map(Value::from).unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "element_kind".to_string(),
                                    m.element_kind
                                        .clone()
                                        .map(Value::from)
                                        .unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "role".to_string(),
                                    m.role.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "test_id".to_string(),
                                    m.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                let top_children = m
                                    .top_children
                                    .iter()
                                    .map(|c| {
                                        let mut c_obj = Map::new();
                                        c_obj.insert("child".to_string(), Value::from(c.child));
                                        c_obj.insert(
                                            "measure_time_us".to_string(),
                                            Value::from(c.measure_time_us),
                                        );
                                        c_obj.insert("calls".to_string(), Value::from(c.calls));
                                        c_obj.insert(
                                            "element".to_string(),
                                            c.element.map(Value::from).unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "element_kind".to_string(),
                                            c.element_kind
                                                .clone()
                                                .map(Value::from)
                                                .unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "role".to_string(),
                                            c.role.clone().map(Value::from).unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "test_id".to_string(),
                                            c.test_id
                                                .clone()
                                                .map(Value::from)
                                                .unwrap_or(Value::Null),
                                        );
                                        Value::Object(c_obj)
                                    })
                                    .collect::<Vec<_>>();
                                m_obj
                                    .insert("top_children".to_string(), Value::Array(top_children));
                                Value::Object(m_obj)
                            })
                            .collect::<Vec<_>>();
                        s_obj.insert("top_measures".to_string(), Value::Array(top_measures));
                        s_obj.insert(
                            "root_role".to_string(),
                            s.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        s_obj.insert(
                            "root_test_id".to_string(),
                            s.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(s_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_layout_engine_solves".to_string(),
                    Value::Array(top_layout_engine_solves),
                );

                let paint_widget_hotspots = row
                    .paint_widget_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "element_kind".to_string(),
                            h.element_kind
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "widget_type".to_string(),
                            h.widget_type
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert("paint_time_us".to_string(), Value::from(h.paint_time_us));
                        h_obj.insert(
                            "inclusive_time_us".to_string(),
                            Value::from(h.inclusive_time_us),
                        );
                        h_obj.insert(
                            "inclusive_scene_ops_delta".to_string(),
                            Value::from(h.inclusive_scene_ops_delta),
                        );
                        h_obj.insert(
                            "exclusive_scene_ops_delta".to_string(),
                            Value::from(h.exclusive_scene_ops_delta),
                        );
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "paint_widget_hotspots".to_string(),
                    Value::Array(paint_widget_hotspots),
                );

                let paint_text_prepare_hotspots = row
                    .paint_text_prepare_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "element_kind".to_string(),
                            h.element_kind
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "prepare_time_us".to_string(),
                            Value::from(h.prepare_time_us),
                        );
                        h_obj.insert("text_len".to_string(), Value::from(h.text_len));
                        h_obj.insert(
                            "max_width".to_string(),
                            h.max_width.map(|v| Value::from(v)).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "wrap".to_string(),
                            h.wrap.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "overflow".to_string(),
                            h.overflow.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "scale_factor".to_string(),
                            h.scale_factor
                                .map(|v| Value::from(v))
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert("reasons_mask".to_string(), Value::from(h.reasons_mask));
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "paint_text_prepare_hotspots".to_string(),
                    Value::Array(paint_text_prepare_hotspots),
                );

                let model_change_hotspots = row
                    .model_change_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("model".to_string(), Value::from(h.model));
                        h_obj.insert(
                            "observation_edges".to_string(),
                            Value::from(h.observation_edges),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "model_change_hotspots".to_string(),
                    Value::Array(model_change_hotspots),
                );

                let model_change_unobserved = row
                    .model_change_unobserved
                    .iter()
                    .map(|u| {
                        let mut u_obj = Map::new();
                        u_obj.insert("model".to_string(), Value::from(u.model));
                        u_obj.insert(
                            "created_type".to_string(),
                            u.created_type
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        u_obj.insert(
                            "created_at".to_string(),
                            u.created_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(u_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "model_change_unobserved".to_string(),
                    Value::Array(model_change_unobserved),
                );

                let global_change_hotspots = row
                    .global_change_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("type_name".to_string(), Value::from(h.type_name.clone()));
                        h_obj.insert(
                            "observation_edges".to_string(),
                            Value::from(h.observation_edges),
                        );
                        h_obj.insert(
                            "changed_at".to_string(),
                            h.changed_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "global_change_hotspots".to_string(),
                    Value::Array(global_change_hotspots),
                );

                let global_change_unobserved = row
                    .global_change_unobserved
                    .iter()
                    .map(|u| {
                        let mut u_obj = Map::new();
                        u_obj.insert("type_name".to_string(), Value::from(u.type_name.clone()));
                        u_obj.insert(
                            "changed_at".to_string(),
                            u.changed_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(u_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "global_change_unobserved".to_string(),
                    Value::Array(global_change_unobserved),
                );

                Value::Object(obj)
            })
            .collect::<Vec<_>>();

        root.insert("top".to_string(), Value::Array(top));
        Value::Object(root)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BundleStatsOptions {
    pub(super) warmup_frames: u64,
}

pub(super) fn bundle_stats_from_path(
    bundle_path: &Path,
    top: usize,
    sort: BundleStatsSort,
    opts: BundleStatsOptions,
) -> Result<BundleStatsReport, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    bundle_stats_from_json_with_options(&bundle, top, sort, opts)
}

pub(super) fn check_bundle_for_stale_paint(
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_stale_paint_json(&bundle, bundle_path, test_id, eps)
}

pub(super) fn check_bundle_for_stale_paint_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut suspicious: Vec<String> = Vec::new();
    let mut missing_scene_fingerprint = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_y: Option<f64> = None;
        let mut prev_fp: Option<u64> = None;
        for s in snaps {
            let y = semantics_node_y_for_test_id(s, test_id);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }
            let (Some(y), Some(fp)) = (y, fp) else {
                prev_y = y;
                prev_fp = fp;
                continue;
            };

            if let (Some(prev_y), Some(prev_fp)) = (prev_y, prev_fp) {
                if (y - prev_y).abs() >= eps as f64 && fp == prev_fp {
                    let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let paint_nodes_performed = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_nodes_performed"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let paint_replayed_ops = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_cache_replayed_ops"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    suspicious.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} test_id={test_id} delta_y={:.2} scene_fingerprint=0x{:016x} paint_nodes_performed={paint_nodes_performed} paint_cache_replayed_ops={paint_replayed_ops}",
                        y - prev_y,
                        fp
                    ));
                    if suspicious.len() >= 8 {
                        break;
                    }
                }
            }

            prev_y = Some(y);
            prev_fp = Some(fp);
        }
    }

    if missing_scene_fingerprint {
        return Err(format!(
            "stale paint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale paint suspected (semantics bounds moved but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in suspicious {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_stale_scene(
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_stale_scene_json(&bundle, bundle_path, test_id, eps)
}

#[derive(Debug, Clone, Default)]
pub(super) struct SemanticsChangedRepaintedScan {
    missing_scene_fingerprint: bool,
    missing_semantics_fingerprint: bool,
    suspicious_lines: Vec<String>,
    pub(super) findings: Vec<serde_json::Value>,
}

pub(super) fn check_bundle_for_semantics_changed_repainted(
    bundle_path: &Path,
    warmup_frames: u64,
    dump_json: bool,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let scan = scan_semantics_changed_repainted_json(&bundle, warmup_frames);
    if dump_json && !scan.findings.is_empty() {
        let out_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
        let out_path = out_dir.join("check.semantics_changed_repainted.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "semantics_changed_repainted",
            "bundle_json": bundle_path.display().to_string(),
            "warmup_frames": warmup_frames,
            "findings": scan.findings,
        });
        let _ = write_json_value(&out_path, &payload);
    }

    check_bundle_for_semantics_changed_repainted_json(&bundle, bundle_path, warmup_frames)
}

pub(super) fn check_bundle_for_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let scan = scan_semantics_changed_repainted_json(bundle, warmup_frames);

    if scan.missing_scene_fingerprint {
        return Err(format!(
            "semantics repaint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.missing_semantics_fingerprint {
        return Err(format!(
            "semantics repaint check requires `semantics_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.suspicious_lines.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "missing repaint suspected (semantics fingerprint changed but scene fingerprint did not)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in scan.suspicious_lines {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn scan_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    warmup_frames: u64,
) -> SemanticsChangedRepaintedScan {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    if windows.is_empty() {
        return SemanticsChangedRepaintedScan::default();
    }

    let mut scan = SemanticsChangedRepaintedScan::default();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_scene_fingerprint: Option<u64> = None;
        let mut prev_semantics_fingerprint: Option<u64> = None;
        let mut prev_tick_id: u64 = 0;
        let mut prev_frame_id: u64 = 0;
        let mut prev_snapshot: Option<&serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let scene_fingerprint = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if scene_fingerprint.is_none() {
                scan.missing_scene_fingerprint = true;
            }

            let semantics_fingerprint = s.get("semantics_fingerprint").and_then(|v| v.as_u64());
            if semantics_fingerprint.is_none() {
                scan.missing_semantics_fingerprint = true;
            }

            let (Some(scene_fingerprint), Some(semantics_fingerprint)) =
                (scene_fingerprint, semantics_fingerprint)
            else {
                prev_scene_fingerprint = None;
                prev_semantics_fingerprint = None;
                prev_tick_id = tick_id;
                prev_frame_id = frame_id;
                prev_snapshot = Some(s);
                continue;
            };

            if let (Some(prev_scene), Some(prev_sem)) =
                (prev_scene_fingerprint, prev_semantics_fingerprint)
            {
                let semantics_changed = semantics_fingerprint != prev_sem;
                let scene_unchanged = scene_fingerprint == prev_scene;
                if semantics_changed && scene_unchanged {
                    let paint_nodes_performed = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_nodes_performed"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let paint_cache_replayed_ops = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_cache_replayed_ops"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    let diff_detail = prev_snapshot
                        .map(|prev| semantics_diff_detail(prev, s))
                        .unwrap_or(serde_json::Value::Null);

                    scan.findings.push(serde_json::json!({
                        "window": window_id,
                        "prev": {
                            "tick_id": prev_tick_id,
                            "frame_id": prev_frame_id,
                            "scene_fingerprint": prev_scene,
                            "semantics_fingerprint": prev_sem,
                        },
                        "now": {
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "scene_fingerprint": scene_fingerprint,
                            "semantics_fingerprint": semantics_fingerprint,
                        },
                        "paint_nodes_performed": paint_nodes_performed,
                        "paint_cache_replayed_ops": paint_cache_replayed_ops,
                        "semantics_diff": diff_detail,
                    }));

                    let mut detail = String::new();
                    if let Some(prev) = prev_snapshot {
                        let diff = semantics_diff_summary(prev, s);
                        if !diff.is_empty() {
                            detail.push(' ');
                            detail.push_str(&diff);
                        }
                    }

                    scan.suspicious_lines.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} prev_tick={prev_tick_id} prev_frame={prev_frame_id} semantics_fingerprint=0x{semantics_fingerprint:016x} prev_semantics_fingerprint=0x{prev_sem:016x} scene_fingerprint=0x{scene_fingerprint:016x} paint_nodes_performed={paint_nodes_performed} paint_cache_replayed_ops={paint_cache_replayed_ops}{detail}"
                    ));
                    if scan.suspicious_lines.len() >= 8 {
                        break;
                    }
                }
            }

            prev_scene_fingerprint = Some(scene_fingerprint);
            prev_semantics_fingerprint = Some(semantics_fingerprint);
            prev_tick_id = tick_id;
            prev_frame_id = frame_id;
            prev_snapshot = Some(s);
        }
    }

    scan
}

fn semantics_diff_detail(
    before: &serde_json::Value,
    after: &serde_json::Value,
) -> serde_json::Value {
    use serde_json::json;
    use std::collections::{HashMap, HashSet};

    let before_nodes = before
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let after_nodes = after
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());

    let (Some(before_nodes), Some(after_nodes)) = (before_nodes, after_nodes) else {
        return serde_json::Value::Null;
    };

    let mut before_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in before_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        before_by_id.insert(id, node);
    }

    let mut after_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in after_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        after_by_id.insert(id, node);
    }

    let before_ids: HashSet<u64> = before_by_id.keys().copied().collect();
    let after_ids: HashSet<u64> = after_by_id.keys().copied().collect();

    let mut added: Vec<u64> = after_ids.difference(&before_ids).copied().collect();
    let mut removed: Vec<u64> = before_ids.difference(&after_ids).copied().collect();
    added.sort_unstable();
    removed.sort_unstable();

    let mut changed: Vec<(u64, u64)> = Vec::new(); // (score, id)
    for id in before_ids.intersection(&after_ids).copied() {
        let Some(a) = after_by_id.get(&id).copied() else {
            continue;
        };
        let Some(b) = before_by_id.get(&id).copied() else {
            continue;
        };
        let fp_a = semantics_node_fingerprint_json(a);
        let fp_b = semantics_node_fingerprint_json(b);
        if fp_a != fp_b {
            let score = semantics_node_score_json(a);
            changed.push((score, id));
        }
    }
    changed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let sample_len = 6usize;

    let added_nodes = added
        .iter()
        .take(sample_len)
        .map(|id| semantics_node_summary_json(*id, after_by_id.get(id).copied()))
        .collect::<Vec<_>>();
    let removed_nodes = removed
        .iter()
        .take(sample_len)
        .map(|id| semantics_node_summary_json(*id, before_by_id.get(id).copied()))
        .collect::<Vec<_>>();
    let changed_nodes = changed
        .iter()
        .take(sample_len)
        .map(|(_score, id)| {
            let before = semantics_node_summary_json(*id, before_by_id.get(id).copied());
            let after = semantics_node_summary_json(*id, after_by_id.get(id).copied());
            json!({ "id": id, "before": before, "after": after })
        })
        .collect::<Vec<_>>();

    json!({
        "counts": {
            "added": added.len(),
            "removed": removed.len(),
            "changed": changed.len(),
        },
        "samples": {
            "added_nodes": added_nodes,
            "removed_nodes": removed_nodes,
            "changed_nodes": changed_nodes,
        }
    })
}

fn semantics_node_summary_json(id: u64, node: Option<&serde_json::Value>) -> serde_json::Value {
    use serde_json::json;
    let Some(node) = node else {
        return json!({ "id": id });
    };

    let role = node.get("role").and_then(|v| v.as_str());
    let parent = node.get("parent").and_then(|v| v.as_u64());
    let test_id = node.get("test_id").and_then(|v| v.as_str());
    let label = node.get("label").and_then(|v| v.as_str());
    let value = node.get("value").and_then(|v| v.as_str());

    let bounds = node.get("bounds").and_then(|b| {
        Some(json!({
            "x": b.get("x").and_then(|v| v.as_f64()),
            "y": b.get("y").and_then(|v| v.as_f64()),
            "w": b.get("w").and_then(|v| v.as_f64()),
            "h": b.get("h").and_then(|v| v.as_f64()),
        }))
    });

    json!({
        "id": id,
        "parent": parent,
        "role": role,
        "test_id": test_id,
        "label": label,
        "value": value,
        "bounds": bounds,
    })
}

fn semantics_diff_summary(before: &serde_json::Value, after: &serde_json::Value) -> String {
    let before_nodes = before
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let after_nodes = after
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());

    let (Some(before_nodes), Some(after_nodes)) = (before_nodes, after_nodes) else {
        return String::new();
    };

    use std::collections::{HashMap, HashSet};

    let mut before_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in before_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        before_by_id.insert(id, node);
    }

    let mut after_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in after_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        after_by_id.insert(id, node);
    }

    let before_ids: HashSet<u64> = before_by_id.keys().copied().collect();
    let after_ids: HashSet<u64> = after_by_id.keys().copied().collect();

    let mut added: Vec<u64> = after_ids.difference(&before_ids).copied().collect();
    let mut removed: Vec<u64> = before_ids.difference(&after_ids).copied().collect();
    added.sort_unstable();
    removed.sort_unstable();

    let mut changed: Vec<(u64, u64, u64)> = Vec::new(); // (score, id, fp_after)
    for id in before_ids.intersection(&after_ids).copied() {
        let Some(a) = after_by_id.get(&id).copied() else {
            continue;
        };
        let Some(b) = before_by_id.get(&id).copied() else {
            continue;
        };
        let fp_a = semantics_node_fingerprint_json(a);
        let fp_b = semantics_node_fingerprint_json(b);
        if fp_a != fp_b {
            // Score heuristic: test_id changes are the most useful to report.
            let score = semantics_node_score_json(a);
            changed.push((score, id, fp_a));
        }
    }

    if added.is_empty() && removed.is_empty() && changed.is_empty() {
        return String::new();
    }

    changed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let mut out = String::new();
    out.push_str("semantics_diff={");
    out.push_str(&format!(
        "added={} removed={} changed={}",
        added.len(),
        removed.len(),
        changed.len()
    ));

    let sample_len = 6usize;
    if !changed.is_empty() {
        out.push_str(" changed_nodes=[");
        for (i, (_score, id, _fp)) in changed.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = after_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if changed.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    if !added.is_empty() {
        out.push_str(" added_nodes=[");
        for (i, id) in added.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = after_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if added.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    if !removed.is_empty() {
        out.push_str(" removed_nodes=[");
        for (i, id) in removed.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = before_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if removed.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    out.push('}');
    out
}

fn semantics_node_score_json(node: &serde_json::Value) -> u64 {
    // Higher is "more useful for debugging".
    let mut score: u64 = 0;
    if node.get("test_id").and_then(|v| v.as_str()).is_some() {
        score += 10_000;
    }
    if node.get("label").and_then(|v| v.as_str()).is_some() {
        score += 1_000;
    }
    if node.get("value").and_then(|v| v.as_str()).is_some() {
        score += 500;
    }
    score
}

fn semantics_node_label_json(id: u64, node: Option<&serde_json::Value>) -> String {
    let Some(node) = node else {
        return format!("id={id}");
    };
    let role = node
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let test_id = node
        .get("test_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());
    let label = node
        .get("label")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());
    let value = node
        .get("value")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());

    let mut out = format!("id={id} role={role}");
    if let Some(v) = test_id {
        out.push_str(" test_id=");
        out.push_str(v);
    }
    if let Some(v) = label {
        out.push_str(" label=");
        out.push_str(v);
    }
    if let Some(v) = value {
        out.push_str(" value=");
        out.push_str(v);
    }
    out
}

fn semantics_node_fingerprint_json(node: &serde_json::Value) -> u64 {
    use std::hash::{Hash, Hasher};

    // Use a stable hash for a curated subset of fields.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    node.get("id").and_then(|v| v.as_u64()).hash(&mut hasher);
    node.get("parent")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("role").and_then(|v| v.as_str()).hash(&mut hasher);

    if let Some(bounds) = node.get("bounds") {
        if let Some(v) = bounds.get("x").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("y").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("w").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("h").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
    }

    if let Some(flags) = node.get("flags") {
        flags
            .get("focused")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("captured")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("disabled")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("selected")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("expanded")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("checked")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
    }

    node.get("test_id")
        .and_then(|v| v.as_str())
        .hash(&mut hasher);
    node.get("active_descendant")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("pos_in_set")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("set_size")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("label").and_then(|v| v.as_str()).hash(&mut hasher);
    node.get("value").and_then(|v| v.as_str()).hash(&mut hasher);

    if let Some(actions) = node.get("actions") {
        actions
            .get("focus")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("invoke")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("set_value")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("set_text_selection")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
    }

    hasher.finish()
}

pub(super) fn check_bundle_for_stale_scene_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut suspicious: Vec<String> = Vec::new();
    let mut missing_scene_fingerprint = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_y: Option<f64> = None;
        let mut prev_label: Option<String> = None;
        let mut prev_value: Option<String> = None;
        let mut prev_fp: Option<u64> = None;

        for s in snaps {
            let (y, label, value) = semantics_node_fields_for_test_id(s, test_id);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }

            let Some(fp) = fp else {
                prev_y = y;
                prev_label = label;
                prev_value = value;
                prev_fp = None;
                continue;
            };

            if let (Some(prev_fp), Some(prev_y)) = (prev_fp, prev_y) {
                let moved = y
                    .zip(Some(prev_y))
                    .is_some_and(|(y, prev_y)| (y - prev_y).abs() >= eps as f64);
                let label_changed = label.as_deref() != prev_label.as_deref();
                let value_changed = value.as_deref() != prev_value.as_deref();
                let changed = moved || label_changed || value_changed;

                if changed && fp == prev_fp {
                    let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let label_len_prev = prev_label.as_deref().map(|s| s.len()).unwrap_or(0);
                    let label_len_now = label.as_deref().map(|s| s.len()).unwrap_or(0);
                    let value_len_prev = prev_value.as_deref().map(|s| s.len()).unwrap_or(0);
                    let value_len_now = value.as_deref().map(|s| s.len()).unwrap_or(0);
                    let delta_y = y
                        .zip(Some(prev_y))
                        .map(|(y, prev_y)| y - prev_y)
                        .unwrap_or(0.0);
                    suspicious.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} test_id={test_id} changed={{moved={moved} label={label_changed} value={value_changed}}} delta_y={delta_y:.2} label_len={label_len_prev}->{label_len_now} value_len={value_len_prev}->{value_len_now} scene_fingerprint=0x{fp:016x}",
                    ));
                    if suspicious.len() >= 8 {
                        break;
                    }
                }
            }

            prev_y = y;
            prev_label = label;
            prev_value = value;
            prev_fp = Some(fp);
        }
    }

    if missing_scene_fingerprint {
        return Err(format!(
            "stale scene check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale scene suspected (semantics changed but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in suspicious {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

fn semantics_node_y_for_test_id(snapshot: &serde_json::Value, test_id: &str) -> Option<f64> {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())?;
    let node = nodes.iter().find(|n| {
        n.get("test_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == test_id)
    })?;
    node.get("bounds")
        .and_then(|v| v.get("y"))
        .and_then(|v| v.as_f64())
}

fn semantics_node_fields_for_test_id(
    snapshot: &serde_json::Value,
    test_id: &str,
) -> (Option<f64>, Option<String>, Option<String>) {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let Some(nodes) = nodes else {
        return (None, None, None);
    };
    let node = nodes.iter().find(|n| {
        n.get("test_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == test_id)
    });
    let Some(node) = node else {
        return (None, None, None);
    };
    let y = node
        .get("bounds")
        .and_then(|v| v.get("y"))
        .and_then(|v| v.as_f64());
    let label = node
        .get("label")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let value = node
        .get("value")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    (y, label, value)
}

fn first_wheel_frame_id_for_window(window: &serde_json::Value) -> Option<u64> {
    window
        .get("events")
        .and_then(|v| v.as_array())?
        .iter()
        .filter(|e| e.get("kind").and_then(|v| v.as_str()) == Some("pointer.wheel"))
        .filter_map(|e| e.get("frame_id").and_then(|v| v.as_u64()))
        .min()
}

fn first_scroll_offset_change_frame_id_for_window(
    window: &serde_json::Value,
    warmup_frames: u64,
) -> Option<u64> {
    let snaps = window
        .get("snapshots")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    snaps
        .iter()
        .filter_map(|s| {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64())?;
            if frame_id < warmup_frames {
                return None;
            }
            let changes = s
                .get("debug")
                .and_then(|v| v.get("scroll_handle_changes"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            let any_offset_changed = changes.iter().any(|c| {
                c.get("offset_changed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            });
            any_offset_changed.then_some(frame_id)
        })
        .min()
}

fn semantics_node_id_for_test_id(snapshot: &serde_json::Value, test_id: &str) -> Option<u64> {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())?;
    nodes
        .iter()
        .find(|n| {
            n.get("test_id")
                .and_then(|v| v.as_str())
                .is_some_and(|id| id == test_id)
        })?
        .get("id")
        .and_then(|v| v.as_u64())
}

fn hit_test_node_id(snapshot: &serde_json::Value) -> Option<u64> {
    snapshot
        .get("debug")
        .and_then(|v| v.get("hit_test"))
        .and_then(|v| v.get("hit"))
        .and_then(|v| v.as_u64())
}

fn is_descendant(
    mut node: u64,
    ancestor: u64,
    parents: &std::collections::HashMap<u64, u64>,
) -> bool {
    if node == ancestor {
        return true;
    }
    while let Some(parent) = parents.get(&node).copied() {
        if parent == ancestor {
            return true;
        }
        node = parent;
    }
    false
}

fn semantics_parent_map(snapshot: &serde_json::Value) -> std::collections::HashMap<u64, u64> {
    let mut parents = std::collections::HashMap::new();
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let Some(nodes) = nodes else {
        return parents;
    };
    for node in nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(parent) = node.get("parent").and_then(|v| v.as_u64()) else {
            continue;
        };
        parents.insert(id, parent);
    }
    parents
}

pub(super) fn check_bundle_for_wheel_scroll(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_wheel_scroll_json(&bundle, bundle_path, test_id, warmup_frames)
}

pub(super) fn check_bundle_for_wheel_scroll_hit_changes(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_wheel_scroll_hit_changes_json(&bundle, bundle_path, test_id, warmup_frames)
}

pub(super) fn check_bundle_for_wheel_scroll_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut failures: Vec<String> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut before: Option<&serde_json::Value> = None;
        let mut before_frame: u64 = 0;
        let mut after: Option<&serde_json::Value> = None;
        let mut after_frame_id: u64 = 0;
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                if frame_id >= before_frame && frame_id < after_frame {
                    before = Some(s);
                    before_frame = frame_id;
                }
                continue;
            }
            after = Some(s);
            after_frame_id = frame_id;
            break;
        }

        let (Some(before), Some(after)) = (before, after) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_before_or_after_snapshot"
            ));
            continue;
        };

        let Some(hit_before) = hit_test_node_id(before) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_hit_before"
            ));
            continue;
        };
        let Some(hit_after) = hit_test_node_id(after) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} error=missing_hit_after"
            ));
            continue;
        };

        let Some(target_before) = semantics_node_id_for_test_id(before, test_id) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(target_after) = semantics_node_id_for_test_id(after, test_id) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let before_parents = semantics_parent_map(before);
        let after_parents = semantics_parent_map(after);

        if !is_descendant(hit_before, target_before, &before_parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=hit_not_within_target_before hit={hit_before} target={target_before}"
            ));
            continue;
        }

        if is_descendant(hit_after, target_after, &after_parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_still_within_target_after hit={hit_after} target={target_after}"
            ));
        }
    }

    if !any_wheel {
        return Err(format!(
            "wheel scroll check requires at least one pointer.wheel event in the bundle: {}",
            bundle_path.display()
        ));
    }

    if failures.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("wheel scroll check failed (expected hit-test result to move after wheel)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in failures {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_wheel_scroll_hit_changes_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut failures: Vec<String> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut before: Option<&serde_json::Value> = None;
        let mut before_frame: u64 = 0;
        let mut after: Option<&serde_json::Value> = None;
        let mut after_frame_id: u64 = 0;
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                if frame_id >= before_frame && frame_id < after_frame {
                    before = Some(s);
                    before_frame = frame_id;
                }
                continue;
            }
            after = Some(s);
            after_frame_id = frame_id;
            break;
        }

        let (Some(before), Some(after)) = (before, after) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_before_or_after_snapshot"
            ));
            continue;
        };

        let Some(target_before) = semantics_node_id_for_test_id(before, test_id) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(target_after) = semantics_node_id_for_test_id(after, test_id) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let before_parents = semantics_parent_map(before);
        let after_parents = semantics_parent_map(after);

        let Some(hit_before) = hit_test_node_id(before) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_hit_before"
            ));
            continue;
        };
        let Some(hit_after) = hit_test_node_id(after) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} error=missing_hit_after"
            ));
            continue;
        };

        if !is_descendant(hit_before, target_before, &before_parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=hit_not_within_target_before hit={hit_before} target={target_before}"
            ));
            continue;
        }
        if !is_descendant(hit_after, target_after, &after_parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_not_within_target_after hit={hit_after} target={target_after}"
            ));
            continue;
        }

        // Prefer a vlist-driven signal when available: for virtualized surfaces the hit-test node
        // can remain stable (e.g. when hovering a static region), but the scroll offset must move.
        let before_offset = before
            .get("debug")
            .and_then(|v| v.get("virtual_list_windows"))
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .and_then(|v| v.get("offset"))
            .and_then(|v| v.as_f64());
        let after_offset = after
            .get("debug")
            .and_then(|v| v.get("virtual_list_windows"))
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .and_then(|v| v.get("offset"))
            .and_then(|v| v.as_f64());
        if let (Some(a), Some(b)) = (before_offset, after_offset) {
            if (a - b).abs() > 0.1 {
                continue;
            }
        }

        if hit_before == hit_after {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_did_not_change hit={hit_after}"
            ));
        }
    }

    if !any_wheel {
        return Err(format!(
            "wheel scroll hit-change check requires at least one pointer.wheel event in the bundle: {}",
            bundle_path.display()
        ));
    }

    if failures.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "wheel scroll hit-change check failed (expected wheel to affect the scrolled content)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in failures {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_visible_range_refreshes_max_json(
        &bundle,
        bundle_path,
        out_dir,
        max_total_refreshes,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_window_shifts_explainable(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_explainable_json(
        &bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_window_shifts_non_retained_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_total_non_retained_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_non_retained_max_json(
        &bundle,
        bundle_path,
        out_dir,
        max_total_non_retained_shifts,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_window_shifts_kind_max(
    bundle_path: &Path,
    out_dir: &Path,
    kind: &str,
    max_total_kind_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_kind_max_json(
        &bundle,
        bundle_path,
        out_dir,
        kind,
        max_total_kind_shifts,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_policy_key_stable(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_policy_key_stable_json(&bundle, bundle_path, out_dir, warmup_frames)
}

pub(super) fn check_bundle_for_vlist_policy_key_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_signal = false;
    let mut examined_snapshots: u64 = 0;
    let mut by_surface: std::collections::BTreeMap<(u64, u64), std::collections::BTreeSet<u64>> =
        std::collections::BTreeMap::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let vlist_windows = s
                .get("debug")
                .and_then(|v| v.get("virtual_list_windows"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if vlist_windows.is_empty() {
                continue;
            }

            any_signal = true;
            for win in vlist_windows {
                let node = win.get("node").and_then(|v| v.as_u64()).unwrap_or(0);
                let element = win.get("element").and_then(|v| v.as_u64()).unwrap_or(0);
                let policy_key = win.get("policy_key").and_then(|v| v.as_u64()).unwrap_or(0);
                by_surface
                    .entry((node, element))
                    .or_default()
                    .insert(policy_key);
            }
        }
    }

    let offenders: Vec<serde_json::Value> = by_surface
        .iter()
        .filter(|(_, keys)| keys.len() > 1)
        .take(64)
        .map(|((node, element), keys)| {
            serde_json::json!({
                "node": node,
                "element": element,
                "policy_keys": keys.iter().copied().collect::<Vec<u64>>(),
            })
        })
        .collect();

    let out_path = out_dir.join("check.vlist_policy_key_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_policy_key_stable",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "surfaces_seen": by_surface.len(),
        "offenders": offenders,
    });
    write_json_value(&out_path, &payload)?;

    if !any_signal {
        return Err(format!(
            "vlist policy-key stability gate requires debug.virtual_list_windows after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if offenders.is_empty() {
        return Ok(());
    }

    Err(format!(
        "vlist policy-key stability gate failed (expected each vlist surface to keep a stable policy_key after warmup)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_visible_range_refreshes_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_total_refreshes,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_windowed_rows_offset_changes_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_total_offset_changes: u64,
    warmup_frames: u64,
    eps_px: f32,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_windowed_rows_offset_changes_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_total_offset_changes,
        warmup_frames,
        eps_px,
    )
}

pub(super) fn check_bundle_for_windowed_rows_offset_changes_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_total_offset_changes: u64,
    warmup_frames: u64,
    eps_px: f32,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    #[derive(Default)]
    struct SurfaceStats {
        location: Option<serde_json::Value>,
        samples: u64,
        offset_changes: u64,
        visible_start_changes: u64,
        prev_offset_y: Option<f32>,
        prev_visible_start: Option<u64>,
    }

    let mut any_scroll = false;
    let mut examined_snapshots: u64 = 0;
    let mut scroll_offset_changed_events: u64 = 0;
    let mut total_offset_changes: u64 = 0;

    let mut surfaces: std::collections::BTreeMap<(u64, u64), SurfaceStats> =
        std::collections::BTreeMap::new();
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(scroll_frame) = first_scroll_offset_change_frame_id_for_window(w, warmup_frames)
        else {
            continue;
        };
        any_scroll = true;

        let after_frame = scroll_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let scroll_changes = s
                .get("debug")
                .and_then(|v| v.get("scroll_handle_changes"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            for c in scroll_changes {
                if c.get("offset_changed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    scroll_offset_changed_events = scroll_offset_changed_events.saturating_add(1);
                }
            }

            let list = s
                .get("debug")
                .and_then(|v| v.get("windowed_rows_surfaces"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if list.is_empty() {
                continue;
            }

            for entry in list {
                let Some(callsite_id) = entry.get("callsite_id").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Some(offset_y) = entry
                    .get("offset_y")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32)
                else {
                    continue;
                };

                let stats = surfaces.entry((window_id, callsite_id)).or_default();
                stats.samples = stats.samples.saturating_add(1);
                if stats.location.is_none() {
                    stats.location = entry.get("location").cloned();
                }

                if let Some(prev) = stats.prev_offset_y {
                    let delta = offset_y - prev;
                    if delta.abs() >= eps_px {
                        stats.offset_changes = stats.offset_changes.saturating_add(1);
                        total_offset_changes = total_offset_changes.saturating_add(1);

                        if samples.len() < 32 {
                            samples.push(serde_json::json!({
                                "window": window_id,
                                "tick_id": tick_id,
                                "frame_id": frame_id,
                                "callsite_id": callsite_id,
                                "delta_offset_y": delta,
                                "prev_offset_y": prev,
                                "offset_y": offset_y,
                            }));
                        }
                    }
                }
                stats.prev_offset_y = Some(offset_y);

                if let Some(visible_start) = entry.get("visible_start").and_then(|v| v.as_u64()) {
                    if let Some(prev) = stats.prev_visible_start
                        && visible_start != prev
                    {
                        stats.visible_start_changes = stats.visible_start_changes.saturating_add(1);
                    }
                    stats.prev_visible_start = Some(visible_start);
                }
            }
        }
    }

    let out_path = out_dir.join("check.windowed_rows_offset_changes_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "windowed_rows_offset_changes_min",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "eps_px": eps_px,
        "min_total_offset_changes": min_total_offset_changes,
        "any_scroll": any_scroll,
        "examined_snapshots": examined_snapshots,
        "scroll_offset_changed_events": scroll_offset_changed_events,
        "surfaces_seen": surfaces.len(),
        "total_offset_changes": total_offset_changes,
        "surfaces": surfaces.iter().map(|((window, callsite_id), stats)| serde_json::json!({
            "window": window,
            "callsite_id": callsite_id,
            "location": stats.location,
            "samples": stats.samples,
            "offset_changes": stats.offset_changes,
            "visible_start_changes": stats.visible_start_changes,
        })).collect::<Vec<_>>(),
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_scroll {
        return Err(format!(
            "windowed rows offset-change gate requires scroll offset changes after warmup, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if examined_snapshots == 0 {
        return Err(format!(
            "windowed rows offset-change gate requires snapshots after the first scroll change, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if scroll_offset_changed_events == 0 {
        return Err(format!(
            "windowed rows offset-change gate requires debug.scroll_handle_changes events after the first scroll change, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if surfaces.is_empty() {
        return Err(format!(
            "windowed rows offset-change gate requires debug.windowed_rows_surfaces after scroll changes, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if min_total_offset_changes > 0 && total_offset_changes < min_total_offset_changes {
        return Err(format!(
            "expected windowed rows surfaces to observe scroll offset changes, but total_offset_changes={total_offset_changes} was below min_total_offset_changes={min_total_offset_changes} (warmup_frames={warmup_frames}, eps_px={eps_px})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_windowed_rows_visible_start_changes_repainted(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
        &bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    #[derive(Default)]
    struct SurfaceStats {
        location: Option<serde_json::Value>,
        samples: u64,
        visible_start_changes: u64,
        suspicious_visible_start_changes: u64,
        prev_visible_start: Option<u64>,
        prev_scene_fingerprint: Option<u64>,
    }

    let mut any_scroll = false;
    let mut examined_snapshots: u64 = 0;
    let mut scroll_offset_changed_events: u64 = 0;
    let mut total_visible_start_changes: u64 = 0;
    let mut total_suspicious_changes: u64 = 0;
    let mut missing_scene_fingerprint = false;

    let mut surfaces: std::collections::BTreeMap<(u64, u64), SurfaceStats> =
        std::collections::BTreeMap::new();
    let mut suspicious: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(scroll_frame) = first_scroll_offset_change_frame_id_for_window(w, warmup_frames)
        else {
            continue;
        };
        any_scroll = true;

        let after_frame = scroll_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }

            let scroll_changes = s
                .get("debug")
                .and_then(|v| v.get("scroll_handle_changes"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            for c in scroll_changes {
                if c.get("offset_changed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    scroll_offset_changed_events = scroll_offset_changed_events.saturating_add(1);
                }
            }

            let list = s
                .get("debug")
                .and_then(|v| v.get("windowed_rows_surfaces"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if list.is_empty() {
                continue;
            }

            for entry in list {
                let Some(callsite_id) = entry.get("callsite_id").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Some(visible_start) = entry.get("visible_start").and_then(|v| v.as_u64())
                else {
                    continue;
                };
                let Some(fp) = fp else {
                    continue;
                };

                let stats = surfaces.entry((window_id, callsite_id)).or_default();
                stats.samples = stats.samples.saturating_add(1);
                if stats.location.is_none() {
                    stats.location = entry.get("location").cloned();
                }

                if let (Some(prev_start), Some(prev_fp)) =
                    (stats.prev_visible_start, stats.prev_scene_fingerprint)
                {
                    if visible_start != prev_start {
                        stats.visible_start_changes = stats.visible_start_changes.saturating_add(1);
                        total_visible_start_changes = total_visible_start_changes.saturating_add(1);
                        if fp == prev_fp {
                            stats.suspicious_visible_start_changes =
                                stats.suspicious_visible_start_changes.saturating_add(1);
                            total_suspicious_changes = total_suspicious_changes.saturating_add(1);
                            if suspicious.len() < 32 {
                                suspicious.push(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "callsite_id": callsite_id,
                                    "prev_visible_start": prev_start,
                                    "visible_start": visible_start,
                                    "scene_fingerprint": fp,
                                }));
                            }
                        }
                    }
                }

                stats.prev_visible_start = Some(visible_start);
                stats.prev_scene_fingerprint = Some(fp);
            }
        }
    }

    let out_path = out_dir.join("check.windowed_rows_visible_start_changes_repainted.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "windowed_rows_visible_start_changes_repainted",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "any_scroll": any_scroll,
        "examined_snapshots": examined_snapshots,
        "scroll_offset_changed_events": scroll_offset_changed_events,
        "surfaces_seen": surfaces.len(),
        "total_visible_start_changes": total_visible_start_changes,
        "total_suspicious_changes": total_suspicious_changes,
        "surfaces": surfaces.iter().map(|((window, callsite_id), stats)| serde_json::json!({
            "window": window,
            "callsite_id": callsite_id,
            "location": stats.location,
            "samples": stats.samples,
            "visible_start_changes": stats.visible_start_changes,
            "suspicious_visible_start_changes": stats.suspicious_visible_start_changes,
        })).collect::<Vec<_>>(),
        "suspicious_samples": suspicious,
    });
    write_json_value(&out_path, &payload)?;

    if missing_scene_fingerprint {
        return Err(format!(
            "windowed rows repaint gate requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if !any_scroll {
        return Err(format!(
            "windowed rows repaint gate requires scroll offset changes after warmup, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if examined_snapshots == 0 {
        return Err(format!(
            "windowed rows repaint gate requires snapshots after the first scroll change, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if scroll_offset_changed_events == 0 {
        return Err(format!(
            "windowed rows repaint gate requires debug.scroll_handle_changes events after the first scroll change, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if surfaces.is_empty() {
        return Err(format!(
            "windowed rows repaint gate requires debug.windowed_rows_surfaces after scroll changes, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if total_visible_start_changes == 0 {
        return Err(format!(
            "windowed rows repaint gate requires at least one visible_start change after the first scroll change (otherwise stale paint cannot be evaluated)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if total_suspicious_changes == 0 {
        return Ok(());
    }

    Err(format!(
        "windowed rows repaint gate failed (visible_start changed but scene fingerprint did not; suspected stale paint / stale lines)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

pub(super) fn check_bundle_for_layout_fast_path_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_layout_fast_path_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_frames,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_layout_fast_path_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut fast_path_frames: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let taken = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("layout_fast_path_taken"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if taken {
                fast_path_frames = fast_path_frames.saturating_add(1);
            }
        }
    }

    let out_path = out_dir.join("check.layout_fast_path_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "layout_fast_path_min",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_frames": min_frames,
        "examined_snapshots": examined_snapshots,
        "fast_path_frames": fast_path_frames,
    });
    write_json_value(&out_path, &payload)?;

    if examined_snapshots == 0 {
        return Err(format!(
            "layout fast-path gate requires snapshots after warmup, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if fast_path_frames >= min_frames {
        return Ok(());
    }

    Err(format!(
        "layout fast-path gate failed (expected at least {min_frames} frames to take the fast-path after warmup, got {fast_path_frames}; examined_snapshots={examined_snapshots}, warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut total_refreshes: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let refreshes = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("virtual_list_visible_range_refreshes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if refreshes == 0 {
                continue;
            }
            total_refreshes = total_refreshes.saturating_add(refreshes);
            if samples.len() < 32 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "refreshes": refreshes,
                }));
            }
        }
    }

    let out_path = out_dir.join("check.vlist_visible_range_refreshes_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_visible_range_refreshes_min",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_total_refreshes": min_total_refreshes,
        "any_wheel": any_wheel,
        "total_refreshes": total_refreshes,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_wheel {
        return Err(format!(
            "vlist visible-range refresh gate requires wheel events, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if min_total_refreshes > 0 && total_refreshes < min_total_refreshes {
        return Err(format!(
            "expected virtual list visible-range refreshes to occur after wheel events, but total_refreshes={total_refreshes} was below min_total_refreshes={min_total_refreshes} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_vlist_window_shifts_explainable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_signal = false;
    let mut total_shifts: u64 = 0;
    let mut offenders: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let wheel_frame = first_wheel_frame_id_for_window(w);
        let after_frame = wheel_frame.unwrap_or(warmup_frames).max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let list = s
                .get("debug")
                .and_then(|v| v.get("virtual_list_windows"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if list.is_empty() {
                continue;
            }
            any_signal = true;

            for win in list {
                let mismatch = win
                    .get("window_mismatch")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let kind = win
                    .get("window_shift_kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or(if mismatch { "escape" } else { "none" });
                if kind == "none" {
                    continue;
                }
                total_shifts = total_shifts.saturating_add(1);

                let reason = win.get("window_shift_reason").and_then(|v| v.as_str());
                let mode = win.get("window_shift_apply_mode").and_then(|v| v.as_str());
                let invalidation_detail = win
                    .get("window_shift_invalidation_detail")
                    .and_then(|v| v.as_str());
                if reason.is_some() && mode.is_some() {
                    if mode == Some("non_retained_rerender") {
                        let expected_detail = match reason {
                            Some("scroll_to_item") => {
                                Some("scroll_handle_scroll_to_item_window_update")
                            }
                            Some("viewport_resize") => {
                                Some("scroll_handle_viewport_resize_window_update")
                            }
                            Some("items_revision") => {
                                Some("scroll_handle_items_revision_window_update")
                            }
                            _ => match kind {
                                "escape" => Some("scroll_handle_window_update"),
                                "prefetch" => Some("scroll_handle_prefetch_window_update"),
                                _ => None,
                            },
                        };
                        if invalidation_detail.is_none() {
                            offenders = offenders.saturating_add(1);
                            failures.push(format!(
                                "window={window_id} tick_id={tick_id} frame_id={frame_id} error=missing_shift_invalidation_detail kind={kind} apply_mode={mode:?}"
                            ));
                            if samples.len() < 64 {
                                samples.push(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "kind": kind,
                                    "reason": reason,
                                    "apply_mode": mode,
                                    "invalidation_detail": invalidation_detail,
                                    "expected_invalidation_detail": expected_detail,
                                    "node": win.get("node").and_then(|v| v.as_u64()),
                                    "element": win.get("element").and_then(|v| v.as_u64()),
                                    "policy_key": win.get("policy_key").and_then(|v| v.as_u64()),
                                    "inputs_key": win.get("inputs_key").and_then(|v| v.as_u64()),
                                }));
                            }
                        } else if expected_detail.is_some()
                            && invalidation_detail != expected_detail
                        {
                            offenders = offenders.saturating_add(1);
                            failures.push(format!(
                                "window={window_id} tick_id={tick_id} frame_id={frame_id} error=unexpected_shift_invalidation_detail kind={kind} got={invalidation_detail:?} expected={expected_detail:?}"
                            ));
                            if samples.len() < 64 {
                                samples.push(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "kind": kind,
                                    "reason": reason,
                                    "apply_mode": mode,
                                    "invalidation_detail": invalidation_detail,
                                    "expected_invalidation_detail": expected_detail,
                                    "node": win.get("node").and_then(|v| v.as_u64()),
                                    "element": win.get("element").and_then(|v| v.as_u64()),
                                    "policy_key": win.get("policy_key").and_then(|v| v.as_u64()),
                                    "inputs_key": win.get("inputs_key").and_then(|v| v.as_u64()),
                                }));
                            }
                        }
                    }
                    continue;
                }

                offenders = offenders.saturating_add(1);
                failures.push(format!(
                    "window={window_id} tick_id={tick_id} frame_id={frame_id} error=missing_shift_explainability kind={kind} reason={reason:?} apply_mode={mode:?} invalidation_detail={invalidation_detail:?}"
                ));

                if samples.len() < 64 {
                    samples.push(serde_json::json!({
                        "window": window_id,
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                        "kind": kind,
                        "reason": reason,
                        "apply_mode": mode,
                        "invalidation_detail": invalidation_detail,
                        "node": win.get("node").and_then(|v| v.as_u64()),
                        "element": win.get("element").and_then(|v| v.as_u64()),
                        "policy_key": win.get("policy_key").and_then(|v| v.as_u64()),
                        "inputs_key": win.get("inputs_key").and_then(|v| v.as_u64()),
                        "window_range": win.get("window_range"),
                        "prev_window_range": win.get("prev_window_range"),
                        "render_window_range": win.get("render_window_range"),
                        "deferred_scroll_to_item": win.get("deferred_scroll_to_item").and_then(|v| v.as_bool()),
                        "deferred_scroll_consumed": win.get("deferred_scroll_consumed").and_then(|v| v.as_bool()),
                    }));
                }
            }
        }
    }

    let out_path = out_dir.join("check.vlist_window_shifts_explainable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_window_shifts_explainable",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "total_shifts": total_shifts,
        "offenders": offenders,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_signal {
        return Err(format!(
            "vlist window-shift explainability gate requires debug.virtual_list_windows after warmup, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if offenders == 0 {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("vlist window-shift explainability gate failed (expected every window shift to have reason + apply_mode)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!("evidence: {}\n", out_path.display()));
    for line in failures.into_iter().take(12) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_prepaint_actions_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_snapshots: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_prepaint_actions_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_snapshots,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_prepaint_actions_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_snapshots: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let after_frame = warmup_frames.saturating_add(1);
    let mut snapshots_with_actions: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let actions = s
                .get("debug")
                .and_then(|v| v.get("prepaint_actions"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if actions.is_empty() {
                continue;
            }

            snapshots_with_actions = snapshots_with_actions.saturating_add(1);
            if samples.len() < 32 {
                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "actions_len": actions.len(),
                }));
            }
        }
    }

    let out_path = out_dir.join("check.prepaint_actions_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "prepaint_actions_min",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_snapshots": min_snapshots,
        "snapshots_with_actions": snapshots_with_actions,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if min_snapshots > 0 && snapshots_with_actions < min_snapshots {
        return Err(format!(
            "expected prepaint actions to be recorded in at least min_snapshots={min_snapshots}, but snapshots_with_actions={snapshots_with_actions} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_chart_sampling_window_shifts_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_chart_sampling_window_shifts_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_actions,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_chart_sampling_window_shifts_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let after_frame = warmup_frames.saturating_add(1);
    let mut snapshots_examined: u64 = 0;
    let mut total_actions: u64 = 0;
    let mut unique_keys: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            snapshots_examined = snapshots_examined.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let actions = s
                .get("debug")
                .and_then(|v| v.get("prepaint_actions"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if actions.is_empty() {
                continue;
            }

            for a in actions {
                let kind = a.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                if kind != "chart_sampling_window_shift" {
                    continue;
                }
                total_actions = total_actions.saturating_add(1);

                let key = a
                    .get("chart_sampling_window_key")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if key != 0 {
                    unique_keys.insert(key);
                }

                if samples.len() < 32 {
                    samples.push(serde_json::json!({
                        "window": window_id,
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                        "chart_sampling_window_key": key,
                    }));
                }
            }
        }
    }

    let out_path = out_dir.join("check.chart_sampling_window_shifts_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "chart_sampling_window_shifts_min",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_actions": min_actions,
        "snapshots_examined": snapshots_examined,
        "total_actions": total_actions,
        "unique_keys": unique_keys.into_iter().collect::<Vec<u64>>(),
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if min_actions > 0 && total_actions < min_actions {
        return Err(format!(
            "expected chart sampling window shift actions to be recorded at least min_actions={min_actions}, but total_actions={total_actions} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_node_graph_cull_window_shifts_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_node_graph_cull_window_shifts_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_actions,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_node_graph_cull_window_shifts_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let after_frame = warmup_frames.saturating_add(1);
    let mut snapshots_examined: u64 = 0;
    let mut total_actions: u64 = 0;
    let mut unique_keys: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            snapshots_examined = snapshots_examined.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let actions = s
                .get("debug")
                .and_then(|v| v.get("prepaint_actions"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if actions.is_empty() {
                continue;
            }

            for a in actions {
                let kind = a.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                if kind != "node_graph_cull_window_shift" {
                    continue;
                }
                total_actions = total_actions.saturating_add(1);

                let key = a
                    .get("node_graph_cull_window_key")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if key != 0 {
                    unique_keys.insert(key);
                }

                if samples.len() < 32 {
                    samples.push(serde_json::json!({
                        "window": window_id,
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                        "node_graph_cull_window_key": key,
                    }));
                }
            }
        }
    }

    let out_path = out_dir.join("check.node_graph_cull_window_shifts_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "node_graph_cull_window_shifts_min",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_actions": min_actions,
        "snapshots_examined": snapshots_examined,
        "total_actions": total_actions,
        "unique_keys": unique_keys.into_iter().collect::<Vec<u64>>(),
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if min_actions > 0 && total_actions < min_actions {
        return Err(format!(
            "expected node graph cull window shift actions to be recorded at least min_actions={min_actions}, but total_actions={total_actions} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_node_graph_cull_window_shifts_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_node_graph_cull_window_shifts_max_json(
        &bundle,
        bundle_path,
        out_dir,
        max_actions,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_node_graph_cull_window_shifts_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let after_frame = warmup_frames.saturating_add(1);
    let mut snapshots_examined: u64 = 0;
    let mut total_actions: u64 = 0;
    let mut unique_keys: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            snapshots_examined = snapshots_examined.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let actions = s
                .get("debug")
                .and_then(|v| v.get("prepaint_actions"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if actions.is_empty() {
                continue;
            }

            for a in actions {
                let kind = a.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                if kind != "node_graph_cull_window_shift" {
                    continue;
                }
                total_actions = total_actions.saturating_add(1);

                let key = a
                    .get("node_graph_cull_window_key")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if key != 0 {
                    unique_keys.insert(key);
                }

                if samples.len() < 32 {
                    samples.push(serde_json::json!({
                        "window": window_id,
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                        "node_graph_cull_window_key": key,
                    }));
                }
            }
        }
    }

    let out_path = out_dir.join("check.node_graph_cull_window_shifts_max.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "node_graph_cull_window_shifts_max",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_actions": max_actions,
        "snapshots_examined": snapshots_examined,
        "total_actions": total_actions,
        "unique_keys": unique_keys.into_iter().collect::<Vec<u64>>(),
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if total_actions > max_actions {
        return Err(format!(
            "expected node graph cull window shift actions to stay under max_actions={max_actions}, but total_actions={total_actions} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_vlist_window_shifts_non_retained_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_total_non_retained_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut snapshots_examined: u64 = 0;
    let mut total_non_retained_shifts: u64 = 0;
    let mut total_shifts: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            snapshots_examined = snapshots_examined.saturating_add(1);

            let debug_stats = s.get("debug").and_then(|v| v.get("stats"));
            let window_shifts_total = debug_stats
                .and_then(|v| v.get("virtual_list_window_shifts_total"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let window_shifts_non_retained = debug_stats
                .and_then(|v| v.get("virtual_list_window_shifts_non_retained"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            total_shifts = total_shifts.saturating_add(window_shifts_total);
            if window_shifts_non_retained == 0 {
                continue;
            }
            total_non_retained_shifts =
                total_non_retained_shifts.saturating_add(window_shifts_non_retained);

            if samples.len() < 64 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let shift_samples = s
                    .get("debug")
                    .and_then(|v| v.get("virtual_list_window_shift_samples"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().take(8).cloned().collect::<Vec<_>>())
                    .unwrap_or_default();

                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "non_retained_shifts": window_shifts_non_retained,
                    "window_shifts_total": window_shifts_total,
                    "shift_samples": shift_samples,
                }));
            }
        }
    }

    let out_path = out_dir.join("check.vlist_window_shifts_non_retained_max.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_window_shifts_non_retained_max",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_total_non_retained_shifts": max_total_non_retained_shifts,
        "snapshots_examined": snapshots_examined,
        "total_window_shifts": total_shifts,
        "total_non_retained_shifts": total_non_retained_shifts,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if total_non_retained_shifts > max_total_non_retained_shifts {
        return Err(format!(
            "vlist non-retained window-shift gate failed: total_non_retained_shifts={total_non_retained_shifts} exceeded max_total_non_retained_shifts={max_total_non_retained_shifts} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_vlist_window_shifts_kind_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    kind: &str,
    max_total_kind_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let kind = match kind {
        "prefetch" | "escape" => kind,
        _ => {
            return Err(format!(
                "vlist window-shift kind must be one of: prefetch|escape (got: {kind})"
            ));
        }
    };

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut snapshots_examined: u64 = 0;
    let mut total_kind_shifts: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "invalid bundle.json: missing snapshots".to_string())?;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            snapshots_examined = snapshots_examined.saturating_add(1);

            let shift_entries = s
                .get("debug")
                .and_then(|v| v.get("virtual_list_windows"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter(|w| {
                            w.get("source")
                                .and_then(|v| v.as_str())
                                .is_some_and(|s| s == "prepaint")
                                && w.get("window_shift_kind")
                                    .and_then(|v| v.as_str())
                                    .is_some_and(|k| k == kind)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if shift_entries.is_empty() {
                continue;
            }

            let mut unique_entries: Vec<&serde_json::Value> = Vec::new();
            let mut seen_keys: std::collections::HashSet<(
                Option<u64>,
                Option<u64>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<bool>,
            )> = std::collections::HashSet::new();
            for w in shift_entries {
                let key = (
                    w.get("node").and_then(|v| v.as_u64()),
                    w.get("element").and_then(|v| v.as_u64()),
                    w.get("source")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    w.get("window_shift_kind")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    w.get("window_shift_reason")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    w.get("window_shift_apply_mode")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    w.get("window_mismatch").and_then(|v| v.as_bool()),
                );
                if seen_keys.insert(key) {
                    unique_entries.push(w);
                }
            }
            if unique_entries.is_empty() {
                continue;
            }

            total_kind_shifts = total_kind_shifts.saturating_add(unique_entries.len() as u64);

            if samples.len() < 64 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let entries = unique_entries
                    .iter()
                    .take(4)
                    .map(|w| {
                        serde_json::json!({
                            "node": w.get("node").cloned().unwrap_or(serde_json::Value::Null),
                            "element": w.get("element").cloned().unwrap_or(serde_json::Value::Null),
                            "window_shift_kind": w.get("window_shift_kind").cloned().unwrap_or(serde_json::Value::Null),
                            "window_shift_reason": w.get("window_shift_reason").cloned().unwrap_or(serde_json::Value::Null),
                            "window_shift_apply_mode": w.get("window_shift_apply_mode").cloned().unwrap_or(serde_json::Value::Null),
                            "window_mismatch": w.get("window_mismatch").cloned().unwrap_or(serde_json::Value::Null),
                        })
                    })
                    .collect::<Vec<_>>();

                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "kind": kind,
                    "shifts_in_frame": unique_entries.len(),
                    "entries": entries,
                }));
            }
        }
    }

    let out_path = out_dir.join(format!("check.vlist_window_shifts_{kind}_max.json"));
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": format!("vlist_window_shifts_{kind}_max"),
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_total_kind_shifts": max_total_kind_shifts,
        "snapshots_examined": snapshots_examined,
        "total_kind_shifts": total_kind_shifts,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if total_kind_shifts > max_total_kind_shifts {
        return Err(format!(
            "vlist window-shift kind gate failed: total_{kind}_shifts={total_kind_shifts} exceeded max_total_kind_shifts={max_total_kind_shifts} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_vlist_window_shifts_have_prepaint_actions(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_have_prepaint_actions_json(
        &bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_window_shifts_have_prepaint_actions_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let after_frame = warmup_frames.saturating_add(1);
    let mut offenders: u64 = 0;
    let mut failures: Vec<String> = Vec::new();
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let debug = s.get("debug").unwrap_or(&serde_json::Value::Null);
            let vlist = debug
                .get("virtual_list_windows")
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if vlist.is_empty() {
                continue;
            }
            let actions = debug
                .get("prepaint_actions")
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);

            let shift_actions: Vec<&serde_json::Value> = actions
                .iter()
                .filter(|a| {
                    a.get("kind").and_then(|v| v.as_str()) == Some("virtual_list_window_shift")
                })
                .collect();

            for win in vlist {
                let source = win.get("source").and_then(|v| v.as_str());
                if source != Some("prepaint") {
                    continue;
                }
                let shift_kind = win
                    .get("window_shift_kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("none");
                if shift_kind == "none" {
                    continue;
                }

                let node = win.get("node").and_then(|v| v.as_u64());
                let element = win.get("element").and_then(|v| v.as_u64());
                let shift_reason = win.get("window_shift_reason").and_then(|v| v.as_str());

                let found = shift_actions.iter().any(|a| {
                    let a_node = a.get("node").and_then(|v| v.as_u64());
                    let a_element = a.get("element").and_then(|v| v.as_u64());
                    let a_kind = a
                        .get("virtual_list_window_shift_kind")
                        .and_then(|v| v.as_str());
                    let a_reason = a
                        .get("virtual_list_window_shift_reason")
                        .and_then(|v| v.as_str());

                    a_node == node
                        && a_element == element
                        && a_kind == Some(shift_kind)
                        && (shift_reason.is_none() || a_reason == shift_reason)
                });

                if !found {
                    offenders = offenders.saturating_add(1);
                    failures.push(format!(
                        "window={window_id} tick_id={tick_id} frame_id={frame_id} error=missing_vlist_window_shift_prepaint_action node={node:?} element={element:?} shift_kind={shift_kind} shift_reason={shift_reason:?}"
                    ));
                    if samples.len() < 64 {
                        samples.push(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "node": node,
                            "element": element,
                            "shift_kind": shift_kind,
                            "shift_reason": shift_reason,
                            "available_shift_actions": shift_actions.iter().take(8).map(|a| serde_json::json!({
                                "node": a.get("node").and_then(|v| v.as_u64()),
                                "element": a.get("element").and_then(|v| v.as_u64()),
                                "shift_kind": a.get("virtual_list_window_shift_kind").and_then(|v| v.as_str()),
                                "shift_reason": a.get("virtual_list_window_shift_reason").and_then(|v| v.as_str()),
                            })).collect::<Vec<_>>(),
                        }));
                    }
                }
            }
        }
    }

    let out_path = out_dir.join("check.vlist_window_shifts_have_prepaint_actions.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_window_shifts_have_prepaint_actions",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "offenders": offenders,
        "failures": failures,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if offenders > 0 {
        return Err(format!(
            "vlist window-shift prepaint-action gate failed: offenders={offenders} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut total_refreshes: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let refreshes = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("virtual_list_visible_range_refreshes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if refreshes == 0 {
                continue;
            }
            total_refreshes = total_refreshes.saturating_add(refreshes);
            if samples.len() < 32 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "refreshes": refreshes,
                }));
            }
        }
    }

    let out_path = out_dir.join("check.vlist_visible_range_refreshes_max.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_visible_range_refreshes_max",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_total_refreshes": max_total_refreshes,
        "any_wheel": any_wheel,
        "total_refreshes": total_refreshes,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_wheel {
        return Err(format!(
            "vlist visible-range refresh gate requires wheel events, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if max_total_refreshes > 0 && total_refreshes > max_total_refreshes {
        return Err(format!(
            "expected virtual list visible-range refreshes to stay under budget after wheel events, but total_refreshes={total_refreshes} exceeded max_total_refreshes={max_total_refreshes} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_drag_cache_root_paint_only(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut good_frames: u64 = 0;
    let mut bad_frames: Vec<String> = Vec::new();
    let mut missing_target_count: u64 = 0;
    let mut any_view_cache_active = false;
    let mut seen_good = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if !view_cache_active {
                continue;
            }

            let Some(target_node_id) = semantics_node_id_for_test_id(s, test_id) else {
                missing_target_count = missing_target_count.saturating_add(1);
                continue;
            };

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| "invalid bundle.json: missing debug.semantics.nodes".to_string())?;
            let mut parents: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
            for n in nodes {
                let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
                    continue;
                };
                if let Some(parent) = n.get("parent").and_then(|v| v.as_u64()) {
                    parents.insert(id, parent);
                }
            }

            let roots = s
                .get("debug")
                .and_then(|v| v.get("cache_roots"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| "invalid bundle.json: missing debug.cache_roots".to_string())?;
            let mut cache_roots: std::collections::HashMap<u64, &serde_json::Value> =
                std::collections::HashMap::new();
            for r in roots {
                if let Some(root) = r.get("root").and_then(|v| v.as_u64()) {
                    cache_roots.insert(root, r);
                }
            }

            let mut current = target_node_id;
            let mut cache_root_node: Option<u64> = None;
            loop {
                if cache_roots.contains_key(&current) {
                    cache_root_node = Some(current);
                    break;
                }
                let Some(parent) = parents.get(&current).copied() else {
                    break;
                };
                current = parent;
            }
            let Some(cache_root_node) = cache_root_node else {
                return Err(format!(
                    "could not resolve a cache root ancestor for test_id={test_id} (node_id={target_node_id}) in bundle: {}",
                    bundle_path.display()
                ));
            };

            let root = cache_roots
                .get(&cache_root_node)
                .ok_or_else(|| "internal error: cache root missing".to_string())?;

            let reused = root
                .get("reused")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let contained_relayout_in_frame = root
                .get("contained_relayout_in_frame")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let dirty = s
                .get("debug")
                .and_then(|v| v.get("dirty_views"))
                .and_then(|v| v.as_array())
                .map_or(false, |dirty| {
                    dirty.iter().any(|d| {
                        d.get("root_node")
                            .and_then(|v| v.as_u64())
                            .is_some_and(|n| n == cache_root_node)
                    })
                });

            let ok = reused && !contained_relayout_in_frame && !dirty;
            if ok {
                good_frames = good_frames.saturating_add(1);
                seen_good = true;
                continue;
            }

            if seen_good {
                bad_frames.push(format!(
                    "window={window_id} frame_id={frame_id} cache_root={cache_root_node} reused={reused} contained_relayout_in_frame={contained_relayout_in_frame} dirty={dirty}"
                ));
            }
        }
    }

    if !bad_frames.is_empty() {
        let mut msg = String::new();
        msg.push_str("expected paint-only drag indicator updates (cache-root reuse, no contained relayout, no dirty view), but found violations after reuse began\n");
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!("test_id: {test_id}\n"));
        for line in bad_frames.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if good_frames == 0 {
        return Err(format!(
            "did not observe any cache-root-reuse paint-only frames for test_id={test_id} \
(any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}, missing_target_count={missing_target_count}) \
in bundle: {}",
            bundle_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_gc_sweep_liveness(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut offenders: Vec<String> = Vec::new();
    let mut offender_samples: Vec<serde_json::Value> = Vec::new();
    let mut offender_taxonomy_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut examined_snapshots: u64 = 0;
    let mut removed_subtrees_total: u64 = 0;
    let mut removed_subtrees_offenders: u64 = 0;

    let mut element_runtime_node_entry_root_overwrites_total: u64 = 0;
    let mut element_runtime_view_cache_reuse_root_element_samples_total: u64 = 0;
    let mut element_runtime_retained_keep_alive_roots_total: u64 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let mut snapshot_node_entry_root_overwrites_len: u64 = 0;
            let mut snapshot_view_cache_reuse_root_element_samples_len: u64 = 0;
            let mut snapshot_retained_keep_alive_roots_len: u64 = 0;

            let element_runtime = s
                .get("debug")
                .and_then(|v| v.get("element_runtime"))
                .and_then(|v| v.as_object());
            if let Some(element_runtime) = element_runtime {
                snapshot_node_entry_root_overwrites_len = element_runtime
                    .get("node_entry_root_overwrites")
                    .and_then(|v| v.as_array())
                    .map(|v| v.len() as u64)
                    .unwrap_or(0);
                snapshot_view_cache_reuse_root_element_samples_len = element_runtime
                    .get("view_cache_reuse_root_element_samples")
                    .and_then(|v| v.as_array())
                    .map(|v| v.len() as u64)
                    .unwrap_or(0);
                snapshot_retained_keep_alive_roots_len = element_runtime
                    .get("retained_keep_alive_roots")
                    .and_then(|v| v.as_array())
                    .map(|v| v.len() as u64)
                    .unwrap_or(0);

                element_runtime_node_entry_root_overwrites_total =
                    element_runtime_node_entry_root_overwrites_total
                        .saturating_add(snapshot_node_entry_root_overwrites_len);
                element_runtime_view_cache_reuse_root_element_samples_total =
                    element_runtime_view_cache_reuse_root_element_samples_total
                        .saturating_add(snapshot_view_cache_reuse_root_element_samples_len);
                element_runtime_retained_keep_alive_roots_total =
                    element_runtime_retained_keep_alive_roots_total
                        .saturating_add(snapshot_retained_keep_alive_roots_len);
            }

            let Some(removed) = s
                .get("debug")
                .and_then(|v| v.get("removed_subtrees"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            for r in removed {
                removed_subtrees_total = removed_subtrees_total.saturating_add(1);
                let unreachable = r
                    .get("unreachable_from_liveness_roots")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let reachable_from_layer_roots = r
                    .get("reachable_from_layer_roots")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let reachable_from_view_cache_roots = r
                    .get("reachable_from_view_cache_roots")
                    .and_then(|v| v.as_bool());
                let root_layer_visible = r.get("root_layer_visible").and_then(|v| v.as_bool());
                let reuse_roots_len = r
                    .get("view_cache_reuse_roots_len")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let under_reuse = reuse_roots_len > 0;
                let reuse_root_nodes_len = r
                    .get("view_cache_reuse_root_nodes_len")
                    .and_then(|v| v.as_u64());
                let trigger_in_keep_alive = r
                    .get("trigger_element_in_view_cache_keep_alive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let trigger_listed_under_reuse_root = r
                    .get("trigger_element_listed_under_reuse_root")
                    .and_then(|v| v.as_u64())
                    .is_some();

                let taxonomy_flags: Vec<&'static str> = {
                    let mut flags: Vec<&'static str> = Vec::new();
                    if snapshot_node_entry_root_overwrites_len > 0 {
                        flags.push("ownership_drift_suspected");
                    }
                    if under_reuse && snapshot_view_cache_reuse_root_element_samples_len == 0 {
                        flags.push("missing_reuse_root_membership_samples");
                    }
                    if trigger_in_keep_alive {
                        flags.push("trigger_in_keep_alive");
                    }
                    if under_reuse && trigger_listed_under_reuse_root {
                        flags.push("trigger_listed_under_reuse_root");
                    }
                    if under_reuse && reachable_from_view_cache_roots.is_none() {
                        flags.push("missing_view_cache_reachability_evidence");
                    }
                    if under_reuse && reuse_root_nodes_len == Some(0) {
                        flags.push("reuse_roots_unmapped");
                    }
                    flags
                };

                let taxonomy = if !unreachable
                    || reachable_from_layer_roots
                    || reachable_from_view_cache_roots == Some(true)
                    || root_layer_visible == Some(true)
                {
                    Some("swept_while_reachable")
                } else if under_reuse && reachable_from_view_cache_roots.is_none() {
                    // Under reuse we expect reachability from reuse roots to be recorded; otherwise
                    // the cache-005 harness won't be actionable from a single bundle.
                    Some("missing_view_cache_reachability_evidence")
                } else if under_reuse && reuse_root_nodes_len == Some(0) {
                    // If we know reuse roots exist but cannot map any to nodes, the window's
                    // identity bookkeeping is inconsistent, so "reachable from reuse roots" is
                    // meaningless for that frame.
                    Some("reuse_roots_unmapped")
                } else if trigger_in_keep_alive {
                    // Keep-alive roots are part of the liveness root set. If the record still
                    // indicates a trigger element is in a keep-alive bucket while being swept as
                    // unreachable, that's almost certainly bookkeeping drift.
                    Some("keep_alive_liveness_mismatch")
                } else if under_reuse && snapshot_view_cache_reuse_root_element_samples_len == 0 {
                    // When reuse roots exist, we expect membership list samples to be exported so
                    // cache-005 failures remain actionable from a single bundle.
                    Some("missing_reuse_root_membership_samples")
                } else if under_reuse && trigger_listed_under_reuse_root {
                    // If the trigger element is recorded as being listed under some reuse root,
                    // but the removal happens as unreachable from reuse roots, the membership/touch
                    // logic is likely stale or incomplete.
                    Some("reuse_membership_mismatch")
                } else {
                    None
                };

                if let Some(taxonomy) = taxonomy {
                    removed_subtrees_offenders = removed_subtrees_offenders.saturating_add(1);
                    *offender_taxonomy_counts
                        .entry(taxonomy.to_string())
                        .or_insert(0) += 1;
                    let root = r.get("root").and_then(|v| v.as_u64()).unwrap_or(0);
                    let root_element_path = r
                        .get("root_element_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");
                    let trigger_path = r
                        .get("trigger_element_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");
                    let mut violations: Vec<&'static str> = Vec::new();
                    if taxonomy == "swept_while_reachable" && !unreachable {
                        violations.push("reachable_from_liveness_roots");
                    }
                    if taxonomy == "swept_while_reachable" && reachable_from_layer_roots {
                        violations.push("reachable_from_layer_roots");
                    }
                    if taxonomy == "swept_while_reachable"
                        && reachable_from_view_cache_roots == Some(true)
                    {
                        violations.push("reachable_from_view_cache_roots");
                    }
                    if taxonomy == "swept_while_reachable" && root_layer_visible == Some(true) {
                        violations.push("root_layer_visible");
                    }
                    offenders.push(format!(
                        "window={window_id} frame_id={frame_id} taxonomy={taxonomy} root={root} unreachable_from_liveness_roots={unreachable} reachable_from_layer_roots={reachable_from_layer_roots} reachable_from_view_cache_roots={reachable_from_view_cache_roots:?} root_layer_visible={root_layer_visible:?} reuse_roots_len={reuse_roots_len} reuse_root_nodes_len={reuse_root_nodes_len:?} trigger_in_keep_alive={trigger_in_keep_alive} trigger_listed_under_reuse_root={trigger_listed_under_reuse_root} root_element_path={root_element_path} trigger_element_path={trigger_path}"
                    ));

                    const MAX_SAMPLES: usize = 128;
                    if offender_samples.len() < MAX_SAMPLES {
                        offender_samples.push(serde_json::json!({
                            "window": window_id,
                            "frame_id": frame_id,
                            "tick_id": s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                            "taxonomy": taxonomy,
                            "taxonomy_flags": taxonomy_flags,
                            "root": r.get("root").and_then(|v| v.as_u64()).unwrap_or(0),
                            "root_root": r.get("root_root").and_then(|v| v.as_u64()),
                            "root_layer": r.get("root_layer").and_then(|v| v.as_u64()),
                            "root_layer_visible": root_layer_visible,
                            "reachable_from_layer_roots": reachable_from_layer_roots,
                            "reachable_from_view_cache_roots": reachable_from_view_cache_roots,
                            "unreachable_from_liveness_roots": unreachable,
                            "violations": violations,
                            "reuse_roots_len": reuse_roots_len,
                            "reuse_root_nodes_len": reuse_root_nodes_len,
                            "trigger_in_keep_alive": trigger_in_keep_alive,
                            "trigger_listed_under_reuse_root": trigger_listed_under_reuse_root,
                            "liveness_layer_roots_len": r.get("liveness_layer_roots_len").and_then(|v| v.as_u64()),
                            "view_cache_reuse_roots_len": r.get("view_cache_reuse_roots_len").and_then(|v| v.as_u64()),
                            "view_cache_reuse_root_nodes_len": r.get("view_cache_reuse_root_nodes_len").and_then(|v| v.as_u64()),
                            "snapshot_node_entry_root_overwrites_len": snapshot_node_entry_root_overwrites_len,
                            "snapshot_view_cache_reuse_root_element_samples_len": snapshot_view_cache_reuse_root_element_samples_len,
                            "snapshot_retained_keep_alive_roots_len": snapshot_retained_keep_alive_roots_len,
                            "root_element": r.get("root_element").and_then(|v| v.as_u64()),
                            "root_element_path": r.get("root_element_path").and_then(|v| v.as_str()),
                            "trigger_element": r.get("trigger_element").and_then(|v| v.as_u64()),
                            "trigger_element_path": r.get("trigger_element_path").and_then(|v| v.as_str()),
                            "trigger_element_in_view_cache_keep_alive": r.get("trigger_element_in_view_cache_keep_alive").and_then(|v| v.as_bool()),
                            "trigger_element_listed_under_reuse_root": r.get("trigger_element_listed_under_reuse_root").and_then(|v| v.as_u64()),
                            "root_root_parent_sever_parent": r.get("root_root_parent_sever_parent").and_then(|v| v.as_u64()),
                            "root_root_parent_sever_location": r.get("root_root_parent_sever_location").and_then(|v| v.as_str()),
                            "root_root_parent_sever_frame_id": r.get("root_root_parent_sever_frame_id").and_then(|v| v.as_u64()),
                        }));
                    }
                }
            }
        }
    }

    // Always write evidence so debugging doesn't require re-running the harness.
    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.gc_sweep_liveness.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "gc_sweep_liveness",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "removed_subtrees_total": removed_subtrees_total,
        "removed_subtrees_offenders": removed_subtrees_offenders,
        "offender_taxonomy_counts": offender_taxonomy_counts,
        "offender_samples": offender_samples,
        "debug_summary": {
            "element_runtime_node_entry_root_overwrites_total": element_runtime_node_entry_root_overwrites_total,
            "element_runtime_view_cache_reuse_root_element_samples_total": element_runtime_view_cache_reuse_root_element_samples_total,
            "element_runtime_retained_keep_alive_roots_total": element_runtime_retained_keep_alive_roots_total,
        },
    });
    write_json_value(&evidence_path, &payload)?;

    if offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("GC sweep liveness violation: removed_subtrees contains entries that appear live or inconsistent with keep-alive/reuse bookkeeping\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
    ));
    msg.push_str(&format!("evidence: {}\n", evidence_path.display()));
    for line in offenders.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_view_cache_reuse_min(
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_view_cache_reuse_min_json(
        &bundle,
        bundle_path,
        min_reuse_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_view_cache_reuse_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reuse_events: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut any_view_cache_active = false;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if !view_cache_active {
                continue;
            }

            let roots = s
                .get("debug")
                .and_then(|v| v.get("cache_roots"))
                .and_then(|v| v.as_array());
            let Some(roots) = roots else {
                continue;
            };

            for r in roots {
                if r.get("reused").and_then(|v| v.as_bool()) == Some(true) {
                    reuse_events = reuse_events.saturating_add(1);
                    if reuse_events >= min_reuse_events {
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_reuse_events} view-cache reuse events, got {reuse_events} \
 (any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) \
 in bundle: {}",
        bundle_path.display()
    ))
}

#[derive(Debug, Clone)]
struct ViewCacheReuseStableWindowReport {
    window: u64,
    examined_snapshots: u64,
    view_cache_active_snapshots: u64,
    non_reuse_cache_inactive_snapshots: u64,
    non_reuse_active_no_signal_snapshots: u64,
    reuse_snapshots: u64,
    reuse_streak_max: u64,
    reuse_streak_tail: u64,
    last_non_reuse: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy)]
struct ViewCacheReuseSignal {
    view_cache_active: bool,
    has_reuse_signal: bool,
    reused_roots: u64,
    paint_cache_replayed_ops: u64,
    cache_roots_present: bool,
}

impl ViewCacheReuseSignal {
    fn no_signal_reason(self) -> &'static str {
        if !self.view_cache_active {
            return "view_cache_inactive";
        }
        "active_no_signal"
    }
}

fn snapshot_view_cache_reuse_signal(snapshot: &serde_json::Value) -> ViewCacheReuseSignal {
    let stats = snapshot.get("debug").and_then(|v| v.get("stats"));
    let view_cache_active = stats
        .and_then(|v| v.get("view_cache_active"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let replayed_ops = stats
        .and_then(|v| v.get("paint_cache_replayed_ops"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let mut reused_roots: u64 = 0;
    let mut cache_roots_present = false;
    if let Some(roots) = snapshot
        .get("debug")
        .and_then(|v| v.get("cache_roots"))
        .and_then(|v| v.as_array())
    {
        cache_roots_present = true;
        for r in roots {
            if r.get("reused").and_then(|v| v.as_bool()) == Some(true) {
                reused_roots = reused_roots.saturating_add(1);
            }
        }
    }

    let has_signal = view_cache_active && (reused_roots > 0 || replayed_ops > 0);
    ViewCacheReuseSignal {
        view_cache_active,
        has_reuse_signal: has_signal,
        reused_roots,
        paint_cache_replayed_ops: replayed_ops,
        cache_roots_present,
    }
}

pub(super) fn check_bundle_for_view_cache_reuse_stable_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_tail_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reports: Vec<ViewCacheReuseStableWindowReport> = Vec::new();
    let mut failures: Vec<serde_json::Value> = Vec::new();

    let mut any_view_cache_active = false;
    let mut best_tail: u64 = 0;

    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut examined_snapshots: u64 = 0;
        let mut view_cache_active_snapshots: u64 = 0;
        let mut non_reuse_cache_inactive_snapshots: u64 = 0;
        let mut non_reuse_active_no_signal_snapshots: u64 = 0;
        let mut reuse_snapshots: u64 = 0;
        let mut reuse_streak: u64 = 0;
        let mut reuse_streak_max: u64 = 0;
        let mut last_non_reuse: Option<serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let stats = s.get("debug").and_then(|v| v.get("stats"));
            let view_cache_active = stats
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if view_cache_active {
                view_cache_active_snapshots = view_cache_active_snapshots.saturating_add(1);
            }

            let signal = snapshot_view_cache_reuse_signal(s);
            if signal.has_reuse_signal {
                reuse_snapshots = reuse_snapshots.saturating_add(1);
                reuse_streak = reuse_streak.saturating_add(1);
                reuse_streak_max = reuse_streak_max.max(reuse_streak);
            } else {
                reuse_streak = 0;
                match signal.no_signal_reason() {
                    "view_cache_inactive" => {
                        non_reuse_cache_inactive_snapshots =
                            non_reuse_cache_inactive_snapshots.saturating_add(1);
                    }
                    _ => {
                        non_reuse_active_no_signal_snapshots =
                            non_reuse_active_no_signal_snapshots.saturating_add(1);
                    }
                }
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                last_non_reuse = Some(serde_json::json!({
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "reason": signal.no_signal_reason(),
                    "view_cache_active": signal.view_cache_active,
                    "cache_roots_present": signal.cache_roots_present,
                    "reused_roots": signal.reused_roots,
                    "paint_cache_replayed_ops": signal.paint_cache_replayed_ops,
                }));
            }
        }

        best_tail = best_tail.max(reuse_streak);

        reports.push(ViewCacheReuseStableWindowReport {
            window,
            examined_snapshots,
            view_cache_active_snapshots,
            non_reuse_cache_inactive_snapshots,
            non_reuse_active_no_signal_snapshots,
            reuse_snapshots,
            reuse_streak_max,
            reuse_streak_tail: reuse_streak,
            last_non_reuse: last_non_reuse.clone(),
        });

        if min_tail_frames > 0 && examined_snapshots < min_tail_frames {
            failures.push(serde_json::json!({
                "window": window,
                "reason": "insufficient_snapshots",
                "examined_snapshots": examined_snapshots,
            }));
        } else if min_tail_frames > 0 && reuse_streak < min_tail_frames {
            failures.push(serde_json::json!({
                "window": window,
                "reason": "reuse_tail_streak_too_small",
                "examined_snapshots": examined_snapshots,
                "view_cache_active_snapshots": view_cache_active_snapshots,
                "non_reuse_cache_inactive_snapshots": non_reuse_cache_inactive_snapshots,
                "non_reuse_active_no_signal_snapshots": non_reuse_active_no_signal_snapshots,
                "reuse_streak_tail": reuse_streak,
                "reuse_streak_max": reuse_streak_max,
                "reuse_snapshots": reuse_snapshots,
                "last_non_reuse": last_non_reuse,
            }));
        }
    }

    let out_path = out_dir.join("check.view_cache_reuse_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "view_cache_reuse_stable",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_tail_frames": min_tail_frames,
        "any_view_cache_active": any_view_cache_active,
        "best_reuse_streak_tail": best_tail,
        "windows": reports.iter().map(|r| serde_json::json!({
            "window": r.window,
            "examined_snapshots": r.examined_snapshots,
            "view_cache_active_snapshots": r.view_cache_active_snapshots,
            "non_reuse_cache_inactive_snapshots": r.non_reuse_cache_inactive_snapshots,
            "non_reuse_active_no_signal_snapshots": r.non_reuse_active_no_signal_snapshots,
            "reuse_snapshots": r.reuse_snapshots,
            "reuse_streak_max": r.reuse_streak_max,
            "reuse_streak_tail": r.reuse_streak_tail,
            "last_non_reuse": r.last_non_reuse,
        })).collect::<Vec<_>>(),
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    if min_tail_frames == 0 {
        return Ok(());
    }
    if !any_view_cache_active {
        return Err(format!(
            "view-cache reuse stable gate requires view_cache_active snapshots, but none were observed (warmup_frames={warmup_frames})\n  hint: enable view-cache for the target demo if applicable (e.g. UI gallery: FRET_UI_GALLERY_VIEW_CACHE=1)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }
    if best_tail >= min_tail_frames {
        return Ok(());
    }

    Err(format!(
        "view-cache reuse stable gate failed (min_tail_frames={min_tail_frames}, best_tail={best_tail}, warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

pub(super) fn check_bundle_for_overlay_synthesis_min(
    bundle_path: &Path,
    min_synthesized_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_overlay_synthesis_min_json(
        &bundle,
        bundle_path,
        min_synthesized_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_overlay_synthesis_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_synthesized_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut synthesized_events: u64 = 0;
    let mut suppression_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    let mut examined_snapshots: u64 = 0;
    let mut any_view_cache_active = false;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;

            let Some(events) = s
                .get("debug")
                .and_then(|v| v.get("overlay_synthesis"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            for e in events {
                let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("unknown");
                let outcome = e
                    .get("outcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                if outcome == "synthesized" {
                    synthesized_events = synthesized_events.saturating_add(1);
                    if synthesized_events >= min_synthesized_events {
                        return Ok(());
                    }
                } else {
                    let key = format!("{kind}/{outcome}");
                    *suppression_counts.entry(key).or_insert(0) += 1;
                }
            }
        }
    }

    let mut suppressions: Vec<(String, u64)> = suppression_counts.into_iter().collect();
    suppressions.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    suppressions.truncate(12);
    let suppressions = if suppressions.is_empty() {
        String::new()
    } else {
        let mut msg = String::new();
        msg.push_str(" suppressions=[");
        for (idx, (k, c)) in suppressions.into_iter().enumerate() {
            if idx > 0 {
                msg.push_str(", ");
            }
            msg.push_str(&format!("{k}:{c}"));
        }
        msg.push(']');
        msg
    };

    Err(format!(
        "expected at least {min_synthesized_events} overlay synthesis events, got {synthesized_events} \
(any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}).{suppressions} \
bundle: {}",
        bundle_path.display()
    ))
}

pub(super) fn check_bundle_for_retained_vlist_reconcile_no_notify_min(
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
        &bundle,
        bundle_path,
        min_reconcile_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reconcile_events: u64 = 0;
    let mut reconcile_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut notify_offenders: Vec<String> = Vec::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let list_count = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let stats_count = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let count = list_count.max(stats_count);
            if count == 0 {
                continue;
            }

            reconcile_frames = reconcile_frames.saturating_add(1);
            reconcile_events = reconcile_events.saturating_add(count);

            let dirty_views = s
                .get("debug")
                .and_then(|v| v.get("dirty_views"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);

            for dv in dirty_views {
                let source = dv
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let detail = dv
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if source == "notify" || detail.contains("notify") {
                    let root_node = dv.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0);
                    notify_offenders.push(format!(
                        "frame_id={frame_id} dirty_view_root_node={root_node} source={source} detail={detail}"
                    ));
                    break;
                }
            }
        }
    }

    if !notify_offenders.is_empty() {
        let mut msg = String::new();
        msg.push_str(
            "retained virtual-list reconcile should not require notify-based dirty views\n",
        );
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!(
            "min_reconcile_events={min_reconcile_events} reconcile_events={reconcile_events} reconcile_frames={reconcile_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
        ));
        for line in notify_offenders.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if reconcile_events < min_reconcile_events {
        return Err(format!(
            "expected at least {min_reconcile_events} retained virtual-list reconcile events, got {reconcile_events} \
(reconcile_frames={reconcile_frames}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) \
bundle: {}",
            bundle_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_retained_vlist_attach_detach_max(
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_attach_detach_max_json(
        &bundle,
        bundle_path,
        max_delta,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_attach_detach_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reconcile_events: u64 = 0;
    let mut reconcile_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut offenders: Vec<String> = Vec::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let list_count = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let stats_count = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let count = list_count.max(stats_count);
            if count == 0 {
                continue;
            }

            reconcile_frames = reconcile_frames.saturating_add(1);
            reconcile_events = reconcile_events.saturating_add(count);

            let records = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            let (attached, detached) = if records.is_empty() {
                let stats = s
                    .get("debug")
                    .and_then(|v| v.get("stats"))
                    .and_then(|v| v.as_object());
                let attached = stats
                    .and_then(|v| v.get("retained_virtual_list_attached_items"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let detached = stats
                    .and_then(|v| v.get("retained_virtual_list_detached_items"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                (attached, detached)
            } else {
                let attached = records
                    .iter()
                    .map(|r| {
                        r.get("attached_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                let detached = records
                    .iter()
                    .map(|r| {
                        r.get("detached_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                (attached, detached)
            };

            let delta = attached.saturating_add(detached);
            if delta > max_delta {
                offenders.push(format!(
                    "frame_id={frame_id} attached={attached} detached={detached} delta={delta} max={max_delta}"
                ));
            }
        }
    }

    if reconcile_events == 0 {
        return Err(format!(
            "expected at least 1 retained virtual-list reconcile event (required for attach/detach max check), got 0 \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
            bundle_path.display()
        ));
    }

    if offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("retained virtual-list attach/detach delta exceeded the configured maximum\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "max_delta={max_delta} reconcile_events={reconcile_events} reconcile_frames={reconcile_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
    ));
    for line in offenders.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_viewport_input_min(
    bundle_path: &Path,
    min_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_viewport_input_min_json(&bundle, bundle_path, min_events, warmup_frames)
}

pub(super) fn check_bundle_for_viewport_input_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut events: u64 = 0;
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(arr) = s
                .get("debug")
                .and_then(|v| v.get("viewport_input"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            events = events.saturating_add(arr.len() as u64);
            if events >= min_events {
                return Ok(());
            }
        }
    }

    Err(format!(
        "expected at least {min_events} viewport input events, got {events} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
        bundle_path.display()
    ))
}

pub(super) fn check_bundle_for_dock_drag_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_dock_drag_min_json(&bundle, bundle_path, min_active_frames, warmup_frames)
}

pub(super) fn check_bundle_for_dock_drag_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut active_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(dock_drag) = s
                .get("debug")
                .and_then(|v| v.get("docking_interaction"))
                .and_then(|v| v.get("dock_drag"))
            else {
                continue;
            };
            if dock_drag.is_object() {
                active_frames = active_frames.saturating_add(1);
                if active_frames >= min_active_frames {
                    return Ok(());
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_active_frames} snapshots with an active dock drag, got {active_frames} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
        bundle_path.display()
    ))
}

pub(super) fn check_bundle_for_viewport_capture_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_viewport_capture_min_json(
        &bundle,
        bundle_path,
        min_active_frames,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_viewport_capture_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut active_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(viewport_capture) = s
                .get("debug")
                .and_then(|v| v.get("docking_interaction"))
                .and_then(|v| v.get("viewport_capture"))
            else {
                continue;
            };
            if viewport_capture.is_object() {
                active_frames = active_frames.saturating_add(1);
                if active_frames >= min_active_frames {
                    return Ok(());
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_active_frames} snapshots with an active viewport capture, got {active_frames} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
        bundle_path.display()
    ))
}

pub(super) fn bundle_stats_from_json_with_options(
    bundle: &serde_json::Value,
    top: usize,
    sort: BundleStatsSort,
    opts: BundleStatsOptions,
) -> Result<BundleStatsReport, String> {
    use std::collections::HashSet;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut out = BundleStatsReport::default();
    out.sort = sort;
    out.warmup_frames = opts.warmup_frames;
    out.windows = windows.len().min(u32::MAX as usize) as u32;

    let mut rows: Vec<BundleStatsSnapshotRow> = Vec::new();
    let mut global_type_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    let mut model_source_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let pointer_move_frame_ids: HashSet<u64> = w
            .get("events")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| {
                        let kind = e.get("kind").and_then(|v| v.as_str())?;
                        if kind != "pointer.move" {
                            return None;
                        }
                        e.get("frame_id").and_then(|v| v.as_u64())
                    })
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        if !pointer_move_frame_ids.is_empty() {
            out.pointer_move_frames_present = true;
        }
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            out.snapshots = out.snapshots.saturating_add(1);
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < opts.warmup_frames {
                out.snapshots_skipped_warmup = out.snapshots_skipped_warmup.saturating_add(1);
                continue;
            }
            out.snapshots_considered = out.snapshots_considered.saturating_add(1);

            let changed_models = s
                .get("changed_models")
                .and_then(|v| v.as_array())
                .map(|v| v.len())
                .unwrap_or(0)
                .min(u32::MAX as usize) as u32;
            let changed_globals_arr = s
                .get("changed_globals")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let changed_globals = changed_globals_arr.len().min(u32::MAX as usize) as u32;
            let mut changed_global_types_sample: Vec<String> = Vec::new();
            for (idx, g) in changed_globals_arr.iter().enumerate() {
                let Some(ty) = g.as_str() else {
                    continue;
                };
                *global_type_counts.entry(ty.to_string()).or_insert(0) += 1;
                if idx < 6 {
                    changed_global_types_sample.push(ty.to_string());
                }
            }

            if let Some(arr) = s
                .get("changed_model_sources_top")
                .and_then(|v| v.as_array())
            {
                for item in arr {
                    let Some(type_name) = item.get("type_name").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(at) = item.get("changed_at").and_then(|v| v.as_object()) else {
                        continue;
                    };
                    let Some(file) = at.get("file").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(line) = at.get("line").and_then(|v| v.as_u64()) else {
                        continue;
                    };
                    let Some(column) = at.get("column").and_then(|v| v.as_u64()) else {
                        continue;
                    };
                    let count = item.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                    let key = format!("{}@{}:{}:{}", type_name, file, line, column);
                    *model_source_counts.entry(key).or_insert(0) += count;
                }
            }

            if changed_models > 0 {
                out.snapshots_with_model_changes =
                    out.snapshots_with_model_changes.saturating_add(1);
            }
            if changed_globals > 0 {
                out.snapshots_with_global_changes =
                    out.snapshots_with_global_changes.saturating_add(1);
            }

            let stats = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.as_object());

            let frame_arena_capacity_estimate_bytes = stats
                .and_then(|m| m.get("frame_arena_capacity_estimate_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let frame_arena_grow_events = stats
                .and_then(|m| m.get("frame_arena_grow_events"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let element_children_vec_pool_reuses = stats
                .and_then(|m| m.get("element_children_vec_pool_reuses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let element_children_vec_pool_misses = stats
                .and_then(|m| m.get("element_children_vec_pool_misses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;

            let layout_time_us = stats
                .and_then(|m| m.get("layout_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let prepaint_time_us = stats
                .and_then(|m| m.get("prepaint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_time_us = stats
                .and_then(|m| m.get("paint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_record_visual_bounds_time_us = stats
                .and_then(|m| m.get("paint_record_visual_bounds_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_record_visual_bounds_calls = stats
                .and_then(|m| m.get("paint_record_visual_bounds_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let paint_cache_key_time_us = stats
                .and_then(|m| m.get("paint_cache_key_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_cache_hit_check_time_us = stats
                .and_then(|m| m.get("paint_cache_hit_check_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_widget_time_us = stats
                .and_then(|m| m.get("paint_widget_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_observation_record_time_us = stats
                .and_then(|m| m.get("paint_observation_record_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_host_widget_observed_models_time_us = stats
                .and_then(|m| m.get("paint_host_widget_observed_models_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_host_widget_observed_models_items = stats
                .and_then(|m| m.get("paint_host_widget_observed_models_items"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_host_widget_observed_globals_time_us = stats
                .and_then(|m| m.get("paint_host_widget_observed_globals_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_host_widget_observed_globals_items = stats
                .and_then(|m| m.get("paint_host_widget_observed_globals_items"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_host_widget_instance_lookup_time_us = stats
                .and_then(|m| m.get("paint_host_widget_instance_lookup_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_host_widget_instance_lookup_calls = stats
                .and_then(|m| m.get("paint_host_widget_instance_lookup_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_text_prepare_time_us = stats
                .and_then(|m| m.get("paint_text_prepare_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_text_prepare_calls = stats
                .and_then(|m| m.get("paint_text_prepare_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let paint_text_prepare_reason_blob_missing = stats
                .and_then(|m| m.get("paint_text_prepare_reason_blob_missing"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_text_prepare_reason_scale_changed = stats
                .and_then(|m| m.get("paint_text_prepare_reason_scale_changed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_text_prepare_reason_text_changed = stats
                .and_then(|m| m.get("paint_text_prepare_reason_text_changed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_text_prepare_reason_rich_changed = stats
                .and_then(|m| m.get("paint_text_prepare_reason_rich_changed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_text_prepare_reason_style_changed = stats
                .and_then(|m| m.get("paint_text_prepare_reason_style_changed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_text_prepare_reason_wrap_changed = stats
                .and_then(|m| m.get("paint_text_prepare_reason_wrap_changed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_text_prepare_reason_overflow_changed = stats
                .and_then(|m| m.get("paint_text_prepare_reason_overflow_changed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_text_prepare_reason_width_changed = stats
                .and_then(|m| m.get("paint_text_prepare_reason_width_changed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_text_prepare_reason_font_stack_changed = stats
                .and_then(|m| m.get("paint_text_prepare_reason_font_stack_changed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_input_context_time_us = stats
                .and_then(|m| m.get("paint_input_context_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_scroll_handle_invalidation_time_us = stats
                .and_then(|m| m.get("paint_scroll_handle_invalidation_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_collect_roots_time_us = stats
                .and_then(|m| m.get("paint_collect_roots_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_publish_text_input_snapshot_time_us = stats
                .and_then(|m| m.get("paint_publish_text_input_snapshot_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_collapse_observations_time_us = stats
                .and_then(|m| m.get("paint_collapse_observations_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_time_us = stats
                .and_then(|m| m.get("dispatch_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_pointer_events = stats
                .and_then(|m| m.get("dispatch_pointer_events"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let dispatch_pointer_event_time_us = stats
                .and_then(|m| m.get("dispatch_pointer_event_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_timer_events = stats
                .and_then(|m| m.get("dispatch_timer_events"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let dispatch_timer_event_time_us = stats
                .and_then(|m| m.get("dispatch_timer_event_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_timer_targeted_events = stats
                .and_then(|m| m.get("dispatch_timer_targeted_events"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let dispatch_timer_targeted_time_us = stats
                .and_then(|m| m.get("dispatch_timer_targeted_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_timer_broadcast_events = stats
                .and_then(|m| m.get("dispatch_timer_broadcast_events"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let dispatch_timer_broadcast_time_us = stats
                .and_then(|m| m.get("dispatch_timer_broadcast_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_timer_broadcast_layers_visited = stats
                .and_then(|m| m.get("dispatch_timer_broadcast_layers_visited"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let dispatch_timer_broadcast_rebuild_visible_layers_time_us = stats
                .and_then(|m| m.get("dispatch_timer_broadcast_rebuild_visible_layers_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_timer_broadcast_loop_time_us = stats
                .and_then(|m| m.get("dispatch_timer_broadcast_loop_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_timer_slowest_event_time_us = stats
                .and_then(|m| m.get("dispatch_timer_slowest_event_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_timer_slowest_token = stats
                .and_then(|m| m.get("dispatch_timer_slowest_token"))
                .and_then(|v| v.as_u64());
            let dispatch_timer_slowest_was_broadcast = stats
                .and_then(|m| m.get("dispatch_timer_slowest_was_broadcast"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let dispatch_other_events = stats
                .and_then(|m| m.get("dispatch_other_events"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let dispatch_other_event_time_us = stats
                .and_then(|m| m.get("dispatch_other_event_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let hit_test_time_us = stats
                .and_then(|m| m.get("hit_test_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_hover_update_time_us = stats
                .and_then(|m| m.get("dispatch_hover_update_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_scroll_handle_invalidation_time_us = stats
                .and_then(|m| m.get("dispatch_scroll_handle_invalidation_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_active_layers_time_us = stats
                .and_then(|m| m.get("dispatch_active_layers_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_input_context_time_us = stats
                .and_then(|m| m.get("dispatch_input_context_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_event_chain_build_time_us = stats
                .and_then(|m| m.get("dispatch_event_chain_build_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_widget_capture_time_us = stats
                .and_then(|m| m.get("dispatch_widget_capture_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_widget_bubble_time_us = stats
                .and_then(|m| m.get("dispatch_widget_bubble_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_cursor_query_time_us = stats
                .and_then(|m| m.get("dispatch_cursor_query_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_pointer_move_layer_observers_time_us = stats
                .and_then(|m| m.get("dispatch_pointer_move_layer_observers_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_synth_hover_observer_time_us = stats
                .and_then(|m| m.get("dispatch_synth_hover_observer_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_cursor_effect_time_us = stats
                .and_then(|m| m.get("dispatch_cursor_effect_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_post_dispatch_snapshot_time_us = stats
                .and_then(|m| m.get("dispatch_post_dispatch_snapshot_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_events = stats
                .and_then(|m| m.get("dispatch_events"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hit_test_queries = stats
                .and_then(|m| m.get("hit_test_queries"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hit_test_bounds_tree_queries = stats
                .and_then(|m| m.get("hit_test_bounds_tree_queries"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hit_test_bounds_tree_disabled = stats
                .and_then(|m| m.get("hit_test_bounds_tree_disabled"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hit_test_bounds_tree_misses = stats
                .and_then(|m| m.get("hit_test_bounds_tree_misses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hit_test_bounds_tree_hits = stats
                .and_then(|m| m.get("hit_test_bounds_tree_hits"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hit_test_bounds_tree_candidate_rejected = stats
                .and_then(|m| m.get("hit_test_bounds_tree_candidate_rejected"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let hit_test_cached_path_time_us = stats
                .and_then(|m| m.get("hit_test_cached_path_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let hit_test_bounds_tree_query_time_us = stats
                .and_then(|m| m.get("hit_test_bounds_tree_query_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let hit_test_candidate_self_only_time_us = stats
                .and_then(|m| m.get("hit_test_candidate_self_only_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let hit_test_fallback_traversal_time_us = stats
                .and_then(|m| m.get("hit_test_fallback_traversal_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let total_time_us = layout_time_us
                .saturating_add(prepaint_time_us)
                .saturating_add(paint_time_us);
            let layout_nodes_performed = stats
                .and_then(|m| m.get("layout_nodes_performed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let paint_nodes_performed = stats
                .and_then(|m| m.get("paint_nodes_performed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let paint_cache_misses = stats
                .and_then(|m| m.get("paint_cache_misses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let paint_cache_replay_time_us = stats
                .and_then(|m| m.get("paint_cache_replay_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_cache_bounds_translate_time_us = stats
                .and_then(|m| m.get("paint_cache_bounds_translate_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_cache_bounds_translated_nodes = stats
                .and_then(|m| m.get("paint_cache_bounds_translated_nodes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let renderer_tick_id = stats
                .and_then(|m| m.get("renderer_tick_id"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_frame_id = stats
                .and_then(|m| m.get("renderer_frame_id"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_us = stats
                .and_then(|m| m.get("renderer_encode_scene_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_us = stats
                .and_then(|m| m.get("renderer_prepare_text_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_svg_us = stats
                .and_then(|m| m.get("renderer_prepare_svg_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_upload_bytes = stats
                .and_then(|m| m.get("renderer_svg_upload_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_image_upload_bytes = stats
                .and_then(|m| m.get("renderer_image_upload_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_raster_budget_bytes = stats
                .and_then(|m| m.get("renderer_svg_raster_budget_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_rasters_live = stats
                .and_then(|m| m.get("renderer_svg_rasters_live"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_standalone_bytes_live = stats
                .and_then(|m| m.get("renderer_svg_standalone_bytes_live"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_mask_atlas_pages_live = stats
                .and_then(|m| m.get("renderer_svg_mask_atlas_pages_live"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_mask_atlas_bytes_live = stats
                .and_then(|m| m.get("renderer_svg_mask_atlas_bytes_live"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_mask_atlas_used_px = stats
                .and_then(|m| m.get("renderer_svg_mask_atlas_used_px"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_mask_atlas_capacity_px = stats
                .and_then(|m| m.get("renderer_svg_mask_atlas_capacity_px"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_raster_cache_hits = stats
                .and_then(|m| m.get("renderer_svg_raster_cache_hits"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_raster_cache_misses = stats
                .and_then(|m| m.get("renderer_svg_raster_cache_misses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_raster_budget_evictions = stats
                .and_then(|m| m.get("renderer_svg_raster_budget_evictions"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_mask_atlas_page_evictions = stats
                .and_then(|m| m.get("renderer_svg_mask_atlas_page_evictions"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_svg_mask_atlas_entries_evicted = stats
                .and_then(|m| m.get("renderer_svg_mask_atlas_entries_evicted"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_text_atlas_upload_bytes = stats
                .and_then(|m| m.get("renderer_text_atlas_upload_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_text_atlas_evicted_pages = stats
                .and_then(|m| m.get("renderer_text_atlas_evicted_pages"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_budget_bytes = stats
                .and_then(|m| m.get("renderer_intermediate_budget_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_in_use_bytes = stats
                .and_then(|m| m.get("renderer_intermediate_in_use_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_peak_in_use_bytes = stats
                .and_then(|m| m.get("renderer_intermediate_peak_in_use_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_release_targets = stats
                .and_then(|m| m.get("renderer_intermediate_release_targets"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_pool_allocations = stats
                .and_then(|m| m.get("renderer_intermediate_pool_allocations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_pool_reuses = stats
                .and_then(|m| m.get("renderer_intermediate_pool_reuses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_pool_releases = stats
                .and_then(|m| m.get("renderer_intermediate_pool_releases"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_pool_evictions = stats
                .and_then(|m| m.get("renderer_intermediate_pool_evictions"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_pool_free_bytes = stats
                .and_then(|m| m.get("renderer_intermediate_pool_free_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_intermediate_pool_free_textures = stats
                .and_then(|m| m.get("renderer_intermediate_pool_free_textures"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_draw_calls = stats
                .and_then(|m| m.get("renderer_draw_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_pipeline_switches = stats
                .and_then(|m| m.get("renderer_pipeline_switches"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_bind_group_switches = stats
                .and_then(|m| m.get("renderer_bind_group_switches"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_scissor_sets = stats
                .and_then(|m| m.get("renderer_scissor_sets"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_scene_encoding_cache_misses = stats
                .and_then(|m| m.get("renderer_scene_encoding_cache_misses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_engine_solves = stats
                .and_then(|m| m.get("layout_engine_solves"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_engine_solve_time_us = stats
                .and_then(|m| m.get("layout_engine_solve_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_collect_roots_time_us = stats
                .and_then(|m| m.get("layout_collect_roots_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_invalidate_scroll_handle_bindings_time_us = stats
                .and_then(|m| m.get("layout_invalidate_scroll_handle_bindings_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_expand_view_cache_invalidations_time_us = stats
                .and_then(|m| m.get("layout_expand_view_cache_invalidations_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_request_build_roots_time_us = stats
                .and_then(|m| m.get("layout_request_build_roots_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_pending_barrier_relayouts_time_us = stats
                .and_then(|m| m.get("layout_pending_barrier_relayouts_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_repair_view_cache_bounds_time_us = stats
                .and_then(|m| m.get("layout_repair_view_cache_bounds_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_contained_view_cache_roots_time_us = stats
                .and_then(|m| m.get("layout_contained_view_cache_roots_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_collapse_layout_observations_time_us = stats
                .and_then(|m| m.get("layout_collapse_layout_observations_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_prepaint_after_layout_time_us = stats
                .and_then(|m| m.get("layout_prepaint_after_layout_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_skipped_engine_frame = stats
                .and_then(|m| m.get("layout_skipped_engine_frame"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let view_cache_contained_relayouts = stats
                .and_then(|m| m.get("view_cache_contained_relayouts"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let view_cache_roots_total = stats
                .and_then(|m| m.get("view_cache_roots_total"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let view_cache_roots_reused = stats
                .and_then(|m| m.get("view_cache_roots_reused"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let view_cache_roots_first_mount = stats
                .and_then(|m| m.get("view_cache_roots_first_mount"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let view_cache_roots_node_recreated = stats
                .and_then(|m| m.get("view_cache_roots_node_recreated"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let view_cache_roots_cache_key_mismatch = stats
                .and_then(|m| m.get("view_cache_roots_cache_key_mismatch"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let view_cache_roots_not_marked_reuse_root = stats
                .and_then(|m| m.get("view_cache_roots_not_marked_reuse_root"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let view_cache_roots_needs_rerender = stats
                .and_then(|m| m.get("view_cache_roots_needs_rerender"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let view_cache_roots_layout_invalidated = stats
                .and_then(|m| m.get("view_cache_roots_layout_invalidated"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let view_cache_roots_manual = stats
                .and_then(|m| m.get("view_cache_roots_manual"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let set_children_barrier_writes = stats
                .and_then(|m| m.get("set_children_barrier_writes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let barrier_relayouts_scheduled = stats
                .and_then(|m| m.get("barrier_relayouts_scheduled"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let barrier_relayouts_performed = stats
                .and_then(|m| m.get("barrier_relayouts_performed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let virtual_list_visible_range_checks = stats
                .and_then(|m| m.get("virtual_list_visible_range_checks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let virtual_list_visible_range_refreshes = stats
                .and_then(|m| m.get("virtual_list_visible_range_refreshes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;

            let propagated_model_change_models = stats
                .and_then(|m| m.get("model_change_models"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let propagated_model_change_observation_edges = stats
                .and_then(|m| m.get("model_change_observation_edges"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let propagated_model_change_unobserved_models = stats
                .and_then(|m| m.get("model_change_unobserved_models"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let propagated_global_change_globals = stats
                .and_then(|m| m.get("global_change_globals"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let propagated_global_change_observation_edges = stats
                .and_then(|m| m.get("global_change_observation_edges"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let propagated_global_change_unobserved_globals = stats
                .and_then(|m| m.get("global_change_unobserved_globals"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;

            if propagated_model_change_models > 0 {
                out.snapshots_with_propagated_model_changes = out
                    .snapshots_with_propagated_model_changes
                    .saturating_add(1);
            }
            if propagated_global_change_globals > 0 {
                out.snapshots_with_propagated_global_changes = out
                    .snapshots_with_propagated_global_changes
                    .saturating_add(1);
            }

            let consider_pointer_move_frame = if pointer_move_frame_ids.is_empty() {
                // Fallback when the bundle does not include event logs.
                dispatch_events > 0
            } else {
                pointer_move_frame_ids.contains(&frame_id) && dispatch_events > 0
            };
            if consider_pointer_move_frame {
                out.pointer_move_frames_considered =
                    out.pointer_move_frames_considered.saturating_add(1);
                if dispatch_time_us > out.pointer_move_max_dispatch_time_us {
                    out.pointer_move_max_dispatch_time_us = dispatch_time_us;
                    out.pointer_move_max_dispatch_window = window_id;
                    out.pointer_move_max_dispatch_tick_id =
                        s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    out.pointer_move_max_dispatch_frame_id = frame_id;
                }
                if hit_test_time_us > out.pointer_move_max_hit_test_time_us {
                    out.pointer_move_max_hit_test_time_us = hit_test_time_us;
                    out.pointer_move_max_hit_test_window = window_id;
                    out.pointer_move_max_hit_test_tick_id =
                        s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    out.pointer_move_max_hit_test_frame_id = frame_id;
                }
                if propagated_global_change_globals > 0 {
                    out.pointer_move_snapshots_with_global_changes = out
                        .pointer_move_snapshots_with_global_changes
                        .saturating_add(1);
                }
            }

            let invalidation_walk_calls = stats
                .and_then(|m| m.get("invalidation_walk_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes = stats
                .and_then(|m| m.get("invalidation_walk_nodes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let model_change_invalidation_roots = stats
                .and_then(|m| m.get("model_change_invalidation_roots"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let global_change_invalidation_roots = stats
                .and_then(|m| m.get("global_change_invalidation_roots"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_model_change = stats
                .and_then(|m| m.get("invalidation_walk_calls_model_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_model_change = stats
                .and_then(|m| m.get("invalidation_walk_nodes_model_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_global_change = stats
                .and_then(|m| m.get("invalidation_walk_calls_global_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let invalidation_walk_nodes_global_change = stats
                .and_then(|m| m.get("invalidation_walk_nodes_global_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let invalidation_walk_calls_hover = stats
                .and_then(|m| m.get("invalidation_walk_calls_hover"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_hover = stats
                .and_then(|m| m.get("invalidation_walk_nodes_hover"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_focus = stats
                .and_then(|m| m.get("invalidation_walk_calls_focus"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_focus = stats
                .and_then(|m| m.get("invalidation_walk_nodes_focus"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_other = stats
                .and_then(|m| m.get("invalidation_walk_calls_other"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_other = stats
                .and_then(|m| m.get("invalidation_walk_nodes_other"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;

            let top_invalidation_walks = snapshot_top_invalidation_walks(s, 3);
            let hover_pressable_target_changes = stats
                .and_then(|m| m.get("hover_pressable_target_changes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hover_hover_region_target_changes = stats
                .and_then(|m| m.get("hover_hover_region_target_changes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hover_declarative_instance_changes = stats
                .and_then(|m| m.get("hover_declarative_instance_changes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hover_declarative_hit_test_invalidations = stats
                .and_then(|m| m.get("hover_declarative_hit_test_invalidations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let hover_declarative_layout_invalidations = stats
                .and_then(|m| m.get("hover_declarative_layout_invalidations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let hover_declarative_paint_invalidations = stats
                .and_then(|m| m.get("hover_declarative_paint_invalidations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let top_hover_declarative_invalidations =
                snapshot_top_hover_declarative_invalidations(s, 3);
            let (
                cache_roots,
                cache_roots_reused,
                cache_roots_contained_relayout,
                cache_replayed_ops,
                top_cache_roots,
                top_contained_relayout_cache_roots,
            ) = snapshot_cache_root_stats(s, 3);
            let top_layout_engine_solves = snapshot_layout_engine_solves(s, 3);
            let paint_widget_hotspots = snapshot_paint_widget_hotspots(s, 3);
            let paint_text_prepare_hotspots = snapshot_paint_text_prepare_hotspots(s, 3);
            let model_change_hotspots = snapshot_model_change_hotspots(s, 3);
            let model_change_unobserved = snapshot_model_change_unobserved(s, 3);
            let global_change_hotspots = snapshot_global_change_hotspots(s, 3);
            let global_change_unobserved = snapshot_global_change_unobserved(s, 3);

            out.sum_layout_time_us = out.sum_layout_time_us.saturating_add(layout_time_us);
            out.sum_prepaint_time_us = out.sum_prepaint_time_us.saturating_add(prepaint_time_us);
            out.sum_paint_time_us = out.sum_paint_time_us.saturating_add(paint_time_us);
            out.sum_total_time_us = out.sum_total_time_us.saturating_add(total_time_us);
            out.sum_cache_roots = out.sum_cache_roots.saturating_add(cache_roots as u64);
            out.sum_cache_roots_reused = out
                .sum_cache_roots_reused
                .saturating_add(cache_roots_reused as u64);
            out.sum_cache_replayed_ops = out
                .sum_cache_replayed_ops
                .saturating_add(cache_replayed_ops);
            out.sum_invalidation_walk_calls = out
                .sum_invalidation_walk_calls
                .saturating_add(invalidation_walk_calls as u64);
            out.sum_invalidation_walk_nodes = out
                .sum_invalidation_walk_nodes
                .saturating_add(invalidation_walk_nodes as u64);
            out.sum_model_change_invalidation_roots = out
                .sum_model_change_invalidation_roots
                .saturating_add(model_change_invalidation_roots as u64);
            out.sum_global_change_invalidation_roots = out
                .sum_global_change_invalidation_roots
                .saturating_add(global_change_invalidation_roots as u64);
            if hover_declarative_layout_invalidations > 0 {
                out.snapshots_with_hover_layout_invalidations = out
                    .snapshots_with_hover_layout_invalidations
                    .saturating_add(1);
            }
            out.sum_hover_layout_invalidations = out
                .sum_hover_layout_invalidations
                .saturating_add(hover_declarative_layout_invalidations as u64);

            out.max_invalidation_walk_calls =
                out.max_invalidation_walk_calls.max(invalidation_walk_calls);
            out.max_invalidation_walk_nodes =
                out.max_invalidation_walk_nodes.max(invalidation_walk_nodes);
            out.max_model_change_invalidation_roots = out
                .max_model_change_invalidation_roots
                .max(model_change_invalidation_roots);
            out.max_global_change_invalidation_roots = out
                .max_global_change_invalidation_roots
                .max(global_change_invalidation_roots);
            if hover_declarative_layout_invalidations > out.max_hover_layout_invalidations {
                out.worst_hover_layout = Some(BundleStatsWorstHoverLayout {
                    window: window_id,
                    tick_id: s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    frame_id: s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    hover_declarative_layout_invalidations,
                    hotspots: snapshot_top_hover_declarative_invalidations(s, 8),
                });
            }
            out.max_hover_layout_invalidations = out
                .max_hover_layout_invalidations
                .max(hover_declarative_layout_invalidations);
            out.max_layout_time_us = out.max_layout_time_us.max(layout_time_us);
            out.max_prepaint_time_us = out.max_prepaint_time_us.max(prepaint_time_us);
            out.max_paint_time_us = out.max_paint_time_us.max(paint_time_us);
            out.max_total_time_us = out.max_total_time_us.max(total_time_us);

            rows.push(BundleStatsSnapshotRow {
                window: window_id,
                tick_id: s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                frame_id: s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                timestamp_unix_ms: s.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
                frame_arena_capacity_estimate_bytes,
                frame_arena_grow_events,
                element_children_vec_pool_reuses,
                element_children_vec_pool_misses,
                layout_time_us,
                layout_collect_roots_time_us,
                layout_invalidate_scroll_handle_bindings_time_us,
                layout_expand_view_cache_invalidations_time_us,
                layout_request_build_roots_time_us,
                layout_pending_barrier_relayouts_time_us,
                layout_repair_view_cache_bounds_time_us,
                layout_contained_view_cache_roots_time_us,
                layout_collapse_layout_observations_time_us,
                layout_prepaint_after_layout_time_us,
                layout_skipped_engine_frame,
                prepaint_time_us,
                paint_time_us,
                paint_record_visual_bounds_time_us,
                paint_record_visual_bounds_calls,
                paint_cache_key_time_us,
                paint_cache_hit_check_time_us,
                paint_widget_time_us,
                paint_observation_record_time_us,
                paint_host_widget_observed_models_time_us,
                paint_host_widget_observed_models_items,
                paint_host_widget_observed_globals_time_us,
                paint_host_widget_observed_globals_items,
                paint_host_widget_instance_lookup_time_us,
                paint_host_widget_instance_lookup_calls,
                paint_text_prepare_time_us,
                paint_text_prepare_calls,
                paint_text_prepare_reason_blob_missing,
                paint_text_prepare_reason_scale_changed,
                paint_text_prepare_reason_text_changed,
                paint_text_prepare_reason_rich_changed,
                paint_text_prepare_reason_style_changed,
                paint_text_prepare_reason_wrap_changed,
                paint_text_prepare_reason_overflow_changed,
                paint_text_prepare_reason_width_changed,
                paint_text_prepare_reason_font_stack_changed,
                paint_input_context_time_us,
                paint_scroll_handle_invalidation_time_us,
                paint_collect_roots_time_us,
                paint_publish_text_input_snapshot_time_us,
                paint_collapse_observations_time_us,
                dispatch_time_us,
                dispatch_pointer_events,
                dispatch_pointer_event_time_us,
                dispatch_timer_events,
                dispatch_timer_event_time_us,
                dispatch_timer_targeted_events,
                dispatch_timer_targeted_time_us,
                dispatch_timer_broadcast_events,
                dispatch_timer_broadcast_time_us,
                dispatch_timer_broadcast_layers_visited,
                dispatch_timer_broadcast_rebuild_visible_layers_time_us,
                dispatch_timer_broadcast_loop_time_us,
                dispatch_timer_slowest_event_time_us,
                dispatch_timer_slowest_token,
                dispatch_timer_slowest_was_broadcast,
                dispatch_other_events,
                dispatch_other_event_time_us,
                hit_test_time_us,
                dispatch_hover_update_time_us,
                dispatch_scroll_handle_invalidation_time_us,
                dispatch_active_layers_time_us,
                dispatch_input_context_time_us,
                dispatch_event_chain_build_time_us,
                dispatch_widget_capture_time_us,
                dispatch_widget_bubble_time_us,
                dispatch_cursor_query_time_us,
                dispatch_pointer_move_layer_observers_time_us,
                dispatch_synth_hover_observer_time_us,
                dispatch_cursor_effect_time_us,
                dispatch_post_dispatch_snapshot_time_us,
                dispatch_events,
                hit_test_queries,
                hit_test_bounds_tree_queries,
                hit_test_bounds_tree_disabled,
                hit_test_bounds_tree_misses,
                hit_test_bounds_tree_hits,
                hit_test_bounds_tree_candidate_rejected,
                hit_test_cached_path_time_us,
                hit_test_bounds_tree_query_time_us,
                hit_test_candidate_self_only_time_us,
                hit_test_fallback_traversal_time_us,
                total_time_us,
                layout_nodes_performed,
                paint_nodes_performed,
                paint_cache_misses,
                paint_cache_replay_time_us,
                paint_cache_bounds_translate_time_us,
                paint_cache_bounds_translated_nodes,
                renderer_tick_id,
                renderer_frame_id,
                renderer_encode_scene_us,
                renderer_prepare_text_us,
                renderer_prepare_svg_us,
                renderer_svg_upload_bytes,
                renderer_image_upload_bytes,
                renderer_svg_raster_budget_bytes,
                renderer_svg_rasters_live,
                renderer_svg_standalone_bytes_live,
                renderer_svg_mask_atlas_pages_live,
                renderer_svg_mask_atlas_bytes_live,
                renderer_svg_mask_atlas_used_px,
                renderer_svg_mask_atlas_capacity_px,
                renderer_svg_raster_cache_hits,
                renderer_svg_raster_cache_misses,
                renderer_svg_raster_budget_evictions,
                renderer_svg_mask_atlas_page_evictions,
                renderer_svg_mask_atlas_entries_evicted,
                renderer_text_atlas_upload_bytes,
                renderer_text_atlas_evicted_pages,
                renderer_intermediate_budget_bytes,
                renderer_intermediate_in_use_bytes,
                renderer_intermediate_peak_in_use_bytes,
                renderer_intermediate_release_targets,
                renderer_intermediate_pool_allocations,
                renderer_intermediate_pool_reuses,
                renderer_intermediate_pool_releases,
                renderer_intermediate_pool_evictions,
                renderer_intermediate_pool_free_bytes,
                renderer_intermediate_pool_free_textures,
                renderer_draw_calls,
                renderer_pipeline_switches,
                renderer_bind_group_switches,
                renderer_scissor_sets,
                renderer_scene_encoding_cache_misses,
                layout_engine_solves,
                layout_engine_solve_time_us,
                changed_models,
                changed_globals,
                changed_global_types_sample,
                propagated_model_change_models,
                propagated_model_change_observation_edges,
                propagated_model_change_unobserved_models,
                propagated_global_change_globals,
                propagated_global_change_observation_edges,
                propagated_global_change_unobserved_globals,
                invalidation_walk_calls,
                invalidation_walk_nodes,
                model_change_invalidation_roots,
                global_change_invalidation_roots,
                invalidation_walk_calls_model_change,
                invalidation_walk_nodes_model_change,
                invalidation_walk_calls_global_change,
                invalidation_walk_nodes_global_change,
                invalidation_walk_calls_hover,
                invalidation_walk_nodes_hover,
                invalidation_walk_calls_focus,
                invalidation_walk_nodes_focus,
                invalidation_walk_calls_other,
                invalidation_walk_nodes_other,
                top_invalidation_walks,
                hover_pressable_target_changes,
                hover_hover_region_target_changes,
                hover_declarative_instance_changes,
                hover_declarative_hit_test_invalidations,
                hover_declarative_layout_invalidations,
                hover_declarative_paint_invalidations,
                top_hover_declarative_invalidations,
                cache_roots,
                cache_roots_reused,
                cache_roots_contained_relayout,
                cache_replayed_ops,
                view_cache_contained_relayouts,
                view_cache_roots_total,
                view_cache_roots_reused,
                view_cache_roots_first_mount,
                view_cache_roots_node_recreated,
                view_cache_roots_cache_key_mismatch,
                view_cache_roots_not_marked_reuse_root,
                view_cache_roots_needs_rerender,
                view_cache_roots_layout_invalidated,
                view_cache_roots_manual,
                set_children_barrier_writes,
                barrier_relayouts_scheduled,
                barrier_relayouts_performed,
                virtual_list_visible_range_checks,
                virtual_list_visible_range_refreshes,
                top_cache_roots,
                top_contained_relayout_cache_roots,
                top_layout_engine_solves,
                paint_widget_hotspots,
                paint_text_prepare_hotspots,
                model_change_hotspots,
                model_change_unobserved,
                global_change_hotspots,
                global_change_unobserved,
            });
        }
    }

    match sort {
        BundleStatsSort::Invalidation => {
            rows.sort_by(|a, b| {
                b.invalidation_walk_nodes
                    .cmp(&a.invalidation_walk_nodes)
                    .then_with(|| b.invalidation_walk_calls.cmp(&a.invalidation_walk_calls))
                    .then_with(|| {
                        b.model_change_invalidation_roots
                            .cmp(&a.model_change_invalidation_roots)
                    })
                    .then_with(|| {
                        b.global_change_invalidation_roots
                            .cmp(&a.global_change_invalidation_roots)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::Time => {
            rows.sort_by(|a, b| {
                b.total_time_us
                    .cmp(&a.total_time_us)
                    .then_with(|| b.layout_time_us.cmp(&a.layout_time_us))
                    .then_with(|| b.paint_time_us.cmp(&a.paint_time_us))
                    .then_with(|| b.invalidation_walk_nodes.cmp(&a.invalidation_walk_nodes))
            });
        }
        BundleStatsSort::Dispatch => {
            rows.sort_by(|a, b| {
                b.dispatch_time_us
                    .cmp(&a.dispatch_time_us)
                    .then_with(|| b.hit_test_time_us.cmp(&a.hit_test_time_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
                    .then_with(|| b.invalidation_walk_nodes.cmp(&a.invalidation_walk_nodes))
            });
        }
        BundleStatsSort::HitTest => {
            rows.sort_by(|a, b| {
                b.hit_test_time_us
                    .cmp(&a.hit_test_time_us)
                    .then_with(|| b.dispatch_time_us.cmp(&a.dispatch_time_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
                    .then_with(|| b.invalidation_walk_nodes.cmp(&a.invalidation_walk_nodes))
            });
        }
        BundleStatsSort::RendererEncodeScene => {
            rows.sort_by(|a, b| {
                b.renderer_encode_scene_us
                    .cmp(&a.renderer_encode_scene_us)
                    .then_with(|| b.renderer_prepare_text_us.cmp(&a.renderer_prepare_text_us))
                    .then_with(|| {
                        b.renderer_pipeline_switches
                            .cmp(&a.renderer_pipeline_switches)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererPrepareText => {
            rows.sort_by(|a, b| {
                b.renderer_prepare_text_us
                    .cmp(&a.renderer_prepare_text_us)
                    .then_with(|| b.renderer_encode_scene_us.cmp(&a.renderer_encode_scene_us))
                    .then_with(|| b.renderer_draw_calls.cmp(&a.renderer_draw_calls))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererDrawCalls => {
            rows.sort_by(|a, b| {
                b.renderer_draw_calls
                    .cmp(&a.renderer_draw_calls)
                    .then_with(|| {
                        b.renderer_pipeline_switches
                            .cmp(&a.renderer_pipeline_switches)
                    })
                    .then_with(|| {
                        b.renderer_bind_group_switches
                            .cmp(&a.renderer_bind_group_switches)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererPipelineSwitches => {
            rows.sort_by(|a, b| {
                b.renderer_pipeline_switches
                    .cmp(&a.renderer_pipeline_switches)
                    .then_with(|| {
                        b.renderer_bind_group_switches
                            .cmp(&a.renderer_bind_group_switches)
                    })
                    .then_with(|| b.renderer_draw_calls.cmp(&a.renderer_draw_calls))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererBindGroupSwitches => {
            rows.sort_by(|a, b| {
                b.renderer_bind_group_switches
                    .cmp(&a.renderer_bind_group_switches)
                    .then_with(|| {
                        b.renderer_pipeline_switches
                            .cmp(&a.renderer_pipeline_switches)
                    })
                    .then_with(|| b.renderer_draw_calls.cmp(&a.renderer_draw_calls))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererTextAtlasUploadBytes => {
            rows.sort_by(|a, b| {
                b.renderer_text_atlas_upload_bytes
                    .cmp(&a.renderer_text_atlas_upload_bytes)
                    .then_with(|| {
                        b.renderer_text_atlas_evicted_pages
                            .cmp(&a.renderer_text_atlas_evicted_pages)
                    })
                    .then_with(|| b.renderer_prepare_text_us.cmp(&a.renderer_prepare_text_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererTextAtlasEvictedPages => {
            rows.sort_by(|a, b| {
                b.renderer_text_atlas_evicted_pages
                    .cmp(&a.renderer_text_atlas_evicted_pages)
                    .then_with(|| {
                        b.renderer_text_atlas_upload_bytes
                            .cmp(&a.renderer_text_atlas_upload_bytes)
                    })
                    .then_with(|| b.renderer_prepare_text_us.cmp(&a.renderer_prepare_text_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererSvgUploadBytes => {
            rows.sort_by(|a, b| {
                b.renderer_svg_upload_bytes
                    .cmp(&a.renderer_svg_upload_bytes)
                    .then_with(|| b.renderer_prepare_svg_us.cmp(&a.renderer_prepare_svg_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererImageUploadBytes => {
            rows.sort_by(|a, b| {
                b.renderer_image_upload_bytes
                    .cmp(&a.renderer_image_upload_bytes)
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererSvgRasterCacheMisses => {
            rows.sort_by(|a, b| {
                b.renderer_svg_raster_cache_misses
                    .cmp(&a.renderer_svg_raster_cache_misses)
                    .then_with(|| {
                        b.renderer_svg_upload_bytes
                            .cmp(&a.renderer_svg_upload_bytes)
                    })
                    .then_with(|| b.renderer_prepare_svg_us.cmp(&a.renderer_prepare_svg_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererSvgRasterBudgetEvictions => {
            rows.sort_by(|a, b| {
                b.renderer_svg_raster_budget_evictions
                    .cmp(&a.renderer_svg_raster_budget_evictions)
                    .then_with(|| {
                        b.renderer_svg_upload_bytes
                            .cmp(&a.renderer_svg_upload_bytes)
                    })
                    .then_with(|| b.renderer_prepare_svg_us.cmp(&a.renderer_prepare_svg_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediateBudgetBytes => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_budget_bytes
                    .cmp(&a.renderer_intermediate_budget_bytes)
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediateInUseBytes => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_in_use_bytes
                    .cmp(&a.renderer_intermediate_in_use_bytes)
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediatePeakInUseBytes => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_peak_in_use_bytes
                    .cmp(&a.renderer_intermediate_peak_in_use_bytes)
                    .then_with(|| {
                        b.renderer_intermediate_pool_evictions
                            .cmp(&a.renderer_intermediate_pool_evictions)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediateReleaseTargets => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_release_targets
                    .cmp(&a.renderer_intermediate_release_targets)
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediatePoolAllocations => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_pool_allocations
                    .cmp(&a.renderer_intermediate_pool_allocations)
                    .then_with(|| {
                        b.renderer_intermediate_pool_evictions
                            .cmp(&a.renderer_intermediate_pool_evictions)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediatePoolReuses => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_pool_reuses
                    .cmp(&a.renderer_intermediate_pool_reuses)
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediatePoolReleases => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_pool_releases
                    .cmp(&a.renderer_intermediate_pool_releases)
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediatePoolEvictions => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_pool_evictions
                    .cmp(&a.renderer_intermediate_pool_evictions)
                    .then_with(|| {
                        b.renderer_intermediate_peak_in_use_bytes
                            .cmp(&a.renderer_intermediate_peak_in_use_bytes)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediatePoolFreeBytes => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_pool_free_bytes
                    .cmp(&a.renderer_intermediate_pool_free_bytes)
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererIntermediatePoolFreeTextures => {
            rows.sort_by(|a, b| {
                b.renderer_intermediate_pool_free_textures
                    .cmp(&a.renderer_intermediate_pool_free_textures)
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
    }
    let mut hotspots: Vec<BundleStatsGlobalTypeHotspot> = global_type_counts
        .into_iter()
        .map(|(type_name, count)| BundleStatsGlobalTypeHotspot { type_name, count })
        .collect();
    hotspots.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.type_name.cmp(&b.type_name))
    });
    hotspots.truncate(top);
    out.global_type_hotspots = hotspots;

    let mut model_hotspots: Vec<BundleStatsModelSourceHotspot> = model_source_counts
        .into_iter()
        .map(|(source, count)| BundleStatsModelSourceHotspot { source, count })
        .collect();
    model_hotspots.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.source.cmp(&b.source)));
    model_hotspots.truncate(top);
    out.model_source_hotspots = model_hotspots;

    out.top = rows.into_iter().take(top).collect();
    Ok(out)
}

fn snapshot_top_invalidation_walks(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsInvalidationWalk> {
    let walks = snapshot
        .get("debug")
        .and_then(|v| v.get("invalidation_walks"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    if walks.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<BundleStatsInvalidationWalk> = walks
        .iter()
        .map(|w| BundleStatsInvalidationWalk {
            root_node: w.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0),
            root_element: w.get("root_element").and_then(|v| v.as_u64()),
            kind: w
                .get("kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            source: w
                .get("source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            detail: w
                .get("detail")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            walked_nodes: w
                .get("walked_nodes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            truncated_at: w.get("truncated_at").and_then(|v| v.as_u64()),
            root_role: None,
            root_test_id: None,
        })
        .collect();

    out.sort_by(|a, b| b.walked_nodes.cmp(&a.walked_nodes));
    out.truncate(max);

    for walk in &mut out {
        let (role, test_id) = snapshot_lookup_semantics(snapshot, walk.root_node);
        walk.root_role = role;
        walk.root_test_id = test_id;
    }

    out
}

fn snapshot_cache_root_stats(
    snapshot: &serde_json::Value,
    max: usize,
) -> (
    u32,
    u32,
    u32,
    u64,
    Vec<BundleStatsCacheRoot>,
    Vec<BundleStatsCacheRoot>,
) {
    let roots = snapshot
        .get("debug")
        .and_then(|v| v.get("cache_roots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if roots.is_empty() {
        return (0, 0, 0, 0, Vec::new(), Vec::new());
    }

    let mut reused: u32 = 0;
    let mut contained_relayout: u32 = 0;
    let mut replayed_ops_sum: u64 = 0;

    let semantics_index = SemanticsIndex::from_snapshot(snapshot);

    let mut out: Vec<BundleStatsCacheRoot> = roots
        .iter()
        .map(|r| {
            let root_node = r.get("root").and_then(|v| v.as_u64()).unwrap_or(0);
            let paint_replayed_ops = r
                .get("paint_replayed_ops")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let reused_flag = r.get("reused").and_then(|v| v.as_bool()).unwrap_or(false);
            if reused_flag {
                reused = reused.saturating_add(1);
            }
            let contained_relayout_in_frame = r
                .get("contained_relayout_in_frame")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if contained_relayout_in_frame {
                contained_relayout = contained_relayout.saturating_add(1);
            }
            replayed_ops_sum = replayed_ops_sum.saturating_add(paint_replayed_ops as u64);

            let (role, test_id) = semantics_index.lookup_for_cache_root(root_node);
            BundleStatsCacheRoot {
                root_node,
                element: r.get("element").and_then(|v| v.as_u64()),
                element_path: r
                    .get("element_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                reused: reused_flag,
                contained_layout: r
                    .get("contained_layout")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                contained_relayout_in_frame,
                paint_replayed_ops,
                reuse_reason: r
                    .get("reuse_reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                root_in_semantics: r.get("root_in_semantics").and_then(|v| v.as_bool()),
                root_role: role,
                root_test_id: test_id,
            }
        })
        .collect();

    out.sort_by(|a, b| b.paint_replayed_ops.cmp(&a.paint_replayed_ops));
    let top_cache_roots: Vec<BundleStatsCacheRoot> = out.iter().take(max).cloned().collect();
    let top_contained_relayout_cache_roots: Vec<BundleStatsCacheRoot> = out
        .iter()
        .filter(|r| r.contained_relayout_in_frame)
        .take(max)
        .cloned()
        .collect();

    (
        roots.len().min(u32::MAX as usize) as u32,
        reused,
        contained_relayout,
        replayed_ops_sum,
        top_cache_roots,
        top_contained_relayout_cache_roots,
    )
}

fn snapshot_top_hover_declarative_invalidations(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsHoverDeclarativeInvalidationHotspot> {
    let items = snapshot
        .get("debug")
        .and_then(|v| v.get("hover_declarative_invalidation_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    if items.is_empty() || max == 0 {
        return Vec::new();
    }

    let mut out: Vec<BundleStatsHoverDeclarativeInvalidationHotspot> = items
        .iter()
        .map(|h| BundleStatsHoverDeclarativeInvalidationHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            hit_test: h
                .get("hit_test")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            layout: h
                .get("layout")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            paint: h
                .get("paint")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            role: None,
            test_id: None,
        })
        .collect();

    out.sort_by(|a, b| {
        b.layout
            .cmp(&a.layout)
            .then_with(|| b.hit_test.cmp(&a.hit_test))
            .then_with(|| b.paint.cmp(&a.paint))
    });
    out.truncate(max);

    for item in &mut out {
        let (role, test_id) = snapshot_lookup_semantics(snapshot, item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn check_report_for_hover_layout_invalidations(
    report: &BundleStatsReport,
    max_allowed: u32,
) -> Result<(), String> {
    if report.max_hover_layout_invalidations <= max_allowed {
        return Ok(());
    }

    let mut extra = String::new();
    if let Some(worst) = report.worst_hover_layout.as_ref() {
        extra.push_str(&format!(
            " worst(window={} tick={} frame={} hover_layout={})",
            worst.window,
            worst.tick_id,
            worst.frame_id,
            worst.hover_declarative_layout_invalidations
        ));
        if !worst.hotspots.is_empty() {
            let items: Vec<String> = worst
                .hotspots
                .iter()
                .take(3)
                .map(|h| {
                    let mut s = format!(
                        "layout={} hit={} paint={} node={}",
                        h.layout, h.hit_test, h.paint, h.node
                    );
                    if let Some(test_id) = h.test_id.as_deref()
                        && !test_id.is_empty()
                    {
                        s.push_str(&format!(" test_id={test_id}"));
                    }
                    if let Some(role) = h.role.as_deref()
                        && !role.is_empty()
                    {
                        s.push_str(&format!(" role={role}"));
                    }
                    s
                })
                .collect();
            extra.push_str(&format!(" hotspots=[{}]", items.join(" | ")));
        }
    }

    Err(format!(
        "hover-attributed declarative layout invalidations detected (max_per_frame={} allowed={max_allowed}).{}",
        report.max_hover_layout_invalidations, extra
    ))
}

fn snapshot_paint_widget_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsPaintWidgetHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("paint_widget_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(snapshot);

    let mut out: Vec<BundleStatsPaintWidgetHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsPaintWidgetHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            widget_type: h
                .get("widget_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            paint_time_us: h.get("paint_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
            inclusive_time_us: h
                .get("inclusive_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            inclusive_scene_ops_delta: h
                .get("inclusive_scene_ops_delta")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            exclusive_scene_ops_delta: h
                .get("exclusive_scene_ops_delta")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

fn snapshot_paint_text_prepare_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsPaintTextPrepareHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("paint_text_prepare_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(snapshot);

    let mut out: Vec<BundleStatsPaintTextPrepareHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsPaintTextPrepareHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            prepare_time_us: h
                .get("prepare_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            text_len: h
                .get("text_len")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            max_width: h
                .get("max_width")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            wrap: h
                .get("wrap")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            overflow: h
                .get("overflow")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            scale_factor: h
                .get("scale_factor")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            reasons_mask: h
                .get("reasons_mask")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u16::MAX as u64) as u16,
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

fn format_text_prepare_reasons(mask: u16) -> String {
    let mut out = String::new();
    let mut push = |name: &str| {
        if !out.is_empty() {
            out.push('|');
        }
        out.push_str(name);
    };
    if mask & (1 << 0) != 0 {
        push("blob");
    }
    if mask & (1 << 1) != 0 {
        push("scale");
    }
    if mask & (1 << 2) != 0 {
        push("text");
    }
    if mask & (1 << 3) != 0 {
        push("rich");
    }
    if mask & (1 << 4) != 0 {
        push("style");
    }
    if mask & (1 << 5) != 0 {
        push("wrap");
    }
    if mask & (1 << 6) != 0 {
        push("overflow");
    }
    if mask & (1 << 7) != 0 {
        push("width");
    }
    if mask & (1 << 8) != 0 {
        push("font");
    }
    if out.is_empty() {
        out.push('0');
    }
    out
}

fn snapshot_layout_engine_solves(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsLayoutEngineSolve> {
    let solves = snapshot
        .get("debug")
        .and_then(|v| v.get("layout_engine_solves"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if solves.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(snapshot);

    let mut out: Vec<BundleStatsLayoutEngineSolve> = solves
        .iter()
        .map(|s| {
            let top_measures = s
                .get("top_measures")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let mut top_measures: Vec<BundleStatsLayoutEngineMeasureHotspot> = top_measures
                .iter()
                .take(3)
                .map(|m| {
                    let children = m
                        .get("top_children")
                        .and_then(|v| v.as_array())
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                    let mut top_children: Vec<BundleStatsLayoutEngineMeasureChildHotspot> =
                        children
                            .iter()
                            .take(3)
                            .map(|c| BundleStatsLayoutEngineMeasureChildHotspot {
                                child: c.get("child").and_then(|v| v.as_u64()).unwrap_or(0),
                                measure_time_us: c
                                    .get("measure_time_us")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                                calls: c.get("calls").and_then(|v| v.as_u64()).unwrap_or(0),
                                element: c.get("element").and_then(|v| v.as_u64()),
                                element_kind: c
                                    .get("element_kind")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                role: None,
                                test_id: None,
                            })
                            .collect();

                    for item in &mut top_children {
                        let (role, test_id) =
                            semantics_index.lookup_for_node_or_ancestor_test_id(item.child);
                        item.role = role;
                        item.test_id = test_id;
                    }

                    BundleStatsLayoutEngineMeasureHotspot {
                        node: m.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
                        measure_time_us: m
                            .get("measure_time_us")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                        calls: m.get("calls").and_then(|v| v.as_u64()).unwrap_or(0),
                        cache_hits: m.get("cache_hits").and_then(|v| v.as_u64()).unwrap_or(0),
                        element: m.get("element").and_then(|v| v.as_u64()),
                        element_kind: m
                            .get("element_kind")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        top_children,
                        role: None,
                        test_id: None,
                    }
                })
                .collect();

            for item in &mut top_measures {
                let (role, test_id) =
                    semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
                item.role = role;
                item.test_id = test_id;
            }

            BundleStatsLayoutEngineSolve {
                root_node: s.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0),
                solve_time_us: s.get("solve_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
                measure_calls: s.get("measure_calls").and_then(|v| v.as_u64()).unwrap_or(0),
                measure_cache_hits: s
                    .get("measure_cache_hits")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                measure_time_us: s
                    .get("measure_time_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                top_measures,
                root_role: None,
                root_test_id: None,
            }
        })
        .collect();

    out.sort_by(|a, b| b.solve_time_us.cmp(&a.solve_time_us));
    out.truncate(max);

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.root_node);
        item.root_role = role;
        item.root_test_id = test_id;
    }

    out
}

fn snapshot_model_change_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsModelChangeHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("model_change_hotspots"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    hotspots
        .iter()
        .take(max)
        .map(|h| BundleStatsModelChangeHotspot {
            model: h.get("model").and_then(|v| v.as_u64()).unwrap_or(0),
            observation_edges: h
                .get("observation_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            changed_at: h
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_model_change_unobserved(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsModelChangeUnobserved> {
    let unobserved = snapshot
        .get("debug")
        .and_then(|v| v.get("model_change_unobserved"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    unobserved
        .iter()
        .take(max)
        .map(|u| BundleStatsModelChangeUnobserved {
            model: u.get("model").and_then(|v| v.as_u64()).unwrap_or(0),
            created_type: u
                .get("created_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: u
                .get("created_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
            changed_at: u
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_global_change_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsGlobalChangeHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("global_change_hotspots"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    hotspots
        .iter()
        .take(max)
        .map(|h| BundleStatsGlobalChangeHotspot {
            type_name: h
                .get("type_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            observation_edges: h
                .get("observation_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            changed_at: h
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_global_change_unobserved(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsGlobalChangeUnobserved> {
    let unobserved = snapshot
        .get("debug")
        .and_then(|v| v.get("global_change_unobserved"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    unobserved
        .iter()
        .take(max)
        .map(|u| BundleStatsGlobalChangeUnobserved {
            type_name: u
                .get("type_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            changed_at: u
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_lookup_semantics(
    snapshot: &serde_json::Value,
    node_id: u64,
) -> (Option<String>, Option<String>) {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    for n in nodes {
        if n.get("id").and_then(|v| v.as_u64()) == Some(node_id) {
            let role = n
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let test_id = n
                .get("test_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return (role, test_id);
        }
    }
    (None, None)
}

#[derive(Debug, Clone)]
struct SemanticsNodeLite {
    id: u64,
    parent: Option<u64>,
    role: Option<String>,
    test_id: Option<String>,
}

#[derive(Debug, Default)]
struct SemanticsIndex {
    by_id: std::collections::HashMap<u64, SemanticsNodeLite>,
    best_descendant_with_test_id: std::collections::HashMap<u64, (Option<String>, Option<String>)>,
}

impl SemanticsIndex {
    fn from_snapshot(snapshot: &serde_json::Value) -> Self {
        let nodes = snapshot
            .get("debug")
            .and_then(|v| v.get("semantics"))
            .and_then(|v| v.get("nodes"))
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let mut by_id: std::collections::HashMap<u64, SemanticsNodeLite> =
            std::collections::HashMap::new();
        by_id.reserve(nodes.len());

        for n in nodes {
            let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
                continue;
            };

            let parent = n.get("parent").and_then(|v| v.as_u64());
            let role = n
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let test_id = n
                .get("test_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            by_id.insert(
                id,
                SemanticsNodeLite {
                    id,
                    parent,
                    role,
                    test_id,
                },
            );
        }

        let mut best_descendant_with_test_id: std::collections::HashMap<
            u64,
            (Option<String>, Option<String>),
        > = std::collections::HashMap::new();

        for node in by_id.values() {
            let Some(test_id) = node.test_id.as_deref() else {
                continue;
            };
            if test_id.is_empty() {
                continue;
            }

            let mut cursor: Option<u64> = Some(node.id);
            let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
            while let Some(id) = cursor {
                if !seen.insert(id) {
                    break;
                }

                best_descendant_with_test_id
                    .entry(id)
                    .or_insert_with(|| (node.role.clone(), node.test_id.clone()));

                cursor = by_id.get(&id).and_then(|n| n.parent);
            }
        }

        Self {
            by_id,
            best_descendant_with_test_id,
        }
    }

    fn lookup_for_cache_root(&self, root_node: u64) -> (Option<String>, Option<String>) {
        if let Some(node) = self.by_id.get(&root_node) {
            return (node.role.clone(), node.test_id.clone());
        }

        if let Some((role, test_id)) = self.best_descendant_with_test_id.get(&root_node) {
            return (role.clone(), test_id.clone());
        }

        (None, None)
    }

    fn lookup_for_node_or_ancestor_test_id(
        &self,
        node_id: u64,
    ) -> (Option<String>, Option<String>) {
        const MAX_PARENT_HOPS: usize = 16;

        let mut role: Option<String> = None;
        let mut current: Option<u64> = Some(node_id);
        for _ in 0..MAX_PARENT_HOPS {
            let Some(id) = current else {
                break;
            };
            let Some(node) = self.by_id.get(&id) else {
                break;
            };
            if role.is_none() {
                role = node.role.clone();
            }
            if node.test_id.as_ref().is_some_and(|s| !s.is_empty()) {
                return (role, node.test_id.clone());
            }
            current = node.parent;
        }

        (role, None)
    }
}

#[derive(Debug, Clone)]
pub(super) struct ScriptResultSummary {
    pub(super) run_id: u64,
    pub(super) stage: Option<String>,
    pub(super) step_index: Option<u64>,
    pub(super) reason: Option<String>,
    pub(super) last_bundle_dir: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct PickResultSummary {
    pub(super) run_id: u64,
    pub(super) stage: Option<String>,
    pub(super) reason: Option<String>,
    pub(super) last_bundle_dir: Option<String>,
    pub(super) selector: Option<serde_json::Value>,
}

pub(super) fn run_script_and_wait(
    src: &Path,
    script_path: &Path,
    script_trigger_path: &Path,
    script_result_path: &Path,
    script_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<ScriptResultSummary, String> {
    fn start_grace_ms(timeout_ms: u64, poll_ms: u64) -> u64 {
        // Give the app a little time to observe the initial trigger file state. On cold start,
        // the first observed stamp is treated as a baseline (not a trigger) to avoid replaying
        // stale scripts when the diagnostics directory is reused.
        //
        // If the external driver touches the file before the app has observed it once, the touch
        // can be consumed as the baseline and the script will never run unless the stamp advances
        // again. We mitigate this by re-touching once after a short grace period if no run starts.
        let baseline_race_ms = poll_ms.saturating_mul(4).max(250).min(5_000);
        baseline_race_ms.min(timeout_ms.saturating_div(2).max(250))
    }

    let prev_run_id = read_script_result_run_id(script_result_path).unwrap_or(0);
    let mut target_run_id: Option<u64> = None;

    write_script(src, script_path)?;
    touch(script_trigger_path)?;

    let start_deadline =
        Instant::now() + Duration::from_millis(start_grace_ms(timeout_ms, poll_ms));
    let mut next_retouch_at = start_deadline;
    let mut retouch_interval_ms: u64 = 2_000;
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for script result (result: {}, trigger: {})",
                script_result_path.display(),
                script_result_trigger_path.display()
            ));
        }

        if target_run_id.is_none() && Instant::now() >= next_retouch_at {
            // See comment in `start_grace_ms`.
            //
            // In `--launch` mode the demo process may start significantly later than the driver
            // touches the trigger (e.g. due to compilation). The in-app contract intentionally
            // treats the first observed stamp as a baseline. Periodic re-touching ensures at
            // least one trigger lands after the app has observed its baseline value, without
            // requiring any changes to the in-app polling contract.
            touch(script_trigger_path)?;
            retouch_interval_ms = (retouch_interval_ms.saturating_mul(2)).min(10_000);
            next_retouch_at = Instant::now() + Duration::from_millis(retouch_interval_ms);
        }

        if let Some(result) = read_script_result(script_result_path) {
            let run_id = result.get("run_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if target_run_id.is_none() && run_id > prev_run_id {
                target_run_id = Some(run_id);
            }

            if Some(run_id) == target_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if matches!(stage.as_deref(), Some("passed") | Some("failed")) {
                    let step_index = result.get("step_index").and_then(|v| v.as_u64());
                    let reason = result
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let last_bundle_dir = result
                        .get("last_bundle_dir")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    return Ok(ScriptResultSummary {
                        run_id,
                        stage,
                        step_index,
                        reason,
                        last_bundle_dir,
                    });
                }
            }
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

pub(super) fn clear_script_result_files(
    script_result_path: &Path,
    script_result_trigger_path: &Path,
) {
    let _ = std::fs::remove_file(script_result_path);
    let _ = std::fs::remove_file(script_result_trigger_path);
}

pub(super) fn report_result_and_exit(result: &ScriptResultSummary) -> ! {
    match result.stage.as_deref() {
        Some("passed") => {
            println!("PASS (run_id={})", result.run_id);
            std::process::exit(0);
        }
        Some("failed") => {
            let reason = result.reason.as_deref().unwrap_or("unknown");
            let last_bundle_dir = result.last_bundle_dir.as_deref().unwrap_or("");
            if last_bundle_dir.is_empty() {
                if let Some(step) = result.step_index {
                    eprintln!(
                        "FAIL (run_id={}) step={} reason={reason}",
                        result.run_id, step
                    );
                } else {
                    eprintln!("FAIL (run_id={}) reason={reason}", result.run_id);
                }
            } else {
                if let Some(step) = result.step_index {
                    eprintln!(
                        "FAIL (run_id={}) step={} reason={reason} last_bundle_dir={last_bundle_dir}",
                        result.run_id, step
                    );
                } else {
                    eprintln!(
                        "FAIL (run_id={}) reason={reason} last_bundle_dir={last_bundle_dir}",
                        result.run_id
                    );
                }
            }
            std::process::exit(1);
        }
        _ => {
            eprintln!("unexpected script stage: {:?}", result);
            std::process::exit(1);
        }
    }
}

fn expected_failure_dump_suffixes(result: &ScriptResultSummary) -> Vec<String> {
    let Some(step_index) = result.step_index else {
        return Vec::new();
    };
    let Some(reason) = result.reason.as_deref() else {
        return Vec::new();
    };

    match reason {
        "wait_until_timeout" => vec![format!("script-step-{step_index:04}-wait_until-timeout")],
        "assert_failed" => vec![format!("script-step-{step_index:04}-assert-failed")],
        "no_semantics_snapshot" => vec![
            format!("script-step-{step_index:04}-wait_until-no-semantics"),
            format!("script-step-{step_index:04}-assert-no-semantics"),
        ],
        _ => Vec::new(),
    }
}

pub(super) fn wait_for_failure_dump_bundle(
    out_dir: &Path,
    result: &ScriptResultSummary,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    let suffixes = expected_failure_dump_suffixes(result);
    if suffixes.is_empty() {
        return None;
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.min(5_000).max(250));
    while Instant::now() < deadline {
        for suffix in &suffixes {
            if let Some(dir) = find_latest_export_dir_with_suffix(out_dir, suffix)
                && dir.join("bundle.json").is_file()
            {
                return Some(dir);
            }
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
    None
}

fn find_latest_export_dir_with_suffix(out_dir: &Path, suffix: &str) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;
    let entries = std::fs::read_dir(out_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(suffix) {
            continue;
        }
        let Some((ts_str, _)) = name.split_once('-') else {
            continue;
        };
        let Ok(ts) = ts_str.parse::<u64>() else {
            continue;
        };
        match &best {
            Some((prev, _)) if *prev >= ts => {}
            _ => best = Some((ts, path)),
        }
    }
    best.map(|(_, p)| p)
}

pub(super) fn run_pick_and_wait(
    pick_trigger_path: &Path,
    pick_result_path: &Path,
    pick_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<PickResultSummary, String> {
    fn start_grace_ms(timeout_ms: u64, poll_ms: u64) -> u64 {
        // Same baseline-race mitigation as `run_script_and_wait`.
        let baseline_race_ms = poll_ms.saturating_mul(4).max(250).min(5_000);
        baseline_race_ms.min(timeout_ms.saturating_div(2).max(250))
    }

    let prev_run_id = read_pick_result_run_id(pick_result_path).unwrap_or(0);
    let mut target_run_id: Option<u64> = None;
    let mut did_retouch = false;

    touch(pick_trigger_path)?;

    let start_deadline =
        Instant::now() + Duration::from_millis(start_grace_ms(timeout_ms, poll_ms));
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for pick result (result: {}, trigger: {})",
                pick_result_path.display(),
                pick_result_trigger_path.display()
            ));
        }

        if !did_retouch && target_run_id.is_none() && Instant::now() >= start_deadline {
            touch(pick_trigger_path)?;
            did_retouch = true;
        }

        if let Some(result) = read_pick_result(pick_result_path) {
            let run_id = result.get("run_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if target_run_id.is_none() && run_id > prev_run_id {
                target_run_id = Some(run_id);
            }

            if Some(run_id) == target_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if matches!(stage.as_deref(), Some("picked")) {
                    let reason = result
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let last_bundle_dir = result
                        .get("last_bundle_dir")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let selector = result
                        .get("selection")
                        .and_then(|v| v.get("selectors"))
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .cloned();

                    return Ok(PickResultSummary {
                        run_id,
                        stage,
                        reason,
                        last_bundle_dir,
                        selector,
                    });
                }
            }
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

pub(super) fn report_pick_result_and_exit(result: &PickResultSummary) -> ! {
    match result.stage.as_deref() {
        Some("picked") => {
            if let Some(sel) = result.selector.as_ref() {
                println!("{}", serde_json::to_string(sel).unwrap_or_default());
            } else {
                println!("PICKED (run_id={})", result.run_id);
            }
            std::process::exit(0);
        }
        Some("failed") => {
            let reason = result.reason.as_deref().unwrap_or("unknown");
            let last_bundle_dir = result.last_bundle_dir.as_deref().unwrap_or("");
            if last_bundle_dir.is_empty() {
                eprintln!("FAIL (run_id={}) reason={reason}", result.run_id);
            } else {
                eprintln!(
                    "FAIL (run_id={}) reason={reason} last_bundle_dir={last_bundle_dir}",
                    result.run_id
                );
            }
            std::process::exit(1);
        }
        _ => {
            eprintln!("unexpected pick stage: {:?}", result);
            std::process::exit(1);
        }
    }
}

pub(super) fn write_pick_script(selector: &serde_json::Value, dst: &Path) -> Result<(), String> {
    let script = serde_json::json!({
        "schema_version": 1,
        "steps": [
            { "type": "click", "target": selector },
            { "type": "wait_frames", "frames": 2 },
            { "type": "capture_bundle", "label": "after-picked-click" }
        ]
    });

    let bytes = serde_json::to_vec_pretty(&script).map_err(|e| e.to_string())?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(dst, bytes).map_err(|e| e.to_string())
}

pub(super) fn apply_pick_to_script(
    src: &Path,
    dst: &Path,
    json_pointer: &str,
    selector: serde_json::Value,
) -> Result<(), String> {
    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;
    let mut script: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    json_pointer_set(&mut script, json_pointer, selector)?;

    let bytes = serde_json::to_vec_pretty(&script).map_err(|e| e.to_string())?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(dst, bytes).map_err(|e| e.to_string())
}

pub(super) fn json_pointer_set(
    root: &mut serde_json::Value,
    pointer: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    if pointer.is_empty() {
        *root = value;
        return Ok(());
    }
    if !pointer.starts_with('/') {
        return Err(format!(
            "invalid JSON pointer (must start with '/'): {pointer}"
        ));
    }

    let mut tokens: Vec<String> = pointer[1..]
        .split('/')
        .map(unescape_json_pointer_token)
        .collect();
    if tokens.is_empty() {
        *root = value;
        return Ok(());
    }

    let last = tokens
        .pop()
        .ok_or_else(|| "invalid JSON pointer".to_string())?;

    let mut cur: &mut serde_json::Value = root;
    for t in tokens {
        match cur {
            serde_json::Value::Object(map) => {
                let Some(next) = map.get_mut(&t) else {
                    return Err(format!("JSON pointer path does not exist: {pointer}"));
                };
                cur = next;
            }
            serde_json::Value::Array(arr) => {
                let idx = t
                    .parse::<usize>()
                    .map_err(|_| format!("JSON pointer expected array index, got: {t}"))?;
                let Some(next) = arr.get_mut(idx) else {
                    return Err(format!("JSON pointer array index out of bounds: {pointer}"));
                };
                cur = next;
            }
            _ => {
                return Err(format!(
                    "JSON pointer path does not resolve to a container: {pointer}"
                ));
            }
        }
    }

    match cur {
        serde_json::Value::Object(map) => {
            map.insert(last, value);
            Ok(())
        }
        serde_json::Value::Array(arr) => {
            if last == "-" {
                arr.push(value);
                return Ok(());
            }

            let idx = last
                .parse::<usize>()
                .map_err(|_| format!("JSON pointer expected array index, got: {last}"))?;
            if idx < arr.len() {
                arr[idx] = value;
                return Ok(());
            }
            if idx == arr.len() {
                arr.push(value);
                return Ok(());
            }
            Err(format!("JSON pointer array index out of bounds: {pointer}"))
        }
        _ => Err(format!(
            "JSON pointer path does not resolve to a container: {pointer}"
        )),
    }
}

pub(super) fn check_bundle_for_retained_vlist_keep_alive_reuse_min(
    bundle_path: &Path,
    min_keep_alive_reuse_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_keep_alive_reuse_min_json(
        &bundle,
        bundle_path,
        min_keep_alive_reuse_frames,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_keep_alive_reuse_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_keep_alive_reuse_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut keep_alive_reuse_frames: u64 = 0;
    let mut offenders: Vec<String> = Vec::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let reconciles = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if reconciles.is_empty() {
                continue;
            }

            let any_keep_alive_reuse = reconciles.iter().any(|r| {
                r.get("reused_from_keep_alive_items")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    > 0
            });

            if any_keep_alive_reuse {
                keep_alive_reuse_frames = keep_alive_reuse_frames.saturating_add(1);
            } else {
                let kept_alive_sum = reconciles
                    .iter()
                    .map(|r| {
                        r.get("kept_alive_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                offenders.push(format!(
                    "frame_id={frame_id} reconciles={count} kept_alive_sum={kept_alive_sum}",
                    count = reconciles.len()
                ));
            }
        }
    }

    if keep_alive_reuse_frames < min_keep_alive_reuse_frames {
        let mut msg = String::new();
        msg.push_str("expected retained virtual-list to reuse keep-alive items\n");
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!(
            "min_keep_alive_reuse_frames={min_keep_alive_reuse_frames} keep_alive_reuse_frames={keep_alive_reuse_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
        ));
        for line in offenders.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    Ok(())
}

pub(super) fn check_bundle_for_retained_vlist_keep_alive_budget(
    bundle_path: &Path,
    min_max_pool_len_after: u64,
    max_total_evicted_items: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_keep_alive_budget_json(
        &bundle,
        bundle_path,
        min_max_pool_len_after,
        max_total_evicted_items,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_keep_alive_budget_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_max_pool_len_after: u64,
    max_total_evicted_items: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let evidence_dir = bundle_path
        .parent()
        .ok_or_else(|| "invalid bundle path: missing parent directory".to_string())?;
    let evidence_path = evidence_dir.join("check.retained_vlist_keep_alive_budget.json");

    let mut examined_snapshots: u64 = 0;
    let mut reconcile_frames: u64 = 0;
    let mut max_pool_len_after: u64 = 0;
    let mut total_evicted_items: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let reconciles = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if reconciles.is_empty() {
                continue;
            }
            reconcile_frames = reconcile_frames.saturating_add(1);

            let mut frame_pool_after_max: u64 = 0;
            let mut frame_evicted_sum: u64 = 0;
            for r in reconciles {
                let pool_after = r
                    .get("keep_alive_pool_len_after")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                frame_pool_after_max = frame_pool_after_max.max(pool_after);

                let evicted = r
                    .get("evicted_keep_alive_items")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                frame_evicted_sum = frame_evicted_sum.saturating_add(evicted);
            }

            max_pool_len_after = max_pool_len_after.max(frame_pool_after_max);
            total_evicted_items = total_evicted_items.saturating_add(frame_evicted_sum);

            if samples.len() < 16 && (frame_pool_after_max > 0 || frame_evicted_sum > 0) {
                samples.push(serde_json::json!({
                    "frame_id": frame_id,
                    "pool_len_after_max": frame_pool_after_max,
                    "evicted_items": frame_evicted_sum,
                }));
            }
        }
    }

    let evidence = serde_json::json!({
        "schema_version": 1,
        "kind": "retained_vlist_keep_alive_budget",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "generated_unix_ms": super::util::now_unix_ms(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "reconcile_frames": reconcile_frames,
        "min_max_pool_len_after": min_max_pool_len_after,
        "max_pool_len_after": max_pool_len_after,
        "max_total_evicted_items": max_total_evicted_items,
        "total_evicted_items": total_evicted_items,
        "samples": samples,
    });
    let bytes = serde_json::to_vec_pretty(&evidence).map_err(|e| e.to_string())?;
    std::fs::write(&evidence_path, bytes).map_err(|e| e.to_string())?;

    if max_pool_len_after < min_max_pool_len_after || total_evicted_items > max_total_evicted_items
    {
        return Err(format!(
            "retained virtual-list keep-alive budget violated\n  bundle: {}\n  evidence: {}\n  min_max_pool_len_after={} max_pool_len_after={}\n  max_total_evicted_items={} total_evicted_items={}",
            bundle_path.display(),
            evidence_path.display(),
            min_max_pool_len_after,
            max_pool_len_after,
            max_total_evicted_items,
            total_evicted_items,
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_notify_hotspot_file_max(
    bundle_path: &Path,
    file_filter: &str,
    max_count: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_notify_hotspot_file_max_json(
        &bundle,
        bundle_path,
        file_filter,
        max_count,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_notify_hotspot_file_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    file_filter: &str,
    max_count: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    fn file_matches(actual: &str, filter: &str) -> bool {
        if filter.is_empty() {
            return false;
        }
        if actual == filter {
            return true;
        }
        let actual_norm = actual.replace('\\', "/");
        let filter_norm = filter.replace('\\', "/");
        actual_norm.ends_with(&filter_norm) || actual_norm.contains(&filter_norm)
    }

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut total_notify_requests: u64 = 0;
    let mut matched_notify_requests: u64 = 0;
    let mut matched_samples: Vec<serde_json::Value> = Vec::new();
    let mut matched_hotspot_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let reqs = s
                .get("debug")
                .and_then(|v| v.get("notify_requests"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);

            for req in reqs {
                total_notify_requests = total_notify_requests.saturating_add(1);

                let file = req.get("file").and_then(|v| v.as_str()).unwrap_or_default();
                let line = req.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                let column = req.get("column").and_then(|v| v.as_u64()).unwrap_or(0);

                let key = format!("{file}:{line}:{column}");
                *matched_hotspot_counts.entry(key).or_insert(0) += 1;

                if file_matches(file, file_filter) {
                    matched_notify_requests = matched_notify_requests.saturating_add(1);
                    if matched_samples.len() < 20 {
                        matched_samples.push(serde_json::json!({
                            "window_id": window_id,
                            "frame_id": frame_id,
                            "caller_node": req.get("caller_node").and_then(|v| v.as_u64()),
                            "target_view": req.get("target_view").and_then(|v| v.as_u64()),
                            "file": file,
                            "line": line,
                            "column": column,
                        }));
                    }
                }
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.notify_hotspots.json");

    let mut top_hotspots: Vec<(String, u64)> = matched_hotspot_counts.into_iter().collect();
    top_hotspots.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top_hotspots: Vec<serde_json::Value> = top_hotspots
        .into_iter()
        .take(30)
        .map(|(key, count)| serde_json::json!({ "key": key, "count": count }))
        .collect();

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "notify_hotspots",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "file_filter": file_filter,
        "max_count": max_count,
        "total_notify_requests": total_notify_requests,
        "matched_notify_requests": matched_notify_requests,
        "matched_samples": matched_samples,
        "top_hotspots": top_hotspots,
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_notify_requests > max_count {
        return Err(format!(
            "notify hotspot file budget exceeded: file_filter={file_filter} matched_notify_requests={matched_notify_requests} max_count={max_count}\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_marker_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_marker_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_marker_present_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut any_marker_present = false;
    let mut last_observed: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let marker_present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("marker_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if marker_present {
                any_marker_present = true;
            }

            let text_len_bytes = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let anchor = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "marker_present": marker_present,
                "text_len_bytes": text_len_bytes,
                "selection": { "anchor": anchor, "caret": caret },
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_marker_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_marker_present",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "any_marker_present": any_marker_present,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor marker gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if any_marker_present {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor marker gate failed (expected code_editor.torture.marker_present=true after warmup)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;
    let mut max_caret: u64 = 0;

    // State machine over `marker_present`:
    // 0: waiting for insert (marker=true)
    // 1: waiting for undo (marker=false)
    // 2: waiting for redo (marker=true)
    // 3: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let marker_present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("marker_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            match state {
                0 if marker_present => state = 1,
                1 if !marker_present => state = 2,
                2 if marker_present => state = 3,
                _ => {}
            }

            let text_len_bytes = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let anchor = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            max_caret = max_caret.max(caret);

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "marker_present": marker_present,
                "text_len_bytes": text_len_bytes,
                "selection": { "anchor": anchor, "caret": caret },
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_marker_undo_redo.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_marker_undo_redo",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "max_caret": max_caret,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor undo/redo gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 && max_caret > 0 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor undo/redo gate failed (expected marker present, then absent, then present again; and caret to advance)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for editable baseline snapshot
    // 1: waiting for an edit to apply (rev/len increase)
    // 2: waiting for read-only snapshot
    // 3: ensure read-only does not mutate (rev/len stable for >=2 snapshots)
    // 4: success
    let mut state: u8 = 0;

    let mut edit_before_rev: u64 = 0;
    let mut edit_before_len: u64 = 0;
    let mut edit_after_rev: u64 = 0;
    let mut edit_after_len: u64 = 0;
    let mut ro_rev: u64 = 0;
    let mut ro_len: u64 = 0;
    let mut ro_samples: u64 = 0;

    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let editable = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("editable"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 if enabled && editable => {
                    edit_before_rev = rev;
                    edit_before_len = len;
                    state = 1;
                }
                1 if enabled && editable && (rev > edit_before_rev || len > edit_before_len) => {
                    edit_after_rev = rev;
                    edit_after_len = len;
                    state = 2;
                }
                2 if enabled && !editable => {
                    ro_rev = rev;
                    ro_len = len;
                    ro_samples = 0;
                    state = 3;
                }
                3 if enabled && !editable => {
                    ro_samples = ro_samples.saturating_add(1);
                    if rev != ro_rev || len != ro_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "expected": { "buffer_revision": ro_rev, "text_len_bytes": ro_len },
                            "observed": { "buffer_revision": rev, "text_len_bytes": len },
                        }));
                        state = 4;
                        break;
                    }
                    if ro_samples >= 2 {
                        state = 4;
                        break;
                    }
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "enabled": enabled,
                "editable": editable,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "state": state,
            }));
        }
        if state == 4 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_read_only_blocks_edits.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_read_only_blocks_edits",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "edit_before": { "buffer_revision": edit_before_rev, "text_len_bytes": edit_before_len },
        "edit_after": { "buffer_revision": edit_after_rev, "text_len_bytes": edit_after_len },
        "read_only_baseline": { "buffer_revision": ro_rev, "text_len_bytes": ro_len },
        "read_only_samples": ro_samples,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor read-only gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if let Some(violation) = violation {
        return Err(format!(
            "ui-gallery code-editor read-only gate failed (buffer mutated while interaction.editable=false)\n  bundle: {}\n  evidence: {}\n  violation: {}",
            bundle_path.display(),
            evidence_path.display(),
            violation
        ));
    }

    if state == 4 && edit_after_rev > edit_before_rev && ro_samples >= 2 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor read-only gate failed (expected: edit applies, then read-only holds revision stable)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for editable baseline snapshot
    // 1: waiting for an edit to apply (rev/len increase)
    // 2: waiting for read-only snapshot
    // 3: ensure read-only does not mutate (rev/len stable for >=2 snapshots)
    // 4: success
    let mut state: u8 = 0;

    let mut edit_before_rev: u64 = 0;
    let mut edit_before_len: u64 = 0;
    let mut edit_after_rev: u64 = 0;
    let mut edit_after_len: u64 = 0;
    let mut ro_rev: u64 = 0;
    let mut ro_len: u64 = 0;
    let mut ro_samples: u64 = 0;

    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let editable = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("editable"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 if enabled && editable => {
                    edit_before_rev = rev;
                    edit_before_len = len;
                    state = 1;
                }
                1 if enabled && editable && (rev > edit_before_rev || len > edit_before_len) => {
                    edit_after_rev = rev;
                    edit_after_len = len;
                    state = 2;
                }
                2 if enabled && !editable => {
                    ro_rev = rev;
                    ro_len = len;
                    ro_samples = 0;
                    state = 3;
                }
                3 if enabled && !editable => {
                    ro_samples = ro_samples.saturating_add(1);
                    if rev != ro_rev || len != ro_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "expected": { "buffer_revision": ro_rev, "text_len_bytes": ro_len },
                            "observed": { "buffer_revision": rev, "text_len_bytes": len },
                        }));
                        state = 4;
                        break;
                    }
                    if ro_samples >= 2 {
                        state = 4;
                        break;
                    }
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "enabled": enabled,
                "editable": editable,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "state": state,
            }));
        }
        if state == 4 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_read_only_blocks_edits.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_read_only_blocks_edits",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "edit_before": { "buffer_revision": edit_before_rev, "text_len_bytes": edit_before_len },
        "edit_after": { "buffer_revision": edit_after_rev, "text_len_bytes": edit_after_len },
        "read_only_baseline": { "buffer_revision": ro_rev, "text_len_bytes": ro_len },
        "read_only_samples": ro_samples,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor read-only gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if let Some(violation) = violation {
        return Err(format!(
            "ui-gallery markdown editor read-only gate failed (buffer mutated while interaction.editable=false)\n  bundle: {}\n  evidence: {}\n  violation: {}",
            bundle_path.display(),
            evidence_path.display(),
            violation
        ));
    }

    if state == 4 && edit_after_rev > edit_before_rev && ro_samples >= 2 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor read-only gate failed (expected: edit applies, then read-only holds revision stable)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;
    let mut disabled_semantics_matched: u64 = 0;
    let mut disabled_semantics_checked: u64 = 0;
    let mut disabled_focus_violation: Option<serde_json::Value> = None;
    let mut disabled_composition_violation: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for editable baseline snapshot
    // 1: waiting for a disabled snapshot
    // 2: ensure disabled does not mutate (rev/len/caret stable for >=2 snapshots)
    // 3: success
    let mut state: u8 = 0;

    let mut edit_before_rev: u64 = 0;
    let mut edit_before_len: u64 = 0;
    let mut edit_before_caret: u64 = 0;

    let mut disabled_rev: u64 = 0;
    let mut disabled_len: u64 = 0;
    let mut disabled_caret: u64 = 0;
    let mut disabled_samples: u64 = 0;

    let mut violation: Option<serde_json::Value> = None;
    let mut failed: bool = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let editable = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("editable"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if !enabled {
                disabled_semantics_checked = disabled_semantics_checked.saturating_add(1);

                let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
                if let Some(viewport_node_id) = viewport_node_id {
                    let nodes = s
                        .get("debug")
                        .and_then(|v| v.get("semantics"))
                        .and_then(|v| v.get("nodes"))
                        .and_then(|v| v.as_array())
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);

                    if !nodes.is_empty() {
                        let parents = semantics_parent_map(s);

                        let mut cur = viewport_node_id;
                        let mut text_field: Option<&serde_json::Value> = None;
                        for _ in 0..128 {
                            let node = nodes
                                .iter()
                                .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                            let Some(node) = node else {
                                break;
                            };
                            if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                                text_field = Some(node);
                                break;
                            }
                            let Some(parent) = parents.get(&cur).copied() else {
                                break;
                            };
                            cur = parent;
                        }

                        if let Some(text_field) = text_field {
                            disabled_semantics_matched =
                                disabled_semantics_matched.saturating_add(1);

                            let focused = text_field
                                .get("flags")
                                .and_then(|v| v.get("focused"))
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);

                            if focused && disabled_focus_violation.is_none() {
                                disabled_focus_violation = Some(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "viewport_test_id": VIEWPORT_TEST_ID,
                                    "viewport_node": viewport_node_id,
                                    "text_field_node": cur,
                                    "focused": focused,
                                }));
                                failed = true;
                            }

                            let text_composition = text_field.get("text_composition");
                            let composition = text_composition.and_then(|v| {
                                if let Some(arr) = v.as_array()
                                    && arr.len() == 2
                                {
                                    let a = arr[0].as_u64()?;
                                    let b = arr[1].as_u64()?;
                                    return Some((a, b));
                                }
                                if let Some(obj) = v.as_object() {
                                    if let Some((a, b)) = obj.get("anchor").and_then(|a| {
                                        Some((a.as_u64()?, obj.get("focus")?.as_u64()?))
                                    }) {
                                        return Some((a, b));
                                    }
                                    if let Some((a, b)) = obj.get("start").and_then(|a| {
                                        Some((a.as_u64()?, obj.get("end")?.as_u64()?))
                                    }) {
                                        return Some((a, b));
                                    }
                                }
                                None
                            });
                            let comp_norm =
                                composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

                            if comp_norm.is_some() && disabled_composition_violation.is_none() {
                                disabled_composition_violation = Some(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "viewport_test_id": VIEWPORT_TEST_ID,
                                    "viewport_node": viewport_node_id,
                                    "text_field_node": cur,
                                    "text_composition": comp_norm.map(|(a,b)| [a,b]),
                                }));
                                failed = true;
                            }
                        }
                    }
                }
            }

            match state {
                0 if enabled && editable => {
                    edit_before_rev = rev;
                    edit_before_len = len;
                    edit_before_caret = caret;
                    state = 1;
                }
                1 if !enabled => {
                    disabled_rev = rev;
                    disabled_len = len;
                    disabled_caret = caret;
                    disabled_samples = 0;
                    state = 2;
                }
                2 if !enabled => {
                    disabled_samples = disabled_samples.saturating_add(1);
                    if rev != disabled_rev || len != disabled_len || caret != disabled_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "expected": {
                                "buffer_revision": disabled_rev,
                                "text_len_bytes": disabled_len,
                                "selection_caret": disabled_caret
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "selection_caret": caret
                            },
                        }));
                        state = 3;
                        break;
                    }
                    if disabled_samples >= 2 {
                        state = 3;
                        break;
                    }
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "enabled": enabled,
                "editable": editable,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "selection_caret": caret,
                "disabled_semantics_matched": disabled_semantics_matched,
                "state": state,
            }));
        }
        if state == 3 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_disabled_blocks_edits.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_disabled_blocks_edits",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "editable_baseline": {
            "buffer_revision": edit_before_rev,
            "text_len_bytes": edit_before_len,
            "selection_caret": edit_before_caret
        },
        "disabled_baseline": {
            "buffer_revision": disabled_rev,
            "text_len_bytes": disabled_len,
            "selection_caret": disabled_caret,
            "samples": disabled_samples
        },
        "disabled_semantics_checked": disabled_semantics_checked,
        "disabled_semantics_matched": disabled_semantics_matched,
        "disabled_focus_violation": disabled_focus_violation,
        "disabled_composition_violation": disabled_composition_violation,
        "violation": violation,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor disabled gate requires fret_ui_gallery app snapshots after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if violation.is_some() {
        return Err(format!(
            "ui-gallery markdown editor disabled gate observed mutation while disabled\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if failed {
        return Err(format!(
            "ui-gallery markdown editor disabled gate observed focus/composition while disabled\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 && disabled_samples >= 2 && disabled_semantics_matched > 0 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor disabled gate failed (expected: disabled holds revision/len/caret stable, and is not focused with no composition)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (wrap A)
    // 1: waiting for wrap to toggle to B (wrap != A)
    // 2: waiting for wrap to return to A
    // 3: success
    let mut state: u8 = 0;

    let mut baseline_wrap_cols: Option<u64> = None;
    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_caret: u64 = 0;

    let mut toggled_wrap_cols: Option<u64> = None;

    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 => {
                    baseline_wrap_cols = wrap_cols;
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_caret = caret;
                    state = 1;
                }
                1 if wrap_cols != baseline_wrap_cols => {
                    toggled_wrap_cols = wrap_cols;
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "toggled",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "wrap": { "baseline_soft_wrap_cols": baseline_wrap_cols, "observed_soft_wrap_cols": wrap_cols },
                        }));
                        state = 3;
                        break;
                    }
                    state = 2;
                }
                2 if wrap_cols == baseline_wrap_cols => {
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "returned",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "wrap": { "baseline_soft_wrap_cols": baseline_wrap_cols, "observed_soft_wrap_cols": wrap_cols },
                        }));
                        state = 3;
                        break;
                    }
                    state = 3;
                    break;
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "caret": caret,
                "state": state,
            }));
        }
        if state == 3 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_soft_wrap_toggle_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_soft_wrap_toggle_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "soft_wrap_cols": baseline_wrap_cols,
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "caret": baseline_caret,
        },
        "toggled_soft_wrap_cols": toggled_wrap_cols,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor soft-wrap toggle gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if let Some(violation) = violation {
        return Err(format!(
            "ui-gallery markdown editor soft-wrap toggle gate failed (caret/rev/len changed across wrap toggles)\n  bundle: {}\n  evidence: {}\n  violation: {}",
            bundle_path.display(),
            evidence_path.display(),
            violation
        ));
    }

    if state == 3 && toggled_wrap_cols != baseline_wrap_cols {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor soft-wrap toggle gate failed (expected: wrap toggles twice, and caret/rev/len remain stable)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_word_boundary(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_word_boundary_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_word_boundary_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for caret=0 (collapsed)
    // 1: waiting for caret=5 (collapsed) (UnicodeWord treats `can't` as a single word)
    // 2: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            match state {
                0 => {
                    if focused && sel_lo == 0 && sel_hi == 0 {
                        state = 1;
                    }
                }
                1 => {
                    if focused && (sel_lo == 5 && sel_hi == 5 || sel_lo == 0 && sel_hi == 5) {
                        state = 2;
                        break;
                    }
                }
                _ => {}
            }
        }
        if state == 2 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_markdown_editor_word_boundary.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_word_boundary",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence_normalized": [
            {"text_selection":[0,0]},
            {"text_selection_any_of":[[5,5],[0,5]]}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor word-boundary gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 2 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor word-boundary gate failed (expected caret to move 0 -> 5 for can't)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_web_ime_bridge_enabled(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_web_ime_bridge_enabled_json(&bundle, bundle_path, warmup_frames)
}

pub(super) fn check_bundle_for_ui_gallery_web_ime_bridge_enabled_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut satisfied: bool = false;
    let mut observed_focus_true: bool = false;
    let mut last_observed: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let web_ime = s
                .get("debug")
                .and_then(|v| v.get("web_ime_bridge"))
                .and_then(|v| v.as_object());
            let Some(web_ime) = web_ime else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let enabled = web_ime
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mount_kind = web_ime.get("mount_kind").and_then(|v| v.as_str());
            let position_mode = web_ime.get("position_mode").and_then(|v| v.as_str());
            let textarea_has_focus = web_ime.get("textarea_has_focus").and_then(|v| v.as_bool());
            let cursor_area_set_seen = web_ime
                .get("cursor_area_set_seen")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let last_cursor_area = web_ime.get("last_cursor_area").cloned();

            observed_focus_true |= textarea_has_focus == Some(true);

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "enabled": enabled,
                "textarea_has_focus": textarea_has_focus,
                "mount_kind": mount_kind,
                "position_mode": position_mode,
                "cursor_area_set_seen": cursor_area_set_seen,
                "last_cursor_area": last_cursor_area,
            }));

            if enabled
                && mount_kind.is_some()
                && position_mode.is_some()
                && textarea_has_focus.is_some()
                && cursor_area_set_seen > 0
            {
                satisfied = true;
                break;
            }
        }
        if satisfied {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_web_ime_bridge_enabled.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_web_ime_bridge_enabled",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matched_snapshots": matched_snapshots,
        "satisfied": satisfied,
        "observed_focus_true": observed_focus_true,
        "last_observed": last_observed,
        "expected": {
            "selected_page": "markdown_editor_source",
            "web_ime_bridge": {
                "enabled": true,
                "mount_kind": "non_null",
                "position_mode": "non_null",
                "textarea_has_focus": "some(true_or_false)",
                "cursor_area_set_seen_gt": 0
            }
        }
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery web-ime bridge gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery web-ime bridge gate requires debug.web_ime_bridge snapshots on selected_page=markdown_editor_source after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}, ui_gallery_snapshots={ui_gallery_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if satisfied {
        return Ok(());
    }

    Err(format!(
        "ui-gallery web-ime bridge gate failed (expected bridge to be enabled with mount/position metadata and cursor area updates; focus may be best-effort)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

// ADR 0179: triple-click should select the logical line.
pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";
    const EXPECTED_LINE_END_ANY_OF: [u64; 2] = [6, 7]; // "hello\n" (LF) or "hello\r\n" (CRLF)

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for caret=0 (collapsed)
    // 1: waiting for selection=0..line_end (including the trailing newline when present)
    // 2: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            match state {
                0 => {
                    if focused && sel_lo == 0 && sel_hi == 0 {
                        state = 1;
                    }
                }
                1 => {
                    if focused && sel_lo == 0 && EXPECTED_LINE_END_ANY_OF.contains(&sel_hi) {
                        state = 2;
                        break;
                    }
                }
                _ => {}
            }
        }
        if state == 2 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_line_boundary_triple_click.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_line_boundary_triple_click",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence_normalized": [
            {"text_selection":[0,0]},
            {"text_selection_any_of":[[0,6],[0,7]]}
        ],
        "expected_line_end_any_of": EXPECTED_LINE_END_ANY_OF,
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor line-boundary (triple-click) gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 2 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor line-boundary (triple-click) gate failed (expected selection to expand 0..line_end including trailing newline)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for caret=2 (collapsed), no composition
    // 1: waiting for composition=2..4 and caret=4 (collapsed)
    // 2: waiting for caret=2 (collapsed), no composition
    // 3: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    if let Some((a, b)) = obj
                        .get("anchor")
                        .and_then(|a| Some((a.as_u64()?, obj.get("focus")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                    if let Some((a, b)) = obj
                        .get("start")
                        .and_then(|a| Some((a.as_u64()?, obj.get("end")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            match state {
                0 => {
                    if focused && sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if focused && sel_lo == 4 && sel_hi == 4 && comp_norm == Some((2, 4)) {
                        state = 2;
                    }
                }
                2 => {
                    if focused && sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 3;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_a11y_composition.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_a11y_composition",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence_normalized": [
            {"text_selection":[2,2],"text_composition":null},
            {"text_selection":[4,4],"text_composition":[2,4]},
            {"text_selection":[2,2],"text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor a11y-composition gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor a11y-composition gate failed (expected: caret 2, then composition 2..4 with caret 4, then clear back to caret 2)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";
    const EXPECTED_SOFT_WRAP_COLS: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    let mut saw_soft_wrap: bool = false;
    let mut last_soft_wrap_cols: Option<u64> = None;

    // State machine:
    // 0: waiting for caret=2 (collapsed), no composition
    // 1: waiting for composition=2..4 and caret=4 (collapsed)
    // 2: waiting for caret=2 (collapsed), no composition
    // 3: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            if let Some(app_snapshot) = s.get("app_snapshot") {
                let kind = app_snapshot
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let selected_page = app_snapshot
                    .get("selected_page")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if kind == "fret_ui_gallery" && selected_page == "markdown_editor_source" {
                    let cols = app_snapshot
                        .get("code_editor")
                        .and_then(|v| v.get("soft_wrap_cols"))
                        .and_then(|v| v.as_u64());
                    last_soft_wrap_cols = cols;
                    if cols == Some(EXPECTED_SOFT_WRAP_COLS) {
                        saw_soft_wrap = true;
                    }
                }
            }

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    if let Some((a, b)) = obj
                        .get("anchor")
                        .and_then(|a| Some((a.as_u64()?, obj.get("focus")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                    if let Some((a, b)) = obj
                        .get("start")
                        .and_then(|a| Some((a.as_u64()?, obj.get("end")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "soft_wrap_cols": last_soft_wrap_cols,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            match state {
                0 => {
                    if focused && sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if focused && sel_lo == 4 && sel_hi == 4 && comp_norm == Some((2, 4)) {
                        state = 2;
                    }
                }
                2 => {
                    if focused && sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 3;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_markdown_editor_source_a11y_composition_soft_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_a11y_composition_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_soft_wrap_cols": EXPECTED_SOFT_WRAP_COLS,
        "saw_soft_wrap": saw_soft_wrap,
        "expected_sequence_normalized": [
            {"text_selection":[2,2],"text_composition":null},
            {"text_selection":[4,4],"text_composition":[2,4]},
            {"text_selection":[2,2],"text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor a11y-composition (soft-wrap) gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if !saw_soft_wrap {
        return Err(format!(
            "ui-gallery markdown editor a11y-composition (soft-wrap) gate requires observing soft_wrap_cols={EXPECTED_SOFT_WRAP_COLS} in app snapshots, but none were observed\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor a11y-composition (soft-wrap) gate failed (expected: caret 2, then composition 2..4 with caret 4, then clear back to caret 2)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";
    const WRAP_COLS: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for wrap=80 and caret=0 (collapsed)
    // 1: waiting for caret=80 (End over visual row)
    // 2: waiting for caret=81 and len to increase by 1 (typed a single byte)
    // 3: waiting for caret=0 (Ctrl+Home)
    // 4: waiting for caret=80 again (End over visual row) with edited len
    // 5: success
    let mut state: u8 = 0;

    let mut baseline_len_bytes: Option<u64> = None;
    let mut edited_len_bytes: Option<u64> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if wrap_cols != WRAP_COLS {
                continue;
            }

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let len_bytes = text_field
                .get("value")
                .and_then(|v| v.as_str())
                .and_then(|s| {
                    parse_redacted_len_bytes(s).or_else(|| {
                        let trimmed = s.trim_start();
                        if trimmed.starts_with("<redacted") {
                            return None;
                        }
                        Some(s.len() as u64)
                    })
                });

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "len_bytes": len_bytes,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            if !focused {
                continue;
            }
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };
            if sel_lo != sel_hi {
                continue;
            }
            let caret = sel_lo;

            let Some(len_bytes) = len_bytes else {
                continue;
            };
            if len_bytes <= WRAP_COLS {
                continue;
            }

            match state {
                0 => {
                    if caret == 0 {
                        baseline_len_bytes = Some(len_bytes);
                        state = 1;
                    }
                }
                1 => {
                    if caret == WRAP_COLS {
                        state = 2;
                    }
                }
                2 => {
                    if caret == WRAP_COLS + 1 {
                        if let Some(base) = baseline_len_bytes
                            && len_bytes == base.saturating_add(1)
                        {
                            edited_len_bytes = Some(len_bytes);
                            state = 3;
                        }
                    }
                }
                3 => {
                    if caret == 0 {
                        state = 4;
                    }
                }
                4 => {
                    if caret == WRAP_COLS {
                        if edited_len_bytes == Some(len_bytes) {
                            state = 5;
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
        if state == 5 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "baseline_len_bytes": baseline_len_bytes,
        "edited_len_bytes": edited_len_bytes,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence": [
            {"text_selection":[0,0]},
            {"text_selection":[WRAP_COLS,WRAP_COLS]},
            {"text_selection":[WRAP_COLS+1,WRAP_COLS+1], "len_bytes":"baseline+1"},
            {"text_selection":[0,0]},
            {"text_selection":[WRAP_COLS,WRAP_COLS], "len_bytes":"baseline+1"}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor soft-wrap editing gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor soft-wrap editing gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} and soft_wrap_cols=80 after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 5 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor soft-wrap editing gate failed (expected: caret 0 -> 80 -> 81 (len+1) -> 0 -> 80 under soft_wrap_cols=80)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_folds_placeholder_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_placeholder_present",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor folds gate failed (expected fold placeholder to be observed at least once)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_under_wrap_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_under_wrap_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_under_wrap_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds-under-wrap gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds-under-wrap gate requires soft_wrap_cols != null and folds_fixture=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_under_wrap_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor folds-under-wrap gate failed (expected fold placeholder to be observed at least once under soft wrap)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds-under-inline-preedit gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds-under-inline-preedit gate requires soft_wrap_cols != null and folds_fixture=true and markdown_editor_source.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Err(format!(
            "ui-gallery markdown editor folds-under-inline-preedit gate failed (expected fold placeholder to be absent while inline preedit is active)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_inlays_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_present",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor inlays gate failed (expected inlay to be observed at least once)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_under_wrap_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_under_wrap_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_under_wrap_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays-under-wrap gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays-under-wrap gate requires soft_wrap_cols != null and inlays_fixture=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_under_wrap_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor inlays-under-wrap gate failed (expected inlay to be observed at least once under soft wrap)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays-under-inline-preedit gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays-under-inline-preedit gate requires soft_wrap_cols != null and inlays_fixture=true and markdown_editor_source.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Err(format!(
            "ui-gallery markdown editor inlays-under-inline-preedit gate failed (expected inlays to be absent while inline preedit is active)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (folds A)
    // 1: waiting for folds to toggle to B (folds != A)
    // 2: waiting for folds to return to A
    // 3: success
    let mut state: u8 = 0;

    let mut baseline_folds: Option<bool> = None;
    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_caret: u64 = 0;

    let mut toggled_folds: Option<bool> = None;
    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if wrap_cols.is_some() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 => {
                    baseline_folds = Some(folds_fixture);
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_caret = caret;
                    state = 1;
                }
                1 if Some(folds_fixture) != baseline_folds => {
                    toggled_folds = Some(folds_fixture);
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "toggled",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "folds_fixture": { "baseline": baseline_folds, "observed": folds_fixture },
                        }));
                        state = 3;
                        break;
                    }
                    state = 2;
                }
                2 if Some(folds_fixture) == baseline_folds => {
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "returned",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "folds_fixture": { "baseline": baseline_folds, "observed": folds_fixture },
                        }));
                        state = 3;
                        break;
                    }
                    state = 3;
                    break;
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "folds_fixture": folds_fixture,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "caret": caret,
                "state": state,
            }));
        }
        if state == 3 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_folds_toggle_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_toggle_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "folds_fixture": baseline_folds,
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "caret": baseline_caret,
        },
        "toggled_folds_fixture": toggled_folds,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds toggle gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 && violation.is_none() && toggled_folds.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor folds toggle gate failed (expected: folds_fixture toggles and returns without changing buffer_revision/text_len_bytes/caret)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (folds off; caret inside fold span)
    // 1: waiting for folds on (caret clamped to fold start; buffer unchanged)
    // 2: success
    let mut state: u8 = 0;

    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_caret: u64 = 0;

    let mut fold_span_start: u64 = 0;
    let mut fold_span_end: u64 = 0;

    let mut clamp_observed: Option<serde_json::Value> = None;
    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if wrap_cols.is_some() {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if preedit_active {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let fold_span = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("fixture_span_line0"));
            let placeholder_present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let span_start = fold_span
                .and_then(|v| v.get("start"))
                .and_then(|v| v.as_u64());
            let span_end = fold_span
                .and_then(|v| v.get("end"))
                .and_then(|v| v.as_u64());

            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let inside_fold = match (span_start, span_end) {
                (Some(start), Some(end)) if start < end => caret > start && caret < end,
                _ => false,
            };

            match state {
                0 => {
                    if folds_fixture {
                        continue;
                    }
                    let Some(start) = span_start else {
                        continue;
                    };
                    let Some(end) = span_end else {
                        continue;
                    };
                    if start >= end || !inside_fold {
                        continue;
                    }

                    fold_span_start = start;
                    fold_span_end = end;
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_caret = caret;
                    state = 1;
                }
                1 if folds_fixture => {
                    // The UI Gallery model toggle (`folds_fixture`) may be observed before the view
                    // updates have propagated to the decorated line text. Gate only once the
                    // placeholder is visible, which implies decorations are applied.
                    if !placeholder_present {
                        continue;
                    }

                    if rev != baseline_rev || len != baseline_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "folds_on",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                            },
                            "caret": caret,
                            "fold_span_line0": { "start": fold_span_start, "end": fold_span_end },
                        }));
                        state = 2;
                        break;
                    }

                    if caret == fold_span_start
                        && !(caret > fold_span_start && caret < fold_span_end)
                    {
                        clamp_observed = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "buffer_revision": rev,
                            "text_len_bytes": len,
                            "caret": caret,
                            "fold_span_line0": { "start": fold_span_start, "end": fold_span_end },
                        }));
                        state = 2;
                        break;
                    }

                    if caret > fold_span_start && caret < fold_span_end {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "folds_on",
                            "expected": {
                                "clamped_caret": fold_span_start,
                                "caret_not_inside_fold_span": true,
                            },
                            "observed": {
                                "caret": caret,
                                "caret_inside_fold_span": true,
                            },
                            "fold_span_line0": { "start": fold_span_start, "end": fold_span_end },
                        }));
                        state = 2;
                        break;
                    }
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "line0_placeholder_present": placeholder_present,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "caret": caret,
                "fold_span_line0": {
                    "start": span_start,
                    "end": span_end,
                },
                "state": state,
            }));
        }
        if state == 2 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "caret": baseline_caret,
            "fold_span_line0": { "start": fold_span_start, "end": fold_span_end },
        },
        "clamp_observed": clamp_observed,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds clamp-selection gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 2 && clamp_observed.is_some() && violation.is_none() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor folds clamp-selection gate failed (expected: with folds_fixture=false, caret inside fold span; then when folds_fixture=true caret clamps to fold start without mutating buffer)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (inlays A)
    // 1: waiting for inlays to toggle to B (inlays != A)
    // 2: waiting for inlays to return to A
    // 3: success
    let mut state: u8 = 0;

    let mut baseline_inlays: Option<bool> = None;
    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_caret: u64 = 0;

    let mut toggled_inlays: Option<bool> = None;
    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if wrap_cols.is_some() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 => {
                    baseline_inlays = Some(inlays_fixture);
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_caret = caret;
                    state = 1;
                }
                1 if Some(inlays_fixture) != baseline_inlays => {
                    toggled_inlays = Some(inlays_fixture);
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "toggled",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "inlays_fixture": { "baseline": baseline_inlays, "observed": inlays_fixture },
                        }));
                        state = 3;
                        break;
                    }
                    state = 2;
                }
                2 if Some(inlays_fixture) == baseline_inlays => {
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "returned",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "inlays_fixture": { "baseline": baseline_inlays, "observed": inlays_fixture },
                        }));
                        state = 3;
                        break;
                    }
                    state = 3;
                    break;
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "inlays_fixture": inlays_fixture,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "caret": caret,
                "state": state,
            }));
        }
        if state == 3 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_inlays_toggle_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_toggle_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "inlays_fixture": baseline_inlays,
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "caret": baseline_caret,
        },
        "toggled_inlays_fixture": toggled_inlays,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays toggle gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 && violation.is_none() && toggled_inlays.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor inlays toggle gate failed (expected: inlays_fixture toggles and returns without changing buffer_revision/text_len_bytes/caret)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline (inlays off, caret=2)
    // 1: waiting for inlays applied (fixture=true, line0_present=true, caret=2)
    // 2: waiting for caret to move right across the inlay (caret=3)
    // 3: waiting for caret to move left back to baseline (caret=2)
    // 4: success
    let mut state: u8 = 0;

    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;

    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if wrap_cols.is_some() {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if preedit_active {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if folds_fixture {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let inlay_present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let anchor = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let collapsed = anchor == caret;

            match state {
                0 => {
                    if inlays_fixture || inlay_present || !collapsed || caret != 2 || len != 5 {
                        // Keep scanning until we observe the baseline caret position with inlays off.
                    } else {
                        baseline_rev = rev;
                        baseline_len = len;
                        state = 1;
                    }
                }
                1 => {
                    if rev != baseline_rev || len != baseline_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "inlays_applied",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                            },
                        }));
                        state = 4;
                        break;
                    }

                    if inlays_fixture && inlay_present && collapsed && caret == 2 {
                        state = 2;
                    }
                }
                2 => {
                    if !(inlays_fixture && inlay_present) {
                        // Wait until the inlay is applied.
                    } else if rev != baseline_rev || len != baseline_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "move_right",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                            },
                        }));
                        state = 4;
                        break;
                    } else if collapsed && caret == 3 {
                        state = 3;
                    }
                }
                3 => {
                    if !(inlays_fixture && inlay_present) {
                        // Wait until the inlay is applied.
                    } else if rev != baseline_rev || len != baseline_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "move_left",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                            },
                        }));
                        state = 4;
                        break;
                    } else if inlays_fixture && inlay_present && collapsed && caret == 2 {
                        state = 4;
                        break;
                    }
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "folds_fixture": folds_fixture,
                "inlays_fixture": inlays_fixture,
                "line0_inlay_present": inlay_present,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "selection": { "anchor": anchor, "caret": caret },
                "state": state,
            }));
        }
        if state == 4 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_markdown_editor_source_inlays_caret_navigation_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_caret_navigation_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "expected_caret": 2,
        },
        "violation": violation,
        "last_observed": last_observed,
        "expected_sequence": [
            { "inlays_fixture": false, "line0_inlay_present": false, "caret": 2 },
            { "inlays_fixture": true, "line0_inlay_present": true, "caret": 2 },
            { "inlays_fixture": true, "line0_inlay_present": true, "caret": 3 },
            { "inlays_fixture": true, "line0_inlay_present": true, "caret": 2 }
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays caret-navigation gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if let Some(violation) = violation {
        return Err(format!(
            "ui-gallery markdown editor inlays caret-navigation gate failed (buffer mutated)\n  bundle: {}\n  evidence: {}\n  violation: {}",
            bundle_path.display(),
            evidence_path.display(),
            violation
        ));
    }

    if state == 4 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor inlays caret-navigation gate failed (expected caret to move across the inlay without mutating buffer)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    // This gate is intentionally strict: after warmup + stats reset, code-editor-grade interactions
    // (soft-wrap, pointer drag, vertical moves) should route through renderer geometry rather than
    // the MVP monospace heuristic.
    const MAX_POINTER_FALLBACKS: u64 = 0;
    const MAX_CARET_RECT_FALLBACKS: u64 = 0;
    const MAX_VERTICAL_MOVE_FALLBACKS: u64 = 0;

    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut max_pointer_fallbacks_observed = 0u64;
    let mut max_caret_rect_fallbacks_observed = 0u64;
    let mut max_vertical_move_fallbacks_observed = 0u64;
    let mut max_pointer_fallbacks_observed_global = 0u64;
    let mut max_caret_rect_fallbacks_observed_global = 0u64;
    let mut max_vertical_move_fallbacks_observed_global = 0u64;
    let mut resets_observed = 0u64;
    let mut segment_start_observed = None::<serde_json::Value>;
    let mut prev_fallbacks = None::<(u64, u64, u64)>;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            let cache_stats = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("cache_stats"));

            let pointer_fallbacks = cache_stats
                .and_then(|v| v.get("geom_pointer_hit_test_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret_rect_fallbacks = cache_stats
                .and_then(|v| v.get("geom_caret_rect_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let vertical_move_fallbacks = cache_stats
                .and_then(|v| v.get("geom_vertical_move_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            max_pointer_fallbacks_observed_global =
                max_pointer_fallbacks_observed_global.max(pointer_fallbacks);
            max_caret_rect_fallbacks_observed_global =
                max_caret_rect_fallbacks_observed_global.max(caret_rect_fallbacks);
            max_vertical_move_fallbacks_observed_global =
                max_vertical_move_fallbacks_observed_global.max(vertical_move_fallbacks);

            let reset_detected = prev_fallbacks.is_some_and(|prev| {
                pointer_fallbacks < prev.0
                    || caret_rect_fallbacks < prev.1
                    || vertical_move_fallbacks < prev.2
            });
            if reset_detected {
                resets_observed = resets_observed.saturating_add(1);
                segment_start_observed = Some(serde_json::json!( {
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "selected_page": selected_page,
                    "soft_wrap_cols": soft_wrap_cols,
                    "geom_pointer_hit_test_fallbacks": pointer_fallbacks,
                    "geom_caret_rect_fallbacks": caret_rect_fallbacks,
                    "geom_vertical_move_fallbacks": vertical_move_fallbacks,
                }));

                // Start a new “post-reset” segment. We intentionally gate only against the latest
                // segment so scripts can isolate interactions via a "Reset stats" step.
                max_pointer_fallbacks_observed = pointer_fallbacks;
                max_caret_rect_fallbacks_observed = caret_rect_fallbacks;
                max_vertical_move_fallbacks_observed = vertical_move_fallbacks;
            } else {
                max_pointer_fallbacks_observed =
                    max_pointer_fallbacks_observed.max(pointer_fallbacks);
                max_caret_rect_fallbacks_observed =
                    max_caret_rect_fallbacks_observed.max(caret_rect_fallbacks);
                max_vertical_move_fallbacks_observed =
                    max_vertical_move_fallbacks_observed.max(vertical_move_fallbacks);
            }
            prev_fallbacks = Some((
                pointer_fallbacks,
                caret_rect_fallbacks,
                vertical_move_fallbacks,
            ));

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "geom_pointer_hit_test_fallbacks": pointer_fallbacks,
                "geom_caret_rect_fallbacks": caret_rect_fallbacks,
                "geom_vertical_move_fallbacks": vertical_move_fallbacks,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_geom_fallbacks_low.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_geom_fallbacks_low",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "max_pointer_fallbacks": MAX_POINTER_FALLBACKS,
        "max_caret_rect_fallbacks": MAX_CARET_RECT_FALLBACKS,
        "max_vertical_move_fallbacks": MAX_VERTICAL_MOVE_FALLBACKS,
        "max_pointer_fallbacks_observed": max_pointer_fallbacks_observed,
        "max_caret_rect_fallbacks_observed": max_caret_rect_fallbacks_observed,
        "max_vertical_move_fallbacks_observed": max_vertical_move_fallbacks_observed,
        "max_pointer_fallbacks_observed_global": max_pointer_fallbacks_observed_global,
        "max_caret_rect_fallbacks_observed_global": max_caret_rect_fallbacks_observed_global,
        "max_vertical_move_fallbacks_observed_global": max_vertical_move_fallbacks_observed_global,
        "resets_observed": resets_observed,
        "segment_start_observed": segment_start_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor geom fallback gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    let Some(_last) = last_observed.as_ref() else {
        return Err(format!(
            "ui-gallery code-editor geom fallback gate failed (no code_editor_torture snapshot observed after warmup)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    };

    if max_pointer_fallbacks_observed <= MAX_POINTER_FALLBACKS
        && max_caret_rect_fallbacks_observed <= MAX_CARET_RECT_FALLBACKS
        && max_vertical_move_fallbacks_observed <= MAX_VERTICAL_MOVE_FALLBACKS
    {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor geom fallback gate failed (expected fallbacks <= {MAX_POINTER_FALLBACKS}/{MAX_CARET_RECT_FALLBACKS}/{MAX_VERTICAL_MOVE_FALLBACKS}, got pointer={max_pointer_fallbacks_observed} caret_rect={max_caret_rect_fallbacks_observed} vertical_move={max_vertical_move_fallbacks_observed})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_folds_placeholder_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds gate failed (expected fold placeholder to be observed at least once)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_under_wrap_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_under_wrap_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_under_wrap_observed": placeholder_present_under_wrap_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-wrap gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-wrap gate requires soft_wrap_cols != null and folds_fixture=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_under_wrap_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds-under-wrap gate failed (expected fold placeholder to be observed at least once under soft wrap)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit gate requires soft_wrap_cols != null and folds_fixture=true and torture.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit gate failed (expected fold placeholder to be absent while inline preedit is active)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_some() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-unwrapped gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-unwrapped gate requires soft_wrap_cols == null and folds_fixture=true and torture.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds-under-inline-preedit-unwrapped gate failed (expected fold placeholder to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-with-decorations gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-with-decorations gate requires soft_wrap_cols != null and folds_fixture=true and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds-under-inline-preedit-with-decorations gate failed (expected fold placeholder to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            let compose_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("compose_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "compose_inline_preedit": compose_inline_preedit,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-with-decorations-composed gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-with-decorations-composed gate requires soft_wrap_cols != null and folds_fixture=true and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true and torture.compose_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds-under-inline-preedit-with-decorations-composed gate failed (expected fold placeholder to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_inlays_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays gate failed (expected inlay fixture to be observed at least once)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_under_wrap_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_under_wrap_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_code_editor_torture_inlays_present_under_soft_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present_under_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_under_wrap_observed": inlay_present_under_wrap_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-wrap gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-wrap gate requires soft_wrap_cols != null and inlays_fixture=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_under_wrap_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays-under-wrap gate failed (expected inlay text to be observed at least once under soft wrap)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit gate requires soft_wrap_cols != null and inlays_fixture=true and torture.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit gate failed (expected inlay text to be absent while inline preedit is active)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_some() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-unwrapped gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-unwrapped gate requires soft_wrap_cols == null and inlays_fixture=true and torture.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays-under-inline-preedit-unwrapped gate failed (expected inlay text to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-with-decorations gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-with-decorations gate requires soft_wrap_cols != null and inlays_fixture=true and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays-under-inline-preedit-with-decorations gate failed (expected inlay text to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            let compose_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("compose_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "compose_inline_preedit": compose_inline_preedit,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-with-decorations-composed gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-with-decorations-composed gate requires soft_wrap_cols != null and inlays_fixture=true and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true and torture.compose_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays-under-inline-preedit-with-decorations-composed gate failed (expected inlay text to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (folds+inlays A)
    // 1: waiting for folds to toggle to B (folds != A)
    // 2: waiting for inlays to toggle to B (inlays != A)
    // 3: waiting for both to return to A
    // 4: success
    let mut state: u8 = 0;

    let mut baseline_folds: Option<bool> = None;
    let mut baseline_inlays: Option<bool> = None;
    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_anchor: u64 = 0;
    let mut baseline_caret: u64 = 0;

    let mut toggled_folds: Option<bool> = None;
    let mut toggled_inlays: Option<bool> = None;
    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let code_editor = app_snapshot.and_then(|v| v.get("code_editor"));
            let folds_fixture = code_editor
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let inlays_fixture = code_editor
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let torture = code_editor.and_then(|v| v.get("torture"));
            let preedit_active = torture
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }
            let allow_decorations_under_inline_preedit = torture
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }
            let compose_inline_preedit = torture
                .and_then(|v| v.get("compose_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            let rev = torture
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = torture
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let anchor = torture
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = torture
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 => {
                    baseline_folds = Some(folds_fixture);
                    baseline_inlays = Some(inlays_fixture);
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_anchor = anchor;
                    baseline_caret = caret;
                    state = 1;
                }
                1 | 2 | 3 => {
                    if rev != baseline_rev
                        || len != baseline_len
                        || anchor != baseline_anchor
                        || caret != baseline_caret
                    {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "state": state,
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "anchor": baseline_anchor,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "anchor": anchor,
                                "caret": caret,
                            },
                            "fixtures": {
                                "folds_fixture": folds_fixture,
                                "inlays_fixture": inlays_fixture,
                            },
                        }));
                        state = 4;
                        break;
                    }

                    if toggled_folds.is_none() && baseline_folds.is_some_and(|b| folds_fixture != b)
                    {
                        toggled_folds = Some(folds_fixture);
                    }
                    if toggled_inlays.is_none()
                        && baseline_inlays.is_some_and(|b| inlays_fixture != b)
                    {
                        toggled_inlays = Some(inlays_fixture);
                    }

                    if toggled_folds.is_some() && toggled_inlays.is_some() {
                        state = 3;
                    } else if toggled_folds.is_some() || toggled_inlays.is_some() {
                        state = 2;
                    }

                    if state == 3
                        && baseline_folds.is_some_and(|b| folds_fixture == b)
                        && baseline_inlays.is_some_and(|b| inlays_fixture == b)
                    {
                        state = 4;
                        break;
                    }
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "compose_inline_preedit": compose_inline_preedit,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "anchor": anchor,
                "caret": caret,
                "state": state,
            }));
        }
        if state == 4 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "folds_fixture": baseline_folds,
            "inlays_fixture": baseline_inlays,
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "anchor": baseline_anchor,
            "caret": baseline_caret,
        },
        "toggled": {
            "folds_fixture": toggled_folds,
            "inlays_fixture": toggled_inlays,
        },
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed decorations toggle gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 && violation.is_none() && toggled_folds.is_some() && toggled_inlays.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor composed decorations toggle gate failed (expected: folds_fixture and inlays_fixture both toggle at least once while compose_inline_preedit=true, then return without changing buffer_revision/text_len_bytes/anchor/caret)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-torture-viewport";
    const EXPECTED_PREEDIT: &[u8] = b"ab";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut matching_snapshots: u64 = 0;
    let mut matched_semantics_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine (mirrors the non-a11y toggle gate):
    // 0: waiting for baseline snapshot (folds+inlays A)
    // 1: waiting for folds to toggle to B (folds != A)
    // 2: waiting for inlays to toggle to B (inlays != A)
    // 3: waiting for both to return to A
    // 4: success
    let mut state: u8 = 0;

    let mut baseline_folds: Option<bool> = None;
    let mut baseline_inlays: Option<bool> = None;
    let mut toggled_folds: Option<bool> = None;
    let mut toggled_inlays: Option<bool> = None;

    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            let compose_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("compose_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            matching_snapshots = matching_snapshots.saturating_add(1);

            // Track the toggle state machine using the fixture booleans.
            if state == 0 {
                baseline_folds = Some(folds_fixture);
                baseline_inlays = Some(inlays_fixture);
                state = 1;
            } else if state == 1 {
                if let Some(base) = baseline_folds
                    && folds_fixture != base
                {
                    toggled_folds = Some(folds_fixture);
                    state = 2;
                }
            } else if state == 2 {
                if let Some(base) = baseline_inlays
                    && inlays_fixture != base
                {
                    toggled_inlays = Some(inlays_fixture);
                    state = 3;
                }
            } else if state == 3 {
                if baseline_folds.is_some_and(|b| folds_fixture == b)
                    && baseline_inlays.is_some_and(|b| inlays_fixture == b)
                {
                    state = 4;
                }
            }

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                last_observed = Some(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "selected_page": selected_page,
                    "soft_wrap_cols": soft_wrap_cols,
                    "folds_fixture": folds_fixture,
                    "inlays_fixture": inlays_fixture,
                    "preedit_active": preedit_active,
                    "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                    "compose_inline_preedit": compose_inline_preedit,
                    "state": state,
                    "semantics": "missing_viewport_node",
                }));
                continue;
            };
            matched_semantics_snapshots = matched_semantics_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);
            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let value = text_field
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let value_bytes = value.as_bytes();

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    if let Some((a, b)) = obj
                        .get("anchor")
                        .and_then(|a| Some((a.as_u64()?, obj.get("focus")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                    if let Some((a, b)) = obj
                        .get("start")
                        .and_then(|a| Some((a.as_u64()?, obj.get("end")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                }
                None
            });

            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });
            let sel_norm = selection.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "compose_inline_preedit": compose_inline_preedit,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "value_len_bytes": value_bytes.len(),
                "text_selection": sel_norm.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": comp_norm.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((sel_lo, sel_hi)) = sel_norm else {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "missing_text_selection",
                        "last_observed": last_observed,
                    }));
                }
                continue;
            };
            if sel_lo != sel_hi {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "selection_not_collapsed",
                        "selection": [sel_lo, sel_hi],
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            let Some((comp_lo, comp_hi)) = comp_norm else {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "missing_text_composition",
                        "last_observed": last_observed,
                    }));
                }
                continue;
            };
            let value_len = value_bytes.len() as u64;
            if comp_hi > value_len || comp_lo > comp_hi {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "composition_out_of_bounds",
                        "composition": [comp_lo, comp_hi],
                        "value_len_bytes": value_len,
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            let comp_len = (comp_hi - comp_lo) as usize;
            if comp_len != EXPECTED_PREEDIT.len() {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "composition_len_mismatch",
                        "expected_len": EXPECTED_PREEDIT.len(),
                        "composition_len": comp_len,
                        "composition": [comp_lo, comp_hi],
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            let lo = comp_lo as usize;
            let hi = comp_hi as usize;
            if hi > value_bytes.len() || &value_bytes[lo..hi] != EXPECTED_PREEDIT {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "composition_text_mismatch",
                        "expected_preedit_utf8": std::str::from_utf8(EXPECTED_PREEDIT).unwrap_or(""),
                        "observed_preedit_bytes": value_bytes.get(lo..hi).map(|s| s.to_vec()),
                        "composition": [comp_lo, comp_hi],
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            if sel_lo != comp_hi {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "selection_not_at_composition_end",
                        "selection": [sel_lo, sel_hi],
                        "composition": [comp_lo, comp_hi],
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            if state == 4 && violation.is_none() {
                break;
            }
        }
        if state == 4 && violation.is_none() {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "matched_semantics_snapshots": matched_semantics_snapshots,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_preedit_utf8": std::str::from_utf8(EXPECTED_PREEDIT).unwrap_or(""),
        "state": state,
        "baseline": {
            "folds_fixture": baseline_folds,
            "inlays_fixture": baseline_inlays,
        },
        "toggled": {
            "folds_fixture": toggled_folds,
            "inlays_fixture": toggled_inlays,
        },
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed decorations toggle a11y gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed decorations toggle a11y gate requires soft_wrap_cols != null and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true and torture.compose_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matched_semantics_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed decorations toggle a11y gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 && violation.is_none() && toggled_folds.is_some() && toggled_inlays.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor composed decorations toggle a11y gate failed (expected: folds_fixture and inlays_fixture both toggle at least once while compose_inline_preedit=true, and TextField text_composition always points at the expected preedit text)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-torture-viewport";
    const EXPECTED_PREEDIT: &[u8] = b"ab";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut examined_windows: u64 = 0;
    let mut matched_windows: u64 = 0;
    let mut failures: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w
            .get("window")
            .or_else(|| w.get("window_id"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;
        examined_windows = examined_windows.saturating_add(1);

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut before: Option<&serde_json::Value> = None;
        let mut before_frame: u64 = 0;
        let mut after: Option<&serde_json::Value> = None;
        let mut after_frame_id: u64 = 0;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                if frame_id >= before_frame && frame_id < after_frame {
                    before = Some(s);
                    before_frame = frame_id;
                }
                continue;
            }
            after = Some(s);
            after_frame_id = frame_id;
            break;
        }

        let (Some(before), Some(after)) = (before, after) else {
            failures.push(serde_json::json!({
                "window": window_id,
                "wheel_frame": wheel_frame,
                "error": "missing_before_or_after_snapshot",
            }));
            continue;
        };

        let extract = |s: &serde_json::Value| -> Option<serde_json::Value> {
            let app_snapshot = s.get("app_snapshot")?;
            if app_snapshot.get("kind")?.as_str()? != "fret_ui_gallery" {
                return None;
            }
            if app_snapshot.get("selected_page")?.as_str()? != "code_editor_torture" {
                return None;
            }
            if app_snapshot
                .get("code_editor")?
                .get("soft_wrap_cols")?
                .is_null()
            {
                return None;
            }

            let torture = app_snapshot.get("code_editor")?.get("torture")?;
            if torture.get("preedit_active")?.as_bool()? != true {
                return None;
            }
            if torture
                .get("allow_decorations_under_inline_preedit")?
                .as_bool()?
                != true
            {
                return None;
            }
            if torture.get("compose_inline_preedit")?.as_bool()? != true {
                return None;
            }

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID)?;
            let nodes = s.get("debug")?.get("semantics")?.get("nodes")?.as_array()?;
            if nodes.is_empty() {
                return None;
            }

            let parents = semantics_parent_map(s);
            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }
            let text_field = text_field?;

            let value = text_field
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let value_bytes = value.as_bytes();

            let selection = text_field.get("text_selection").and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    return Some((arr[0].as_u64()?, arr[1].as_u64()?));
                }
                if let Some(obj) = v.as_object() {
                    return Some((
                        obj.get("anchor").and_then(|v| v.as_u64())?,
                        obj.get("focus").and_then(|v| v.as_u64())?,
                    ));
                }
                None
            })?;
            let (sel_lo, sel_hi) = if selection.0 <= selection.1 {
                selection
            } else {
                (selection.1, selection.0)
            };

            let composition = text_field.get("text_composition").and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    return Some((arr[0].as_u64()?, arr[1].as_u64()?));
                }
                if let Some(obj) = v.as_object() {
                    if let Some((a, b)) = obj
                        .get("anchor")
                        .and_then(|a| Some((a.as_u64()?, obj.get("focus")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                    if let Some((a, b)) = obj
                        .get("start")
                        .and_then(|a| Some((a.as_u64()?, obj.get("end")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                }
                None
            })?;
            let (comp_lo, comp_hi) = if composition.0 <= composition.1 {
                composition
            } else {
                (composition.1, composition.0)
            };

            let value_len = value_bytes.len() as u64;
            if comp_hi > value_len || comp_lo > comp_hi {
                return None;
            }
            let lo = comp_lo as usize;
            let hi = comp_hi as usize;
            if hi > value_bytes.len() || &value_bytes[lo..hi] != EXPECTED_PREEDIT {
                return None;
            }
            if sel_lo != sel_hi || sel_lo != comp_hi {
                return None;
            }

            Some(serde_json::json!({
                "frame_id": s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                "tick_id": s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                "value_len_bytes": value_bytes.len(),
                "text_selection": [sel_lo, sel_hi],
                "text_composition": [comp_lo, comp_hi],
                "buffer_revision": torture.get("buffer_revision").and_then(|v| v.as_u64()).unwrap_or(0),
                "text_len_bytes": torture.get("text_len_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
            }))
        };

        let before_obs = extract(before);
        let after_obs = extract(after);
        let (Some(before_obs), Some(after_obs)) = (before_obs, after_obs) else {
            failures.push(serde_json::json!({
                "window": window_id,
                "wheel_frame": wheel_frame,
                "after_frame": after_frame,
                "after_frame_id": after_frame_id,
                "error": "missing_matching_before_or_after_observation",
            }));
            continue;
        };

        let before_sel = before_obs.get("text_selection").and_then(|v| v.as_array());
        let after_sel = after_obs.get("text_selection").and_then(|v| v.as_array());
        let before_comp = before_obs
            .get("text_composition")
            .and_then(|v| v.as_array());
        let after_comp = after_obs.get("text_composition").and_then(|v| v.as_array());

        let before_rev = before_obs
            .get("buffer_revision")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let after_rev = after_obs
            .get("buffer_revision")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let before_len = before_obs
            .get("text_len_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let after_len = after_obs
            .get("text_len_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if before_sel != after_sel
            || before_comp != after_comp
            || before_rev != after_rev
            || before_len != after_len
        {
            failures.push(serde_json::json!({
                "window": window_id,
                "wheel_frame": wheel_frame,
                "after_frame": after_frame,
                "after_frame_id": after_frame_id,
                "before": before_obs,
                "after": after_obs,
                "error": "selection_or_composition_or_buffer_changed",
            }));
            continue;
        }

        matched_windows = matched_windows.saturating_add(1);
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_windows": examined_windows,
        "matched_windows": matched_windows,
        "failures": failures,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_preedit_utf8": std::str::from_utf8(EXPECTED_PREEDIT).unwrap_or(""),
    });
    write_json_value(&evidence_path, &payload)?;

    if !any_wheel {
        return Err(format!(
            "ui-gallery code-editor composed preedit wheel gate requires at least one pointer.wheel event in the bundle\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display(),
        ));
    }

    if matched_windows > 0 && failures.is_empty() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor composed preedit wheel gate failed (expected selection+composition+buffer len/rev to be stable across a wheel scroll while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display(),
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-torture-viewport";
    const EXPECTED_PREEDIT: &[u8] = b"ab";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut matching_snapshots: u64 = 0;
    let mut state: u8 = 0;
    let mut baseline: Option<serde_json::Value> = None;
    let mut after: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w
            .get("window")
            .or_else(|| w.get("window_id"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let torture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"));
            let Some(torture) = torture else {
                continue;
            };

            let allow_decorations_under_inline_preedit = torture
                .get("allow_decorations_under_inline_preedit")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            let compose_inline_preedit = torture
                .get("compose_inline_preedit")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            let preedit_active = torture
                .get("preedit_active")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let rev = torture
                .get("buffer_revision")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = torture
                .get("text_len_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let selection_anchor = torture
                .get("selection")
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let selection_caret = torture
                .get("selection")
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            matching_snapshots = matching_snapshots.saturating_add(1);

            // Extract semantics for validation (best-effort).
            let mut sem_value_len: Option<u64> = None;
            let mut sem_sel: Option<(u64, u64)> = None;
            let mut sem_comp: Option<(u64, u64)> = None;
            if let Some(viewport_node_id) = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID) {
                let nodes = s
                    .get("debug")
                    .and_then(|v| v.get("semantics"))
                    .and_then(|v| v.get("nodes"))
                    .and_then(|v| v.as_array())
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);
                if !nodes.is_empty() {
                    let parents = semantics_parent_map(s);
                    let mut cur = viewport_node_id;
                    let mut text_field: Option<&serde_json::Value> = None;
                    for _ in 0..128 {
                        let node = nodes
                            .iter()
                            .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                        let Some(node) = node else {
                            break;
                        };
                        if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                            text_field = Some(node);
                            break;
                        }
                        let Some(parent) = parents.get(&cur).copied() else {
                            break;
                        };
                        cur = parent;
                    }

                    if let Some(text_field) = text_field {
                        let value = text_field
                            .get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let value_bytes = value.as_bytes();
                        sem_value_len = Some(value_bytes.len() as u64);

                        sem_sel = text_field.get("text_selection").and_then(|v| {
                            if let Some(arr) = v.as_array()
                                && arr.len() == 2
                            {
                                return Some((arr[0].as_u64()?, arr[1].as_u64()?));
                            }
                            if let Some(obj) = v.as_object() {
                                return Some((
                                    obj.get("anchor").and_then(|v| v.as_u64())?,
                                    obj.get("focus").and_then(|v| v.as_u64())?,
                                ));
                            }
                            None
                        });

                        sem_comp =
                            text_field.get("text_composition").and_then(|v| {
                                if let Some(arr) = v.as_array()
                                    && arr.len() == 2
                                {
                                    return Some((arr[0].as_u64()?, arr[1].as_u64()?));
                                }
                                if let Some(obj) = v.as_object() {
                                    if let Some((a, b)) = obj.get("anchor").and_then(|a| {
                                        Some((a.as_u64()?, obj.get("focus")?.as_u64()?))
                                    }) {
                                        return Some((a, b));
                                    }
                                    if let Some((a, b)) = obj.get("start").and_then(|a| {
                                        Some((a.as_u64()?, obj.get("end")?.as_u64()?))
                                    }) {
                                        return Some((a, b));
                                    }
                                }
                                None
                            });

                        if let Some((a, b)) = sem_comp {
                            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                            let lo = lo as usize;
                            let hi = hi as usize;
                            if hi <= value_bytes.len()
                                && lo <= hi
                                && &value_bytes[lo..hi] != EXPECTED_PREEDIT
                            {
                                sem_comp = None;
                            }
                        }
                    }
                }
            }

            match state {
                0 => {
                    if preedit_active {
                        let Some((a, b)) = sem_comp else {
                            continue;
                        };
                        let (comp_lo, comp_hi) = if a <= b { (a, b) } else { (b, a) };
                        let Some((sa, sb)) = sem_sel else {
                            continue;
                        };
                        let (sel_lo, sel_hi) = if sa <= sb { (sa, sb) } else { (sb, sa) };
                        if sel_lo == sel_hi && sel_lo == comp_hi {
                            baseline = Some(serde_json::json!({
                                "window": window_id,
                                "frame_id": frame_id,
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "selection_anchor": selection_anchor,
                                "selection_caret": selection_caret,
                                "text_selection": [sel_lo, sel_hi],
                                "text_composition": [comp_lo, comp_hi],
                                "value_len_bytes": sem_value_len,
                            }));
                            state = 1;
                        }
                    }
                }
                1 => {
                    if !preedit_active && selection_anchor != selection_caret {
                        // Preedit cancellation must be non-mutating.
                        let Some(base) = baseline.as_ref() else {
                            continue;
                        };
                        let base_rev = base
                            .get("buffer_revision")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        let base_len = base
                            .get("text_len_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        if rev != base_rev || len != base_len {
                            after = Some(serde_json::json!({
                                "window": window_id,
                                "frame_id": frame_id,
                                "error": "buffer_changed",
                                "baseline": base,
                                "after": {
                                    "buffer_revision": rev,
                                    "text_len_bytes": len,
                                    "selection_anchor": selection_anchor,
                                    "selection_caret": selection_caret,
                                    "text_selection": sem_sel.map(|(a,b)| if a <= b { [a,b] } else { [b,a] }),
                                    "text_composition": sem_comp.map(|(a,b)| if a <= b { [a,b] } else { [b,a] }),
                                    "value_len_bytes": sem_value_len,
                                }
                            }));
                            state = 2;
                            break;
                        }

                        // Composition should be cleared after a pointer-driven selection change.
                        if sem_comp.is_none() {
                            after = Some(serde_json::json!({
                                "window": window_id,
                                "frame_id": frame_id,
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "selection_anchor": selection_anchor,
                                "selection_caret": selection_caret,
                                "text_selection": sem_sel.map(|(a,b)| if a <= b { [a,b] } else { [b,a] }),
                                "text_composition": null,
                                "value_len_bytes": sem_value_len,
                            }));
                            state = 3;
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
        if state >= 2 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "state": state,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_preedit_utf8": std::str::from_utf8(EXPECTED_PREEDIT).unwrap_or(""),
        "baseline": baseline,
        "after": after,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed preedit drag-select gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed preedit drag-select gate requires soft_wrap_cols != null and torture.allow_decorations_under_inline_preedit=true and torture.compose_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor composed preedit drag-select gate failed (expected: observe preedit composition once, then a pointer-driven drag selection cancels preedit without mutating buffer)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_word_boundary(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_word_boundary_json(&bundle, bundle_path, warmup_frames)
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_word_boundary_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_IDS: [&str; 2] = [
        "ui-gallery-code-editor-word-gate-viewport",
        "ui-gallery-code-editor-word-gate-soft-wrap-viewport",
    ];

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for caret=0 in Identifier mode
    // 1: waiting for caret=3 (Identifier splits `can't` around the apostrophe, `can|'t`)
    // 2: waiting for caret=0 in UnicodeWord mode
    // 3: waiting for caret=5 (UnicodeWord treats `can't` as a single word)
    // 4: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);
            for viewport_test_id in VIEWPORT_TEST_IDS {
                let Some(viewport_node_id) = semantics_node_id_for_test_id(s, viewport_test_id)
                else {
                    continue;
                };
                matched_snapshots = matched_snapshots.saturating_add(1);

                let mut cur = viewport_node_id;
                let mut text_field: Option<&serde_json::Value> = None;
                for _ in 0..128 {
                    let node = nodes
                        .iter()
                        .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                    let Some(node) = node else {
                        break;
                    };
                    if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                        text_field = Some(node);
                        break;
                    }
                    let Some(parent) = parents.get(&cur).copied() else {
                        break;
                    };
                    cur = parent;
                }

                let Some(text_field) = text_field else {
                    continue;
                };

                let text_selection = text_field.get("text_selection");
                let selection = text_selection.and_then(|v| {
                    if let Some(arr) = v.as_array()
                        && arr.len() == 2
                    {
                        let a = arr[0].as_u64()?;
                        let b = arr[1].as_u64()?;
                        return Some((a, b));
                    }
                    if let Some(obj) = v.as_object() {
                        let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                        let b = obj.get("focus").and_then(|v| v.as_u64())?;
                        return Some((a, b));
                    }
                    None
                });

                let focused = text_field
                    .get("flags")
                    .and_then(|v| v.get("focused"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                last_observed = Some(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "viewport_test_id": viewport_test_id,
                    "viewport_node": viewport_node_id,
                    "text_field_node": cur,
                    "focused": focused,
                    "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                    "state": state,
                }));

                let Some((mut a, mut b)) = selection else {
                    continue;
                };
                if !focused {
                    continue;
                }
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                }

                match state {
                    0 => {
                        if a == 0 && b == 0 {
                            state = 1;
                        }
                    }
                    1 => {
                        if (a == 3 && b == 3) || (a == 0 && b == 3) || (a == 4 && b == 5) {
                            state = 2;
                        }
                    }
                    2 => {
                        if a == 0 && b == 0 {
                            state = 3;
                        }
                    }
                    3 => {
                        if (a == 5 && b == 5) || (a == 0 && b == 5) {
                            state = 4;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_code_editor_word_boundary.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_word_boundary",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_ids": VIEWPORT_TEST_IDS,
        "expected_sequence": [
            {"selection":[0,0]},
            {"selection_any_of":[[3,3],[0,3],[4,5]]},
            {"selection":[0,0]},
            {"selection_any_of":[[5,5],[0,5]]}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor word-boundary gate requires semantics snapshots with viewport test_ids={:?} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            VIEWPORT_TEST_IDS,
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor word-boundary gate failed (expected selection sequence [0,0] -> [3,3]/[0,3]/[4,5] -> [0,0] -> [5,5]/[0,5] for can't)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_selection(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_selection_json(&bundle, bundle_path, warmup_frames)
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_selection_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-selection-gate-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine for "hello world":
    //
    // 0: waiting for caret=0 (collapsed)
    // 1: waiting for selection=0..5 ("hello", normalized)
    // 2: waiting for caret=11 (collapsed, end of string)
    // 3: waiting for selection=0..11 (select all, normalized)
    // 4: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (lo, hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            match state {
                0 => {
                    if lo == 0 && hi == 0 {
                        state = 1;
                    }
                }
                1 => {
                    if lo == 0 && hi == 5 {
                        state = 2;
                    }
                }
                2 => {
                    if lo == 11 && hi == 11 {
                        state = 3;
                    }
                }
                3 => {
                    if lo == 0 && hi == 11 {
                        state = 4;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_code_editor_a11y_selection.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_selection",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence_normalized": [
            {"min":0,"max":0},
            {"min":0,"max":5},
            {"min":11,"max":11},
            {"min":0,"max":11}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-selection gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-selection gate failed (expected selection sequence for \"hello world\": 0..0 -> 0..5 -> 11..11 -> 0..11)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_composition(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_composition_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-composition-gate-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for caret=2 (collapsed), no composition
    // 1: waiting for composition=2..4 and caret=4 (collapsed)
    // 2: waiting for caret=2 (collapsed), no composition
    // 3: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    if let Some((a, b)) = obj
                        .get("anchor")
                        .and_then(|a| Some((a.as_u64()?, obj.get("focus")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                    if let Some((a, b)) = obj
                        .get("start")
                        .and_then(|a| Some((a.as_u64()?, obj.get("end")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            match state {
                0 => {
                    if sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if sel_lo == 4 && sel_hi == 4 && comp_norm == Some((2, 4)) {
                        state = 2;
                    }
                }
                2 => {
                    if sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 3;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_code_editor_a11y_composition.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_composition",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence_normalized": [
            {"text_selection":[2,2],"text_composition":null},
            {"text_selection":[4,4],"text_composition":[2,4]},
            {"text_selection":[2,2],"text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-composition gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-composition gate failed (expected selection/composition sequence: caret 2..2 (no composition) -> caret 4..4 (composition 2..4) -> caret 2..2 (no composition))\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport";
    const WRAP_COLS: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    let mut expected_len_bytes: Option<u64> = None;

    // State machine (single long line, wrap at 80 cols):
    //
    // 0: waiting for caret=0
    // 1: waiting for caret=80 (End over visual row)
    // 2: waiting for caret=len (Ctrl+End clamps to document bounds)
    // 3: waiting for selection=0..len (Ctrl+A)
    // 4: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let len_bytes = text_field
                .get("value")
                .and_then(|v| v.as_str())
                .and_then(|s| {
                    parse_redacted_len_bytes(s).or_else(|| {
                        let trimmed = s.trim_start();
                        if trimmed.starts_with("<redacted") {
                            return None;
                        }
                        Some(s.len() as u64)
                    })
                });
            if let Some(len_bytes) = len_bytes {
                if expected_len_bytes.is_none() {
                    expected_len_bytes = Some(len_bytes);
                }
            }
            let Some(len_bytes) = expected_len_bytes else {
                continue;
            };
            if len_bytes <= WRAP_COLS {
                continue;
            }

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "len_bytes": len_bytes,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (lo, hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            match state {
                0 => {
                    if lo == 0 && hi == 0 {
                        state = 1;
                    }
                }
                1 => {
                    if lo == WRAP_COLS && hi == WRAP_COLS {
                        state = 2;
                    }
                }
                2 => {
                    if lo == len_bytes && hi == len_bytes {
                        state = 3;
                    }
                }
                3 => {
                    if lo == 0 && hi == len_bytes {
                        state = 4;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_code_editor_a11y_selection_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_selection_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "wrap_cols": WRAP_COLS,
        "expected_len_bytes": expected_len_bytes,
        "expected_sequence_template": [
            {"min":0,"max":0},
            {"min":WRAP_COLS,"max":WRAP_COLS},
            {"min":"len_bytes","max":"len_bytes"},
            {"min":0,"max":"len_bytes"}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-selection-wrap gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if expected_len_bytes.is_none() {
        return Err(format!(
            "ui-gallery code-editor a11y-selection-wrap gate requires a text_field semantics node with a value/len, but none was observed\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if expected_len_bytes.unwrap_or(0) <= WRAP_COLS {
        return Err(format!(
            "ui-gallery code-editor a11y-selection-wrap gate requires len_bytes > {WRAP_COLS}, but observed len_bytes={:?}\n  bundle: {}\n  evidence: {}",
            expected_len_bytes,
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-selection-wrap gate failed (expected: caret 0..0 -> caret {WRAP_COLS}..{WRAP_COLS} (End) -> caret len..len (Ctrl+End) -> selection 0..len (Ctrl+A))\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport";
    const PREEDIT_START: u64 = 78;
    const PREEDIT_END: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    //
    // 0: waiting for caret=78 (collapsed), no composition
    // 1: waiting for caret=80 (collapsed), composition=78..80
    // 2: waiting for caret=78 (collapsed), no composition
    // 3: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("start").and_then(|v| v.as_u64())?;
                    let b = obj.get("end").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };
            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            match state {
                0 => {
                    if sel_lo == PREEDIT_START && sel_hi == PREEDIT_START && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if sel_lo == PREEDIT_END
                        && sel_hi == PREEDIT_END
                        && comp_norm == Some((PREEDIT_START, PREEDIT_END))
                    {
                        state = 2;
                    }
                }
                2 => {
                    if sel_lo == PREEDIT_START && sel_hi == PREEDIT_START && comp_norm.is_none() {
                        state = 3;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_a11y_composition_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_composition_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence_normalized": [
            {"text_selection":[PREEDIT_START,PREEDIT_START],"text_composition":null},
            {"text_selection":[PREEDIT_END,PREEDIT_END],"text_composition":[PREEDIT_START,PREEDIT_END]},
            {"text_selection":[PREEDIT_START,PREEDIT_START],"text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-wrap gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-composition-wrap gate failed (expected selection/composition sequence: caret 78..78 (no composition) -> caret 80..80 (composition 78..80) -> caret 78..78 (no composition))\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_scroll(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_scroll_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_scroll_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport";
    const PREEDIT_START: u64 = 78;
    const PREEDIT_END: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    let mut preedit_observed_frame: Option<u64> = None;
    let mut scroll_after_preedit_frame: Option<u64> = None;
    let mut preedit_observed_after_scroll_frame: Option<u64> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("start").and_then(|v| v.as_u64())?;
                    let b = obj.get("end").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let scroll_offset_changed = s
                .get("debug")
                .and_then(|v| v.get("scroll_handle_changes"))
                .and_then(|v| v.as_array())
                .is_some_and(|changes| {
                    changes.iter().any(|c| {
                        c.get("offset_changed")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    })
                });

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "scroll_offset_changed": scroll_offset_changed,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "preedit_observed_frame": preedit_observed_frame,
                "scroll_after_preedit_frame": scroll_after_preedit_frame,
                "preedit_observed_after_scroll_frame": preedit_observed_after_scroll_frame,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };
            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            let is_expected_preedit = sel_lo == PREEDIT_END
                && sel_hi == PREEDIT_END
                && comp_norm == Some((PREEDIT_START, PREEDIT_END));

            if preedit_observed_frame.is_none() && is_expected_preedit {
                preedit_observed_frame = Some(frame_id);
            }

            if scroll_after_preedit_frame.is_none()
                && preedit_observed_frame.is_some()
                && scroll_offset_changed
            {
                scroll_after_preedit_frame = Some(frame_id);
            }

            if preedit_observed_after_scroll_frame.is_none()
                && scroll_after_preedit_frame.is_some()
                && is_expected_preedit
            {
                preedit_observed_after_scroll_frame = Some(frame_id);
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_a11y_composition_wrap_scroll.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_composition_wrap_scroll",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "preedit_observed_frame": preedit_observed_frame,
        "scroll_after_preedit_frame": scroll_after_preedit_frame,
        "preedit_observed_after_scroll_frame": preedit_observed_after_scroll_frame,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_preedit": {"text_selection":[PREEDIT_END,PREEDIT_END],"text_composition":[PREEDIT_START,PREEDIT_END]},
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-wrap-scroll gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if preedit_observed_frame.is_none() {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-wrap-scroll gate requires observing an inline preedit (selection 80..80, composition 78..80), but none was observed\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if scroll_after_preedit_frame.is_none() {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-wrap-scroll gate requires observing a scroll offset change after preedit is active, but none was observed\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if preedit_observed_after_scroll_frame.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-composition-wrap-scroll gate failed (expected preedit to remain active after scroll while composing)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_drag(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_composition_drag_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_drag_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-composition-drag-gate-viewport";
    const PREEDIT_START: u64 = 78;
    const PREEDIT_END: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    //
    // 0: waiting for caret=78 (collapsed), no composition
    // 1: waiting for caret=80 (collapsed), composition=78..80
    // 2: waiting for a non-collapsed selection, no composition (drag selection clears preedit deterministically)
    // 3: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("start").and_then(|v| v.as_u64())?;
                    let b = obj.get("end").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };
            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            match state {
                0 => {
                    if sel_lo == PREEDIT_START && sel_hi == PREEDIT_START && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if sel_lo == PREEDIT_END
                        && sel_hi == PREEDIT_END
                        && comp_norm == Some((PREEDIT_START, PREEDIT_END))
                    {
                        state = 2;
                    }
                }
                2 => {
                    if sel_lo != sel_hi && comp_norm.is_none() {
                        state = 3;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_a11y_composition_drag.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_composition_drag",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "preedit": {"start": PREEDIT_START, "end": PREEDIT_END},
        "expected_sequence": [
            {"text_selection":[PREEDIT_START,PREEDIT_START],"text_composition":null},
            {"text_selection":[PREEDIT_END,PREEDIT_END],"text_composition":[PREEDIT_START,PREEDIT_END]},
            {"text_selection":"non-collapsed","text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-drag gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-composition-drag gate failed (expected: caret 78..78 (no composition) -> caret 80..80 (composition 78..80) -> non-collapsed selection (no composition))\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

fn parse_redacted_len_bytes(value: &str) -> Option<u64> {
    let value = value.trim();
    if !value.starts_with("<redacted") {
        return None;
    }
    let idx = value.find("len=")?;
    let digits = value[(idx + "len=".len())..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok()
}

fn unescape_json_pointer_token(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut it = raw.chars();
    while let Some(c) = it.next() {
        if c == '~' {
            match it.next() {
                Some('0') => out.push('~'),
                Some('1') => out.push('/'),
                Some(other) => {
                    out.push('~');
                    out.push(other);
                }
                None => out.push('~'),
            }
        } else {
            out.push(c);
        }
    }
    out
}
