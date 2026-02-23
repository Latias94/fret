use std::borrow::Cow;
use std::path::{Path, PathBuf};

mod debug_stats_gates;
mod drag_cache_gates;
mod gc_gates;
mod hover_layout_checks;
mod interaction_gates;
mod notify_gates;
mod overlay_gates;
mod retained_vlist_gates;
mod script_runtime;
mod semantics;
mod stale;
mod ui_gallery_code_editor;
mod ui_gallery_markdown_editor;
mod ui_gallery_text_gates;
mod view_cache_gates;
mod vlist;
mod wheel_scroll;
mod windowed_rows;
pub(super) use script_runtime::{
    ScriptResultSummary, apply_pick_to_script, clear_script_result_files,
    report_pick_result_and_exit, report_result_and_exit, run_pick_and_wait, run_script_and_wait,
    wait_for_failure_dump_bundle, write_pick_script,
};
use semantics::{semantics_node_id_for_test_id, semantics_parent_map};
#[cfg(test)]
pub(super) use stale::SemanticsChangedRepaintedScan;
pub(super) use ui_gallery_code_editor::*;
pub(super) use ui_gallery_markdown_editor::*;
use wheel_scroll::first_wheel_frame_id_for_window;

pub(super) fn check_out_dir_for_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps(
    out_dir: &Path,
) -> Result<(), String> {
    ui_gallery_text_gates::check_out_dir_for_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps(out_dir)
}

fn compact_string_middle<'a>(s: &'a str, head_bytes: usize, tail_bytes: usize) -> Cow<'a, str> {
    // Keep `diag stats` output readable: element paths can be extremely long on Windows
    // (workspace root + nested debug identity chain). Prefer keeping both the root prefix and the
    // final "file:line:col" tail, which is usually the most actionable part.
    let min_len = head_bytes.saturating_add(tail_bytes).saturating_add(3);
    if s.len() <= min_len {
        return Cow::Borrowed(s);
    }

    let mut head = head_bytes.min(s.len());
    while head > 0 && !s.is_char_boundary(head) {
        head -= 1;
    }

    let mut tail_start = s.len().saturating_sub(tail_bytes.min(s.len()));
    while tail_start < s.len() && !s.is_char_boundary(tail_start) {
        tail_start += 1;
    }

    Cow::Owned(format!("{}...{}", &s[..head], &s[tail_start..]))
}

fn compact_debug_path<'a>(path: &'a str) -> Cow<'a, str> {
    compact_string_middle(path, 72, 160)
}

pub(super) fn check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(
    out_dir: &Path,
) -> Result<(), String> {
    ui_gallery_text_gates::check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(out_dir)
}

pub(super) fn check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(
    out_dir: &Path,
) -> Result<(), String> {
    ui_gallery_text_gates::check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(out_dir)
}

pub(super) fn check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(
    out_dir: &Path,
) -> Result<(), String> {
    ui_gallery_text_gates::check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(out_dir)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum BundleStatsSort {
    #[default]
    Invalidation,
    Time,
    UiThreadCpuTime,
    UiThreadCpuCycles,
    Dispatch,
    HitTest,
    RendererEncodeScene,
    RendererEnsurePipelines,
    RendererPlanCompile,
    RendererUpload,
    RendererRecordPasses,
    RendererEncoderFinish,
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
            "cpu_time" | "cpu_us" | "ui_thread_cpu_time" => Ok(Self::UiThreadCpuTime),
            "cpu_cycles" | "cycles" | "ui_thread_cpu_cycles" => Ok(Self::UiThreadCpuCycles),
            "dispatch" => Ok(Self::Dispatch),
            "hit_test" => Ok(Self::HitTest),
            "encode_scene" | "encode" | "renderer_encode_scene" => Ok(Self::RendererEncodeScene),
            "ensure_pipelines" | "ensure" | "renderer_ensure_pipelines" => {
                Ok(Self::RendererEnsurePipelines)
            }
            "plan_compile" | "plan" | "renderer_plan_compile" => Ok(Self::RendererPlanCompile),
            "upload" | "uploads" | "renderer_upload" => Ok(Self::RendererUpload),
            "record_passes" | "record" | "renderer_record_passes" => Ok(Self::RendererRecordPasses),
            "encoder_finish" | "finish" | "renderer_encoder_finish" => {
                Ok(Self::RendererEncoderFinish)
            }
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
                "invalid --sort value: {other} (expected: invalidation|time|cpu_time|cpu_cycles|dispatch|hit_test|encode_scene|ensure_pipelines|plan_compile|upload|record_passes|encoder_finish|prepare_text|draw_calls|pipeline_switches|bind_group_switches|atlas_upload_bytes|atlas_evicted_pages|svg_upload_bytes|image_upload_bytes|svg_cache_misses|svg_evictions|intermediate_budget_bytes|intermediate_in_use_bytes|intermediate_peak_bytes|intermediate_release_targets|intermediate_allocations|intermediate_reuses|intermediate_releases|pool_evictions|intermediate_free_bytes|intermediate_free_textures)"
            )),
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Invalidation => "invalidation",
            Self::Time => "time",
            Self::UiThreadCpuTime => "cpu_time",
            Self::UiThreadCpuCycles => "cpu_cycles",
            Self::Dispatch => "dispatch",
            Self::HitTest => "hit_test",
            Self::RendererEncodeScene => "encode_scene",
            Self::RendererEnsurePipelines => "ensure_pipelines",
            Self::RendererPlanCompile => "plan_compile",
            Self::RendererUpload => "upload",
            Self::RendererRecordPasses => "record_passes",
            Self::RendererEncoderFinish => "encoder_finish",
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
    pub(super) snapshots_considered: u32,
    pub(super) snapshots_skipped_warmup: u32,
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
    sum_layout_collect_roots_time_us: u64,
    sum_layout_invalidate_scroll_handle_bindings_time_us: u64,
    sum_layout_expand_view_cache_invalidations_time_us: u64,
    sum_layout_request_build_roots_time_us: u64,
    sum_layout_roots_time_us: u64,
    sum_layout_collapse_layout_observations_time_us: u64,
    sum_layout_time_us: u64,
    sum_layout_view_cache_time_us: u64,
    sum_layout_prepaint_after_layout_time_us: u64,
    sum_layout_observation_record_time_us: u64,
    sum_layout_observation_record_models_items: u64,
    sum_layout_observation_record_globals_items: u64,
    sum_prepaint_time_us: u64,
    sum_paint_time_us: u64,
    sum_total_time_us: u64,
    sum_ui_thread_cpu_time_us: u64,
    sum_ui_thread_cpu_cycle_time_delta_cycles: u64,
    sum_layout_engine_solve_time_us: u64,
    sum_cache_roots: u64,
    sum_cache_roots_reused: u64,
    sum_cache_replayed_ops: u64,
    pub(super) sum_invalidation_walk_calls: u64,
    pub(super) sum_invalidation_walk_nodes: u64,
    sum_model_change_invalidation_roots: u64,
    sum_global_change_invalidation_roots: u64,
    pub(super) sum_hover_layout_invalidations: u64,
    max_layout_collect_roots_time_us: u64,
    max_layout_invalidate_scroll_handle_bindings_time_us: u64,
    max_layout_expand_view_cache_invalidations_time_us: u64,
    max_layout_request_build_roots_time_us: u64,
    max_layout_roots_time_us: u64,
    max_layout_collapse_layout_observations_time_us: u64,
    max_layout_time_us: u64,
    max_layout_view_cache_time_us: u64,
    max_layout_prepaint_after_layout_time_us: u64,
    max_layout_observation_record_time_us: u64,
    max_layout_observation_record_models_items: u32,
    max_layout_observation_record_globals_items: u32,
    pub(super) max_prepaint_time_us: u64,
    pub(super) max_paint_time_us: u64,
    pub(super) max_total_time_us: u64,
    pub(super) max_ui_thread_cpu_time_us: u64,
    pub(super) max_ui_thread_cpu_cycle_time_delta_cycles: u64,
    pub(super) max_layout_engine_solve_time_us: u64,
    pub(super) max_renderer_encode_scene_us: u64,
    pub(super) max_renderer_ensure_pipelines_us: u64,
    pub(super) max_renderer_plan_compile_us: u64,
    pub(super) max_renderer_upload_us: u64,
    pub(super) max_renderer_record_passes_us: u64,
    pub(super) max_renderer_encoder_finish_us: u64,
    pub(super) max_renderer_prepare_svg_us: u64,
    pub(super) max_renderer_prepare_text_us: u64,
    pub(super) max_invalidation_walk_calls: u32,
    pub(super) max_invalidation_walk_nodes: u32,
    max_model_change_invalidation_roots: u32,
    max_global_change_invalidation_roots: u32,
    pub(super) max_hover_layout_invalidations: u32,
    pub(super) p50_total_time_us: u64,
    pub(super) p95_total_time_us: u64,
    pub(super) p50_ui_thread_cpu_time_us: u64,
    pub(super) p95_ui_thread_cpu_time_us: u64,
    pub(super) p50_ui_thread_cpu_cycle_time_delta_cycles: u64,
    pub(super) p95_ui_thread_cpu_cycle_time_delta_cycles: u64,
    pub(super) p50_layout_time_us: u64,
    pub(super) p95_layout_time_us: u64,
    pub(super) p50_layout_collect_roots_time_us: u64,
    pub(super) p95_layout_collect_roots_time_us: u64,
    pub(super) p50_layout_request_build_roots_time_us: u64,
    pub(super) p95_layout_request_build_roots_time_us: u64,
    pub(super) p50_layout_roots_time_us: u64,
    pub(super) p95_layout_roots_time_us: u64,
    pub(super) p50_layout_view_cache_time_us: u64,
    pub(super) p95_layout_view_cache_time_us: u64,
    pub(super) p50_layout_collapse_layout_observations_time_us: u64,
    pub(super) p95_layout_collapse_layout_observations_time_us: u64,
    pub(super) p50_layout_prepaint_after_layout_time_us: u64,
    pub(super) p95_layout_prepaint_after_layout_time_us: u64,
    pub(super) p50_prepaint_time_us: u64,
    pub(super) p95_prepaint_time_us: u64,
    pub(super) p50_paint_time_us: u64,
    pub(super) p95_paint_time_us: u64,
    pub(super) p50_paint_input_context_time_us: u64,
    pub(super) p95_paint_input_context_time_us: u64,
    pub(super) p50_paint_scroll_handle_invalidation_time_us: u64,
    pub(super) p95_paint_scroll_handle_invalidation_time_us: u64,
    pub(super) p50_paint_collect_roots_time_us: u64,
    pub(super) p95_paint_collect_roots_time_us: u64,
    pub(super) p50_paint_publish_text_input_snapshot_time_us: u64,
    pub(super) p95_paint_publish_text_input_snapshot_time_us: u64,
    pub(super) p50_paint_collapse_observations_time_us: u64,
    pub(super) p95_paint_collapse_observations_time_us: u64,
    pub(super) p50_layout_engine_solve_time_us: u64,
    pub(super) p95_layout_engine_solve_time_us: u64,
    pub(super) p50_dispatch_time_us: u64,
    pub(super) p95_dispatch_time_us: u64,
    pub(super) p50_hit_test_time_us: u64,
    pub(super) p95_hit_test_time_us: u64,
    pub(super) p50_paint_widget_time_us: u64,
    pub(super) p95_paint_widget_time_us: u64,
    pub(super) p50_paint_text_prepare_time_us: u64,
    pub(super) p95_paint_text_prepare_time_us: u64,
    pub(super) p50_renderer_encode_scene_us: u64,
    pub(super) p95_renderer_encode_scene_us: u64,
    pub(super) p50_renderer_ensure_pipelines_us: u64,
    pub(super) p95_renderer_ensure_pipelines_us: u64,
    pub(super) p50_renderer_plan_compile_us: u64,
    pub(super) p95_renderer_plan_compile_us: u64,
    pub(super) p50_renderer_upload_us: u64,
    pub(super) p95_renderer_upload_us: u64,
    pub(super) p50_renderer_record_passes_us: u64,
    pub(super) p95_renderer_record_passes_us: u64,
    pub(super) p50_renderer_encoder_finish_us: u64,
    pub(super) p95_renderer_encoder_finish_us: u64,
    pub(super) p50_renderer_prepare_svg_us: u64,
    pub(super) p95_renderer_prepare_svg_us: u64,
    pub(super) p50_renderer_prepare_text_us: u64,
    pub(super) p95_renderer_prepare_text_us: u64,
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
    pub(super) ui_thread_cpu_time_us: u64,
    pub(super) ui_thread_cpu_total_time_us: u64,
    pub(super) ui_thread_cpu_cycle_time_delta_cycles: u64,
    pub(super) ui_thread_cpu_cycle_time_total_cycles: u64,
    pub(super) layout_time_us: u64,
    pub(super) layout_collect_roots_time_us: u64,
    pub(super) layout_invalidate_scroll_handle_bindings_time_us: u64,
    pub(super) layout_expand_view_cache_invalidations_time_us: u64,
    pub(super) layout_request_build_roots_time_us: u64,
    pub(super) layout_roots_time_us: u64,
    pub(super) layout_pending_barrier_relayouts_time_us: u64,
    pub(super) layout_barrier_relayouts_time_us: u64,
    pub(super) layout_repair_view_cache_bounds_time_us: u64,
    pub(super) layout_contained_view_cache_roots_time_us: u64,
    pub(super) layout_collapse_layout_observations_time_us: u64,
    pub(super) layout_observation_record_time_us: u64,
    pub(super) layout_observation_record_models_items: u32,
    pub(super) layout_observation_record_globals_items: u32,
    pub(super) layout_view_cache_time_us: u64,
    pub(super) layout_semantics_refresh_time_us: u64,
    pub(super) layout_focus_repair_time_us: u64,
    pub(super) layout_deferred_cleanup_time_us: u64,
    pub(super) layout_prepaint_after_layout_time_us: u64,
    pub(super) layout_skipped_engine_frame: bool,
    pub(super) layout_fast_path_taken: bool,
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
    pub(super) renderer_ensure_pipelines_us: u64,
    pub(super) renderer_plan_compile_us: u64,
    pub(super) renderer_upload_us: u64,
    pub(super) renderer_record_passes_us: u64,
    pub(super) renderer_encoder_finish_us: u64,
    pub(super) renderer_prepare_text_us: u64,
    pub(super) renderer_prepare_svg_us: u64,
    pub(super) renderer_svg_upload_bytes: u64,
    pub(super) renderer_image_upload_bytes: u64,

    pub(super) renderer_render_target_updates_ingest_unknown: u64,
    pub(super) renderer_render_target_updates_ingest_owned: u64,
    pub(super) renderer_render_target_updates_ingest_external_zero_copy: u64,
    pub(super) renderer_render_target_updates_ingest_gpu_copy: u64,
    pub(super) renderer_render_target_updates_ingest_cpu_upload: u64,
    pub(super) renderer_render_target_updates_requested_ingest_unknown: u64,
    pub(super) renderer_render_target_updates_requested_ingest_owned: u64,
    pub(super) renderer_render_target_updates_requested_ingest_external_zero_copy: u64,
    pub(super) renderer_render_target_updates_requested_ingest_gpu_copy: u64,
    pub(super) renderer_render_target_updates_requested_ingest_cpu_upload: u64,
    pub(super) renderer_render_target_updates_ingest_fallbacks: u64,

    pub(super) renderer_viewport_draw_calls: u64,
    pub(super) renderer_viewport_draw_calls_ingest_unknown: u64,
    pub(super) renderer_viewport_draw_calls_ingest_owned: u64,
    pub(super) renderer_viewport_draw_calls_ingest_external_zero_copy: u64,
    pub(super) renderer_viewport_draw_calls_ingest_gpu_copy: u64,
    pub(super) renderer_viewport_draw_calls_ingest_cpu_upload: u64,
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
    pub(super) renderer_material_quad_ops: u64,
    pub(super) renderer_material_sampled_quad_ops: u64,
    pub(super) renderer_material_distinct: u64,
    pub(super) renderer_material_unknown_ids: u64,
    pub(super) renderer_material_degraded_due_to_budget: u64,
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
    pub(super) layout_hotspots: Vec<BundleStatsLayoutHotspot>,
    pub(super) widget_measure_hotspots: Vec<BundleStatsWidgetMeasureHotspot>,
    pub(super) paint_widget_hotspots: Vec<BundleStatsPaintWidgetHotspot>,
    pub(super) paint_text_prepare_hotspots: Vec<BundleStatsPaintTextPrepareHotspot>,
    pub(super) model_change_hotspots: Vec<BundleStatsModelChangeHotspot>,
    pub(super) model_change_unobserved: Vec<BundleStatsModelChangeUnobserved>,
    pub(super) global_change_hotspots: Vec<BundleStatsGlobalChangeHotspot>,
    pub(super) global_change_unobserved: Vec<BundleStatsGlobalChangeUnobserved>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) element_path: Option<String>,
    pub(super) widget_type: Option<String>,
    pub(super) layout_time_us: u64,
    pub(super) inclusive_time_us: u64,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsWidgetMeasureHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) element_path: Option<String>,
    pub(super) widget_type: Option<String>,
    pub(super) measure_time_us: u64,
    pub(super) inclusive_time_us: u64,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
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
    pub(super) root_element_path: Option<String>,
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
    pub(super) root_element: Option<u64>,
    pub(super) root_element_kind: Option<String>,
    pub(super) root_element_path: Option<String>,
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
    pub(super) fn print_human_brief(&self, bundle_path: &Path) {
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
            "time p50/p95 (us): total={}/{} cpu_time={}/{} layout={}/{} prepaint={}/{} paint={}/{} dispatch={}/{} hit_test={}/{}",
            self.p50_total_time_us,
            self.p95_total_time_us,
            self.p50_ui_thread_cpu_time_us,
            self.p95_ui_thread_cpu_time_us,
            self.p50_layout_time_us,
            self.p95_layout_time_us,
            self.p50_prepaint_time_us,
            self.p95_prepaint_time_us,
            self.p50_paint_time_us,
            self.p95_paint_time_us,
            self.p50_dispatch_time_us,
            self.p95_dispatch_time_us,
            self.p50_hit_test_time_us,
            self.p95_hit_test_time_us
        );
        println!(
            "hot p50/p95 (us): layout.engine_solve={}/{} paint.widget={}/{} paint.text_prepare={}/{}",
            self.p50_layout_engine_solve_time_us,
            self.p95_layout_engine_solve_time_us,
            self.p50_paint_widget_time_us,
            self.p95_paint_widget_time_us,
            self.p50_paint_text_prepare_time_us,
            self.p95_paint_text_prepare_time_us
        );
        if self.p95_renderer_encode_scene_us > 0
            || self.p95_renderer_upload_us > 0
            || self.p95_renderer_record_passes_us > 0
            || self.p95_renderer_encoder_finish_us > 0
            || self.p95_renderer_prepare_text_us > 0
            || self.p95_renderer_prepare_svg_us > 0
            || self.max_renderer_encode_scene_us > 0
            || self.max_renderer_upload_us > 0
            || self.max_renderer_record_passes_us > 0
            || self.max_renderer_encoder_finish_us > 0
            || self.max_renderer_prepare_text_us > 0
            || self.max_renderer_prepare_svg_us > 0
        {
            println!(
                "renderer p95/max (us): upload={}/{} record={}/{} finish={}/{} encode={}/{} text={}/{} svg={}/{}",
                self.p95_renderer_upload_us,
                self.max_renderer_upload_us,
                self.p95_renderer_record_passes_us,
                self.max_renderer_record_passes_us,
                self.p95_renderer_encoder_finish_us,
                self.max_renderer_encoder_finish_us,
                self.p95_renderer_encode_scene_us,
                self.max_renderer_encode_scene_us,
                self.p95_renderer_prepare_text_us,
                self.max_renderer_prepare_text_us,
                self.p95_renderer_prepare_svg_us,
                self.max_renderer_prepare_svg_us,
            );
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
            let mut line = format!(
                "  window={} tick={} frame={} ts={} cpu.us={} cpu.cycles={} time.us(total/layout/prepaint/paint)={}/{}/{}/{} layout.solve_us={} paint.cache_misses={} layout.nodes={} paint.nodes={} paint.elem_bounds_us={} paint.elem_bounds_calls={} cache_roots={} cache.reused={} cache.replayed_ops={} cache.replay_us={} cache.translate_us={} cache.translate_nodes={} contained_relayouts={} cache.contained_relayout_roots={} barrier(set_children/scheduled/performed)={}/{}/{} vlist(range_checks/refreshes)={}/{} inv.calls={} inv.nodes={} by_src.calls(hover/focus/other)={}/{}/{} by_src.nodes(hover/focus/other)={}/{}/{} hover.decl_inv(layout/hit/paint)={}/{}/{} roots.model={} roots.global={} changed.models={} changed.globals={} propagated.models={} propagated.edges={} unobs.models={} propagated.globals={} propagated.global_edges={} unobs.globals={}",
                row.window,
                row.tick_id,
                row.frame_id,
                ts,
                row.ui_thread_cpu_time_us,
                row.ui_thread_cpu_cycle_time_delta_cycles,
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
            if row.renderer_encode_scene_us > 0
                || row.renderer_prepare_text_us > 0
                || row.renderer_prepare_svg_us > 0
                || row.renderer_upload_us > 0
                || row.renderer_record_passes_us > 0
            {
                line.push_str(&format!(
                    " renderer.us(encode/ensure/plan/upload/record/finish/svg/text)={}/{}/{}/{}/{}/{}/{}/{}",
                    row.renderer_encode_scene_us,
                    row.renderer_ensure_pipelines_us,
                    row.renderer_plan_compile_us,
                    row.renderer_upload_us,
                    row.renderer_record_passes_us,
                    row.renderer_encoder_finish_us,
                    row.renderer_prepare_svg_us,
                    row.renderer_prepare_text_us,
                ));
            }
            println!("{line}");
        }
    }

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
            "time p50/p95 (us): total={}/{} cpu_time={}/{} layout={}/{} prepaint={}/{} paint={}/{} dispatch={}/{} hit_test={}/{}",
            self.p50_total_time_us,
            self.p95_total_time_us,
            self.p50_ui_thread_cpu_time_us,
            self.p95_ui_thread_cpu_time_us,
            self.p50_layout_time_us,
            self.p95_layout_time_us,
            self.p50_prepaint_time_us,
            self.p95_prepaint_time_us,
            self.p50_paint_time_us,
            self.p95_paint_time_us,
            self.p50_dispatch_time_us,
            self.p95_dispatch_time_us,
            self.p50_hit_test_time_us,
            self.p95_hit_test_time_us
        );
        if self.p50_ui_thread_cpu_cycle_time_delta_cycles > 0
            || self.p95_ui_thread_cpu_cycle_time_delta_cycles > 0
            || self.max_ui_thread_cpu_cycle_time_delta_cycles > 0
        {
            println!(
                "cpu cycles p50/p95/max: {}/{}/{}",
                self.p50_ui_thread_cpu_cycle_time_delta_cycles,
                self.p95_ui_thread_cpu_cycle_time_delta_cycles,
                self.max_ui_thread_cpu_cycle_time_delta_cycles
            );
        }
        println!(
            "hot p50/p95 (us): layout.engine_solve={}/{} paint.widget={}/{} paint.text_prepare={}/{}",
            self.p50_layout_engine_solve_time_us,
            self.p95_layout_engine_solve_time_us,
            self.p50_paint_widget_time_us,
            self.p95_paint_widget_time_us,
            self.p50_paint_text_prepare_time_us,
            self.p95_paint_text_prepare_time_us
        );
        if self.p50_renderer_encode_scene_us > 0
            || self.p95_renderer_encode_scene_us > 0
            || self.p50_renderer_upload_us > 0
            || self.p95_renderer_upload_us > 0
            || self.p50_renderer_record_passes_us > 0
            || self.p95_renderer_record_passes_us > 0
        {
            println!(
                "renderer p50/p95 (us): encode={}/{} ensure={}/{} plan={}/{} upload={}/{} record={}/{} finish={}/{} svg={}/{} text={}/{}",
                self.p50_renderer_encode_scene_us,
                self.p95_renderer_encode_scene_us,
                self.p50_renderer_ensure_pipelines_us,
                self.p95_renderer_ensure_pipelines_us,
                self.p50_renderer_plan_compile_us,
                self.p95_renderer_plan_compile_us,
                self.p50_renderer_upload_us,
                self.p95_renderer_upload_us,
                self.p50_renderer_record_passes_us,
                self.p95_renderer_record_passes_us,
                self.p50_renderer_encoder_finish_us,
                self.p95_renderer_encoder_finish_us,
                self.p50_renderer_prepare_svg_us,
                self.p95_renderer_prepare_svg_us,
                self.p50_renderer_prepare_text_us,
                self.p95_renderer_prepare_text_us,
            );
        }
        println!(
            "layout breakdown p50/p95 (us): roots={}/{} request_build_roots={}/{} view_cache={}/{} collapse_obs={}/{} prepaint_after_layout={}/{}",
            self.p50_layout_roots_time_us,
            self.p95_layout_roots_time_us,
            self.p50_layout_request_build_roots_time_us,
            self.p95_layout_request_build_roots_time_us,
            self.p50_layout_view_cache_time_us,
            self.p95_layout_view_cache_time_us,
            self.p50_layout_collapse_layout_observations_time_us,
            self.p95_layout_collapse_layout_observations_time_us,
            self.p50_layout_prepaint_after_layout_time_us,
            self.p95_layout_prepaint_after_layout_time_us
        );
        println!(
            "paint breakdown p50/p95 (us): input_ctx={}/{} scroll_inv={}/{} collect_roots={}/{} text_snapshot={}/{} collapse={}/{}",
            self.p50_paint_input_context_time_us,
            self.p95_paint_input_context_time_us,
            self.p50_paint_scroll_handle_invalidation_time_us,
            self.p95_paint_scroll_handle_invalidation_time_us,
            self.p50_paint_collect_roots_time_us,
            self.p95_paint_collect_roots_time_us,
            self.p50_paint_publish_text_input_snapshot_time_us,
            self.p95_paint_publish_text_input_snapshot_time_us,
            self.p50_paint_collapse_observations_time_us,
            self.p95_paint_collapse_observations_time_us
        );
        if self.sum_layout_observation_record_time_us > 0
            || self.sum_layout_observation_record_models_items > 0
            || self.sum_layout_observation_record_globals_items > 0
            || self.max_layout_observation_record_time_us > 0
        {
            println!(
                "layout obs_record sum (us): time={} items(models/globals)={}/{}",
                self.sum_layout_observation_record_time_us,
                self.sum_layout_observation_record_models_items,
                self.sum_layout_observation_record_globals_items
            );
            println!(
                "layout obs_record max (us): time={} items(models/globals)={}/{}",
                self.max_layout_observation_record_time_us,
                self.max_layout_observation_record_models_items,
                self.max_layout_observation_record_globals_items
            );
        }
        println!(
            "time max (us): total={} layout={} prepaint={} paint={}",
            self.max_total_time_us,
            self.max_layout_time_us,
            self.max_prepaint_time_us,
            self.max_paint_time_us
        );
        if self.max_renderer_encode_scene_us > 0
            || self.max_renderer_upload_us > 0
            || self.max_renderer_record_passes_us > 0
        {
            println!(
                "renderer max (us): encode={} ensure={} plan={} upload={} record={} finish={} svg={} text={}",
                self.max_renderer_encode_scene_us,
                self.max_renderer_ensure_pipelines_us,
                self.max_renderer_plan_compile_us,
                self.max_renderer_upload_us,
                self.max_renderer_record_passes_us,
                self.max_renderer_encoder_finish_us,
                self.max_renderer_prepare_svg_us,
                self.max_renderer_prepare_text_us,
            );
        }
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
            let mut line = format!(
                "  window={} tick={} frame={} ts={} cpu.us={} cpu.cycles={} time.us(total/layout/prepaint/paint)={}/{}/{}/{} layout.solve_us={} paint.cache_misses={} layout.nodes={} paint.nodes={} paint.elem_bounds_us={} paint.elem_bounds_calls={} cache_roots={} cache.reused={} cache.replayed_ops={} cache.replay_us={} cache.translate_us={} cache.translate_nodes={} contained_relayouts={} cache.contained_relayout_roots={} barrier(set_children/scheduled/performed)={}/{}/{} vlist(range_checks/refreshes)={}/{} inv.calls={} inv.nodes={} by_src.calls(hover/focus/other)={}/{}/{} by_src.nodes(hover/focus/other)={}/{}/{} hover.decl_inv(layout/hit/paint)={}/{}/{} roots.model={} roots.global={} changed.models={} changed.globals={} propagated.models={} propagated.edges={} unobs.models={} propagated.globals={} propagated.global_edges={} unobs.globals={}",
                row.window,
                row.tick_id,
                row.frame_id,
                ts,
                row.ui_thread_cpu_time_us,
                row.ui_thread_cpu_cycle_time_delta_cycles,
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
            if row.renderer_encode_scene_us > 0
                || row.renderer_prepare_text_us > 0
                || row.renderer_prepare_svg_us > 0
                || row.renderer_upload_us > 0
                || row.renderer_record_passes_us > 0
            {
                line.push_str(&format!(
                    " renderer.us(encode/ensure/plan/upload/record/finish/svg/text)={}/{}/{}/{}/{}/{}/{}/{}",
                    row.renderer_encode_scene_us,
                    row.renderer_ensure_pipelines_us,
                    row.renderer_plan_compile_us,
                    row.renderer_upload_us,
                    row.renderer_record_passes_us,
                    row.renderer_encoder_finish_us,
                    row.renderer_prepare_svg_us,
                    row.renderer_prepare_text_us,
                ));
            }
            println!("{line}");
            if row.layout_observation_record_time_us > 0
                || row.layout_observation_record_models_items > 0
                || row.layout_observation_record_globals_items > 0
            {
                println!(
                    "    layout_obs_record.us(time)={} items(models/globals)={}/{}",
                    row.layout_observation_record_time_us,
                    row.layout_observation_record_models_items,
                    row.layout_observation_record_globals_items
                );
            }
            if row.layout_roots_time_us > 0
                || row.layout_request_build_roots_time_us > 0
                || row.layout_view_cache_time_us > 0
                || row.layout_collapse_layout_observations_time_us > 0
                || row.layout_prepaint_after_layout_time_us > 0
                || row.layout_expand_view_cache_invalidations_time_us > 0
            {
                println!(
                    "    layout_breakdown.us(roots/request_build_roots/view_cache/collapse_obs/prepaint_after_layout)={}/{}/{}/{}/{} view_cache_inv_us={}",
                    row.layout_roots_time_us,
                    row.layout_request_build_roots_time_us,
                    row.layout_view_cache_time_us,
                    row.layout_collapse_layout_observations_time_us,
                    row.layout_prepaint_after_layout_time_us,
                    row.layout_expand_view_cache_invalidations_time_us,
                );
            }
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
                        if let Some(path) = w.root_element_path.as_deref()
                            && !path.is_empty()
                        {
                            s.push_str(&format!(" element_path={}", elide_middle(path, 120)));
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
                            let path = compact_debug_path(path);
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
                            let path = compact_debug_path(path);
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
                        if let Some(kind) = s.root_element_kind.as_deref()
                            && !kind.is_empty()
                        {
                            out.push_str(&format!(" root.kind={kind}"));
                        }
                        if let Some(el) = s.root_element {
                            out.push_str(&format!(" root.element={el}"));
                        }
                        if let Some(path) = s.root_element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            out.push_str(&format!(" root.path={path}"));
                        }
                        if let Some(m) = s.top_measures.first()
                            && m.measure_time_us > 0
                            && m.node != 0
                        {
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
                            if let Some(c) = m.top_children.first()
                                && c.measure_time_us > 0
                                && c.child != 0
                            {
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
                        out
                    })
                    .collect();
                println!("    top_layout_engine_solves: {}", items.join(" | "));
            }
            if !row.layout_hotspots.is_empty() {
                let items: Vec<String> = row
                    .layout_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut out = format!(
                            "us={} incl.us={} node={}",
                            h.layout_time_us, h.inclusive_time_us, h.node
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            out.push_str(&format!(" role={role}"));
                        }
                        if let Some(widget) = h.widget_type.as_deref()
                            && !widget.is_empty()
                        {
                            out.push_str(&format!(" widget={widget}"));
                        }
                        if let Some(kind) = h.element_kind.as_deref()
                            && !kind.is_empty()
                        {
                            out.push_str(&format!(" kind={kind}"));
                        }
                        if let Some(el) = h.element {
                            out.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = h.element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            out.push_str(&format!(" path={path}"));
                        }
                        out
                    })
                    .collect();
                println!("    layout_hotspots: {}", items.join(" | "));
            }
            if !row.widget_measure_hotspots.is_empty() {
                let items: Vec<String> = row
                    .widget_measure_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut out = format!(
                            "us={} incl.us={} node={}",
                            h.measure_time_us, h.inclusive_time_us, h.node
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            out.push_str(&format!(" role={role}"));
                        }
                        if let Some(widget) = h.widget_type.as_deref()
                            && !widget.is_empty()
                        {
                            out.push_str(&format!(" widget={widget}"));
                        }
                        if let Some(kind) = h.element_kind.as_deref()
                            && !kind.is_empty()
                        {
                            out.push_str(&format!(" kind={kind}"));
                        }
                        if let Some(el) = h.element {
                            out.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = h.element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            out.push_str(&format!(" path={path}"));
                        }
                        out
                    })
                    .collect();
                println!("    widget_measure_hotspots: {}", items.join(" | "));
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

        fn avg_us(sum: u64, n: u32) -> u64 {
            if n == 0 {
                return 0;
            }
            sum / (n as u64)
        }

        fn pct(numer: u64, denom: u64) -> f64 {
            if denom == 0 {
                return 0.0;
            }
            (numer as f64) * 100.0 / (denom as f64)
        }

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
            "layout_collect_roots_time_us".to_string(),
            Value::from(self.sum_layout_collect_roots_time_us),
        );
        sum.insert(
            "layout_invalidate_scroll_handle_bindings_time_us".to_string(),
            Value::from(self.sum_layout_invalidate_scroll_handle_bindings_time_us),
        );
        sum.insert(
            "layout_expand_view_cache_invalidations_time_us".to_string(),
            Value::from(self.sum_layout_expand_view_cache_invalidations_time_us),
        );
        sum.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(self.sum_layout_request_build_roots_time_us),
        );
        sum.insert(
            "layout_roots_time_us".to_string(),
            Value::from(self.sum_layout_roots_time_us),
        );
        sum.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(self.sum_layout_collapse_layout_observations_time_us),
        );
        sum.insert(
            "layout_time_us".to_string(),
            Value::from(self.sum_layout_time_us),
        );
        sum.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(self.sum_layout_view_cache_time_us),
        );
        sum.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(self.sum_layout_prepaint_after_layout_time_us),
        );
        sum.insert(
            "layout_observation_record_time_us".to_string(),
            Value::from(self.sum_layout_observation_record_time_us),
        );
        sum.insert(
            "layout_observation_record_models_items".to_string(),
            Value::from(self.sum_layout_observation_record_models_items),
        );
        sum.insert(
            "layout_observation_record_globals_items".to_string(),
            Value::from(self.sum_layout_observation_record_globals_items),
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
        sum.insert(
            "ui_thread_cpu_time_us".to_string(),
            Value::from(self.sum_ui_thread_cpu_time_us),
        );
        sum.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(self.sum_ui_thread_cpu_cycle_time_delta_cycles),
        );
        sum.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(self.sum_layout_engine_solve_time_us),
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
            "layout_collect_roots_time_us".to_string(),
            Value::from(self.max_layout_collect_roots_time_us),
        );
        max.insert(
            "layout_invalidate_scroll_handle_bindings_time_us".to_string(),
            Value::from(self.max_layout_invalidate_scroll_handle_bindings_time_us),
        );
        max.insert(
            "layout_expand_view_cache_invalidations_time_us".to_string(),
            Value::from(self.max_layout_expand_view_cache_invalidations_time_us),
        );
        max.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(self.max_layout_request_build_roots_time_us),
        );
        max.insert(
            "layout_roots_time_us".to_string(),
            Value::from(self.max_layout_roots_time_us),
        );
        max.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(self.max_layout_collapse_layout_observations_time_us),
        );
        max.insert(
            "layout_time_us".to_string(),
            Value::from(self.max_layout_time_us),
        );
        max.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(self.max_layout_view_cache_time_us),
        );
        max.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(self.max_layout_prepaint_after_layout_time_us),
        );
        max.insert(
            "layout_observation_record_time_us".to_string(),
            Value::from(self.max_layout_observation_record_time_us),
        );
        max.insert(
            "layout_observation_record_models_items".to_string(),
            Value::from(self.max_layout_observation_record_models_items),
        );
        max.insert(
            "layout_observation_record_globals_items".to_string(),
            Value::from(self.max_layout_observation_record_globals_items),
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
            "ui_thread_cpu_time_us".to_string(),
            Value::from(self.max_ui_thread_cpu_time_us),
        );
        max.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(self.max_ui_thread_cpu_cycle_time_delta_cycles),
        );
        max.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(self.max_layout_engine_solve_time_us),
        );
        max.insert(
            "renderer_encode_scene_us".to_string(),
            Value::from(self.max_renderer_encode_scene_us),
        );
        max.insert(
            "renderer_ensure_pipelines_us".to_string(),
            Value::from(self.max_renderer_ensure_pipelines_us),
        );
        max.insert(
            "renderer_plan_compile_us".to_string(),
            Value::from(self.max_renderer_plan_compile_us),
        );
        max.insert(
            "renderer_upload_us".to_string(),
            Value::from(self.max_renderer_upload_us),
        );
        max.insert(
            "renderer_record_passes_us".to_string(),
            Value::from(self.max_renderer_record_passes_us),
        );
        max.insert(
            "renderer_encoder_finish_us".to_string(),
            Value::from(self.max_renderer_encoder_finish_us),
        );
        max.insert(
            "renderer_prepare_svg_us".to_string(),
            Value::from(self.max_renderer_prepare_svg_us),
        );
        max.insert(
            "renderer_prepare_text_us".to_string(),
            Value::from(self.max_renderer_prepare_text_us),
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

        let mut avg = Map::new();
        avg.insert(
            "layout_collect_roots_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_collect_roots_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_invalidate_scroll_handle_bindings_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_invalidate_scroll_handle_bindings_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_expand_view_cache_invalidations_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_expand_view_cache_invalidations_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_request_build_roots_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_roots_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_roots_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_collapse_layout_observations_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_time_us".to_string(),
            Value::from(avg_us(self.sum_layout_time_us, self.snapshots_considered)),
        );
        avg.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_view_cache_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_prepaint_after_layout_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_observation_record_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_observation_record_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_observation_record_models_items".to_string(),
            Value::from(avg_us(
                self.sum_layout_observation_record_models_items,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_observation_record_globals_items".to_string(),
            Value::from(avg_us(
                self.sum_layout_observation_record_globals_items,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "prepaint_time_us".to_string(),
            Value::from(avg_us(self.sum_prepaint_time_us, self.snapshots_considered)),
        );
        avg.insert(
            "paint_time_us".to_string(),
            Value::from(avg_us(self.sum_paint_time_us, self.snapshots_considered)),
        );
        avg.insert(
            "total_time_us".to_string(),
            Value::from(avg_us(self.sum_total_time_us, self.snapshots_considered)),
        );
        avg.insert(
            "ui_thread_cpu_time_us".to_string(),
            Value::from(avg_us(
                self.sum_ui_thread_cpu_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(avg_us(
                self.sum_ui_thread_cpu_cycle_time_delta_cycles,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_engine_solve_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "cache_roots".to_string(),
            Value::from(avg_us(self.sum_cache_roots, self.snapshots_considered)),
        );
        avg.insert(
            "cache_roots_reused".to_string(),
            Value::from(avg_us(
                self.sum_cache_roots_reused,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "cache_replayed_ops".to_string(),
            Value::from(avg_us(
                self.sum_cache_replayed_ops,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(avg_us(
                self.sum_invalidation_walk_calls,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(avg_us(
                self.sum_invalidation_walk_nodes,
                self.snapshots_considered,
            )),
        );
        root.insert("avg".to_string(), Value::Object(avg));

        let mut p50 = Map::new();
        p50.insert(
            "total_time_us".to_string(),
            Value::from(self.p50_total_time_us),
        );
        p50.insert(
            "ui_thread_cpu_time_us".to_string(),
            Value::from(self.p50_ui_thread_cpu_time_us),
        );
        p50.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(self.p50_ui_thread_cpu_cycle_time_delta_cycles),
        );
        p50.insert(
            "layout_time_us".to_string(),
            Value::from(self.p50_layout_time_us),
        );
        p50.insert(
            "layout_collect_roots_time_us".to_string(),
            Value::from(self.p50_layout_collect_roots_time_us),
        );
        p50.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(self.p50_layout_request_build_roots_time_us),
        );
        p50.insert(
            "layout_roots_time_us".to_string(),
            Value::from(self.p50_layout_roots_time_us),
        );
        p50.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(self.p50_layout_view_cache_time_us),
        );
        p50.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(self.p50_layout_collapse_layout_observations_time_us),
        );
        p50.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(self.p50_layout_prepaint_after_layout_time_us),
        );
        p50.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.p50_prepaint_time_us),
        );
        p50.insert(
            "paint_time_us".to_string(),
            Value::from(self.p50_paint_time_us),
        );
        p50.insert(
            "paint_input_context_time_us".to_string(),
            Value::from(self.p50_paint_input_context_time_us),
        );
        p50.insert(
            "paint_scroll_handle_invalidation_time_us".to_string(),
            Value::from(self.p50_paint_scroll_handle_invalidation_time_us),
        );
        p50.insert(
            "paint_collect_roots_time_us".to_string(),
            Value::from(self.p50_paint_collect_roots_time_us),
        );
        p50.insert(
            "paint_publish_text_input_snapshot_time_us".to_string(),
            Value::from(self.p50_paint_publish_text_input_snapshot_time_us),
        );
        p50.insert(
            "paint_collapse_observations_time_us".to_string(),
            Value::from(self.p50_paint_collapse_observations_time_us),
        );
        p50.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(self.p50_layout_engine_solve_time_us),
        );
        p50.insert(
            "dispatch_time_us".to_string(),
            Value::from(self.p50_dispatch_time_us),
        );
        p50.insert(
            "hit_test_time_us".to_string(),
            Value::from(self.p50_hit_test_time_us),
        );
        p50.insert(
            "paint_widget_time_us".to_string(),
            Value::from(self.p50_paint_widget_time_us),
        );
        p50.insert(
            "paint_text_prepare_time_us".to_string(),
            Value::from(self.p50_paint_text_prepare_time_us),
        );
        p50.insert(
            "renderer_encode_scene_us".to_string(),
            Value::from(self.p50_renderer_encode_scene_us),
        );
        p50.insert(
            "renderer_ensure_pipelines_us".to_string(),
            Value::from(self.p50_renderer_ensure_pipelines_us),
        );
        p50.insert(
            "renderer_plan_compile_us".to_string(),
            Value::from(self.p50_renderer_plan_compile_us),
        );
        p50.insert(
            "renderer_upload_us".to_string(),
            Value::from(self.p50_renderer_upload_us),
        );
        p50.insert(
            "renderer_record_passes_us".to_string(),
            Value::from(self.p50_renderer_record_passes_us),
        );
        p50.insert(
            "renderer_encoder_finish_us".to_string(),
            Value::from(self.p50_renderer_encoder_finish_us),
        );
        p50.insert(
            "renderer_prepare_svg_us".to_string(),
            Value::from(self.p50_renderer_prepare_svg_us),
        );
        p50.insert(
            "renderer_prepare_text_us".to_string(),
            Value::from(self.p50_renderer_prepare_text_us),
        );
        root.insert("p50".to_string(), Value::Object(p50));

        let mut p95 = Map::new();
        p95.insert(
            "total_time_us".to_string(),
            Value::from(self.p95_total_time_us),
        );
        p95.insert(
            "ui_thread_cpu_time_us".to_string(),
            Value::from(self.p95_ui_thread_cpu_time_us),
        );
        p95.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(self.p95_ui_thread_cpu_cycle_time_delta_cycles),
        );
        p95.insert(
            "layout_time_us".to_string(),
            Value::from(self.p95_layout_time_us),
        );
        p95.insert(
            "layout_collect_roots_time_us".to_string(),
            Value::from(self.p95_layout_collect_roots_time_us),
        );
        p95.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(self.p95_layout_request_build_roots_time_us),
        );
        p95.insert(
            "layout_roots_time_us".to_string(),
            Value::from(self.p95_layout_roots_time_us),
        );
        p95.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(self.p95_layout_view_cache_time_us),
        );
        p95.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(self.p95_layout_collapse_layout_observations_time_us),
        );
        p95.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(self.p95_layout_prepaint_after_layout_time_us),
        );
        p95.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.p95_prepaint_time_us),
        );
        p95.insert(
            "paint_time_us".to_string(),
            Value::from(self.p95_paint_time_us),
        );
        p95.insert(
            "paint_input_context_time_us".to_string(),
            Value::from(self.p95_paint_input_context_time_us),
        );
        p95.insert(
            "paint_scroll_handle_invalidation_time_us".to_string(),
            Value::from(self.p95_paint_scroll_handle_invalidation_time_us),
        );
        p95.insert(
            "paint_collect_roots_time_us".to_string(),
            Value::from(self.p95_paint_collect_roots_time_us),
        );
        p95.insert(
            "paint_publish_text_input_snapshot_time_us".to_string(),
            Value::from(self.p95_paint_publish_text_input_snapshot_time_us),
        );
        p95.insert(
            "paint_collapse_observations_time_us".to_string(),
            Value::from(self.p95_paint_collapse_observations_time_us),
        );
        p95.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(self.p95_layout_engine_solve_time_us),
        );
        p95.insert(
            "dispatch_time_us".to_string(),
            Value::from(self.p95_dispatch_time_us),
        );
        p95.insert(
            "hit_test_time_us".to_string(),
            Value::from(self.p95_hit_test_time_us),
        );
        p95.insert(
            "paint_widget_time_us".to_string(),
            Value::from(self.p95_paint_widget_time_us),
        );
        p95.insert(
            "paint_text_prepare_time_us".to_string(),
            Value::from(self.p95_paint_text_prepare_time_us),
        );
        p95.insert(
            "renderer_encode_scene_us".to_string(),
            Value::from(self.p95_renderer_encode_scene_us),
        );
        p95.insert(
            "renderer_ensure_pipelines_us".to_string(),
            Value::from(self.p95_renderer_ensure_pipelines_us),
        );
        p95.insert(
            "renderer_plan_compile_us".to_string(),
            Value::from(self.p95_renderer_plan_compile_us),
        );
        p95.insert(
            "renderer_upload_us".to_string(),
            Value::from(self.p95_renderer_upload_us),
        );
        p95.insert(
            "renderer_record_passes_us".to_string(),
            Value::from(self.p95_renderer_record_passes_us),
        );
        p95.insert(
            "renderer_encoder_finish_us".to_string(),
            Value::from(self.p95_renderer_encoder_finish_us),
        );
        p95.insert(
            "renderer_prepare_svg_us".to_string(),
            Value::from(self.p95_renderer_prepare_svg_us),
        );
        p95.insert(
            "renderer_prepare_text_us".to_string(),
            Value::from(self.p95_renderer_prepare_text_us),
        );
        root.insert("p95".to_string(), Value::Object(p95));

        root.insert(
            "budget_pct".to_string(),
            serde_json::json!({
                "layout_of_total": pct(self.sum_layout_time_us, self.sum_total_time_us),
                "prepaint_of_total": pct(self.sum_prepaint_time_us, self.sum_total_time_us),
                "paint_of_total": pct(self.sum_paint_time_us, self.sum_total_time_us),
                "layout_obs_record_of_layout": pct(self.sum_layout_observation_record_time_us, self.sum_layout_time_us),
                "layout_obs_record_of_total": pct(self.sum_layout_observation_record_time_us, self.sum_total_time_us),
            }),
        );

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
                    "ui_thread_cpu_time_us".to_string(),
                    Value::from(row.ui_thread_cpu_time_us),
                );
                obj.insert(
                    "ui_thread_cpu_total_time_us".to_string(),
                    Value::from(row.ui_thread_cpu_total_time_us),
                );
                obj.insert(
                    "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
                    Value::from(row.ui_thread_cpu_cycle_time_delta_cycles),
                );
                obj.insert(
                    "ui_thread_cpu_cycle_time_total_cycles".to_string(),
                    Value::from(row.ui_thread_cpu_cycle_time_total_cycles),
                );
                obj.insert(
                    "layout_time_us".to_string(),
                    Value::from(row.layout_time_us),
                );
                obj.insert(
                    "renderer_tick_id".to_string(),
                    Value::from(row.renderer_tick_id),
                );
                obj.insert(
                    "renderer_frame_id".to_string(),
                    Value::from(row.renderer_frame_id),
                );
                obj.insert(
                    "renderer_encode_scene_us".to_string(),
                    Value::from(row.renderer_encode_scene_us),
                );
                obj.insert(
                    "renderer_ensure_pipelines_us".to_string(),
                    Value::from(row.renderer_ensure_pipelines_us),
                );
                obj.insert(
                    "renderer_plan_compile_us".to_string(),
                    Value::from(row.renderer_plan_compile_us),
                );
                obj.insert(
                    "renderer_upload_us".to_string(),
                    Value::from(row.renderer_upload_us),
                );
                obj.insert(
                    "renderer_record_passes_us".to_string(),
                    Value::from(row.renderer_record_passes_us),
                );
                obj.insert(
                    "renderer_encoder_finish_us".to_string(),
                    Value::from(row.renderer_encoder_finish_us),
                );
                obj.insert(
                    "renderer_prepare_svg_us".to_string(),
                    Value::from(row.renderer_prepare_svg_us),
                );
                obj.insert(
                    "renderer_prepare_text_us".to_string(),
                    Value::from(row.renderer_prepare_text_us),
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
                    "layout_roots_time_us".to_string(),
                    Value::from(row.layout_roots_time_us),
                );
                obj.insert(
                    "layout_pending_barrier_relayouts_time_us".to_string(),
                    Value::from(row.layout_pending_barrier_relayouts_time_us),
                );
                obj.insert(
                    "layout_barrier_relayouts_time_us".to_string(),
                    Value::from(row.layout_barrier_relayouts_time_us),
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
                    "layout_observation_record_time_us".to_string(),
                    Value::from(row.layout_observation_record_time_us),
                );
                obj.insert(
                    "layout_observation_record_models_items".to_string(),
                    Value::from(row.layout_observation_record_models_items),
                );
                obj.insert(
                    "layout_observation_record_globals_items".to_string(),
                    Value::from(row.layout_observation_record_globals_items),
                );
                obj.insert(
                    "layout_view_cache_time_us".to_string(),
                    Value::from(row.layout_view_cache_time_us),
                );
                obj.insert(
                    "layout_semantics_refresh_time_us".to_string(),
                    Value::from(row.layout_semantics_refresh_time_us),
                );
                obj.insert(
                    "layout_focus_repair_time_us".to_string(),
                    Value::from(row.layout_focus_repair_time_us),
                );
                obj.insert(
                    "layout_deferred_cleanup_time_us".to_string(),
                    Value::from(row.layout_deferred_cleanup_time_us),
                );
                obj.insert(
                    "layout_prepaint_after_layout_time_us".to_string(),
                    Value::from(row.layout_prepaint_after_layout_time_us),
                );
                obj.insert(
                    "layout_skipped_engine_frame".to_string(),
                    Value::from(row.layout_skipped_engine_frame),
                );
                obj.insert(
                    "layout_fast_path_taken".to_string(),
                    Value::from(row.layout_fast_path_taken),
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
                            "root_element_path".to_string(),
                            w.root_element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
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
                        s_obj.insert(
                            "root_element".to_string(),
                            s.root_element.map(Value::from).unwrap_or(Value::Null),
                        );
                        s_obj.insert(
                            "root_element_kind".to_string(),
                            s.root_element_kind
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        s_obj.insert(
                            "root_element_path".to_string(),
                            s.root_element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
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

                let layout_hotspots = row
                    .layout_hotspots
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
                            "element_path".to_string(),
                            h.element_path
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
                        h_obj.insert("layout_time_us".to_string(), Value::from(h.layout_time_us));
                        h_obj.insert(
                            "inclusive_time_us".to_string(),
                            Value::from(h.inclusive_time_us),
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
                obj.insert("layout_hotspots".to_string(), Value::Array(layout_hotspots));

                let widget_measure_hotspots = row
                    .widget_measure_hotspots
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
                            "element_path".to_string(),
                            h.element_path
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
                        h_obj.insert(
                            "measure_time_us".to_string(),
                            Value::from(h.measure_time_us),
                        );
                        h_obj.insert(
                            "inclusive_time_us".to_string(),
                            Value::from(h.inclusive_time_us),
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
                    "widget_measure_hotspots".to_string(),
                    Value::Array(widget_measure_hotspots),
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
                            h.max_width.map(Value::from).unwrap_or(Value::Null),
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
                            h.scale_factor.map(Value::from).unwrap_or(Value::Null),
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

#[derive(Debug, Clone)]
pub(super) struct BundleStatsDiffReport {
    a_path: PathBuf,
    b_path: PathBuf,
    sort: BundleStatsSort,
    warmup_frames: u64,
    top: usize,
    deltas: Vec<BundleStatsDiffDelta>,
}

#[derive(Debug, Clone)]
pub(super) struct BundleStatsDiffDelta {
    key: &'static str,
    a: u64,
    b: u64,
}

impl BundleStatsDiffDelta {
    fn delta_us(&self) -> i64 {
        (self.b as i64).saturating_sub(self.a as i64)
    }

    fn delta_pct(&self) -> Option<f64> {
        if self.a == 0 {
            return None;
        }
        Some(((self.b as f64) - (self.a as f64)) * 100.0 / (self.a as f64))
    }

    fn abs_delta_us(&self) -> u64 {
        self.delta_us().unsigned_abs()
    }
}

impl BundleStatsDiffReport {
    pub(super) fn print_human(&self) {
        println!("bundle_a: {}", self.a_path.display());
        println!("bundle_b: {}", self.b_path.display());
        println!(
            "diff: sort={} warmup_frames={}",
            self.sort.as_str(),
            self.warmup_frames
        );
        if self.deltas.is_empty() {
            println!("diff: ok (no metrics)");
            return;
        }

        println!("top (by |delta_us|):");
        for d in self.deltas.iter().take(self.top.max(1)) {
            let delta_us = d.delta_us();
            let sign = if delta_us >= 0 { "+" } else { "-" };
            let abs = delta_us.unsigned_abs();
            let pct = d
                .delta_pct()
                .map(|v| format!("{v:.1}%"))
                .unwrap_or_else(|| "-".to_string());
            println!(
                "  {key}: a={a} b={b} delta_us={sign}{abs} delta_pct={pct}",
                key = d.key,
                a = d.a,
                b = d.b
            );
        }
    }

    pub(super) fn to_json(&self) -> serde_json::Value {
        let deltas = self
            .deltas
            .iter()
            .map(|d| {
                serde_json::json!({
                    "key": d.key,
                    "a": d.a,
                    "b": d.b,
                    "delta_us": d.delta_us(),
                    "delta_pct": d.delta_pct(),
                    "abs_delta_us": d.abs_delta_us(),
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "schema_version": 1,
            "bundle_a": self.a_path.display().to_string(),
            "bundle_b": self.b_path.display().to_string(),
            "sort": self.sort.as_str(),
            "warmup_frames": self.warmup_frames,
            "top": self.top,
            "deltas": deltas,
        })
    }
}

fn sort_diff_deltas_in_place(deltas: &mut [BundleStatsDiffDelta]) {
    deltas.sort_by(|a, b| {
        b.abs_delta_us()
            .cmp(&a.abs_delta_us())
            .then_with(|| a.key.cmp(b.key))
    });
}

pub(super) fn bundle_stats_diff_from_paths(
    a_bundle_path: &Path,
    b_bundle_path: &Path,
    top: usize,
    sort: BundleStatsSort,
    opts: BundleStatsOptions,
) -> Result<BundleStatsDiffReport, String> {
    let mut a = bundle_stats_from_path(a_bundle_path, 0, sort, opts)?;
    let mut b = bundle_stats_from_path(b_bundle_path, 0, sort, opts)?;
    if opts.warmup_frames > 0 && (a.snapshots_considered == 0 || b.snapshots_considered == 0) {
        let fallback_opts = BundleStatsOptions::default();
        if a.snapshots_considered == 0 {
            a = bundle_stats_from_path(a_bundle_path, 0, sort, fallback_opts)?;
        }
        if b.snapshots_considered == 0 {
            b = bundle_stats_from_path(b_bundle_path, 0, sort, fallback_opts)?;
        }
    }

    // Curated, time-in-us metrics (keep this list small and stable).
    let mut deltas = vec![
        BundleStatsDiffDelta {
            key: "avg.total_time_us",
            a: if a.snapshots_considered == 0 {
                0
            } else {
                a.sum_total_time_us / (a.snapshots_considered as u64)
            },
            b: if b.snapshots_considered == 0 {
                0
            } else {
                b.sum_total_time_us / (b.snapshots_considered as u64)
            },
        },
        BundleStatsDiffDelta {
            key: "avg.layout_time_us",
            a: if a.snapshots_considered == 0 {
                0
            } else {
                a.sum_layout_time_us / (a.snapshots_considered as u64)
            },
            b: if b.snapshots_considered == 0 {
                0
            } else {
                b.sum_layout_time_us / (b.snapshots_considered as u64)
            },
        },
        BundleStatsDiffDelta {
            key: "avg.layout_request_build_roots_time_us",
            a: if a.snapshots_considered == 0 {
                0
            } else {
                a.sum_layout_request_build_roots_time_us / (a.snapshots_considered as u64)
            },
            b: if b.snapshots_considered == 0 {
                0
            } else {
                b.sum_layout_request_build_roots_time_us / (b.snapshots_considered as u64)
            },
        },
        BundleStatsDiffDelta {
            key: "avg.layout_roots_time_us",
            a: if a.snapshots_considered == 0 {
                0
            } else {
                a.sum_layout_roots_time_us / (a.snapshots_considered as u64)
            },
            b: if b.snapshots_considered == 0 {
                0
            } else {
                b.sum_layout_roots_time_us / (b.snapshots_considered as u64)
            },
        },
        BundleStatsDiffDelta {
            key: "avg.layout_engine_solve_time_us",
            a: if a.snapshots_considered == 0 {
                0
            } else {
                a.sum_layout_engine_solve_time_us / (a.snapshots_considered as u64)
            },
            b: if b.snapshots_considered == 0 {
                0
            } else {
                b.sum_layout_engine_solve_time_us / (b.snapshots_considered as u64)
            },
        },
        BundleStatsDiffDelta {
            key: "avg.prepaint_time_us",
            a: if a.snapshots_considered == 0 {
                0
            } else {
                a.sum_prepaint_time_us / (a.snapshots_considered as u64)
            },
            b: if b.snapshots_considered == 0 {
                0
            } else {
                b.sum_prepaint_time_us / (b.snapshots_considered as u64)
            },
        },
        BundleStatsDiffDelta {
            key: "avg.paint_time_us",
            a: if a.snapshots_considered == 0 {
                0
            } else {
                a.sum_paint_time_us / (a.snapshots_considered as u64)
            },
            b: if b.snapshots_considered == 0 {
                0
            } else {
                b.sum_paint_time_us / (b.snapshots_considered as u64)
            },
        },
        BundleStatsDiffDelta {
            key: "avg.layout_obs_record_time_us",
            a: if a.snapshots_considered == 0 {
                0
            } else {
                a.sum_layout_observation_record_time_us / (a.snapshots_considered as u64)
            },
            b: if b.snapshots_considered == 0 {
                0
            } else {
                b.sum_layout_observation_record_time_us / (b.snapshots_considered as u64)
            },
        },
        BundleStatsDiffDelta {
            key: "max.total_time_us",
            a: a.max_total_time_us,
            b: b.max_total_time_us,
        },
        BundleStatsDiffDelta {
            key: "max.layout_time_us",
            a: a.max_layout_time_us,
            b: b.max_layout_time_us,
        },
        BundleStatsDiffDelta {
            key: "max.layout_request_build_roots_time_us",
            a: a.max_layout_request_build_roots_time_us,
            b: b.max_layout_request_build_roots_time_us,
        },
        BundleStatsDiffDelta {
            key: "max.layout_roots_time_us",
            a: a.max_layout_roots_time_us,
            b: b.max_layout_roots_time_us,
        },
        BundleStatsDiffDelta {
            key: "max.layout_engine_solve_time_us",
            a: a.max_layout_engine_solve_time_us,
            b: b.max_layout_engine_solve_time_us,
        },
        BundleStatsDiffDelta {
            key: "max.prepaint_time_us",
            a: a.max_prepaint_time_us,
            b: b.max_prepaint_time_us,
        },
        BundleStatsDiffDelta {
            key: "max.paint_time_us",
            a: a.max_paint_time_us,
            b: b.max_paint_time_us,
        },
        BundleStatsDiffDelta {
            key: "max.layout_obs_record_time_us",
            a: a.max_layout_observation_record_time_us,
            b: b.max_layout_observation_record_time_us,
        },
        BundleStatsDiffDelta {
            key: "pointer_move.max_dispatch_time_us",
            a: a.pointer_move_max_dispatch_time_us,
            b: b.pointer_move_max_dispatch_time_us,
        },
        BundleStatsDiffDelta {
            key: "pointer_move.max_hit_test_time_us",
            a: a.pointer_move_max_hit_test_time_us,
            b: b.pointer_move_max_hit_test_time_us,
        },
    ];

    sort_diff_deltas_in_place(&mut deltas);

    Ok(BundleStatsDiffReport {
        a_path: a_bundle_path.to_path_buf(),
        b_path: b_bundle_path.to_path_buf(),
        sort,
        warmup_frames: opts.warmup_frames,
        top,
        deltas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_diff_sorts_by_abs_delta_then_key() {
        let mut deltas = vec![
            BundleStatsDiffDelta {
                key: "b",
                a: 10,
                b: 20,
            }, // +10
            BundleStatsDiffDelta {
                key: "a",
                a: 30,
                b: 20,
            }, // -10
            BundleStatsDiffDelta {
                key: "z",
                a: 0,
                b: 25,
            }, // +25
        ];
        sort_diff_deltas_in_place(&mut deltas);
        assert_eq!(deltas[0].key, "z");
        assert_eq!(deltas[1].key, "a");
        assert_eq!(deltas[2].key, "b");
    }

    #[test]
    fn stats_json_includes_avg_and_budget() {
        let report = BundleStatsReport {
            sort: BundleStatsSort::Time,
            snapshots_considered: 2,
            sum_total_time_us: 100,
            sum_layout_time_us: 40,
            sum_prepaint_time_us: 10,
            sum_paint_time_us: 50,
            sum_layout_observation_record_time_us: 6,
            ..Default::default()
        };

        let json = report.to_json();
        assert!(json.get("avg").is_some());
        assert!(json.get("budget_pct").is_some());
    }
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
    stale::check_bundle_for_stale_paint(bundle_path, test_id, eps)
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_stale_paint_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    stale::check_bundle_for_stale_paint_json(bundle, bundle_path, test_id, eps)
}

pub(super) fn check_bundle_for_stale_scene(
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    stale::check_bundle_for_stale_scene(bundle_path, test_id, eps)
}

pub(super) fn check_bundle_for_semantics_changed_repainted(
    bundle_path: &Path,
    warmup_frames: u64,
    dump_json: bool,
) -> Result<(), String> {
    stale::check_bundle_for_semantics_changed_repainted(bundle_path, warmup_frames, dump_json)
}

#[cfg(test)]
pub(super) fn check_bundle_for_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    stale::check_bundle_for_semantics_changed_repainted_json(bundle, bundle_path, warmup_frames)
}

#[cfg(test)]
pub(super) fn scan_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    warmup_frames: u64,
) -> SemanticsChangedRepaintedScan {
    stale::scan_semantics_changed_repainted_json(bundle, warmup_frames)
}

#[cfg(test)]
pub(super) fn check_bundle_for_stale_scene_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    stale::check_bundle_for_stale_scene_json(bundle, bundle_path, test_id, eps)
}

pub(super) fn check_bundle_for_wheel_scroll(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    wheel_scroll::check_bundle_for_wheel_scroll(bundle_path, test_id, warmup_frames)
}

pub(super) fn check_bundle_for_wheel_scroll_hit_changes(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    wheel_scroll::check_bundle_for_wheel_scroll_hit_changes(bundle_path, test_id, warmup_frames)
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_wheel_scroll_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    wheel_scroll::check_bundle_for_wheel_scroll_json(bundle, bundle_path, test_id, warmup_frames)
}

#[cfg(test)]
pub(super) fn check_bundle_for_wheel_scroll_hit_changes_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    wheel_scroll::check_bundle_for_wheel_scroll_hit_changes_json(
        bundle,
        bundle_path,
        test_id,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_visible_range_refreshes_max(
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
    vlist::check_bundle_for_vlist_window_shifts_explainable(bundle_path, out_dir, warmup_frames)
}

pub(super) fn check_bundle_for_vlist_window_shifts_non_retained_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_total_non_retained_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_window_shifts_non_retained_max(
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
    vlist::check_bundle_for_vlist_window_shifts_kind_max(
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
    vlist::check_bundle_for_vlist_policy_key_stable(bundle_path, out_dir, warmup_frames)
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_vlist_policy_key_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_policy_key_stable_json(
        bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_visible_range_refreshes_min(
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
    windowed_rows::check_bundle_for_windowed_rows_offset_changes_min(
        bundle_path,
        out_dir,
        min_total_offset_changes,
        warmup_frames,
        eps_px,
    )
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_windowed_rows_offset_changes_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_total_offset_changes: u64,
    warmup_frames: u64,
    eps_px: f32,
) -> Result<(), String> {
    windowed_rows::check_bundle_for_windowed_rows_offset_changes_min_json(
        bundle,
        bundle_path,
        out_dir,
        min_total_offset_changes,
        warmup_frames,
        eps_px,
    )
}

pub(super) fn check_bundle_for_windowed_rows_visible_start_changes_repainted(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    windowed_rows::check_bundle_for_windowed_rows_visible_start_changes_repainted(
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    windowed_rows::check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
        bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_layout_fast_path_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    debug_stats_gates::check_bundle_for_layout_fast_path_min(
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
    debug_stats_gates::check_bundle_for_layout_fast_path_min_json(
        bundle,
        bundle_path,
        out_dir,
        min_frames,
        warmup_frames,
    )
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_vlist_visible_range_refreshes_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_visible_range_refreshes_min_json(
        bundle,
        bundle_path,
        out_dir,
        min_total_refreshes,
        warmup_frames,
    )
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_vlist_window_shifts_explainable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_window_shifts_explainable_json(
        bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_prepaint_actions_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_snapshots: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    debug_stats_gates::check_bundle_for_prepaint_actions_min(
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
    debug_stats_gates::check_bundle_for_prepaint_actions_min_json(
        bundle,
        bundle_path,
        out_dir,
        min_snapshots,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_chart_sampling_window_shifts_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    debug_stats_gates::check_bundle_for_chart_sampling_window_shifts_min(
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
    debug_stats_gates::check_bundle_for_chart_sampling_window_shifts_min_json(
        bundle,
        bundle_path,
        out_dir,
        min_actions,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_node_graph_cull_window_shifts_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    debug_stats_gates::check_bundle_for_node_graph_cull_window_shifts_min(
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
    debug_stats_gates::check_bundle_for_node_graph_cull_window_shifts_min_json(
        bundle,
        bundle_path,
        out_dir,
        min_actions,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_node_graph_cull_window_shifts_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    debug_stats_gates::check_bundle_for_node_graph_cull_window_shifts_max(
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
    debug_stats_gates::check_bundle_for_node_graph_cull_window_shifts_max_json(
        bundle,
        bundle_path,
        out_dir,
        max_actions,
        warmup_frames,
    )
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_vlist_window_shifts_non_retained_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_total_non_retained_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_window_shifts_non_retained_max_json(
        bundle,
        bundle_path,
        out_dir,
        max_total_non_retained_shifts,
        warmup_frames,
    )
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_vlist_window_shifts_kind_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    kind: &str,
    max_total_kind_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_window_shifts_kind_max_json(
        bundle,
        bundle_path,
        out_dir,
        kind,
        max_total_kind_shifts,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_window_shifts_have_prepaint_actions(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_window_shifts_have_prepaint_actions(
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_vlist_window_shifts_have_prepaint_actions_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_window_shifts_have_prepaint_actions_json(
        bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn check_bundle_for_vlist_visible_range_refreshes_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    vlist::check_bundle_for_vlist_visible_range_refreshes_max_json(
        bundle,
        bundle_path,
        out_dir,
        max_total_refreshes,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_drag_cache_root_paint_only(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    drag_cache_gates::check_bundle_for_drag_cache_root_paint_only(
        bundle_path,
        test_id,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_gc_sweep_liveness(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    gc_gates::check_bundle_for_gc_sweep_liveness(bundle_path, warmup_frames)
}

pub(super) fn check_bundle_for_view_cache_reuse_min(
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    view_cache_gates::check_bundle_for_view_cache_reuse_min(
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
    view_cache_gates::check_bundle_for_view_cache_reuse_min_json(
        bundle,
        bundle_path,
        min_reuse_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_view_cache_reuse_stable_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_tail_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    view_cache_gates::check_bundle_for_view_cache_reuse_stable_min(
        bundle_path,
        out_dir,
        min_tail_frames,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_overlay_synthesis_min(
    bundle_path: &Path,
    min_synthesized_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    overlay_gates::check_bundle_for_overlay_synthesis_min(
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
    overlay_gates::check_bundle_for_overlay_synthesis_min_json(
        bundle,
        bundle_path,
        min_synthesized_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_reconcile_no_notify_min(
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    retained_vlist_gates::check_bundle_for_retained_vlist_reconcile_no_notify_min(
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
    retained_vlist_gates::check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
        bundle,
        bundle_path,
        min_reconcile_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_attach_detach_max(
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    retained_vlist_gates::check_bundle_for_retained_vlist_attach_detach_max(
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
    retained_vlist_gates::check_bundle_for_retained_vlist_attach_detach_max_json(
        bundle,
        bundle_path,
        max_delta,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_viewport_input_min(
    bundle_path: &Path,
    min_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    interaction_gates::check_bundle_for_viewport_input_min(bundle_path, min_events, warmup_frames)
}

pub(super) fn check_bundle_for_viewport_input_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    interaction_gates::check_bundle_for_viewport_input_min_json(
        bundle,
        bundle_path,
        min_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_dock_drag_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    interaction_gates::check_bundle_for_dock_drag_min(bundle_path, min_active_frames, warmup_frames)
}

pub(super) fn check_bundle_for_dock_drag_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    interaction_gates::check_bundle_for_dock_drag_min_json(
        bundle,
        bundle_path,
        min_active_frames,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_viewport_capture_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    interaction_gates::check_bundle_for_viewport_capture_min(
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
    interaction_gates::check_bundle_for_viewport_capture_min_json(
        bundle,
        bundle_path,
        min_active_frames,
        warmup_frames,
    )
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

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut out = BundleStatsReport {
        sort,
        warmup_frames: opts.warmup_frames,
        windows: windows.len().min(u32::MAX as usize) as u32,
        ..Default::default()
    };

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
            let ui_thread_cpu_time_us = stats
                .and_then(|m| m.get("ui_thread_cpu_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let ui_thread_cpu_total_time_us = stats
                .and_then(|m| m.get("ui_thread_cpu_total_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let ui_thread_cpu_cycle_time_delta_cycles = stats
                .and_then(|m| m.get("ui_thread_cpu_cycle_time_delta_cycles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let ui_thread_cpu_cycle_time_total_cycles = stats
                .and_then(|m| m.get("ui_thread_cpu_cycle_time_total_cycles"))
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
            let renderer_ensure_pipelines_us = stats
                .and_then(|m| m.get("renderer_ensure_pipelines_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_plan_compile_us = stats
                .and_then(|m| m.get("renderer_plan_compile_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_upload_us = stats
                .and_then(|m| m.get("renderer_upload_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_record_passes_us = stats
                .and_then(|m| m.get("renderer_record_passes_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encoder_finish_us = stats
                .and_then(|m| m.get("renderer_encoder_finish_us"))
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

            let renderer_render_target_updates_ingest_unknown = stats
                .and_then(|m| m.get("renderer_render_target_updates_ingest_unknown"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_ingest_owned = stats
                .and_then(|m| m.get("renderer_render_target_updates_ingest_owned"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_ingest_external_zero_copy = stats
                .and_then(|m| m.get("renderer_render_target_updates_ingest_external_zero_copy"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_ingest_gpu_copy = stats
                .and_then(|m| m.get("renderer_render_target_updates_ingest_gpu_copy"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_ingest_cpu_upload = stats
                .and_then(|m| m.get("renderer_render_target_updates_ingest_cpu_upload"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_requested_ingest_unknown = stats
                .and_then(|m| m.get("renderer_render_target_updates_requested_ingest_unknown"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_requested_ingest_owned = stats
                .and_then(|m| m.get("renderer_render_target_updates_requested_ingest_owned"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_requested_ingest_external_zero_copy = stats
                .and_then(|m| {
                    m.get("renderer_render_target_updates_requested_ingest_external_zero_copy")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_requested_ingest_gpu_copy = stats
                .and_then(|m| m.get("renderer_render_target_updates_requested_ingest_gpu_copy"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_requested_ingest_cpu_upload = stats
                .and_then(|m| m.get("renderer_render_target_updates_requested_ingest_cpu_upload"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_target_updates_ingest_fallbacks = stats
                .and_then(|m| m.get("renderer_render_target_updates_ingest_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let renderer_viewport_draw_calls = stats
                .and_then(|m| m.get("renderer_viewport_draw_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_viewport_draw_calls_ingest_unknown = stats
                .and_then(|m| m.get("renderer_viewport_draw_calls_ingest_unknown"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_viewport_draw_calls_ingest_owned = stats
                .and_then(|m| m.get("renderer_viewport_draw_calls_ingest_owned"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_viewport_draw_calls_ingest_external_zero_copy = stats
                .and_then(|m| m.get("renderer_viewport_draw_calls_ingest_external_zero_copy"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_viewport_draw_calls_ingest_gpu_copy = stats
                .and_then(|m| m.get("renderer_viewport_draw_calls_ingest_gpu_copy"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_viewport_draw_calls_ingest_cpu_upload = stats
                .and_then(|m| m.get("renderer_viewport_draw_calls_ingest_cpu_upload"))
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
            let renderer_material_quad_ops = stats
                .and_then(|m| m.get("renderer_material_quad_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_material_sampled_quad_ops = stats
                .and_then(|m| m.get("renderer_material_sampled_quad_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_material_distinct = stats
                .and_then(|m| m.get("renderer_material_distinct"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_material_unknown_ids = stats
                .and_then(|m| m.get("renderer_material_unknown_ids"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_material_degraded_due_to_budget = stats
                .and_then(|m| m.get("renderer_material_degraded_due_to_budget"))
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
            let layout_roots_time_us = stats
                .and_then(|m| m.get("layout_roots_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_pending_barrier_relayouts_time_us = stats
                .and_then(|m| m.get("layout_pending_barrier_relayouts_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_barrier_relayouts_time_us = stats
                .and_then(|m| m.get("layout_barrier_relayouts_time_us"))
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
            let layout_observation_record_time_us = stats
                .and_then(|m| m.get("layout_observation_record_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_observation_record_models_items = stats
                .and_then(|m| m.get("layout_observation_record_models_items"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let layout_observation_record_globals_items = stats
                .and_then(|m| m.get("layout_observation_record_globals_items"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let layout_view_cache_time_us = stats
                .and_then(|m| m.get("layout_view_cache_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_semantics_refresh_time_us = stats
                .and_then(|m| m.get("layout_semantics_refresh_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_focus_repair_time_us = stats
                .and_then(|m| m.get("layout_focus_repair_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_deferred_cleanup_time_us = stats
                .and_then(|m| m.get("layout_deferred_cleanup_time_us"))
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
            let layout_fast_path_taken = stats
                .and_then(|m| m.get("layout_fast_path_taken"))
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

            let top_invalidation_walks = snapshot_top_invalidation_walks(&semantics, s, 3);
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
                snapshot_top_hover_declarative_invalidations(&semantics, s, 3);
            let (
                cache_roots,
                cache_roots_reused,
                cache_roots_contained_relayout,
                cache_replayed_ops,
                top_cache_roots,
                top_contained_relayout_cache_roots,
            ) = snapshot_cache_root_stats(&semantics, s, 3);
            let top_layout_engine_solves = snapshot_layout_engine_solves(&semantics, s, 3);
            let layout_hotspots = snapshot_layout_hotspots(&semantics, s, 3);
            let widget_measure_hotspots = snapshot_widget_measure_hotspots(&semantics, s, 3);
            let paint_widget_hotspots = snapshot_paint_widget_hotspots(&semantics, s, 3);
            let paint_text_prepare_hotspots =
                snapshot_paint_text_prepare_hotspots(&semantics, s, 3);
            let model_change_hotspots = snapshot_model_change_hotspots(s, 3);
            let model_change_unobserved = snapshot_model_change_unobserved(s, 3);
            let global_change_hotspots = snapshot_global_change_hotspots(s, 3);
            let global_change_unobserved = snapshot_global_change_unobserved(s, 3);

            out.sum_layout_time_us = out.sum_layout_time_us.saturating_add(layout_time_us);
            out.sum_layout_collect_roots_time_us = out
                .sum_layout_collect_roots_time_us
                .saturating_add(layout_collect_roots_time_us);
            out.sum_layout_invalidate_scroll_handle_bindings_time_us = out
                .sum_layout_invalidate_scroll_handle_bindings_time_us
                .saturating_add(layout_invalidate_scroll_handle_bindings_time_us);
            out.sum_layout_expand_view_cache_invalidations_time_us = out
                .sum_layout_expand_view_cache_invalidations_time_us
                .saturating_add(layout_expand_view_cache_invalidations_time_us);
            out.sum_layout_request_build_roots_time_us = out
                .sum_layout_request_build_roots_time_us
                .saturating_add(layout_request_build_roots_time_us);
            out.sum_layout_roots_time_us = out
                .sum_layout_roots_time_us
                .saturating_add(layout_roots_time_us);
            out.sum_layout_collapse_layout_observations_time_us = out
                .sum_layout_collapse_layout_observations_time_us
                .saturating_add(layout_collapse_layout_observations_time_us);
            out.sum_layout_view_cache_time_us = out
                .sum_layout_view_cache_time_us
                .saturating_add(layout_view_cache_time_us);
            out.sum_layout_prepaint_after_layout_time_us = out
                .sum_layout_prepaint_after_layout_time_us
                .saturating_add(layout_prepaint_after_layout_time_us);
            out.sum_layout_observation_record_time_us = out
                .sum_layout_observation_record_time_us
                .saturating_add(layout_observation_record_time_us);
            out.sum_layout_observation_record_models_items = out
                .sum_layout_observation_record_models_items
                .saturating_add(layout_observation_record_models_items as u64);
            out.sum_layout_observation_record_globals_items = out
                .sum_layout_observation_record_globals_items
                .saturating_add(layout_observation_record_globals_items as u64);
            out.sum_prepaint_time_us = out.sum_prepaint_time_us.saturating_add(prepaint_time_us);
            out.sum_paint_time_us = out.sum_paint_time_us.saturating_add(paint_time_us);
            out.sum_total_time_us = out.sum_total_time_us.saturating_add(total_time_us);
            out.sum_ui_thread_cpu_time_us = out
                .sum_ui_thread_cpu_time_us
                .saturating_add(ui_thread_cpu_time_us);
            out.sum_ui_thread_cpu_cycle_time_delta_cycles = out
                .sum_ui_thread_cpu_cycle_time_delta_cycles
                .saturating_add(ui_thread_cpu_cycle_time_delta_cycles);
            out.sum_layout_engine_solve_time_us = out
                .sum_layout_engine_solve_time_us
                .saturating_add(layout_engine_solve_time_us);
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
                    hotspots: snapshot_top_hover_declarative_invalidations(&semantics, s, 8),
                });
            }
            out.max_hover_layout_invalidations = out
                .max_hover_layout_invalidations
                .max(hover_declarative_layout_invalidations);
            out.max_layout_time_us = out.max_layout_time_us.max(layout_time_us);
            out.max_layout_collect_roots_time_us = out
                .max_layout_collect_roots_time_us
                .max(layout_collect_roots_time_us);
            out.max_layout_invalidate_scroll_handle_bindings_time_us = out
                .max_layout_invalidate_scroll_handle_bindings_time_us
                .max(layout_invalidate_scroll_handle_bindings_time_us);
            out.max_layout_expand_view_cache_invalidations_time_us = out
                .max_layout_expand_view_cache_invalidations_time_us
                .max(layout_expand_view_cache_invalidations_time_us);
            out.max_layout_request_build_roots_time_us = out
                .max_layout_request_build_roots_time_us
                .max(layout_request_build_roots_time_us);
            out.max_layout_roots_time_us = out.max_layout_roots_time_us.max(layout_roots_time_us);
            out.max_layout_view_cache_time_us = out
                .max_layout_view_cache_time_us
                .max(layout_view_cache_time_us);
            out.max_layout_collapse_layout_observations_time_us = out
                .max_layout_collapse_layout_observations_time_us
                .max(layout_collapse_layout_observations_time_us);
            out.max_layout_prepaint_after_layout_time_us = out
                .max_layout_prepaint_after_layout_time_us
                .max(layout_prepaint_after_layout_time_us);
            out.max_layout_observation_record_time_us = out
                .max_layout_observation_record_time_us
                .max(layout_observation_record_time_us);
            out.max_layout_observation_record_models_items = out
                .max_layout_observation_record_models_items
                .max(layout_observation_record_models_items);
            out.max_layout_observation_record_globals_items = out
                .max_layout_observation_record_globals_items
                .max(layout_observation_record_globals_items);
            out.max_prepaint_time_us = out.max_prepaint_time_us.max(prepaint_time_us);
            out.max_paint_time_us = out.max_paint_time_us.max(paint_time_us);
            out.max_total_time_us = out.max_total_time_us.max(total_time_us);
            out.max_ui_thread_cpu_time_us =
                out.max_ui_thread_cpu_time_us.max(ui_thread_cpu_time_us);
            out.max_ui_thread_cpu_cycle_time_delta_cycles = out
                .max_ui_thread_cpu_cycle_time_delta_cycles
                .max(ui_thread_cpu_cycle_time_delta_cycles);
            out.max_layout_engine_solve_time_us = out
                .max_layout_engine_solve_time_us
                .max(layout_engine_solve_time_us);
            out.max_renderer_encode_scene_us = out
                .max_renderer_encode_scene_us
                .max(renderer_encode_scene_us);
            out.max_renderer_ensure_pipelines_us = out
                .max_renderer_ensure_pipelines_us
                .max(renderer_ensure_pipelines_us);
            out.max_renderer_plan_compile_us = out
                .max_renderer_plan_compile_us
                .max(renderer_plan_compile_us);
            out.max_renderer_upload_us = out.max_renderer_upload_us.max(renderer_upload_us);
            out.max_renderer_record_passes_us = out
                .max_renderer_record_passes_us
                .max(renderer_record_passes_us);
            out.max_renderer_encoder_finish_us = out
                .max_renderer_encoder_finish_us
                .max(renderer_encoder_finish_us);
            out.max_renderer_prepare_svg_us =
                out.max_renderer_prepare_svg_us.max(renderer_prepare_svg_us);
            out.max_renderer_prepare_text_us = out
                .max_renderer_prepare_text_us
                .max(renderer_prepare_text_us);

            rows.push(BundleStatsSnapshotRow {
                window: window_id,
                tick_id: s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                frame_id: s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                timestamp_unix_ms: s.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
                frame_arena_capacity_estimate_bytes,
                frame_arena_grow_events,
                element_children_vec_pool_reuses,
                element_children_vec_pool_misses,
                ui_thread_cpu_time_us,
                ui_thread_cpu_total_time_us,
                ui_thread_cpu_cycle_time_delta_cycles,
                ui_thread_cpu_cycle_time_total_cycles,
                layout_time_us,
                layout_collect_roots_time_us,
                layout_invalidate_scroll_handle_bindings_time_us,
                layout_expand_view_cache_invalidations_time_us,
                layout_request_build_roots_time_us,
                layout_roots_time_us,
                layout_pending_barrier_relayouts_time_us,
                layout_barrier_relayouts_time_us,
                layout_repair_view_cache_bounds_time_us,
                layout_contained_view_cache_roots_time_us,
                layout_collapse_layout_observations_time_us,
                layout_observation_record_time_us,
                layout_observation_record_models_items,
                layout_observation_record_globals_items,
                layout_view_cache_time_us,
                layout_semantics_refresh_time_us,
                layout_focus_repair_time_us,
                layout_deferred_cleanup_time_us,
                layout_prepaint_after_layout_time_us,
                layout_skipped_engine_frame,
                layout_fast_path_taken,
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
                renderer_ensure_pipelines_us,
                renderer_plan_compile_us,
                renderer_upload_us,
                renderer_record_passes_us,
                renderer_encoder_finish_us,
                renderer_prepare_text_us,
                renderer_prepare_svg_us,
                renderer_svg_upload_bytes,
                renderer_image_upload_bytes,
                renderer_render_target_updates_ingest_unknown,
                renderer_render_target_updates_ingest_owned,
                renderer_render_target_updates_ingest_external_zero_copy,
                renderer_render_target_updates_ingest_gpu_copy,
                renderer_render_target_updates_ingest_cpu_upload,
                renderer_render_target_updates_requested_ingest_unknown,
                renderer_render_target_updates_requested_ingest_owned,
                renderer_render_target_updates_requested_ingest_external_zero_copy,
                renderer_render_target_updates_requested_ingest_gpu_copy,
                renderer_render_target_updates_requested_ingest_cpu_upload,
                renderer_render_target_updates_ingest_fallbacks,
                renderer_viewport_draw_calls,
                renderer_viewport_draw_calls_ingest_unknown,
                renderer_viewport_draw_calls_ingest_owned,
                renderer_viewport_draw_calls_ingest_external_zero_copy,
                renderer_viewport_draw_calls_ingest_gpu_copy,
                renderer_viewport_draw_calls_ingest_cpu_upload,
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
                renderer_material_quad_ops,
                renderer_material_sampled_quad_ops,
                renderer_material_distinct,
                renderer_material_unknown_ids,
                renderer_material_degraded_due_to_budget,
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
                layout_hotspots,
                widget_measure_hotspots,
                paint_widget_hotspots,
                paint_text_prepare_hotspots,
                model_change_hotspots,
                model_change_unobserved,
                global_change_hotspots,
                global_change_unobserved,
            });
        }
    }

    fn p50_p95(values: impl Iterator<Item = u64>) -> (u64, u64) {
        let mut sorted: Vec<u64> = values.collect();
        if sorted.is_empty() {
            return (0, 0);
        }
        sorted.sort_unstable();
        let p50 = crate::percentile_nearest_rank_sorted(&sorted, 0.50);
        let p95 = crate::percentile_nearest_rank_sorted(&sorted, 0.95);
        (p50, p95)
    }

    (out.p50_total_time_us, out.p95_total_time_us) = p50_p95(rows.iter().map(|r| r.total_time_us));
    (out.p50_ui_thread_cpu_time_us, out.p95_ui_thread_cpu_time_us) =
        p50_p95(rows.iter().map(|r| r.ui_thread_cpu_time_us));
    (
        out.p50_ui_thread_cpu_cycle_time_delta_cycles,
        out.p95_ui_thread_cpu_cycle_time_delta_cycles,
    ) = p50_p95(rows.iter().map(|r| r.ui_thread_cpu_cycle_time_delta_cycles));
    (out.p50_layout_time_us, out.p95_layout_time_us) =
        p50_p95(rows.iter().map(|r| r.layout_time_us));
    (
        out.p50_layout_collect_roots_time_us,
        out.p95_layout_collect_roots_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_collect_roots_time_us));
    (
        out.p50_layout_request_build_roots_time_us,
        out.p95_layout_request_build_roots_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_request_build_roots_time_us));
    (out.p50_layout_roots_time_us, out.p95_layout_roots_time_us) =
        p50_p95(rows.iter().map(|r| r.layout_roots_time_us));
    (
        out.p50_layout_view_cache_time_us,
        out.p95_layout_view_cache_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_view_cache_time_us));
    (
        out.p50_layout_collapse_layout_observations_time_us,
        out.p95_layout_collapse_layout_observations_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_collapse_layout_observations_time_us),
    );
    (
        out.p50_layout_prepaint_after_layout_time_us,
        out.p95_layout_prepaint_after_layout_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_prepaint_after_layout_time_us));
    (out.p50_prepaint_time_us, out.p95_prepaint_time_us) =
        p50_p95(rows.iter().map(|r| r.prepaint_time_us));
    (out.p50_paint_time_us, out.p95_paint_time_us) = p50_p95(rows.iter().map(|r| r.paint_time_us));
    (
        out.p50_paint_input_context_time_us,
        out.p95_paint_input_context_time_us,
    ) = p50_p95(rows.iter().map(|r| r.paint_input_context_time_us));
    (
        out.p50_paint_scroll_handle_invalidation_time_us,
        out.p95_paint_scroll_handle_invalidation_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_scroll_handle_invalidation_time_us),
    );
    (
        out.p50_paint_collect_roots_time_us,
        out.p95_paint_collect_roots_time_us,
    ) = p50_p95(rows.iter().map(|r| r.paint_collect_roots_time_us));
    (
        out.p50_paint_publish_text_input_snapshot_time_us,
        out.p95_paint_publish_text_input_snapshot_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_publish_text_input_snapshot_time_us),
    );
    (
        out.p50_paint_collapse_observations_time_us,
        out.p95_paint_collapse_observations_time_us,
    ) = p50_p95(rows.iter().map(|r| r.paint_collapse_observations_time_us));
    (
        out.p50_layout_engine_solve_time_us,
        out.p95_layout_engine_solve_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_engine_solve_time_us));
    (out.p50_dispatch_time_us, out.p95_dispatch_time_us) =
        p50_p95(rows.iter().map(|r| r.dispatch_time_us));
    (out.p50_hit_test_time_us, out.p95_hit_test_time_us) =
        p50_p95(rows.iter().map(|r| r.hit_test_time_us));
    (out.p50_paint_widget_time_us, out.p95_paint_widget_time_us) =
        p50_p95(rows.iter().map(|r| r.paint_widget_time_us));
    (
        out.p50_paint_text_prepare_time_us,
        out.p95_paint_text_prepare_time_us,
    ) = p50_p95(rows.iter().map(|r| r.paint_text_prepare_time_us));
    (
        out.p50_renderer_encode_scene_us,
        out.p95_renderer_encode_scene_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_encode_scene_us));
    (
        out.p50_renderer_ensure_pipelines_us,
        out.p95_renderer_ensure_pipelines_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_ensure_pipelines_us));
    (
        out.p50_renderer_plan_compile_us,
        out.p95_renderer_plan_compile_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_plan_compile_us));
    (out.p50_renderer_upload_us, out.p95_renderer_upload_us) =
        p50_p95(rows.iter().map(|r| r.renderer_upload_us));
    (
        out.p50_renderer_record_passes_us,
        out.p95_renderer_record_passes_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_record_passes_us));
    (
        out.p50_renderer_encoder_finish_us,
        out.p95_renderer_encoder_finish_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_encoder_finish_us));
    (
        out.p50_renderer_prepare_svg_us,
        out.p95_renderer_prepare_svg_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_prepare_svg_us));
    (
        out.p50_renderer_prepare_text_us,
        out.p95_renderer_prepare_text_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_prepare_text_us));

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
        BundleStatsSort::UiThreadCpuTime => {
            rows.sort_by(|a, b| {
                b.ui_thread_cpu_time_us
                    .cmp(&a.ui_thread_cpu_time_us)
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
                    .then_with(|| b.layout_time_us.cmp(&a.layout_time_us))
                    .then_with(|| b.paint_time_us.cmp(&a.paint_time_us))
                    .then_with(|| b.invalidation_walk_nodes.cmp(&a.invalidation_walk_nodes))
            });
        }
        BundleStatsSort::UiThreadCpuCycles => {
            rows.sort_by(|a, b| {
                b.ui_thread_cpu_cycle_time_delta_cycles
                    .cmp(&a.ui_thread_cpu_cycle_time_delta_cycles)
                    .then_with(|| b.ui_thread_cpu_time_us.cmp(&a.ui_thread_cpu_time_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
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
        BundleStatsSort::RendererEnsurePipelines => {
            rows.sort_by(|a, b| {
                b.renderer_ensure_pipelines_us
                    .cmp(&a.renderer_ensure_pipelines_us)
                    .then_with(|| b.renderer_plan_compile_us.cmp(&a.renderer_plan_compile_us))
                    .then_with(|| b.renderer_encode_scene_us.cmp(&a.renderer_encode_scene_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererPlanCompile => {
            rows.sort_by(|a, b| {
                b.renderer_plan_compile_us
                    .cmp(&a.renderer_plan_compile_us)
                    .then_with(|| b.renderer_encode_scene_us.cmp(&a.renderer_encode_scene_us))
                    .then_with(|| {
                        b.renderer_record_passes_us
                            .cmp(&a.renderer_record_passes_us)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererUpload => {
            rows.sort_by(|a, b| {
                b.renderer_upload_us
                    .cmp(&a.renderer_upload_us)
                    .then_with(|| {
                        b.renderer_ensure_pipelines_us
                            .cmp(&a.renderer_ensure_pipelines_us)
                    })
                    .then_with(|| b.renderer_plan_compile_us.cmp(&a.renderer_plan_compile_us))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererRecordPasses => {
            rows.sort_by(|a, b| {
                b.renderer_record_passes_us
                    .cmp(&a.renderer_record_passes_us)
                    .then_with(|| b.renderer_upload_us.cmp(&a.renderer_upload_us))
                    .then_with(|| b.renderer_draw_calls.cmp(&a.renderer_draw_calls))
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::RendererEncoderFinish => {
            rows.sort_by(|a, b| {
                b.renderer_encoder_finish_us
                    .cmp(&a.renderer_encoder_finish_us)
                    .then_with(|| {
                        b.renderer_record_passes_us
                            .cmp(&a.renderer_record_passes_us)
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

fn elide_middle(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let len = s.chars().count();
    if len <= max_chars {
        return s.to_string();
    }

    // Keep output compact but still searchable by both prefix and suffix.
    let head = max_chars / 2;
    let tail = max_chars.saturating_sub(head + 1);
    let head_str: String = s.chars().take(head).collect();
    let tail_str: String = s
        .chars()
        .rev()
        .take(tail)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head_str}…{tail_str}")
}

fn snapshot_top_invalidation_walks(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
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
            root_element_path: w
                .get("root_element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
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
        let (role, test_id) = snapshot_lookup_semantics(semantics, snapshot, walk.root_node);
        walk.root_role = role;
        walk.root_test_id = test_id;
    }

    out
}

fn snapshot_cache_root_stats(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
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

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

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
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
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
        let (role, test_id) = snapshot_lookup_semantics(semantics, snapshot, item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn check_report_for_hover_layout_invalidations(
    report: &BundleStatsReport,
    max_allowed: u32,
) -> Result<(), String> {
    hover_layout_checks::check_report_for_hover_layout_invalidations(report, max_allowed)
}

fn snapshot_paint_widget_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
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

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

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

fn snapshot_layout_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsLayoutHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("layout_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsLayoutHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsLayoutHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            element_path: h
                .get("element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            widget_type: h
                .get("widget_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            layout_time_us: h
                .get("layout_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            inclusive_time_us: h
                .get("inclusive_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
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

fn snapshot_widget_measure_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsWidgetMeasureHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("widget_measure_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsWidgetMeasureHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsWidgetMeasureHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            element_path: h
                .get("element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            widget_type: h
                .get("widget_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            measure_time_us: h
                .get("measure_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            inclusive_time_us: h
                .get("inclusive_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
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
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
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

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

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
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
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

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

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
                root_element: s.get("root_element").and_then(|v| v.as_u64()),
                root_element_kind: s
                    .get("root_element_kind")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                root_element_path: s
                    .get("root_element_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
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
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    node_id: u64,
) -> (Option<String>, Option<String>) {
    let nodes = semantics.nodes(snapshot).unwrap_or(&[]);

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
    fn from_snapshot(
        semantics: &crate::json_bundle::SemanticsResolver<'_>,
        snapshot: &serde_json::Value,
    ) -> Self {
        let nodes = semantics.nodes(snapshot).unwrap_or(&[]);

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
    notify_gates::check_bundle_for_notify_hotspot_file_max(
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
    notify_gates::check_bundle_for_notify_hotspot_file_max_json(
        bundle,
        bundle_path,
        file_filter,
        max_count,
        warmup_frames,
    )
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
