fn snapshot_code_editor_paint_perf(
    snapshot: &serde_json::Value,
) -> Option<BundleStatsCodeEditorPaintPerf> {
    let p = snapshot
        .get("app_snapshot")?
        .get("code_editor")?
        .get("torture")?
        .get("paint_perf")?;

    macro_rules! u64_field {
        ($key:literal) => {
            p.get($key).and_then(|v| v.as_u64()).unwrap_or(0)
        };
    }
    macro_rules! us_field {
        ($us_key:literal, $ns_key:literal) => {
            p.get($ns_key)
                .and_then(|v| v.as_u64())
                .map(|ns| ns / 1_000)
                .unwrap_or_else(|| u64_field!($us_key))
        };
    }

    Some(BundleStatsCodeEditorPaintPerf {
        frame_seq: u64_field!("frame_seq"),
        visible_start: u64_field!("visible_start"),
        visible_end: u64_field!("visible_end"),
        visible_rows: u64_field!("visible_rows"),
        cache_base_entries: u64_field!("cache_base_entries"),
        cache_frame_min_entries: u64_field!("cache_frame_min_entries"),
        cache_effective_entries: u64_field!("cache_effective_entries"),
        rows_painted: u64_field!("rows_painted"),
        rows_drew_rich: u64_field!("rows_drew_rich"),
        rows_scene_replayed: u64_field!("rows_scene_replayed"),
        rows_scene_prepaint_planned: u64_field!("rows_scene_prepaint_planned"),
        rows_scene_prepaint_plan_used: u64_field!("rows_scene_prepaint_plan_used"),
        rows_scene_stored: u64_field!("rows_scene_stored"),
        rows_scene_stored_at_visible_start: u64_field!("rows_scene_stored_at_visible_start"),
        rows_scene_stored_at_visible_end: u64_field!("rows_scene_stored_at_visible_end"),
        row_scene_ops_stored: u64_field!("row_scene_ops_stored"),
        rows_scene_prepaint_edge_stored: u64_field!("rows_scene_prepaint_edge_stored"),
        row_scene_prepaint_edge_ops_stored: u64_field!("row_scene_prepaint_edge_ops_stored"),
        rows_scene_prepaint_candidates: u64_field!("rows_scene_prepaint_candidates"),
        rows_scene_prepaint_skip_no_cache: u64_field!("rows_scene_prepaint_skip_no_cache"),
        rows_scene_prepaint_skip_unsupported_key: u64_field!(
            "rows_scene_prepaint_skip_unsupported_key"
        ),
        rows_scene_prepaint_skip_preedit: u64_field!("rows_scene_prepaint_skip_preedit"),
        rows_scene_prepaint_skip_syntax_empty: u64_field!(
            "rows_scene_prepaint_skip_syntax_empty"
        ),
        rows_scene_prepaint_skip_key_mismatch: u64_field!(
            "rows_scene_prepaint_skip_key_mismatch"
        ),
        rows_scene_prepaint_plan_cache_hits: u64_field!(
            "rows_scene_prepaint_plan_cache_hits"
        ),
        rows_scene_prepaint_plan_cache_rejects: u64_field!(
            "rows_scene_prepaint_plan_cache_rejects"
        ),
        rows_scene_fast_miss_no_entry: u64_field!("rows_scene_fast_miss_no_entry"),
        rows_scene_fast_miss_key_mismatch: u64_field!("rows_scene_fast_miss_key_mismatch"),
        rows_scene_full_miss_no_entry: u64_field!("rows_scene_full_miss_no_entry"),
        rows_scene_full_miss_key_mismatch: u64_field!("rows_scene_full_miss_key_mismatch"),
        quads_selection: u64_field!("quads_selection"),
        quads_caret: u64_field!("quads_caret"),
        syntax_rows_stored: u64_field!("syntax_rows_stored"),
        us_total: us_field!("us_total", "ns_total"),
        us_row_text: us_field!("us_row_text", "ns_row_text"),
        us_baseline_measure: us_field!("us_baseline_measure", "ns_baseline_measure"),
        us_row_content_resolve: us_field!("us_row_content_resolve", "ns_row_content_resolve"),
        us_row_rich_cache_compare: us_field!(
            "us_row_rich_cache_compare",
            "ns_row_rich_cache_compare"
        ),
        us_row_geom_key: us_field!("us_row_geom_key", "ns_row_geom_key"),
        us_rich_materialize: us_field!("us_rich_materialize", "ns_rich_materialize"),
        us_text_draw: us_field!("us_text_draw", "ns_text_draw"),
        us_row_scene_key: us_field!("us_row_scene_key", "ns_row_scene_key"),
        us_row_scene_fast_probe: us_field!(
            "us_row_scene_fast_probe",
            "ns_row_scene_fast_probe"
        ),
        us_row_scene_full_probe: us_field!(
            "us_row_scene_full_probe",
            "ns_row_scene_full_probe"
        ),
        us_row_scene_fast_key_compare: us_field!(
            "us_row_scene_fast_key_compare",
            "ns_row_scene_fast_key_compare"
        ),
        us_row_scene_full_key_compare: us_field!(
            "us_row_scene_full_key_compare",
            "ns_row_scene_full_key_compare"
        ),
        us_row_scene_replay_setup: us_field!(
            "us_row_scene_replay_setup",
            "ns_row_scene_replay_setup"
        ),
        us_row_scene_replay_touch: us_field!(
            "us_row_scene_replay_touch",
            "ns_row_scene_replay_touch"
        ),
        us_row_scene_replay_ops: us_field!(
            "us_row_scene_replay_ops",
            "ns_row_scene_replay_ops"
        ),
        us_row_scene_prepaint_plan: us_field!(
            "us_row_scene_prepaint_plan",
            "ns_row_scene_prepaint_plan"
        ),
        us_row_scene_prepaint_probe: us_field!(
            "us_row_scene_prepaint_probe",
            "ns_row_scene_prepaint_probe"
        ),
        us_row_scene_prepaint_key_compare: us_field!(
            "us_row_scene_prepaint_key_compare",
            "ns_row_scene_prepaint_key_compare"
        ),
        us_row_scene_capture_ops: us_field!(
            "us_row_scene_capture_ops",
            "ns_row_scene_capture_ops"
        ),
        us_row_scene_store: us_field!("us_row_scene_store", "ns_row_scene_store"),
        us_row_scene_prepaint_edge_store: us_field!(
            "us_row_scene_prepaint_edge_store",
            "ns_row_scene_prepaint_edge_store"
        ),
        us_row_scene_fast_path: us_field!("us_row_scene_fast_path", "ns_row_scene_fast_path"),
        us_row_scene_full_path: us_field!("us_row_scene_full_path", "ns_row_scene_full_path"),
        us_syntax_spans: us_field!("us_syntax_spans", "ns_syntax_spans"),
        us_syntax_slice: us_field!("us_syntax_slice", "ns_syntax_slice"),
        us_syntax_highlight: us_field!("us_syntax_highlight", "ns_syntax_highlight"),
        us_syntax_distribute: us_field!("us_syntax_distribute", "ns_syntax_distribute"),
        us_syntax_store: us_field!("us_syntax_store", "ns_syntax_store"),
        us_selection_rects: us_field!("us_selection_rects", "ns_selection_rects"),
        us_caret_x: us_field!("us_caret_x", "ns_caret_x"),
        us_caret_stops: us_field!("us_caret_stops", "ns_caret_stops"),
        us_caret_rect: us_field!("us_caret_rect", "ns_caret_rect"),
        us_row_geom_cache: us_field!("us_row_geom_cache", "ns_row_geom_cache"),
        us_row_geom_resolve: us_field!("us_row_geom_resolve", "ns_row_geom_resolve"),
        us_row_overlay: us_field!("us_row_overlay", "ns_row_overlay"),
        us_frame_overlay_prepare: us_field!(
            "us_frame_overlay_prepare",
            "ns_frame_overlay_prepare"
        ),
        surface_rows_iterated: u64_field!("surface_rows_iterated"),
        surface_rows_with_rect: u64_field!("surface_rows_with_rect"),
        us_windowed_surface_paint_callback: us_field!(
            "us_windowed_surface_paint_callback",
            "ns_windowed_surface_paint_callback"
        ),
        us_windowed_surface_frame_lookup: us_field!(
            "us_windowed_surface_frame_lookup",
            "ns_windowed_surface_frame_lookup"
        ),
        us_windowed_surface_hook: us_field!(
            "us_windowed_surface_hook",
            "ns_windowed_surface_hook"
        ),
        us_windowed_surface_row_loop: us_field!(
            "us_windowed_surface_row_loop",
            "ns_windowed_surface_row_loop"
        ),
        us_windowed_surface_row_rect: us_field!(
            "us_windowed_surface_row_rect",
            "ns_windowed_surface_row_rect"
        ),
        us_windowed_surface_row_paint: us_field!(
            "us_windowed_surface_row_paint",
            "ns_windowed_surface_row_paint"
        ),
        us_windowed_surface_non_row: us_field!(
            "us_windowed_surface_non_row",
            "ns_windowed_surface_non_row"
        ),
        us_windowed_surface_row_callback_gap: us_field!(
            "us_windowed_surface_row_callback_gap",
            "ns_windowed_surface_row_callback_gap"
        ),
        us_torture_autoscroll: us_field!("us_torture_autoscroll", "ns_torture_autoscroll"),
        us_torture_overlay: us_field!("us_torture_overlay", "ns_torture_overlay"),
    })
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
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut out = BundleStatsReport {
        sort,
        warmup_frames: opts.warmup_frames,
        source_bundle_schema_version: crate::compat::bundle::bundle_schema_version_from_value(
            bundle,
        ),
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
            let element_children_vec_pool_grow_events = stats
                .and_then(|m| m.get("element_children_vec_pool_grow_events"))
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
            let paint_host_widget_observed_deps_calls = stats
                .and_then(|m| m.get("paint_host_widget_observed_deps_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_host_widget_observed_deps_empty_calls = stats
                .and_then(|m| m.get("paint_host_widget_observed_deps_empty_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_host_widget_observed_models_non_empty_calls = stats
                .and_then(|m| m.get("paint_host_widget_observed_models_non_empty_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let paint_host_widget_observed_globals_non_empty_calls = stats
                .and_then(|m| m.get("paint_host_widget_observed_globals_non_empty_calls"))
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
            let dispatch_inner_body_time_us = stats
                .and_then(|m| m.get("dispatch_inner_body_time_us"))
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
            let dispatch_input_state_update_time_us = stats
                .and_then(|m| m.get("dispatch_input_state_update_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_context_build_time_us = stats
                .and_then(|m| m.get("dispatch_context_build_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_prelude_time_us = stats
                .and_then(|m| m.get("dispatch_prelude_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_pointer_arbitration_time_us = stats
                .and_then(|m| m.get("dispatch_pointer_arbitration_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_pointer_target_routing_time_us = stats
                .and_then(|m| m.get("dispatch_pointer_target_routing_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_post_widget_control_flow_time_us = stats
                .and_then(|m| m.get("dispatch_post_widget_control_flow_time_us"))
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
            let window_runtime_snapshot_focus_repair_time_us = stats
                .and_then(|m| m.get("window_runtime_snapshot_focus_repair_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let window_runtime_snapshot_input_context_time_us = stats
                .and_then(|m| m.get("window_runtime_snapshot_input_context_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let window_runtime_snapshot_command_availability_time_us = stats
                .and_then(|m| m.get("window_runtime_snapshot_command_availability_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let window_runtime_snapshot_widget_command_count = stats
                .and_then(|m| m.get("window_runtime_snapshot_widget_command_count"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let window_runtime_snapshot_command_registry_collect_time_us = stats
                .and_then(|m| {
                    m.get("window_runtime_snapshot_command_registry_collect_time_us")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let window_runtime_snapshot_command_availability_eval_time_us = stats
                .and_then(|m| m.get("window_runtime_snapshot_command_availability_eval_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let window_runtime_snapshot_shortcut_overlay_time_us = stats
                .and_then(|m| m.get("window_runtime_snapshot_shortcut_overlay_time_us"))
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
            let renderer_prepare_text_collect_pin_keys_us = stats
                .and_then(|m| m.get("renderer_prepare_text_collect_pin_keys_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_bucket_delta_us = stats
                .and_then(|m| m.get("renderer_prepare_text_bucket_delta_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_prewarm_us = stats
                .and_then(|m| m.get("renderer_prepare_text_prewarm_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_pin_bucket_update_us = stats
                .and_then(|m| m.get("renderer_prepare_text_pin_bucket_update_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_flush_uploads_us = stats
                .and_then(|m| m.get("renderer_prepare_text_flush_uploads_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_scene_text_blobs = stats
                .and_then(|m| m.get("renderer_prepare_text_scene_text_blobs"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_pinned_glyph_keys = stats
                .and_then(|m| m.get("renderer_prepare_text_pinned_glyph_keys"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_prewarm_glyph_keys = stats
                .and_then(|m| m.get("renderer_prepare_text_prewarm_glyph_keys"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_retained_glyph_keys = stats
                .and_then(|m| m.get("renderer_prepare_text_retained_glyph_keys"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_added_glyph_keys = stats
                .and_then(|m| m.get("renderer_prepare_text_added_glyph_keys"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_text_removed_glyph_keys = stats
                .and_then(|m| m.get("renderer_prepare_text_removed_glyph_keys"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_prepare_svg_us = stats
                .and_then(|m| m.get("renderer_prepare_svg_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_uniform_bytes = stats
                .and_then(|m| m.get("renderer_uniform_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_instance_bytes = stats
                .and_then(|m| m.get("renderer_instance_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_vertex_bytes = stats
                .and_then(|m| m.get("renderer_vertex_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_stack_us = stats
                .and_then(|m| m.get("renderer_encode_scene_stack_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_clip_us = stats
                .and_then(|m| m.get("renderer_encode_scene_clip_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_mask_us = stats
                .and_then(|m| m.get("renderer_encode_scene_mask_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_effect_us = stats
                .and_then(|m| m.get("renderer_encode_scene_effect_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_quad_us = stats
                .and_then(|m| m.get("renderer_encode_scene_quad_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_image_us = stats
                .and_then(|m| m.get("renderer_encode_scene_image_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_us = stats
                .and_then(|m| m.get("renderer_encode_scene_text_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_path_us = stats
                .and_then(|m| m.get("renderer_encode_scene_path_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_viewport_us = stats
                .and_then(|m| m.get("renderer_encode_scene_viewport_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_flush_us = stats
                .and_then(|m| m.get("renderer_encode_scene_flush_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_shadow_us = stats
                .and_then(|m| m.get("renderer_encode_scene_text_shadow_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_setup_us = stats
                .and_then(|m| m.get("renderer_encode_scene_text_setup_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_glyphs_us = stats
                .and_then(|m| m.get("renderer_encode_scene_text_glyphs_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_vertex_grow_events = stats
                .and_then(|m| m.get("renderer_encode_scene_text_vertex_grow_events"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_glyph_transform_us = stats
                .and_then(|m| m.get("renderer_encode_scene_text_glyph_transform_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_glyph_emit_us = stats
                .and_then(|m| m.get("renderer_encode_scene_text_glyph_emit_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_group_flush_us = stats
                .and_then(|m| m.get("renderer_encode_scene_text_group_flush_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_transform_fast_path_glyphs = stats
                .and_then(|m| m.get("renderer_encode_scene_text_transform_fast_path_glyphs"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_transform_generic_glyphs = stats
                .and_then(|m| m.get("renderer_encode_scene_text_transform_generic_glyphs"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_stack_ops = stats
                .and_then(|m| m.get("renderer_encode_scene_stack_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_clip_ops = stats
                .and_then(|m| m.get("renderer_encode_scene_clip_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_mask_ops = stats
                .and_then(|m| m.get("renderer_encode_scene_mask_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_effect_ops = stats
                .and_then(|m| m.get("renderer_encode_scene_effect_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_quad_ops = stats
                .and_then(|m| m.get("renderer_encode_scene_quad_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_image_ops = stats
                .and_then(|m| m.get("renderer_encode_scene_image_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_text_ops = stats
                .and_then(|m| m.get("renderer_encode_scene_text_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_path_ops = stats
                .and_then(|m| m.get("renderer_encode_scene_path_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_viewport_ops = stats
                .and_then(|m| m.get("renderer_encode_scene_viewport_ops"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_encode_scene_flushes = stats
                .and_then(|m| m.get("renderer_encode_scene_flushes"))
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
            let renderer_intermediate_full_target_bytes = stats
                .and_then(|m| m.get("renderer_intermediate_full_target_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_effect_chain_budget_samples = stats
                .and_then(|m| m.get("renderer_render_plan_effect_chain_budget_samples"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_effect_chain_effective_budget_min_bytes = stats
                .and_then(|m| m.get("renderer_render_plan_effect_chain_effective_budget_min_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_effect_chain_effective_budget_max_bytes = stats
                .and_then(|m| m.get("renderer_render_plan_effect_chain_effective_budget_max_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_effect_chain_other_live_max_bytes = stats
                .and_then(|m| m.get("renderer_render_plan_effect_chain_other_live_max_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_custom_effect_chain_budget_samples = stats
                .and_then(|m| m.get("renderer_render_plan_custom_effect_chain_budget_samples"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_custom_effect_chain_effective_budget_min_bytes = stats
                .and_then(|m| {
                    m.get("renderer_render_plan_custom_effect_chain_effective_budget_min_bytes")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_custom_effect_chain_effective_budget_max_bytes = stats
                .and_then(|m| {
                    m.get("renderer_render_plan_custom_effect_chain_effective_budget_max_bytes")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_custom_effect_chain_other_live_max_bytes = stats
                .and_then(|m| m.get("renderer_render_plan_custom_effect_chain_other_live_max_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_custom_effect_chain_base_required_max_bytes = stats
                .and_then(|m| {
                    m.get("renderer_render_plan_custom_effect_chain_base_required_max_bytes")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_custom_effect_chain_optional_required_max_bytes = stats
                .and_then(|m| {
                    m.get("renderer_render_plan_custom_effect_chain_optional_required_max_bytes")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_custom_effect_chain_base_required_full_targets_max = stats
                .and_then(|m| {
                    m.get("renderer_render_plan_custom_effect_chain_base_required_full_targets_max")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_custom_effect_chain_optional_mask_max_bytes = stats
                .and_then(|m| {
                    m.get("renderer_render_plan_custom_effect_chain_optional_mask_max_bytes")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes = stats
                .and_then(|m| {
                    m.get("renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes")
                })
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
            let renderer_custom_effect_v1_steps_requested = stats
                .and_then(|m| m.get("renderer_custom_effect_v1_steps_requested"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v1_passes_emitted = stats
                .and_then(|m| m.get("renderer_custom_effect_v1_passes_emitted"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v2_steps_requested = stats
                .and_then(|m| m.get("renderer_custom_effect_v2_steps_requested"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v2_passes_emitted = stats
                .and_then(|m| m.get("renderer_custom_effect_v2_passes_emitted"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v2_user_image_incompatible_fallbacks = stats
                .and_then(|m| m.get("renderer_custom_effect_v2_user_image_incompatible_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_steps_requested = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_steps_requested"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_passes_emitted = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_passes_emitted"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_user0_image_incompatible_fallbacks = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_user0_image_incompatible_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_user1_image_incompatible_fallbacks = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_user1_image_incompatible_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_pyramid_cache_hits = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_pyramid_cache_hits"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_pyramid_cache_misses = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_pyramid_cache_misses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_sources_raw_requested = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_sources_raw_requested"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_sources_raw_distinct = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_sources_raw_distinct"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_sources_raw_aliased_to_src = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_sources_raw_aliased_to_src"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_sources_pyramid_requested = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_sources_pyramid_requested"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2 = stats
                .and_then(|m| m.get("renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero = stats
                .and_then(|m| {
                    m.get("renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient = stats
                .and_then(|m| {
                    m.get(
                        "renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient",
                    )
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_requested = stats
                .and_then(|m| m.get("renderer_backdrop_source_groups_requested"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_applied_raw = stats
                .and_then(|m| m.get("renderer_backdrop_source_groups_applied_raw"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_raw_degraded_budget_zero = stats
                .and_then(|m| m.get("renderer_backdrop_source_groups_raw_degraded_budget_zero"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_raw_degraded_budget_insufficient = stats
                .and_then(|m| {
                    m.get("renderer_backdrop_source_groups_raw_degraded_budget_insufficient")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_raw_degraded_target_exhausted = stats
                .and_then(|m| {
                    m.get("renderer_backdrop_source_groups_raw_degraded_target_exhausted")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_pyramid_requested = stats
                .and_then(|m| m.get("renderer_backdrop_source_groups_pyramid_requested"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_pyramid_applied_levels_ge2 = stats
                .and_then(|m| m.get("renderer_backdrop_source_groups_pyramid_applied_levels_ge2"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero = stats
                .and_then(|m| {
                    m.get("renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient = stats
                .and_then(|m| {
                    m.get(
                        "renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient",
                    )
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable = stats
                .and_then(|m| {
                    m.get("renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable")
                })
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
            let layout_clean_geometry_solve_skip_rejections = stats
                .and_then(|m| m.get("layout_clean_geometry_solve_skip_rejections"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let layout_clean_geometry_solve_skip_first_rejection = stats
                .and_then(|m| m.get("layout_clean_geometry_solve_skip_first_rejection"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let layout_clean_geometry_solve_skip_first_detail = stats
                .and_then(|m| m.get("layout_clean_geometry_solve_skip_first_detail"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let layout_clean_geometry_solve_skip_first_element_kind = stats
                .and_then(|m| m.get("layout_clean_geometry_solve_skip_first_element_kind"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let layout_engine_child_rect_queries = stats
                .and_then(|m| m.get("layout_engine_child_rect_queries"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_engine_child_rect_time_us = stats
                .and_then(|m| m.get("layout_engine_child_rect_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_engine_widget_fallback_solves = stats
                .and_then(|m| m.get("layout_engine_widget_fallback_solves"))
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
            let layout_request_build_roots_take_engine_time_us = stats
                .and_then(|m| m.get("layout_request_build_roots_take_engine_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_request_build_roots_phase1_time_us = stats
                .and_then(|m| m.get("layout_request_build_roots_phase1_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_request_build_roots_phase2_time_us = stats
                .and_then(|m| m.get("layout_request_build_roots_phase2_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_request_build_roots_phase2_clean_geometry_proof_time_us = stats
                .and_then(|m| {
                    m.get("layout_request_build_roots_phase2_clean_geometry_proof_time_us")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_nodes = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_nodes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_boundaries = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_boundaries"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_leaf_shortcut_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_leaf_shortcut_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_node_state_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_node_state_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_contract_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_contract_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_record_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_record_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_contract_eval_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_contract_eval_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_child_bounds_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_origin_only_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_child_bounds_origin_only_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us = stats
                .and_then(|m| {
                    m.get("layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us =
                stats
                    .and_then(|m| {
                        m.get(
                            "layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us",
                        )
                    })
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us =
                stats
                    .and_then(|m| {
                        m.get(
                            "layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us",
                        )
                    })
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us =
                stats
                    .and_then(|m| {
                        m.get(
                            "layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us",
                        )
                    })
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us =
                stats
                    .and_then(|m| {
                        m.get(
                            "layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us",
                        )
                    })
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us = stats
                .and_then(|m| {
                    m.get("layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us = stats
                .and_then(|m| {
                    m.get("layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_container_px_insets_time_us = stats
                .and_then(|m| {
                    m.get("layout_clean_geometry_proof_child_bounds_container_px_insets_time_us")
                })
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us =
                stats
                    .and_then(|m| {
                        m.get(
                            "layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us",
                        )
                    })
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
            let layout_clean_geometry_proof_text_metrics_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_text_metrics_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_child_prev_bounds_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_child_prev_bounds_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_proof_emit_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_proof_emit_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_apply_nodes = stats
                .and_then(|m| m.get("layout_clean_geometry_apply_nodes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_apply_fallback_layouts = stats
                .and_then(|m| m.get("layout_clean_geometry_apply_fallback_layouts"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_apply_fallback_layouts_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_apply_fallback_layouts_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_apply_fallback_layouts_top_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_apply_fallback_layouts_top_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_apply_fallback_layouts_top_kind = stats
                .and_then(|m| m.get("layout_clean_geometry_apply_fallback_layouts_top_kind"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let layout_clean_geometry_scroll_side_effect_fast_paths = stats
                .and_then(|m| m.get("layout_clean_geometry_scroll_side_effect_fast_paths"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_clean_geometry_apply_paint_fingerprint_time_us = stats
                .and_then(|m| m.get("layout_clean_geometry_apply_paint_fingerprint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_request_build_roots_phase2_compute_time_us = stats
                .and_then(|m| m.get("layout_request_build_roots_phase2_compute_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_request_build_roots_put_engine_time_us = stats
                .and_then(|m| m.get("layout_request_build_roots_put_engine_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_roots_time_us = stats
                .and_then(|m| m.get("layout_roots_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_roots_apply_time_us = stats
                .and_then(|m| m.get("layout_roots_apply_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_roots_flush_viewport_time_us = stats
                .and_then(|m| m.get("layout_roots_flush_viewport_time_us"))
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
            let layout_request_build_roots = snapshot_layout_request_build_roots(&semantics, s, 3);
            let scroll_layout_profiles = snapshot_scroll_layout_profiles(&semantics, s, 3);
            let top_layout_engine_solves = snapshot_layout_engine_solves(&semantics, s, 3);
            let layout_hotspots = snapshot_layout_hotspots(&semantics, s, 3);
            let widget_measure_hotspots = snapshot_widget_measure_hotspots(&semantics, s, 3);
            let paint_widget_hotspots_all =
                snapshot_paint_widget_hotspots(&semantics, s, PAINT_WIDGET_HOTSPOT_SUMMARY_TOP_N);
            out.paint_widget_hotspot_summary.observe_frame(
                &paint_widget_hotspots_all,
                PAINT_WIDGET_HOTSPOT_SUMMARY_TOP_N,
            );
            let paint_widget_hotspots = paint_widget_hotspots_all
                .iter()
                .take(PAINT_WIDGET_HOTSPOT_ROW_TOP_N)
                .cloned()
                .collect::<Vec<_>>();
            let paint_text_prepare_hotspots =
                snapshot_paint_text_prepare_hotspots(&semantics, s, 3);
            let command_availability_hotspots = snapshot_command_availability_hotspots(s, 5);
            let model_change_hotspots = snapshot_model_change_hotspots(s, 3);
            let model_change_unobserved = snapshot_model_change_unobserved(s, 3);
            let global_change_hotspots = snapshot_global_change_hotspots(s, 3);
            let global_change_unobserved = snapshot_global_change_unobserved(s, 3);
            let code_editor_paint_perf = snapshot_code_editor_paint_perf(s);
            if let Some(perf) = code_editor_paint_perf.as_ref() {
                out.code_editor_paint_perf.observe(perf);
            }

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
            out.sum_layout_request_build_roots_take_engine_time_us = out
                .sum_layout_request_build_roots_take_engine_time_us
                .saturating_add(layout_request_build_roots_take_engine_time_us);
            out.sum_layout_request_build_roots_phase1_time_us = out
                .sum_layout_request_build_roots_phase1_time_us
                .saturating_add(layout_request_build_roots_phase1_time_us);
            out.sum_layout_request_build_roots_phase2_time_us = out
                .sum_layout_request_build_roots_phase2_time_us
                .saturating_add(layout_request_build_roots_phase2_time_us);
            out.sum_layout_request_build_roots_phase2_clean_geometry_proof_time_us = out
                .sum_layout_request_build_roots_phase2_clean_geometry_proof_time_us
                .saturating_add(layout_request_build_roots_phase2_clean_geometry_proof_time_us);
            out.sum_layout_clean_geometry_proof_nodes = out
                .sum_layout_clean_geometry_proof_nodes
                .saturating_add(layout_clean_geometry_proof_nodes);
            out.sum_layout_clean_geometry_proof_boundaries = out
                .sum_layout_clean_geometry_proof_boundaries
                .saturating_add(layout_clean_geometry_proof_boundaries);
            out.sum_layout_clean_geometry_proof_leaf_shortcut_time_us = out
                .sum_layout_clean_geometry_proof_leaf_shortcut_time_us
                .saturating_add(layout_clean_geometry_proof_leaf_shortcut_time_us);
            out.sum_layout_clean_geometry_proof_node_state_time_us = out
                .sum_layout_clean_geometry_proof_node_state_time_us
                .saturating_add(layout_clean_geometry_proof_node_state_time_us);
            out.sum_layout_clean_geometry_proof_contract_time_us = out
                .sum_layout_clean_geometry_proof_contract_time_us
                .saturating_add(layout_clean_geometry_proof_contract_time_us);
            out.sum_layout_clean_geometry_proof_record_time_us = out
                .sum_layout_clean_geometry_proof_record_time_us
                .saturating_add(layout_clean_geometry_proof_record_time_us);
            out.sum_layout_clean_geometry_proof_contract_eval_time_us = out
                .sum_layout_clean_geometry_proof_contract_eval_time_us
                .saturating_add(layout_clean_geometry_proof_contract_eval_time_us);
            out.sum_layout_clean_geometry_proof_child_bounds_time_us = out
                .sum_layout_clean_geometry_proof_child_bounds_time_us
                .saturating_add(layout_clean_geometry_proof_child_bounds_time_us);
            out.sum_layout_clean_geometry_proof_child_bounds_origin_only_time_us = out
                .sum_layout_clean_geometry_proof_child_bounds_origin_only_time_us
                .saturating_add(layout_clean_geometry_proof_child_bounds_origin_only_time_us);
            out.sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us = out
                .sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us
                .saturating_add(
                    layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us,
                );
            out.sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us =
                out.sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us
                    .saturating_add(
                        layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us,
                    );
            out.sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us =
                out.sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us
                    .saturating_add(
                        layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us,
                    );
            out.sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us =
                out.sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us
                    .saturating_add(
                        layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us,
                    );
            out.sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us =
                out.sum_layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us
                    .saturating_add(
                        layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us,
                    );
            out.sum_layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us = out
                .sum_layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us
                .saturating_add(layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us);
            out.sum_layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us = out
                .sum_layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us
                .saturating_add(
                    layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us,
                );
            out.sum_layout_clean_geometry_proof_child_bounds_container_px_insets_time_us = out
                .sum_layout_clean_geometry_proof_child_bounds_container_px_insets_time_us
                .saturating_add(layout_clean_geometry_proof_child_bounds_container_px_insets_time_us);
            out.sum_layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us =
                out.sum_layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us
                    .saturating_add(
                        layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us,
                    );
            out.sum_layout_clean_geometry_proof_text_metrics_time_us = out
                .sum_layout_clean_geometry_proof_text_metrics_time_us
                .saturating_add(layout_clean_geometry_proof_text_metrics_time_us);
            out.sum_layout_clean_geometry_proof_child_prev_bounds_time_us = out
                .sum_layout_clean_geometry_proof_child_prev_bounds_time_us
                .saturating_add(layout_clean_geometry_proof_child_prev_bounds_time_us);
            out.sum_layout_clean_geometry_proof_emit_time_us = out
                .sum_layout_clean_geometry_proof_emit_time_us
                .saturating_add(layout_clean_geometry_proof_emit_time_us);
            out.sum_layout_clean_geometry_apply_nodes = out
                .sum_layout_clean_geometry_apply_nodes
                .saturating_add(layout_clean_geometry_apply_nodes);
            out.sum_layout_clean_geometry_apply_fallback_layouts = out
                .sum_layout_clean_geometry_apply_fallback_layouts
                .saturating_add(layout_clean_geometry_apply_fallback_layouts);
            out.sum_layout_clean_geometry_apply_fallback_layouts_time_us = out
                .sum_layout_clean_geometry_apply_fallback_layouts_time_us
                .saturating_add(layout_clean_geometry_apply_fallback_layouts_time_us);
            out.sum_layout_clean_geometry_apply_fallback_layouts_top_time_us = out
                .sum_layout_clean_geometry_apply_fallback_layouts_top_time_us
                .saturating_add(layout_clean_geometry_apply_fallback_layouts_top_time_us);
            out.sum_layout_clean_geometry_scroll_side_effect_fast_paths = out
                .sum_layout_clean_geometry_scroll_side_effect_fast_paths
                .saturating_add(layout_clean_geometry_scroll_side_effect_fast_paths);
            out.sum_layout_clean_geometry_apply_paint_fingerprint_time_us = out
                .sum_layout_clean_geometry_apply_paint_fingerprint_time_us
                .saturating_add(layout_clean_geometry_apply_paint_fingerprint_time_us);
            out.sum_layout_request_build_roots_phase2_compute_time_us = out
                .sum_layout_request_build_roots_phase2_compute_time_us
                .saturating_add(layout_request_build_roots_phase2_compute_time_us);
            out.sum_layout_request_build_roots_put_engine_time_us = out
                .sum_layout_request_build_roots_put_engine_time_us
                .saturating_add(layout_request_build_roots_put_engine_time_us);
            out.sum_layout_roots_time_us = out
                .sum_layout_roots_time_us
                .saturating_add(layout_roots_time_us);
            out.sum_layout_roots_apply_time_us = out
                .sum_layout_roots_apply_time_us
                .saturating_add(layout_roots_apply_time_us);
            out.sum_layout_roots_flush_viewport_time_us = out
                .sum_layout_roots_flush_viewport_time_us
                .saturating_add(layout_roots_flush_viewport_time_us);
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
            out.max_layout_request_build_roots_take_engine_time_us = out
                .max_layout_request_build_roots_take_engine_time_us
                .max(layout_request_build_roots_take_engine_time_us);
            out.max_layout_request_build_roots_phase1_time_us = out
                .max_layout_request_build_roots_phase1_time_us
                .max(layout_request_build_roots_phase1_time_us);
            out.max_layout_request_build_roots_phase2_time_us = out
                .max_layout_request_build_roots_phase2_time_us
                .max(layout_request_build_roots_phase2_time_us);
            out.max_layout_request_build_roots_phase2_clean_geometry_proof_time_us = out
                .max_layout_request_build_roots_phase2_clean_geometry_proof_time_us
                .max(layout_request_build_roots_phase2_clean_geometry_proof_time_us);
            out.max_layout_clean_geometry_proof_nodes = out
                .max_layout_clean_geometry_proof_nodes
                .max(layout_clean_geometry_proof_nodes);
            out.max_layout_clean_geometry_proof_boundaries = out
                .max_layout_clean_geometry_proof_boundaries
                .max(layout_clean_geometry_proof_boundaries);
            out.max_layout_clean_geometry_proof_leaf_shortcut_time_us = out
                .max_layout_clean_geometry_proof_leaf_shortcut_time_us
                .max(layout_clean_geometry_proof_leaf_shortcut_time_us);
            out.max_layout_clean_geometry_proof_node_state_time_us = out
                .max_layout_clean_geometry_proof_node_state_time_us
                .max(layout_clean_geometry_proof_node_state_time_us);
            out.max_layout_clean_geometry_proof_contract_time_us = out
                .max_layout_clean_geometry_proof_contract_time_us
                .max(layout_clean_geometry_proof_contract_time_us);
            out.max_layout_clean_geometry_proof_record_time_us = out
                .max_layout_clean_geometry_proof_record_time_us
                .max(layout_clean_geometry_proof_record_time_us);
            out.max_layout_clean_geometry_proof_contract_eval_time_us = out
                .max_layout_clean_geometry_proof_contract_eval_time_us
                .max(layout_clean_geometry_proof_contract_eval_time_us);
            out.max_layout_clean_geometry_proof_child_bounds_time_us = out
                .max_layout_clean_geometry_proof_child_bounds_time_us
                .max(layout_clean_geometry_proof_child_bounds_time_us);
            out.max_layout_clean_geometry_proof_child_bounds_origin_only_time_us = out
                .max_layout_clean_geometry_proof_child_bounds_origin_only_time_us
                .max(layout_clean_geometry_proof_child_bounds_origin_only_time_us);
            out.max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us = out
                .max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us
                .max(layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us);
            out.max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us =
                out.max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us
                    .max(
                        layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us,
                    );
            out.max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us =
                out.max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us
                    .max(
                        layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us,
                    );
            out.max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us =
                out.max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us
                    .max(
                        layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us,
                    );
            out.max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us =
                out.max_layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us
                    .max(
                        layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us,
                    );
            out.max_layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us = out
                .max_layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us
                .max(layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us);
            out.max_layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us = out
                .max_layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us
                .max(layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us);
            out.max_layout_clean_geometry_proof_child_bounds_container_px_insets_time_us = out
                .max_layout_clean_geometry_proof_child_bounds_container_px_insets_time_us
                .max(layout_clean_geometry_proof_child_bounds_container_px_insets_time_us);
            out.max_layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us =
                out.max_layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us
                    .max(
                        layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us,
                    );
            out.max_layout_clean_geometry_proof_text_metrics_time_us = out
                .max_layout_clean_geometry_proof_text_metrics_time_us
                .max(layout_clean_geometry_proof_text_metrics_time_us);
            out.max_layout_clean_geometry_proof_child_prev_bounds_time_us = out
                .max_layout_clean_geometry_proof_child_prev_bounds_time_us
                .max(layout_clean_geometry_proof_child_prev_bounds_time_us);
            out.max_layout_clean_geometry_proof_emit_time_us = out
                .max_layout_clean_geometry_proof_emit_time_us
                .max(layout_clean_geometry_proof_emit_time_us);
            out.max_layout_clean_geometry_apply_nodes = out
                .max_layout_clean_geometry_apply_nodes
                .max(layout_clean_geometry_apply_nodes);
            out.max_layout_clean_geometry_apply_fallback_layouts = out
                .max_layout_clean_geometry_apply_fallback_layouts
                .max(layout_clean_geometry_apply_fallback_layouts);
            out.max_layout_clean_geometry_apply_fallback_layouts_time_us = out
                .max_layout_clean_geometry_apply_fallback_layouts_time_us
                .max(layout_clean_geometry_apply_fallback_layouts_time_us);
            if layout_clean_geometry_apply_fallback_layouts_top_time_us
                > out.max_layout_clean_geometry_apply_fallback_layouts_top_time_us
            {
                out.max_layout_clean_geometry_apply_fallback_layouts_top_time_us =
                    layout_clean_geometry_apply_fallback_layouts_top_time_us;
                out.max_layout_clean_geometry_apply_fallback_layouts_top_kind =
                    layout_clean_geometry_apply_fallback_layouts_top_kind.clone();
            }
            out.max_layout_clean_geometry_scroll_side_effect_fast_paths = out
                .max_layout_clean_geometry_scroll_side_effect_fast_paths
                .max(layout_clean_geometry_scroll_side_effect_fast_paths);
            out.max_layout_clean_geometry_apply_paint_fingerprint_time_us = out
                .max_layout_clean_geometry_apply_paint_fingerprint_time_us
                .max(layout_clean_geometry_apply_paint_fingerprint_time_us);
            out.max_layout_request_build_roots_phase2_compute_time_us = out
                .max_layout_request_build_roots_phase2_compute_time_us
                .max(layout_request_build_roots_phase2_compute_time_us);
            out.max_layout_request_build_roots_put_engine_time_us = out
                .max_layout_request_build_roots_put_engine_time_us
                .max(layout_request_build_roots_put_engine_time_us);
            out.max_layout_roots_time_us = out.max_layout_roots_time_us.max(layout_roots_time_us);
            out.max_layout_roots_apply_time_us =
                out.max_layout_roots_apply_time_us.max(layout_roots_apply_time_us);
            out.max_layout_roots_flush_viewport_time_us = out
                .max_layout_roots_flush_viewport_time_us
                .max(layout_roots_flush_viewport_time_us);
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
            out.max_paint_record_visual_bounds_time_us = out
                .max_paint_record_visual_bounds_time_us
                .max(paint_record_visual_bounds_time_us);
            out.max_paint_record_visual_bounds_calls = out
                .max_paint_record_visual_bounds_calls
                .max(paint_record_visual_bounds_calls);
            out.max_paint_cache_key_time_us = out
                .max_paint_cache_key_time_us
                .max(paint_cache_key_time_us);
            out.max_paint_cache_hit_check_time_us = out
                .max_paint_cache_hit_check_time_us
                .max(paint_cache_hit_check_time_us);
            out.max_paint_observation_record_time_us = out
                .max_paint_observation_record_time_us
                .max(paint_observation_record_time_us);
            out.max_paint_host_widget_observed_models_time_us = out
                .max_paint_host_widget_observed_models_time_us
                .max(paint_host_widget_observed_models_time_us);
            out.max_paint_host_widget_observed_models_items = out
                .max_paint_host_widget_observed_models_items
                .max(paint_host_widget_observed_models_items);
            out.max_paint_host_widget_observed_globals_time_us = out
                .max_paint_host_widget_observed_globals_time_us
                .max(paint_host_widget_observed_globals_time_us);
            out.max_paint_host_widget_observed_globals_items = out
                .max_paint_host_widget_observed_globals_items
                .max(paint_host_widget_observed_globals_items);
            out.max_paint_host_widget_observed_deps_calls = out
                .max_paint_host_widget_observed_deps_calls
                .max(paint_host_widget_observed_deps_calls);
            out.max_paint_host_widget_observed_deps_empty_calls = out
                .max_paint_host_widget_observed_deps_empty_calls
                .max(paint_host_widget_observed_deps_empty_calls);
            out.max_paint_host_widget_observed_models_non_empty_calls = out
                .max_paint_host_widget_observed_models_non_empty_calls
                .max(paint_host_widget_observed_models_non_empty_calls);
            out.max_paint_host_widget_observed_globals_non_empty_calls = out
                .max_paint_host_widget_observed_globals_non_empty_calls
                .max(paint_host_widget_observed_globals_non_empty_calls);
            out.max_paint_host_widget_instance_lookup_time_us = out
                .max_paint_host_widget_instance_lookup_time_us
                .max(paint_host_widget_instance_lookup_time_us);
            out.max_paint_host_widget_instance_lookup_calls = out
                .max_paint_host_widget_instance_lookup_calls
                .max(paint_host_widget_instance_lookup_calls);
            out.max_total_time_us = out.max_total_time_us.max(total_time_us);
            out.max_ui_thread_cpu_time_us =
                out.max_ui_thread_cpu_time_us.max(ui_thread_cpu_time_us);
            out.max_ui_thread_cpu_cycle_time_delta_cycles = out
                .max_ui_thread_cpu_cycle_time_delta_cycles
                .max(ui_thread_cpu_cycle_time_delta_cycles);
            out.max_layout_engine_solve_time_us = out
                .max_layout_engine_solve_time_us
                .max(layout_engine_solve_time_us);
            let dispatch_accounted_time_us = hit_test_time_us
                .saturating_add(dispatch_hover_update_time_us)
                .saturating_add(dispatch_input_state_update_time_us)
                .saturating_add(dispatch_context_build_time_us)
                .saturating_add(dispatch_prelude_time_us)
                .saturating_add(dispatch_pointer_arbitration_time_us)
                .saturating_add(dispatch_pointer_target_routing_time_us)
                .saturating_add(dispatch_post_widget_control_flow_time_us)
                .saturating_add(dispatch_scroll_handle_invalidation_time_us)
                .saturating_add(dispatch_active_layers_time_us)
                .saturating_add(dispatch_input_context_time_us)
                .saturating_add(dispatch_event_chain_build_time_us)
                .saturating_add(dispatch_widget_capture_time_us)
                .saturating_add(dispatch_widget_bubble_time_us)
                .saturating_add(dispatch_cursor_query_time_us)
                .saturating_add(dispatch_pointer_move_layer_observers_time_us)
                .saturating_add(dispatch_synth_hover_observer_time_us)
                .saturating_add(dispatch_cursor_effect_time_us)
                .saturating_add(dispatch_post_dispatch_snapshot_time_us);
            out.max_dispatch_accounted_time_us = out
                .max_dispatch_accounted_time_us
                .max(dispatch_accounted_time_us);
            out.max_dispatch_unattributed_time_us = out
                .max_dispatch_unattributed_time_us
                .max(dispatch_time_us.saturating_sub(dispatch_accounted_time_us));
            let dispatch_body_time_us = if dispatch_inner_body_time_us == 0 {
                dispatch_time_us
            } else {
                dispatch_inner_body_time_us
            };
            out.max_dispatch_inner_body_unattributed_time_us = out
                .max_dispatch_inner_body_unattributed_time_us
                .max(dispatch_body_time_us.saturating_sub(dispatch_accounted_time_us));
            let dispatch_runtime_wrapper_time_us = if dispatch_inner_body_time_us == 0 {
                0
            } else {
                dispatch_time_us.saturating_sub(dispatch_inner_body_time_us)
            };
            out.max_dispatch_runtime_wrapper_time_us = out
                .max_dispatch_runtime_wrapper_time_us
                .max(dispatch_runtime_wrapper_time_us);
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
            out.max_renderer_prepare_text_collect_pin_keys_us = out
                .max_renderer_prepare_text_collect_pin_keys_us
                .max(renderer_prepare_text_collect_pin_keys_us);
            out.max_renderer_prepare_text_bucket_delta_us = out
                .max_renderer_prepare_text_bucket_delta_us
                .max(renderer_prepare_text_bucket_delta_us);
            out.max_renderer_prepare_text_prewarm_us = out
                .max_renderer_prepare_text_prewarm_us
                .max(renderer_prepare_text_prewarm_us);
            out.max_renderer_prepare_text_pin_bucket_update_us = out
                .max_renderer_prepare_text_pin_bucket_update_us
                .max(renderer_prepare_text_pin_bucket_update_us);
            out.max_renderer_prepare_text_flush_uploads_us = out
                .max_renderer_prepare_text_flush_uploads_us
                .max(renderer_prepare_text_flush_uploads_us);

            rows.push(BundleStatsSnapshotRow {
                window: window_id,
                tick_id: s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                frame_id: s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                timestamp_unix_ms: s.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
                frame_arena_capacity_estimate_bytes,
                frame_arena_grow_events,
                element_children_vec_pool_reuses,
                element_children_vec_pool_misses,
                element_children_vec_pool_grow_events,
                ui_thread_cpu_time_us,
                ui_thread_cpu_total_time_us,
                ui_thread_cpu_cycle_time_delta_cycles,
                ui_thread_cpu_cycle_time_total_cycles,
                layout_time_us,
                layout_collect_roots_time_us,
                layout_invalidate_scroll_handle_bindings_time_us,
                layout_expand_view_cache_invalidations_time_us,
                layout_request_build_roots_time_us,
                layout_request_build_roots_take_engine_time_us,
                layout_request_build_roots_phase1_time_us,
                layout_request_build_roots_phase2_time_us,
                layout_request_build_roots_phase2_clean_geometry_proof_time_us,
                layout_clean_geometry_proof_nodes,
                layout_clean_geometry_proof_boundaries,
                layout_clean_geometry_proof_leaf_shortcut_time_us,
                layout_clean_geometry_proof_node_state_time_us,
                layout_clean_geometry_proof_contract_time_us,
                layout_clean_geometry_proof_record_time_us,
                layout_clean_geometry_proof_contract_eval_time_us,
                layout_clean_geometry_proof_child_bounds_time_us,
                layout_clean_geometry_proof_child_bounds_origin_only_time_us,
                layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us,
                layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us,
                layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us,
                layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us,
                layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us,
                layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us,
                layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us,
                layout_clean_geometry_proof_child_bounds_container_px_insets_time_us,
                layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us,
                layout_clean_geometry_proof_text_metrics_time_us,
                layout_clean_geometry_proof_child_prev_bounds_time_us,
                layout_clean_geometry_proof_emit_time_us,
                layout_clean_geometry_apply_nodes,
                layout_clean_geometry_apply_fallback_layouts,
                layout_clean_geometry_apply_fallback_layouts_time_us,
                layout_clean_geometry_apply_fallback_layouts_top_time_us,
                layout_clean_geometry_apply_fallback_layouts_top_kind,
                layout_clean_geometry_scroll_side_effect_fast_paths,
                layout_clean_geometry_apply_paint_fingerprint_time_us,
                layout_request_build_roots_phase2_compute_time_us,
                layout_request_build_roots_put_engine_time_us,
                layout_roots_time_us,
                layout_roots_apply_time_us,
                layout_roots_flush_viewport_time_us,
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
                paint_host_widget_observed_deps_calls,
                paint_host_widget_observed_deps_empty_calls,
                paint_host_widget_observed_models_non_empty_calls,
                paint_host_widget_observed_globals_non_empty_calls,
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
                code_editor_paint_perf,
                dispatch_time_us,
                dispatch_inner_body_time_us,
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
                dispatch_input_state_update_time_us,
                dispatch_context_build_time_us,
                dispatch_prelude_time_us,
                dispatch_pointer_arbitration_time_us,
                dispatch_pointer_target_routing_time_us,
                dispatch_post_widget_control_flow_time_us,
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
                window_runtime_snapshot_focus_repair_time_us,
                window_runtime_snapshot_input_context_time_us,
                window_runtime_snapshot_command_availability_time_us,
                window_runtime_snapshot_widget_command_count,
                window_runtime_snapshot_command_registry_collect_time_us,
                window_runtime_snapshot_command_availability_eval_time_us,
                window_runtime_snapshot_shortcut_overlay_time_us,
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
                renderer_prepare_text_collect_pin_keys_us,
                renderer_prepare_text_bucket_delta_us,
                renderer_prepare_text_prewarm_us,
                renderer_prepare_text_pin_bucket_update_us,
                renderer_prepare_text_flush_uploads_us,
                renderer_prepare_text_scene_text_blobs,
                renderer_prepare_text_pinned_glyph_keys,
                renderer_prepare_text_prewarm_glyph_keys,
                renderer_prepare_text_retained_glyph_keys,
                renderer_prepare_text_added_glyph_keys,
                renderer_prepare_text_removed_glyph_keys,
                renderer_prepare_svg_us,
                renderer_uniform_bytes,
                renderer_instance_bytes,
                renderer_vertex_bytes,
                renderer_encode_scene_stack_us,
                renderer_encode_scene_clip_us,
                renderer_encode_scene_mask_us,
                renderer_encode_scene_effect_us,
                renderer_encode_scene_quad_us,
                renderer_encode_scene_image_us,
                renderer_encode_scene_text_us,
                renderer_encode_scene_path_us,
                renderer_encode_scene_viewport_us,
                renderer_encode_scene_flush_us,
                renderer_encode_scene_text_shadow_us,
                renderer_encode_scene_text_setup_us,
                renderer_encode_scene_text_glyphs_us,
                renderer_encode_scene_text_glyph_transform_us,
                renderer_encode_scene_text_glyph_emit_us,
                renderer_encode_scene_text_group_flush_us,
                renderer_encode_scene_text_vertex_grow_events,
                renderer_encode_scene_text_transform_fast_path_glyphs,
                renderer_encode_scene_text_transform_generic_glyphs,
                renderer_encode_scene_stack_ops,
                renderer_encode_scene_clip_ops,
                renderer_encode_scene_mask_ops,
                renderer_encode_scene_effect_ops,
                renderer_encode_scene_quad_ops,
                renderer_encode_scene_image_ops,
                renderer_encode_scene_text_ops,
                renderer_encode_scene_path_ops,
                renderer_encode_scene_viewport_ops,
                renderer_encode_scene_flushes,
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
                renderer_intermediate_full_target_bytes,
                renderer_render_plan_effect_chain_budget_samples,
                renderer_render_plan_effect_chain_effective_budget_min_bytes,
                renderer_render_plan_effect_chain_effective_budget_max_bytes,
                renderer_render_plan_effect_chain_other_live_max_bytes,
                renderer_render_plan_custom_effect_chain_budget_samples,
                renderer_render_plan_custom_effect_chain_effective_budget_min_bytes,
                renderer_render_plan_custom_effect_chain_effective_budget_max_bytes,
                renderer_render_plan_custom_effect_chain_other_live_max_bytes,
                renderer_render_plan_custom_effect_chain_base_required_max_bytes,
                renderer_render_plan_custom_effect_chain_optional_required_max_bytes,
                renderer_render_plan_custom_effect_chain_base_required_full_targets_max,
                renderer_render_plan_custom_effect_chain_optional_mask_max_bytes,
                renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes,
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
                renderer_custom_effect_v1_steps_requested,
                renderer_custom_effect_v1_passes_emitted,
                renderer_custom_effect_v2_steps_requested,
                renderer_custom_effect_v2_passes_emitted,
                renderer_custom_effect_v2_user_image_incompatible_fallbacks,
                renderer_custom_effect_v3_steps_requested,
                renderer_custom_effect_v3_passes_emitted,
                renderer_custom_effect_v3_user0_image_incompatible_fallbacks,
                renderer_custom_effect_v3_user1_image_incompatible_fallbacks,
                renderer_custom_effect_v3_pyramid_cache_hits,
                renderer_custom_effect_v3_pyramid_cache_misses,
                renderer_custom_effect_v3_sources_raw_requested,
                renderer_custom_effect_v3_sources_raw_distinct,
                renderer_custom_effect_v3_sources_raw_aliased_to_src,
                renderer_custom_effect_v3_sources_pyramid_requested,
                renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2,
                renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero,
                renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient,
                renderer_backdrop_source_groups_requested,
                renderer_backdrop_source_groups_applied_raw,
                renderer_backdrop_source_groups_raw_degraded_budget_zero,
                renderer_backdrop_source_groups_raw_degraded_budget_insufficient,
                renderer_backdrop_source_groups_raw_degraded_target_exhausted,
                renderer_backdrop_source_groups_pyramid_requested,
                renderer_backdrop_source_groups_pyramid_applied_levels_ge2,
                renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero,
                renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient,
                renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable,
                layout_engine_solves,
                layout_engine_solve_time_us,
                layout_clean_geometry_solve_skip_rejections,
                layout_clean_geometry_solve_skip_first_rejection,
                layout_clean_geometry_solve_skip_first_detail,
                layout_clean_geometry_solve_skip_first_element_kind,
                layout_engine_child_rect_queries,
                layout_engine_child_rect_time_us,
                layout_engine_widget_fallback_solves,
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
                layout_request_build_roots,
                scroll_layout_profiles,
                top_layout_engine_solves,
                layout_hotspots,
                widget_measure_hotspots,
                paint_widget_hotspots,
                paint_text_prepare_hotspots,
                command_availability_hotspots,
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

    fn code_editor_paint_perf_percentile<'a>(
        values: impl Iterator<Item = &'a BundleStatsCodeEditorPaintPerf>,
        percentile: f64,
    ) -> BundleStatsCodeEditorPaintPerfTotals {
        let values: Vec<&BundleStatsCodeEditorPaintPerf> = values.collect();
        if values.is_empty() {
            return BundleStatsCodeEditorPaintPerfTotals::default();
        }

        fn percentile_value(values: impl Iterator<Item = u64>, percentile: f64) -> u64 {
            let mut sorted: Vec<u64> = values.collect();
            sorted.sort_unstable();
            crate::percentile_nearest_rank_sorted(&sorted, percentile)
        }

        macro_rules! metric {
            ($field:ident) => {
                percentile_value(values.iter().map(|p| p.$field), percentile)
            };
        }

        BundleStatsCodeEditorPaintPerfTotals {
            rows_painted: metric!(rows_painted),
            rows_drew_rich: metric!(rows_drew_rich),
            rows_scene_replayed: metric!(rows_scene_replayed),
            rows_scene_prepaint_planned: metric!(rows_scene_prepaint_planned),
            rows_scene_prepaint_plan_used: metric!(rows_scene_prepaint_plan_used),
            rows_scene_stored: metric!(rows_scene_stored),
            rows_scene_stored_at_visible_start: metric!(rows_scene_stored_at_visible_start),
            rows_scene_stored_at_visible_end: metric!(rows_scene_stored_at_visible_end),
            row_scene_ops_stored: metric!(row_scene_ops_stored),
            rows_scene_prepaint_edge_stored: metric!(rows_scene_prepaint_edge_stored),
            row_scene_prepaint_edge_ops_stored: metric!(row_scene_prepaint_edge_ops_stored),
            rows_scene_prepaint_candidates: metric!(rows_scene_prepaint_candidates),
            rows_scene_prepaint_skip_no_cache: metric!(rows_scene_prepaint_skip_no_cache),
            rows_scene_prepaint_skip_unsupported_key: metric!(
                rows_scene_prepaint_skip_unsupported_key
            ),
            rows_scene_prepaint_skip_preedit: metric!(rows_scene_prepaint_skip_preedit),
            rows_scene_prepaint_skip_syntax_empty: metric!(rows_scene_prepaint_skip_syntax_empty),
            rows_scene_prepaint_skip_key_mismatch: metric!(
                rows_scene_prepaint_skip_key_mismatch
            ),
            rows_scene_prepaint_plan_cache_hits: metric!(rows_scene_prepaint_plan_cache_hits),
            rows_scene_prepaint_plan_cache_rejects: metric!(
                rows_scene_prepaint_plan_cache_rejects
            ),
            rows_scene_fast_miss_no_entry: metric!(rows_scene_fast_miss_no_entry),
            rows_scene_fast_miss_key_mismatch: metric!(rows_scene_fast_miss_key_mismatch),
            rows_scene_full_miss_no_entry: metric!(rows_scene_full_miss_no_entry),
            rows_scene_full_miss_key_mismatch: metric!(rows_scene_full_miss_key_mismatch),
            quads_selection: metric!(quads_selection),
            quads_caret: metric!(quads_caret),
            syntax_rows_stored: metric!(syntax_rows_stored),
            us_total: metric!(us_total),
            us_row_text: metric!(us_row_text),
            us_baseline_measure: metric!(us_baseline_measure),
            us_row_content_resolve: metric!(us_row_content_resolve),
            us_row_rich_cache_compare: metric!(us_row_rich_cache_compare),
            us_row_geom_key: metric!(us_row_geom_key),
            us_rich_materialize: metric!(us_rich_materialize),
            us_text_draw: metric!(us_text_draw),
            us_row_scene_key: metric!(us_row_scene_key),
            us_row_scene_fast_probe: metric!(us_row_scene_fast_probe),
            us_row_scene_full_probe: metric!(us_row_scene_full_probe),
            us_row_scene_fast_key_compare: metric!(us_row_scene_fast_key_compare),
            us_row_scene_full_key_compare: metric!(us_row_scene_full_key_compare),
            us_row_scene_replay_setup: metric!(us_row_scene_replay_setup),
            us_row_scene_replay_touch: metric!(us_row_scene_replay_touch),
            us_row_scene_replay_ops: metric!(us_row_scene_replay_ops),
            us_row_scene_prepaint_plan: metric!(us_row_scene_prepaint_plan),
            us_row_scene_prepaint_probe: metric!(us_row_scene_prepaint_probe),
            us_row_scene_prepaint_key_compare: metric!(us_row_scene_prepaint_key_compare),
            us_row_scene_capture_ops: metric!(us_row_scene_capture_ops),
            us_row_scene_store: metric!(us_row_scene_store),
            us_row_scene_prepaint_edge_store: metric!(us_row_scene_prepaint_edge_store),
            us_row_scene_fast_path: metric!(us_row_scene_fast_path),
            us_row_scene_full_path: metric!(us_row_scene_full_path),
            us_syntax_spans: metric!(us_syntax_spans),
            us_syntax_slice: metric!(us_syntax_slice),
            us_syntax_highlight: metric!(us_syntax_highlight),
            us_syntax_distribute: metric!(us_syntax_distribute),
            us_syntax_store: metric!(us_syntax_store),
            us_selection_rects: metric!(us_selection_rects),
            us_caret_x: metric!(us_caret_x),
            us_caret_stops: metric!(us_caret_stops),
            us_caret_rect: metric!(us_caret_rect),
            us_row_geom_cache: metric!(us_row_geom_cache),
            us_row_geom_resolve: metric!(us_row_geom_resolve),
            us_row_overlay: metric!(us_row_overlay),
            us_frame_overlay_prepare: metric!(us_frame_overlay_prepare),
            surface_rows_iterated: metric!(surface_rows_iterated),
            surface_rows_with_rect: metric!(surface_rows_with_rect),
            us_windowed_surface_paint_callback: metric!(us_windowed_surface_paint_callback),
            us_windowed_surface_frame_lookup: metric!(us_windowed_surface_frame_lookup),
            us_windowed_surface_hook: metric!(us_windowed_surface_hook),
            us_windowed_surface_row_loop: metric!(us_windowed_surface_row_loop),
            us_windowed_surface_row_rect: metric!(us_windowed_surface_row_rect),
            us_windowed_surface_row_paint: metric!(us_windowed_surface_row_paint),
            us_windowed_surface_non_row: metric!(us_windowed_surface_non_row),
            us_windowed_surface_row_callback_gap: metric!(
                us_windowed_surface_row_callback_gap
            ),
            us_torture_autoscroll: metric!(us_torture_autoscroll),
            us_torture_overlay: metric!(us_torture_overlay),
        }
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
    (
        out.p50_layout_request_build_roots_take_engine_time_us,
        out.p95_layout_request_build_roots_take_engine_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_request_build_roots_take_engine_time_us));
    (
        out.p50_layout_request_build_roots_phase1_time_us,
        out.p95_layout_request_build_roots_phase1_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_request_build_roots_phase1_time_us));
    (
        out.p50_layout_request_build_roots_phase2_time_us,
        out.p95_layout_request_build_roots_phase2_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_request_build_roots_phase2_time_us));
    (
        out.p50_layout_request_build_roots_phase2_clean_geometry_proof_time_us,
        out.p95_layout_request_build_roots_phase2_clean_geometry_proof_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_request_build_roots_phase2_clean_geometry_proof_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_nodes,
        out.p95_layout_clean_geometry_proof_nodes,
    ) = p50_p95(rows.iter().map(|r| r.layout_clean_geometry_proof_nodes));
    (
        out.p50_layout_clean_geometry_proof_boundaries,
        out.p95_layout_clean_geometry_proof_boundaries,
    ) = p50_p95(rows.iter().map(|r| r.layout_clean_geometry_proof_boundaries));
    (
        out.p50_layout_clean_geometry_proof_leaf_shortcut_time_us,
        out.p95_layout_clean_geometry_proof_leaf_shortcut_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_leaf_shortcut_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_node_state_time_us,
        out.p95_layout_clean_geometry_proof_node_state_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_node_state_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_contract_time_us,
        out.p95_layout_clean_geometry_proof_contract_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_clean_geometry_proof_contract_time_us));
    (
        out.p50_layout_clean_geometry_proof_record_time_us,
        out.p95_layout_clean_geometry_proof_record_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_clean_geometry_proof_record_time_us));
    (
        out.p50_layout_clean_geometry_proof_contract_eval_time_us,
        out.p95_layout_clean_geometry_proof_contract_eval_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_contract_eval_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_clean_geometry_proof_child_bounds_time_us));
    (
        out.p50_layout_clean_geometry_proof_child_bounds_origin_only_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_origin_only_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_bounds_origin_only_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_bounds_preserve_local_origins_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_bounds_preserve_local_origins_style_lookup_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_bounds_preserve_local_origins_prev_bounds_lookup_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_bounds_preserve_local_origins_absolute_child_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_bounds_preserve_local_origins_relative_child_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_bounds_vertical_no_wrap_flex_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_bounds_horizontal_fixed_flex_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_container_px_insets_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_container_px_insets_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_bounds_container_px_insets_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us,
        out.p95_layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us,
    ) = p50_p95(
        rows.iter().map(|r| {
            r.layout_clean_geometry_proof_child_bounds_single_column_auto_rows_grid_time_us
        }),
    );
    (
        out.p50_layout_clean_geometry_proof_text_metrics_time_us,
        out.p95_layout_clean_geometry_proof_text_metrics_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_clean_geometry_proof_text_metrics_time_us));
    (
        out.p50_layout_clean_geometry_proof_child_prev_bounds_time_us,
        out.p95_layout_clean_geometry_proof_child_prev_bounds_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_proof_child_prev_bounds_time_us),
    );
    (
        out.p50_layout_clean_geometry_proof_emit_time_us,
        out.p95_layout_clean_geometry_proof_emit_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_clean_geometry_proof_emit_time_us));
    (
        out.p50_layout_clean_geometry_apply_nodes,
        out.p95_layout_clean_geometry_apply_nodes,
    ) = p50_p95(rows.iter().map(|r| r.layout_clean_geometry_apply_nodes));
    (
        out.p50_layout_clean_geometry_apply_fallback_layouts,
        out.p95_layout_clean_geometry_apply_fallback_layouts,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_apply_fallback_layouts),
    );
    (
        out.p50_layout_clean_geometry_apply_fallback_layouts_time_us,
        out.p95_layout_clean_geometry_apply_fallback_layouts_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_apply_fallback_layouts_time_us),
    );
    (
        out.p50_layout_clean_geometry_apply_fallback_layouts_top_time_us,
        out.p95_layout_clean_geometry_apply_fallback_layouts_top_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_apply_fallback_layouts_top_time_us),
    );
    (
        out.p50_layout_clean_geometry_scroll_side_effect_fast_paths,
        out.p95_layout_clean_geometry_scroll_side_effect_fast_paths,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_scroll_side_effect_fast_paths),
    );
    (
        out.p50_layout_clean_geometry_apply_paint_fingerprint_time_us,
        out.p95_layout_clean_geometry_apply_paint_fingerprint_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.layout_clean_geometry_apply_paint_fingerprint_time_us),
    );
    (
        out.p50_layout_request_build_roots_phase2_compute_time_us,
        out.p95_layout_request_build_roots_phase2_compute_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_request_build_roots_phase2_compute_time_us));
    (
        out.p50_layout_request_build_roots_put_engine_time_us,
        out.p95_layout_request_build_roots_put_engine_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_request_build_roots_put_engine_time_us));
    (out.p50_layout_roots_time_us, out.p95_layout_roots_time_us) =
        p50_p95(rows.iter().map(|r| r.layout_roots_time_us));
    (out.p50_layout_roots_apply_time_us, out.p95_layout_roots_apply_time_us) =
        p50_p95(rows.iter().map(|r| r.layout_roots_apply_time_us));
    (
        out.p50_layout_roots_flush_viewport_time_us,
        out.p95_layout_roots_flush_viewport_time_us,
    ) = p50_p95(rows.iter().map(|r| r.layout_roots_flush_viewport_time_us));
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
        out.p50_paint_record_visual_bounds_time_us,
        out.p95_paint_record_visual_bounds_time_us,
    ) = p50_p95(rows.iter().map(|r| r.paint_record_visual_bounds_time_us));
    (
        out.p50_paint_record_visual_bounds_calls,
        out.p95_paint_record_visual_bounds_calls,
    ) = p50_p95(rows.iter().map(|r| r.paint_record_visual_bounds_calls as u64));
    (
        out.p50_paint_cache_key_time_us,
        out.p95_paint_cache_key_time_us,
    ) = p50_p95(rows.iter().map(|r| r.paint_cache_key_time_us));
    (
        out.p50_paint_cache_hit_check_time_us,
        out.p95_paint_cache_hit_check_time_us,
    ) = p50_p95(rows.iter().map(|r| r.paint_cache_hit_check_time_us));
    (
        out.p50_paint_observation_record_time_us,
        out.p95_paint_observation_record_time_us,
    ) = p50_p95(rows.iter().map(|r| r.paint_observation_record_time_us));
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
    (
        out.p50_dispatch_accounted_time_us,
        out.p95_dispatch_accounted_time_us,
    ) = p50_p95(
        rows.iter()
            .map(BundleStatsReport::dispatch_accounted_time_us),
    );
    (
        out.p50_dispatch_unattributed_time_us,
        out.p95_dispatch_unattributed_time_us,
    ) = p50_p95(
        rows.iter()
            .map(BundleStatsReport::dispatch_unattributed_time_us),
    );
    (
        out.p50_dispatch_inner_body_unattributed_time_us,
        out.p95_dispatch_inner_body_unattributed_time_us,
    ) = p50_p95(
        rows.iter()
            .map(BundleStatsReport::dispatch_inner_body_unattributed_time_us),
    );
    (
        out.p50_dispatch_runtime_wrapper_time_us,
        out.p95_dispatch_runtime_wrapper_time_us,
    ) = p50_p95(
        rows.iter()
            .map(BundleStatsReport::dispatch_runtime_wrapper_time_us),
    );
    (out.p50_hit_test_time_us, out.p95_hit_test_time_us) =
        p50_p95(rows.iter().map(|r| r.hit_test_time_us));
    (out.p50_paint_widget_time_us, out.p95_paint_widget_time_us) =
        p50_p95(rows.iter().map(|r| r.paint_widget_time_us));
    (
        out.p50_paint_host_widget_observed_models_time_us,
        out.p95_paint_host_widget_observed_models_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_observed_models_time_us),
    );
    (
        out.p50_paint_host_widget_observed_models_items,
        out.p95_paint_host_widget_observed_models_items,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_observed_models_items as u64),
    );
    (
        out.p50_paint_host_widget_observed_globals_time_us,
        out.p95_paint_host_widget_observed_globals_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_observed_globals_time_us),
    );
    (
        out.p50_paint_host_widget_observed_globals_items,
        out.p95_paint_host_widget_observed_globals_items,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_observed_globals_items as u64),
    );
    (
        out.p50_paint_host_widget_observed_deps_calls,
        out.p95_paint_host_widget_observed_deps_calls,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_observed_deps_calls as u64),
    );
    (
        out.p50_paint_host_widget_observed_deps_empty_calls,
        out.p95_paint_host_widget_observed_deps_empty_calls,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_observed_deps_empty_calls as u64),
    );
    (
        out.p50_paint_host_widget_observed_models_non_empty_calls,
        out.p95_paint_host_widget_observed_models_non_empty_calls,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_observed_models_non_empty_calls as u64),
    );
    (
        out.p50_paint_host_widget_observed_globals_non_empty_calls,
        out.p95_paint_host_widget_observed_globals_non_empty_calls,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_observed_globals_non_empty_calls as u64),
    );
    (
        out.p50_paint_host_widget_instance_lookup_time_us,
        out.p95_paint_host_widget_instance_lookup_time_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_instance_lookup_time_us),
    );
    (
        out.p50_paint_host_widget_instance_lookup_calls,
        out.p95_paint_host_widget_instance_lookup_calls,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.paint_host_widget_instance_lookup_calls as u64),
    );
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
    (
        out.p50_renderer_prepare_text_collect_pin_keys_us,
        out.p95_renderer_prepare_text_collect_pin_keys_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_prepare_text_collect_pin_keys_us));
    (
        out.p50_renderer_prepare_text_bucket_delta_us,
        out.p95_renderer_prepare_text_bucket_delta_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_prepare_text_bucket_delta_us));
    (
        out.p50_renderer_prepare_text_prewarm_us,
        out.p95_renderer_prepare_text_prewarm_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_prepare_text_prewarm_us));
    (
        out.p50_renderer_prepare_text_pin_bucket_update_us,
        out.p95_renderer_prepare_text_pin_bucket_update_us,
    ) = p50_p95(
        rows.iter()
            .map(|r| r.renderer_prepare_text_pin_bucket_update_us),
    );
    (
        out.p50_renderer_prepare_text_flush_uploads_us,
        out.p95_renderer_prepare_text_flush_uploads_us,
    ) = p50_p95(rows.iter().map(|r| r.renderer_prepare_text_flush_uploads_us));
    out.code_editor_paint_perf.p50 = code_editor_paint_perf_percentile(
        rows.iter().filter_map(|r| r.code_editor_paint_perf.as_ref()),
        0.50,
    );
    out.code_editor_paint_perf.p95 = code_editor_paint_perf_percentile(
        rows.iter().filter_map(|r| r.code_editor_paint_perf.as_ref()),
        0.95,
    );

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
        BundleStatsSort::CommandAvailability => {
            rows.sort_by(|a, b| {
                b.window_runtime_snapshot_command_availability_time_us
                    .cmp(&a.window_runtime_snapshot_command_availability_time_us)
                    .then_with(|| {
                        b.window_runtime_snapshot_command_availability_eval_time_us
                            .cmp(&a.window_runtime_snapshot_command_availability_eval_time_us)
                    })
                    .then_with(|| {
                        b.window_runtime_snapshot_command_registry_collect_time_us
                            .cmp(&a.window_runtime_snapshot_command_registry_collect_time_us)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
                    .then_with(|| b.dispatch_time_us.cmp(&a.dispatch_time_us))
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
    let boundaries_by_id: std::collections::HashMap<u64, &serde_json::Value> = snapshot
        .get("debug")
        .and_then(|v| v.get("boundaries"))
        .and_then(|v| v.as_array())
        .map(|boundaries| {
            boundaries
                .iter()
                .filter_map(|b| b.get("id").and_then(|v| v.as_u64()).map(|id| (id, b)))
                .collect()
        })
        .unwrap_or_default();

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
            let boundary = boundaries_by_id.get(&root_node).copied();
            let boundary_layout_dependency = boundary
                .and_then(|v| v.get("layout_dependency"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let layout_dependency = boundary_layout_dependency.clone().or_else(|| {
                r.get("layout_dependency")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });
            BundleStatsCacheRoot {
                root_node,
                element: r.get("element").and_then(|v| v.as_u64()),
                element_path: r
                    .get("element_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                reused: reused_flag,
                layout_dependency,
                contained_relayout_in_frame,
                paint_replayed_ops,
                reuse_reason: r
                    .get("reuse_reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                root_in_semantics: r.get("root_in_semantics").and_then(|v| v.as_bool()),
                root_role: role,
                root_test_id: test_id,
                boundary_kind: boundary
                    .and_then(|v| v.get("kind"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                boundary_layout_dependency,
                boundary_build_outcome: boundary
                    .and_then(|v| v.get("build_outcome"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                boundary_reuse_reason: boundary
                    .and_then(|v| v.get("reuse_reason"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                boundary_layout_outcome: boundary
                    .and_then(|v| v.get("layout_outcome"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                boundary_prepaint_owner: boundary
                    .and_then(|v| v.get("prepaint_owner"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                boundary_paint_outcome: boundary
                    .and_then(|v| v.get("paint_outcome"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
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
