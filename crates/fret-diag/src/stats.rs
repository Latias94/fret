use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod bundle_stats_snapshot;
mod bundle_stats_sort;
mod debug_stats_gates;
mod drag_cache_gates;
mod drag_cache_gates_streaming;
mod frames_index_gates;
mod gc_gates;
mod gc_gates_streaming;
mod hello_world_compare;
mod hover_layout_checks;
mod interaction_gates;
mod notify_gates;
mod notify_gates_streaming;
mod overlay_gates;
mod pixels_changed;
mod resource_loading;
mod retained_vlist_gates;
mod retained_vlist_gates_streaming;
mod script_runtime;
mod semantics;
mod stale;
mod stale_checks_streaming;
mod stale_streaming;
mod ui_gallery_code_editor;
mod ui_gallery_markdown_editor;
mod ui_gallery_text_gates;
mod view_cache_gates;
mod vlist;
mod wheel_events_streaming;
mod wheel_scroll;
mod wheel_scroll_streaming;
mod windowed_rows;
pub(super) use bundle_stats_sort::BundleStatsSort;
pub(super) use script_runtime::{
    ScriptResultSummary, apply_pick_to_script, clear_script_result_files,
    report_pick_result_and_exit, report_result_and_exit, run_pick_and_wait, run_script_and_wait,
    wait_for_failure_dump_bundle, write_pick_script,
};
use semantics::{semantics_node_id_for_test_id, semantics_parent_map};
pub(super) use ui_gallery_code_editor::*;
pub(super) use ui_gallery_markdown_editor::*;
use wheel_scroll::first_wheel_frame_id_for_window;

pub(super) use debug_stats_gates::{
    check_bundle_for_chart_sampling_window_shifts_min, check_bundle_for_layout_fast_path_min,
    check_bundle_for_node_graph_cull_window_shifts_max,
    check_bundle_for_node_graph_cull_window_shifts_min, check_bundle_for_prepaint_actions_min,
};
pub(super) use drag_cache_gates::check_bundle_for_drag_cache_root_paint_only;
pub(super) use frames_index_gates::{
    check_frames_index_for_dock_drag_min, check_frames_index_for_idle_no_paint_min,
    check_frames_index_for_overlay_synthesis_min, check_frames_index_for_view_cache_reuse_min,
    check_frames_index_for_view_cache_reuse_stable_min,
    check_frames_index_for_viewport_capture_min, check_frames_index_for_viewport_input_min,
};
pub(super) use gc_gates::check_bundle_for_gc_sweep_liveness;
pub(super) use gc_gates_streaming::check_bundle_for_gc_sweep_liveness_streaming;
pub(super) use hello_world_compare::check_out_dir_for_hello_world_compare_idle_present_max_delta;
pub(super) use hover_layout_checks::check_report_for_hover_layout_invalidations;
pub(super) use interaction_gates::{
    check_bundle_for_dock_drag_min, check_bundle_for_viewport_capture_min,
    check_bundle_for_viewport_input_min,
};
pub(super) use notify_gates::check_bundle_for_notify_hotspot_file_max;
pub(super) use overlay_gates::check_bundle_for_overlay_synthesis_min;
pub(super) use pixels_changed::{
    check_out_dir_for_pixels_changed, check_out_dir_for_pixels_unchanged,
};
pub(super) use resource_loading::{
    check_bundle_for_asset_load_external_reference_unavailable_max,
    check_bundle_for_asset_load_external_reference_unavailable_max_streaming,
    check_bundle_for_asset_load_io_max, check_bundle_for_asset_load_io_max_streaming,
    check_bundle_for_asset_load_missing_bundle_assets_max,
    check_bundle_for_asset_load_missing_bundle_assets_max_streaming,
    check_bundle_for_asset_load_revision_changes_max,
    check_bundle_for_asset_load_revision_changes_max_streaming,
    check_bundle_for_asset_load_stale_manifest_max,
    check_bundle_for_asset_load_stale_manifest_max_streaming,
    check_bundle_for_asset_load_unsupported_file_max,
    check_bundle_for_asset_load_unsupported_file_max_streaming,
    check_bundle_for_asset_load_unsupported_url_max,
    check_bundle_for_asset_load_unsupported_url_max_streaming,
    check_bundle_for_asset_reload_active_backend,
    check_bundle_for_asset_reload_active_backend_streaming,
    check_bundle_for_asset_reload_configured_backend,
    check_bundle_for_asset_reload_configured_backend_streaming,
    check_bundle_for_asset_reload_epoch_min, check_bundle_for_asset_reload_epoch_min_streaming,
    check_bundle_for_asset_reload_fallback_reason,
    check_bundle_for_asset_reload_fallback_reason_streaming,
    check_bundle_for_bundled_font_baseline_source,
    check_bundle_for_bundled_font_baseline_source_streaming,
};
pub(super) use retained_vlist_gates::{
    check_bundle_for_retained_vlist_attach_detach_max,
    check_bundle_for_retained_vlist_keep_alive_budget,
    check_bundle_for_retained_vlist_keep_alive_reuse_min,
    check_bundle_for_retained_vlist_reconcile_no_notify_min,
};
pub(super) use stale::{
    check_bundle_for_idle_no_paint_min, check_bundle_for_semantics_changed_repainted,
    check_bundle_for_stale_paint, check_bundle_for_stale_scene,
};
pub(super) use stale_checks_streaming::{
    check_bundle_for_stale_paint_streaming, check_bundle_for_stale_scene_streaming,
};
pub(super) use stale_streaming::check_bundle_for_semantics_changed_repainted_streaming;
pub(super) use ui_gallery_text_gates::{
    check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change,
    check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change,
    check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance,
    check_out_dir_for_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps,
};
pub(super) use view_cache_gates::{
    check_bundle_for_view_cache_reuse_min, check_bundle_for_view_cache_reuse_stable_min,
};
pub(super) use vlist::{
    check_bundle_for_vlist_policy_key_stable, check_bundle_for_vlist_visible_range_refreshes_max,
    check_bundle_for_vlist_visible_range_refreshes_min,
    check_bundle_for_vlist_window_shifts_explainable,
    check_bundle_for_vlist_window_shifts_have_prepaint_actions,
    check_bundle_for_vlist_window_shifts_kind_max,
    check_bundle_for_vlist_window_shifts_non_retained_max,
};
pub(super) use wheel_events_streaming::check_bundle_for_wheel_events_max_per_frame;
pub(super) use wheel_scroll::{
    check_bundle_for_wheel_scroll, check_bundle_for_wheel_scroll_hit_changes,
};
pub(super) use windowed_rows::{
    check_bundle_for_windowed_rows_offset_changes_min,
    check_bundle_for_windowed_rows_visible_start_changes_repainted,
};

#[cfg(test)]
#[allow(unused_imports)]
pub(super) use debug_stats_gates::{
    check_bundle_for_chart_sampling_window_shifts_min_json,
    check_bundle_for_layout_fast_path_min_json,
    check_bundle_for_node_graph_cull_window_shifts_max_json,
    check_bundle_for_node_graph_cull_window_shifts_min_json,
    check_bundle_for_prepaint_actions_min_json,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use interaction_gates::{
    check_bundle_for_dock_drag_min_json, check_bundle_for_viewport_capture_min_json,
    check_bundle_for_viewport_input_min_json,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use notify_gates::check_bundle_for_notify_hotspot_file_max_json;
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use overlay_gates::check_bundle_for_overlay_synthesis_min_json;
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use resource_loading::{
    check_bundle_for_asset_load_external_reference_unavailable_max_json,
    check_bundle_for_asset_load_io_max_json,
    check_bundle_for_asset_load_missing_bundle_assets_max_json,
    check_bundle_for_asset_load_revision_changes_max_json,
    check_bundle_for_asset_load_stale_manifest_max_json,
    check_bundle_for_asset_load_unsupported_file_max_json,
    check_bundle_for_asset_load_unsupported_url_max_json,
    check_bundle_for_asset_reload_active_backend_json,
    check_bundle_for_asset_reload_configured_backend_json,
    check_bundle_for_asset_reload_epoch_min_json,
    check_bundle_for_asset_reload_fallback_reason_json,
    check_bundle_for_bundled_font_baseline_source_json,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use retained_vlist_gates::{
    check_bundle_for_retained_vlist_attach_detach_max_json,
    check_bundle_for_retained_vlist_keep_alive_budget_json,
    check_bundle_for_retained_vlist_keep_alive_reuse_min_json,
    check_bundle_for_retained_vlist_reconcile_no_notify_min_json,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use stale::{
    check_bundle_for_semantics_changed_repainted_json, check_bundle_for_stale_paint_json,
    check_bundle_for_stale_scene_json, scan_semantics_changed_repainted_json,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use view_cache_gates::check_bundle_for_view_cache_reuse_min_json;
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use vlist::{
    check_bundle_for_vlist_policy_key_stable_json,
    check_bundle_for_vlist_visible_range_refreshes_max_json,
    check_bundle_for_vlist_visible_range_refreshes_min_json,
    check_bundle_for_vlist_window_shifts_explainable_json,
    check_bundle_for_vlist_window_shifts_have_prepaint_actions_json,
    check_bundle_for_vlist_window_shifts_kind_max_json,
    check_bundle_for_vlist_window_shifts_non_retained_max_json,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use wheel_scroll::{
    check_bundle_for_wheel_scroll_hit_changes_json, check_bundle_for_wheel_scroll_json,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use windowed_rows::{
    check_bundle_for_windowed_rows_offset_changes_min_json,
    check_bundle_for_windowed_rows_visible_start_changes_repainted_json,
};

use bundle_stats_snapshot::{
    SemanticsIndex, format_text_prepare_reasons, snapshot_command_availability_hotspots,
    snapshot_global_change_hotspots, snapshot_global_change_unobserved,
    snapshot_layout_engine_solves, snapshot_layout_hotspots, snapshot_layout_request_build_roots,
    snapshot_lookup_semantics, snapshot_model_change_hotspots, snapshot_model_change_unobserved,
    snapshot_paint_text_prepare_hotspots, snapshot_paint_widget_hotspots,
    snapshot_scroll_layout_profiles, snapshot_widget_measure_hotspots,
};

fn bundle_artifact_alias_pair(bundle_path: &Path) -> (String, String) {
    crate::artifact_alias::bundle_artifact_alias_pair(bundle_path)
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

include!("stats/bundle_stats_report.inc.rs");

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BundleStatsOptions {
    pub(super) warmup_frames: u64,
}

#[derive(Debug, Clone, Default)]
struct BundleStatsFrameFilter {
    script_capture_frames: HashSet<(Option<u64>, u64)>,
}

impl BundleStatsFrameFilter {
    fn insert_script_capture_frame(&mut self, window: Option<u64>, frame_id: u64) {
        self.script_capture_frames.insert((window, frame_id));
    }

    fn contains_script_capture_frame(&self, window: u64, frame_id: u64) -> bool {
        self.script_capture_frames
            .contains(&(Some(window), frame_id))
            || self.script_capture_frames.contains(&(None, frame_id))
    }
}

fn script_result_event_is_capture_frame(e: &serde_json::Value) -> bool {
    let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let note = e.get("note").and_then(|v| v.as_str()).unwrap_or("");
    matches!(
        kind,
        "capture_bundle" | "bundle_dump_requested" | "bundle_dumped"
    ) || note == "capture_bundle"
}

fn script_result_frame_filter(script_result: &serde_json::Value) -> BundleStatsFrameFilter {
    let mut filter = BundleStatsFrameFilter::default();
    let default_window = script_result.get("window").and_then(|v| v.as_u64());
    let events = script_result
        .get("evidence")
        .and_then(|v| v.get("event_log"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    for e in events {
        if !script_result_event_is_capture_frame(e) {
            continue;
        }
        let Some(frame_id) = e.get("frame_id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let window = e.get("window").and_then(|v| v.as_u64()).or(default_window);
        filter.insert_script_capture_frame(window, frame_id);
    }

    filter
}

fn try_read_script_result_json_for_bundle(bundle_path: &Path) -> Option<serde_json::Value> {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let direct = dir.join("script.result.json");
    let from_parent = if dir.file_name().and_then(|s| s.to_str()) == Some("_root") {
        dir.parent().map(|p| p.join("script.result.json"))
    } else {
        None
    };

    let v = if direct.is_file() {
        crate::util::read_json_value(&direct)?
    } else if let Some(from_parent) = from_parent
        && from_parent.is_file()
    {
        crate::util::read_json_value(&from_parent)?
    } else {
        return None;
    };

    if v.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        return None;
    }
    v.get("stage")?;
    Some(v)
}

fn bundle_stats_frame_filter_from_sidecars(bundle_path: &Path) -> BundleStatsFrameFilter {
    let Some(script_result) = try_read_script_result_json_for_bundle(bundle_path) else {
        return BundleStatsFrameFilter::default();
    };
    script_result_frame_filter(&script_result)
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

const TYPICAL_TAIL_DIFF_HIGHLIGHT_KEYS: &[&str] = &[
    "p95.total_time_us",
    "max.total_time_us",
    "p95.layout_time_us",
    "max.layout_time_us",
    "p95.layout_engine_solve_time_us",
    "max.layout_engine_solve_time_us",
    "p95.prepaint_time_us",
    "max.prepaint_time_us",
    "p95.paint_time_us",
    "max.paint_time_us",
    "p95.dispatch_time_us",
    "p95.hit_test_time_us",
    "pointer_move.max_dispatch_time_us",
    "pointer_move.max_hit_test_time_us",
];

fn diff_delta_json(d: &BundleStatsDiffDelta) -> serde_json::Value {
    serde_json::json!({
        "key": d.key,
        "a": d.a,
        "b": d.b,
        "delta_us": d.delta_us(),
        "delta_pct": d.delta_pct(),
        "abs_delta_us": d.abs_delta_us(),
    })
}

impl BundleStatsDiffReport {
    fn typical_tail_highlights(&self) -> Vec<&BundleStatsDiffDelta> {
        TYPICAL_TAIL_DIFF_HIGHLIGHT_KEYS
            .iter()
            .filter_map(|key| self.deltas.iter().find(|delta| delta.key == *key))
            .collect()
    }

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

        let highlights = self.typical_tail_highlights();
        if !highlights.is_empty() {
            println!("typical/tail highlights:");
            for d in highlights {
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
        let deltas = self.deltas.iter().map(diff_delta_json).collect::<Vec<_>>();
        let typical_tail_highlights = self
            .typical_tail_highlights()
            .into_iter()
            .map(diff_delta_json)
            .collect::<Vec<_>>();
        serde_json::json!({
            "schema_version": crate::perf_schema::PERF_STATS_SCHEMA_VERSION,
            "kind": crate::perf_schema::PERF_STATS_DIFF_KIND,
            "schema_policy": crate::perf_schema::schema_policy_json(),
            "bundle_a": self.a_path.display().to_string(),
            "bundle_b": self.b_path.display().to_string(),
            "sort": self.sort.as_str(),
            "warmup_frames": self.warmup_frames,
            "top": self.top,
            "highlights": {
                "typical_tail": typical_tail_highlights,
            },
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
            key: "p95.total_time_us",
            a: a.p95_total_time_us,
            b: b.p95_total_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.layout_time_us",
            a: a.p95_layout_time_us,
            b: b.p95_layout_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.layout_request_build_roots_time_us",
            a: a.p95_layout_request_build_roots_time_us,
            b: b.p95_layout_request_build_roots_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.layout_roots_time_us",
            a: a.p95_layout_roots_time_us,
            b: b.p95_layout_roots_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.layout_engine_solve_time_us",
            a: a.p95_layout_engine_solve_time_us,
            b: b.p95_layout_engine_solve_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.prepaint_time_us",
            a: a.p95_prepaint_time_us,
            b: b.p95_prepaint_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.paint_time_us",
            a: a.p95_paint_time_us,
            b: b.p95_paint_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.dispatch_time_us",
            a: a.p95_dispatch_time_us,
            b: b.p95_dispatch_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.hit_test_time_us",
            a: a.p95_hit_test_time_us,
            b: b.p95_hit_test_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.paint_widget_time_us",
            a: a.p95_paint_widget_time_us,
            b: b.p95_paint_widget_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.paint_text_prepare_time_us",
            a: a.p95_paint_text_prepare_time_us,
            b: b.p95_paint_text_prepare_time_us,
        },
        BundleStatsDiffDelta {
            key: "p95.renderer_encode_scene_us",
            a: a.p95_renderer_encode_scene_us,
            b: b.p95_renderer_encode_scene_us,
        },
        BundleStatsDiffDelta {
            key: "p95.renderer_upload_us",
            a: a.p95_renderer_upload_us,
            b: b.p95_renderer_upload_us,
        },
        BundleStatsDiffDelta {
            key: "p95.renderer_record_passes_us",
            a: a.p95_renderer_record_passes_us,
            b: b.p95_renderer_record_passes_us,
        },
        BundleStatsDiffDelta {
            key: "p95.renderer_encoder_finish_us",
            a: a.p95_renderer_encoder_finish_us,
            b: b.p95_renderer_encoder_finish_us,
        },
        BundleStatsDiffDelta {
            key: "p95.renderer_prepare_text_us",
            a: a.p95_renderer_prepare_text_us,
            b: b.p95_renderer_prepare_text_us,
        },
        BundleStatsDiffDelta {
            key: "p95.renderer_prepare_svg_us",
            a: a.p95_renderer_prepare_svg_us,
            b: b.p95_renderer_prepare_svg_us,
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

pub(super) fn bundle_stats_from_path(
    bundle_path: &Path,
    top: usize,
    sort: BundleStatsSort,
    opts: BundleStatsOptions,
) -> Result<BundleStatsReport, String> {
    const MAX_MATERIALIZED_BUNDLE_BYTES: u64 = 64 * 1024 * 1024;
    let frame_filter = bundle_stats_frame_filter_from_sidecars(bundle_path);
    let file_len = std::fs::metadata(bundle_path)
        .map(|m| m.len())
        .unwrap_or(MAX_MATERIALIZED_BUNDLE_BYTES + 1);
    if file_len > MAX_MATERIALIZED_BUNDLE_BYTES {
        return bundle_stats_from_frames_index(
            bundle_path,
            top,
            sort,
            opts.warmup_frames,
            &frame_filter,
        )
        .map_err(|err| {
            format!(
                "{err}\n\
  bundle: {} ({} MiB)\n\
  hint: prefer schema2 + sidecars + lite triage:\n\
    - fretboard-dev diag doctor --fix-schema2 <bundle_dir> --warmup-frames {}\n\
    - fretboard-dev diag index <bundle_dir> --warmup-frames {}\n\
    - fretboard-dev diag triage --lite <bundle_dir> --warmup-frames {}",
                bundle_path.display(),
                file_len / (1024 * 1024),
                opts.warmup_frames,
                opts.warmup_frames,
                opts.warmup_frames
            )
        });
    }
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    bundle_stats_from_json_with_options_and_filter(&bundle, top, sort, opts, &frame_filter)
}

include!("stats/bundle_stats_compute.inc.rs");

fn bundle_stats_from_frames_index(
    bundle_path: &Path,
    top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
    frame_filter: &BundleStatsFrameFilter,
) -> Result<BundleStatsReport, String> {
    if !matches!(sort, BundleStatsSort::Invalidation | BundleStatsSort::Time) {
        return Err(format!(
            "bundle artifact is too large for full `diag stats`, and stats-lite currently supports `--sort invalidation|time` only (got: {})",
            sort.as_str()
        ));
    }

    fn col_index(columns: &[serde_json::Value], name: &str) -> Option<usize> {
        columns
            .iter()
            .position(|c| c.as_str().is_some_and(|s| s == name))
    }

    fn row_u64(row: &[serde_json::Value], idx: Option<usize>) -> Option<u64> {
        let idx = idx?;
        row.get(idx)?.as_u64()
    }

    fn p50_p95(mut values: Vec<u64>) -> (u64, u64) {
        if values.is_empty() {
            return (0, 0);
        }
        values.sort_unstable();
        let n = values.len();
        let p50 = values[(n - 1) * 50 / 100];
        let p95 = values[(n - 1) * 95 / 100];
        (p50, p95)
    }

    let frames_index_path =
        crate::frames_index::ensure_frames_index_json(bundle_path, warmup_frames)?;
    let Some(frames_index) =
        crate::frames_index::read_frames_index_json_v1(&frames_index_path, warmup_frames)
    else {
        return Err(format!(
            "frames.index.json is missing or invalid (warmup_frames={warmup_frames})"
        ));
    };

    let columns = frames_index
        .get("columns")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing columns".to_string())?;
    let windows = frames_index
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing windows".to_string())?;

    let idx_frame_id = col_index(columns, "frame_id");
    let idx_snapshot_seq = col_index(columns, "window_snapshot_seq");
    let idx_ts = col_index(columns, "timestamp_unix_ms");
    let idx_total = col_index(columns, "total_time_us");
    let idx_layout = col_index(columns, "layout_time_us");
    let idx_prepaint = col_index(columns, "prepaint_time_us");
    let idx_paint = col_index(columns, "paint_time_us");
    let idx_inv_calls = col_index(columns, "invalidation_walk_calls");
    let idx_inv_nodes = col_index(columns, "invalidation_walk_nodes");

    let mut out = BundleStatsReport {
        sort,
        warmup_frames,
        derived_from_frames_index: true,
        source_bundle_schema_version: crate::compat::bundle::sniff_bundle_schema_version(
            bundle_path,
        )
        .ok()
        .flatten()
        .unwrap_or(0)
        .min(u32::MAX as u64) as u32,
        windows: windows.len().min(u32::MAX as usize) as u32,
        ..Default::default()
    };

    let mut rows: Vec<BundleStatsSnapshotRow> = Vec::new();

    let mut total_values: Vec<u64> = Vec::new();
    let mut layout_values: Vec<u64> = Vec::new();
    let mut prepaint_values: Vec<u64> = Vec::new();
    let mut paint_values: Vec<u64> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snapshots_total = w
            .get("snapshots_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        out.snapshots = out
            .snapshots
            .saturating_add(snapshots_total.min(u32::MAX as u64) as u32);

        let rows_arr = w
            .get("rows")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v.as_slice());
        let skipped = snapshots_total.saturating_sub(rows_arr.len() as u64);
        out.snapshots_skipped_warmup = out
            .snapshots_skipped_warmup
            .saturating_add(skipped.min(u32::MAX as u64) as u32);

        for row in rows_arr {
            let Some(row) = row.as_array() else {
                continue;
            };

            let frame_id = row_u64(row, idx_frame_id).unwrap_or(0);
            if frame_filter.contains_script_capture_frame(window_id, frame_id) {
                out.snapshots_skipped_script_capture =
                    out.snapshots_skipped_script_capture.saturating_add(1);
                continue;
            }
            out.snapshots_considered = out.snapshots_considered.saturating_add(1);

            let snapshot_seq = row_u64(row, idx_snapshot_seq).unwrap_or(0);
            let ts = row_u64(row, idx_ts);

            let total = row_u64(row, idx_total).unwrap_or(0);
            let layout = row_u64(row, idx_layout).unwrap_or(0);
            let prepaint = row_u64(row, idx_prepaint).unwrap_or(0);
            let paint = row_u64(row, idx_paint).unwrap_or(0);
            let inv_calls_u64 = row_u64(row, idx_inv_calls).unwrap_or(0);
            let inv_nodes_u64 = row_u64(row, idx_inv_nodes).unwrap_or(0);
            let inv_calls_u32 = inv_calls_u64.min(u32::MAX as u64) as u32;
            let inv_nodes_u32 = inv_nodes_u64.min(u32::MAX as u64) as u32;

            out.sum_total_time_us = out.sum_total_time_us.saturating_add(total);
            out.sum_layout_time_us = out.sum_layout_time_us.saturating_add(layout);
            out.sum_prepaint_time_us = out.sum_prepaint_time_us.saturating_add(prepaint);
            out.sum_paint_time_us = out.sum_paint_time_us.saturating_add(paint);
            out.sum_invalidation_walk_calls = out
                .sum_invalidation_walk_calls
                .saturating_add(inv_calls_u64);
            out.sum_invalidation_walk_nodes = out
                .sum_invalidation_walk_nodes
                .saturating_add(inv_nodes_u64);

            out.max_total_time_us = out.max_total_time_us.max(total);
            out.max_layout_time_us = out.max_layout_time_us.max(layout);
            out.max_prepaint_time_us = out.max_prepaint_time_us.max(prepaint);
            out.max_paint_time_us = out.max_paint_time_us.max(paint);
            out.max_invalidation_walk_calls = out.max_invalidation_walk_calls.max(inv_calls_u32);
            out.max_invalidation_walk_nodes = out.max_invalidation_walk_nodes.max(inv_nodes_u32);

            total_values.push(total);
            layout_values.push(layout);
            prepaint_values.push(prepaint);
            paint_values.push(paint);

            rows.push(BundleStatsSnapshotRow {
                window: window_id,
                tick_id: snapshot_seq,
                frame_id,
                timestamp_unix_ms: ts,
                total_time_us: total,
                layout_time_us: layout,
                prepaint_time_us: prepaint,
                paint_time_us: paint,
                invalidation_walk_calls: inv_calls_u32,
                invalidation_walk_nodes: inv_nodes_u32,
                ..Default::default()
            });
        }
    }

    (out.p50_total_time_us, out.p95_total_time_us) = p50_p95(total_values);
    (out.p50_layout_time_us, out.p95_layout_time_us) = p50_p95(layout_values);
    (out.p50_prepaint_time_us, out.p95_prepaint_time_us) = p50_p95(prepaint_values);
    (out.p50_paint_time_us, out.p95_paint_time_us) = p50_p95(paint_values);

    match sort {
        BundleStatsSort::Invalidation => {
            rows.sort_by(|a, b| {
                b.invalidation_walk_nodes
                    .cmp(&a.invalidation_walk_nodes)
                    .then_with(|| b.invalidation_walk_calls.cmp(&a.invalidation_walk_calls))
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
        _ => {}
    }

    out.top = rows.into_iter().take(top).collect();
    Ok(out)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_stats_diff_dir(prefix: &str) -> PathBuf {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("fret-diag-{prefix}-{}-{now}", std::process::id()))
    }

    fn write_stats_diff_bundle(path: &Path, layout_values: &[u64]) {
        let snapshots = layout_values
            .iter()
            .enumerate()
            .map(|(idx, layout_time_us)| {
                serde_json::json!({
                    "frame_id": idx as u64,
                    "debug": {
                        "stats": {
                            "layout_time_us": layout_time_us,
                            "layout_engine_solve_time_us": *layout_time_us / 2,
                            "prepaint_time_us": 0,
                            "paint_time_us": 0
                        }
                    }
                })
            })
            .collect::<Vec<_>>();
        let bundle = serde_json::json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": snapshots
            }]
        });
        std::fs::create_dir_all(path.parent().expect("bundle path should have parent"))
            .expect("create bundle dir");
        std::fs::write(path, serde_json::to_vec(&bundle).expect("serialize bundle"))
            .expect("write bundle");
    }

    fn write_script_result_for_capture_frame(path: &Path, window: u64, frame_id: u64) {
        let script_result = serde_json::json!({
            "schema_version": 1,
            "stage": "passed",
            "window": window,
            "evidence": {
                "event_log": [
                    {
                        "kind": "step_start",
                        "note": "capture_bundle",
                        "window": window,
                        "frame_id": frame_id,
                        "step_index": 3
                    },
                    {
                        "kind": "bundle_dump_requested",
                        "window": window,
                        "frame_id": frame_id,
                        "step_index": 3
                    }
                ]
            }
        });
        std::fs::write(
            path,
            serde_json::to_vec(&script_result).expect("serialize script result"),
        )
        .expect("write script result");
    }

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
    fn stats_diff_includes_typical_and_tail_highlights() {
        let dir = temp_stats_diff_dir("stats-diff");
        let a_path = dir.join("a").join("bundle.schema2.json");
        let b_path = dir.join("b").join("bundle.schema2.json");
        let a_values = vec![100; 21];
        let mut b_values = vec![110; 20];
        b_values.push(400);
        write_stats_diff_bundle(&a_path, &a_values);
        write_stats_diff_bundle(&b_path, &b_values);

        let report = bundle_stats_diff_from_paths(
            &a_path,
            &b_path,
            20,
            BundleStatsSort::Time,
            BundleStatsOptions::default(),
        )
        .expect("diff report");
        let json = report.to_json();
        let deltas = json
            .get("deltas")
            .and_then(|v| v.as_array())
            .expect("deltas");
        let delta_for = |key: &str| {
            deltas
                .iter()
                .find(|delta| delta.get("key").and_then(|v| v.as_str()) == Some(key))
                .unwrap_or_else(|| panic!("missing diff delta: {key}"))
        };

        assert_eq!(
            delta_for("p95.total_time_us")
                .get("delta_us")
                .and_then(|v| v.as_i64()),
            Some(10)
        );
        assert_eq!(
            delta_for("max.total_time_us")
                .get("delta_us")
                .and_then(|v| v.as_i64()),
            Some(300)
        );

        let highlight_keys = json
            .pointer("/highlights/typical_tail")
            .and_then(|v| v.as_array())
            .expect("typical/tail highlights")
            .iter()
            .filter_map(|delta| delta.get("key").and_then(|v| v.as_str()))
            .collect::<std::collections::BTreeSet<_>>();
        assert!(highlight_keys.contains("p95.total_time_us"));
        assert!(highlight_keys.contains("max.total_time_us"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stats_diff_json_is_versioned_and_additive_only() {
        let report = BundleStatsDiffReport {
            a_path: PathBuf::from("a/bundle.schema2.json"),
            b_path: PathBuf::from("b/bundle.schema2.json"),
            sort: BundleStatsSort::Time,
            warmup_frames: 3,
            top: 5,
            deltas: Vec::new(),
        };

        let json = report.to_json();
        assert_eq!(
            json.get("kind").and_then(|v| v.as_str()),
            Some(crate::perf_schema::PERF_STATS_DIFF_KIND)
        );
        assert_eq!(
            json.get("schema_version").and_then(|v| v.as_u64()),
            Some(crate::perf_schema::PERF_STATS_SCHEMA_VERSION as u64)
        );
        assert_eq!(
            json.pointer("/schema_policy/compatibility")
                .and_then(|v| v.as_str()),
            Some("additive_only")
        );
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
        assert_eq!(
            json.get("kind").and_then(|v| v.as_str()),
            Some(crate::perf_schema::PERF_STATS_KIND)
        );
        assert_eq!(
            json.get("schema_version").and_then(|v| v.as_u64()),
            Some(crate::perf_schema::PERF_STATS_SCHEMA_VERSION as u64)
        );
        assert_eq!(
            json.pointer("/schema_policy/compatibility")
                .and_then(|v| v.as_str()),
            Some("additive_only")
        );
        let registered_keys = json
            .get("registered_perf_keys")
            .and_then(|v| v.as_array())
            .expect("registered perf keys");
        assert!(registered_keys.iter().any(|key| {
            key.get("key").and_then(|v| v.as_str()) == Some("renderer_encode_scene_us")
                && key.get("unit").and_then(|v| v.as_str()) == Some("us")
        }));
        assert!(registered_keys.iter().any(|key| {
            key.get("key").and_then(|v| v.as_str()) == Some("pointer_move.max_dispatch_time_us")
                && key.get("scope").and_then(|v| v.as_str()) == Some("pointer_move")
        }));
        assert!(json.get("avg").is_some());
        assert!(json.get("budget_pct").is_some());
    }

    #[test]
    fn script_result_frame_filter_marks_capture_bundle_frames() {
        let script_result = serde_json::json!({
            "schema_version": 1,
            "stage": "passed",
            "window": 7,
            "evidence": {
                "event_log": [
                    { "kind": "step_start", "note": "set_text_value", "frame_id": 10 },
                    { "kind": "step_start", "note": "capture_bundle", "frame_id": 11 },
                    { "kind": "bundle_dump_requested", "frame_id": 12, "window": 8 },
                    { "kind": "bundle_dumped" }
                ]
            }
        });

        let filter = script_result_frame_filter(&script_result);

        assert!(!filter.contains_script_capture_frame(7, 10));
        assert!(filter.contains_script_capture_frame(7, 11));
        assert!(filter.contains_script_capture_frame(8, 12));
    }

    #[test]
    fn bundle_stats_skips_script_capture_frames_for_top_and_percentiles() {
        let bundle = serde_json::json!({
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 10,
                        "tick_id": 10,
                        "debug": { "stats": { "total_time_us": 100, "layout_time_us": 40, "prepaint_time_us": 10, "paint_time_us": 50 } }
                    },
                    {
                        "frame_id": 11,
                        "tick_id": 11,
                        "debug": { "stats": { "total_time_us": 200, "layout_time_us": 80, "prepaint_time_us": 20, "paint_time_us": 100 } }
                    },
                    {
                        "frame_id": 12,
                        "tick_id": 12,
                        "debug": { "stats": { "total_time_us": 10_000, "layout_time_us": 4_000, "prepaint_time_us": 1_000, "paint_time_us": 5_000 } }
                    }
                ]
            }]
        });
        let mut filter = BundleStatsFrameFilter::default();
        filter.insert_script_capture_frame(Some(1), 12);

        let report = bundle_stats_from_json_with_options_and_filter(
            &bundle,
            3,
            BundleStatsSort::Time,
            BundleStatsOptions { warmup_frames: 0 },
            &filter,
        )
        .expect("bundle stats");

        assert_eq!(report.snapshots, 3);
        assert_eq!(report.snapshots_considered, 2);
        assert_eq!(report.snapshots_skipped_script_capture, 1);
        assert_eq!(report.top.first().map(|row| row.frame_id), Some(11));
        assert_eq!(report.max_total_time_us, 200);
        assert_eq!(report.p95_total_time_us, 200);
        assert_eq!(
            report
                .to_json()
                .get("snapshots_skipped_script_capture")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
    }

    #[test]
    fn bundle_stats_from_path_reads_script_result_and_skips_capture_frames() {
        let dir = temp_stats_diff_dir("stats-script-capture-sidecar");
        let bundle_path = dir.join("bundle.schema2.json");
        let bundle = serde_json::json!({
            "schema_version": 2,
            "windows": [{
                "window": 99,
                "snapshots": [
                    {
                        "frame_id": 41,
                        "tick_id": 41,
                        "debug": { "stats": { "total_time_us": 300, "layout_time_us": 100, "prepaint_time_us": 50, "paint_time_us": 150 } }
                    },
                    {
                        "frame_id": 42,
                        "tick_id": 42,
                        "debug": { "stats": { "total_time_us": 30_000, "layout_time_us": 10_000, "prepaint_time_us": 5_000, "paint_time_us": 15_000 } }
                    }
                ]
            }]
        });
        std::fs::create_dir_all(&dir).expect("create bundle dir");
        std::fs::write(
            &bundle_path,
            serde_json::to_vec(&bundle).expect("serialize bundle"),
        )
        .expect("write bundle");
        write_script_result_for_capture_frame(&dir.join("script.result.json"), 99, 42);

        let report = bundle_stats_from_path(
            &bundle_path,
            2,
            BundleStatsSort::Time,
            BundleStatsOptions { warmup_frames: 0 },
        )
        .expect("bundle stats");

        assert_eq!(report.snapshots_considered, 1);
        assert_eq!(report.snapshots_skipped_script_capture, 1);
        assert_eq!(report.top.first().map(|row| row.frame_id), Some(41));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bundle_stats_from_frames_index_skips_script_capture_frames() {
        let dir = temp_stats_diff_dir("stats-script-capture-frames-index");
        let bundle_path = dir.join("bundle.schema2.json");
        std::fs::create_dir_all(&dir).expect("create bundle dir");
        std::fs::write(&bundle_path, br#"{"schema_version":2,"windows":[]}"#)
            .expect("write bundle");
        std::fs::write(
            dir.join("frames.index.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind": "frames_index",
                "schema_version": 1,
                "warmup_frames": 0,
                "bundle": bundle_path.display().to_string(),
                "features": [
                    "window_aggregates.v1",
                    "window_aggregates.overlay_synthesis.v1",
                    "window_aggregates.view_cache_reuse_streak.v1",
                    "window_aggregates.idle_no_paint.v1"
                ],
                "columns": [
                    "frame_id",
                    "window_snapshot_seq",
                    "timestamp_unix_ms",
                    "total_time_us",
                    "layout_time_us",
                    "prepaint_time_us",
                    "paint_time_us",
                    "invalidation_walk_calls",
                    "invalidation_walk_nodes"
                ],
                "windows": [{
                    "window": 99,
                    "snapshots_total": 2,
                    "rows": [
                        [41, 1, 1000, 300, 100, 50, 150, 1, 10],
                        [42, 2, 2000, 30000, 10000, 5000, 15000, 2, 20]
                    ]
                }]
            }))
            .expect("serialize frames index"),
        )
        .expect("write frames index");
        let mut filter = BundleStatsFrameFilter::default();
        filter.insert_script_capture_frame(Some(99), 42);

        let report =
            bundle_stats_from_frames_index(&bundle_path, 2, BundleStatsSort::Time, 0, &filter)
                .expect("stats-lite bundle stats");

        assert!(report.derived_from_frames_index());
        assert_eq!(report.snapshots_considered, 1);
        assert_eq!(report.snapshots_skipped_script_capture, 1);
        assert_eq!(report.max_total_time_us, 300);
        assert_eq!(report.top.first().map(|row| row.frame_id), Some(41));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bundle_stats_reads_element_children_vec_pool_grow_events() {
        let bundle = serde_json::json!({
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "tick_id": 11,
                    "timestamp_unix_ms": 12,
                    "debug": {
                        "stats": {
                            "total_time_us": 100,
                            "layout_time_us": 20,
                            "paint_time_us": 30,
                            "element_children_vec_pool_reuses": 4,
                            "element_children_vec_pool_misses": 5,
                            "element_children_vec_pool_grow_events": 6
                        }
                    }
                }]
            }]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            1,
            BundleStatsSort::Time,
            BundleStatsOptions { warmup_frames: 0 },
        )
        .expect("bundle stats");

        let top = report.top.first().expect("top row");
        assert_eq!(top.element_children_vec_pool_reuses, 4);
        assert_eq!(top.element_children_vec_pool_misses, 5);
        assert_eq!(top.element_children_vec_pool_grow_events, 6);
    }

    #[test]
    fn bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot() {
        let bundle = serde_json::json!({
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "tick_id": 11,
                    "timestamp_unix_ms": 12,
                    "debug": {
                        "stats": {
                            "total_time_us": 1000,
                            "layout_time_us": 100,
                            "prepaint_time_us": 50,
                            "paint_time_us": 850,
                            "paint_widget_time_us": 700
                        }
                    },
                    "app_snapshot": {
                        "kind": "fret_ui_gallery",
                        "selected_page": "code_editor_torture",
                        "code_editor": {
                            "torture": {
                                "paint_perf": {
                                    "schema_version": 14,
                                    "frame_seq": 7,
                                    "visible_start": 20,
                                    "visible_end": 30,
                                    "visible_rows": 10,
                                    "cache_base_entries": 128,
                                    "cache_frame_min_entries": 32,
                                    "cache_effective_entries": 160,
                                    "rows_painted": 10,
                                    "rows_drew_rich": 3,
                                    "rows_scene_replayed": 8,
                                    "rows_scene_prepaint_planned": 6,
                                    "rows_scene_prepaint_plan_used": 5,
                                    "rows_scene_stored": 2,
                                    "rows_scene_stored_at_visible_start": 1,
                                    "rows_scene_stored_at_visible_end": 1,
                                    "row_scene_ops_stored": 19,
                                    "rows_scene_prepaint_edge_stored": 1,
                                    "row_scene_prepaint_edge_ops_stored": 7,
                                    "rows_scene_prepaint_candidates": 10,
                                    "rows_scene_prepaint_skip_no_cache": 2,
                                    "rows_scene_prepaint_skip_unsupported_key": 1,
                                    "rows_scene_prepaint_skip_preedit": 0,
                                    "rows_scene_prepaint_skip_syntax_empty": 1,
                                    "rows_scene_prepaint_skip_key_mismatch": 1,
                                    "rows_scene_prepaint_plan_cache_hits": 3,
                                    "rows_scene_prepaint_plan_cache_rejects": 1,
                                    "rows_scene_fast_miss_no_entry": 2,
                                    "rows_scene_fast_miss_key_mismatch": 1,
                                    "rows_scene_full_miss_no_entry": 1,
                                    "rows_scene_full_miss_key_mismatch": 1,
                                    "quads_selection": 4,
                                    "quads_caret": 1,
                                    "syntax_rows_stored": 1,
                                    "us_total": 500,
                                    "us_row_content_resolve": 40,
                                    "ns_row_content_resolve": 45900,
                                    "us_row_text": 12,
                                    "us_baseline_measure": 3,
                                    "us_row_rich_cache_compare": 8,
                                    "us_row_geom_key": 14,
                                    "us_rich_materialize": 30,
                                    "us_text_draw": 120,
                                    "us_row_scene_key": 9,
                                    "us_row_scene_fast_probe": 11,
                                    "us_row_scene_full_probe": 13,
                                    "us_row_scene_fast_key_compare": 2,
                                    "ns_row_scene_fast_key_compare": 2900,
                                    "us_row_scene_full_key_compare": 1,
                                    "us_row_scene_replay_setup": 21,
                                    "ns_row_scene_replay_setup": 21300,
                                    "us_row_scene_replay_touch": 5,
                                    "us_row_scene_replay_ops": 25,
                                    "us_row_scene_prepaint_plan": 7,
                                    "us_row_scene_prepaint_probe": 4,
                                    "ns_row_scene_prepaint_key_compare": 6100,
                                    "us_row_scene_capture_ops": 70,
                                    "us_row_scene_store": 20,
                                    "us_row_scene_prepaint_edge_store": 4,
                                    "us_row_scene_fast_path": 15,
                                    "us_row_scene_full_path": 17,
                                    "us_syntax_spans": 60,
                                    "us_syntax_slice": 6,
                                    "us_syntax_highlight": 50,
                                    "us_syntax_distribute": 4,
                                    "us_syntax_store": 10,
                                    "surface_rows_iterated": 10,
                                    "surface_rows_with_rect": 10,
                                    "us_windowed_surface_paint_callback": 620,
                                    "ns_windowed_surface_paint_callback": 621500,
                                    "us_windowed_surface_frame_lookup": 2,
                                    "us_windowed_surface_hook": 30,
                                    "us_windowed_surface_row_loop": 560,
                                    "us_windowed_surface_row_rect": 4,
                                    "us_windowed_surface_row_paint": 530,
                                    "us_windowed_surface_non_row": 90,
                                    "us_windowed_surface_row_callback_gap": 30,
                                    "us_torture_autoscroll": 18,
                                    "ns_torture_autoscroll": 19000,
                                    "us_torture_overlay": 9,
                                    "ns_torture_overlay": 11000
                                }
                            }
                        }
                    }
                }]
            }]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            1,
            BundleStatsSort::Time,
            BundleStatsOptions { warmup_frames: 0 },
        )
        .expect("bundle stats");

        let top = report.top.first().expect("top row");
        let perf = top
            .code_editor_paint_perf
            .as_ref()
            .expect("code editor paint perf");
        assert_eq!(perf.rows_scene_replayed, 8);
        assert_eq!(perf.rows_scene_prepaint_planned, 6);
        assert_eq!(perf.rows_scene_prepaint_plan_used, 5);
        assert_eq!(perf.rows_scene_stored, 2);
        assert_eq!(perf.rows_scene_stored_at_visible_start, 1);
        assert_eq!(perf.rows_scene_stored_at_visible_end, 1);
        assert_eq!(perf.row_scene_ops_stored, 19);
        assert_eq!(perf.rows_scene_prepaint_edge_stored, 1);
        assert_eq!(perf.row_scene_prepaint_edge_ops_stored, 7);
        assert_eq!(perf.rows_scene_prepaint_candidates, 10);
        assert_eq!(perf.rows_scene_prepaint_skip_no_cache, 2);
        assert_eq!(perf.rows_scene_prepaint_skip_unsupported_key, 1);
        assert_eq!(perf.rows_scene_prepaint_skip_syntax_empty, 1);
        assert_eq!(perf.rows_scene_prepaint_skip_key_mismatch, 1);
        assert_eq!(perf.rows_scene_fast_miss_no_entry, 2);
        assert_eq!(perf.rows_scene_fast_miss_key_mismatch, 1);
        assert_eq!(perf.rows_scene_full_miss_no_entry, 1);
        assert_eq!(perf.rows_scene_full_miss_key_mismatch, 1);
        assert_eq!(perf.quads_selection, 4);
        assert_eq!(perf.quads_caret, 1);
        assert_eq!(perf.us_row_scene_replay_setup, 21);
        assert_eq!(perf.us_row_scene_replay_ops, 25);
        assert_eq!(perf.us_row_scene_prepaint_plan, 7);
        assert_eq!(perf.us_row_scene_prepaint_probe, 4);
        assert_eq!(perf.us_row_scene_prepaint_key_compare, 6);
        assert_eq!(perf.us_row_scene_prepaint_edge_store, 4);
        assert_eq!(perf.us_row_scene_capture_ops, 70);
        assert_eq!(perf.us_row_scene_store, 20);
        assert_eq!(perf.us_row_content_resolve, 45);
        assert_eq!(perf.us_row_text, 12);
        assert_eq!(perf.us_row_rich_cache_compare, 8);
        assert_eq!(perf.us_row_geom_key, 14);
        assert_eq!(perf.us_row_scene_fast_key_compare, 2);
        assert_eq!(perf.us_text_draw, 120);
        assert_eq!(perf.us_rich_materialize, 30);
        assert_eq!(perf.surface_rows_iterated, 10);
        assert_eq!(perf.surface_rows_with_rect, 10);
        assert_eq!(perf.us_windowed_surface_paint_callback, 621);
        assert_eq!(perf.us_windowed_surface_row_loop, 560);
        assert_eq!(perf.us_windowed_surface_row_paint, 530);
        assert_eq!(perf.us_windowed_surface_non_row, 90);
        assert_eq!(perf.us_windowed_surface_row_callback_gap, 30);
        assert_eq!(perf.us_torture_autoscroll, 19);
        assert_eq!(perf.us_torture_overlay, 11);

        let json = report.to_json();
        assert_eq!(
            json.pointer("/code_editor_paint_perf/frames")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/sum/us_row_scene_capture_ops")
                .and_then(|v| v.as_u64()),
            Some(70)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/sum/rows_scene_prepaint_planned")
                .and_then(|v| v.as_u64()),
            Some(6)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/sum/rows_scene_prepaint_skip_no_cache")
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/sum/rows_scene_full_miss_no_entry")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/sum/us_row_scene_prepaint_plan")
                .and_then(|v| v.as_u64()),
            Some(7)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/sum/us_row_scene_prepaint_probe")
                .and_then(|v| v.as_u64()),
            Some(4)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/sum/us_row_scene_prepaint_key_compare")
                .and_then(|v| v.as_u64()),
            Some(6)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/sum/us_row_scene_replay_setup")
                .and_then(|v| v.as_u64()),
            Some(21)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/sum/us_row_content_resolve")
                .and_then(|v| v.as_u64()),
            Some(45)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/p95/us_row_scene_replay_ops")
                .and_then(|v| v.as_u64()),
            Some(25)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/p95/us_row_scene_replay_setup")
                .and_then(|v| v.as_u64()),
            Some(21)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/p95/us_row_geom_key")
                .and_then(|v| v.as_u64()),
            Some(14)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/p95/us_windowed_surface_paint_callback")
                .and_then(|v| v.as_u64()),
            Some(621)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/p95/us_windowed_surface_row_callback_gap")
                .and_then(|v| v.as_u64()),
            Some(30)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/p95/us_torture_autoscroll")
                .and_then(|v| v.as_u64()),
            Some(19)
        );
        assert_eq!(
            json.pointer("/code_editor_paint_perf/p95/us_torture_overlay")
                .and_then(|v| v.as_u64()),
            Some(11)
        );
        assert_eq!(
            json.pointer("/top/0/code_editor_paint_perf/rows_scene_replayed")
                .and_then(|v| v.as_u64()),
            Some(8)
        );
        assert_eq!(
            json.pointer("/top/0/code_editor_paint_perf/rows_scene_prepaint_plan_used")
                .and_then(|v| v.as_u64()),
            Some(5)
        );
        assert_eq!(
            json.pointer("/top/0/code_editor_paint_perf/us_row_scene_prepaint_probe")
                .and_then(|v| v.as_u64()),
            Some(4)
        );
        assert_eq!(
            json.pointer("/top/0/code_editor_paint_perf/us_row_scene_prepaint_key_compare")
                .and_then(|v| v.as_u64()),
            Some(6)
        );
        assert_eq!(
            json.pointer("/top/0/code_editor_paint_perf/us_row_scene_replay_setup")
                .and_then(|v| v.as_u64()),
            Some(21)
        );
        assert_eq!(
            json.pointer("/top/0/code_editor_paint_perf/row_scene_ops_stored")
                .and_then(|v| v.as_u64()),
            Some(19)
        );
        assert_eq!(
            json.pointer("/top/0/code_editor_paint_perf/us_windowed_surface_row_paint")
                .and_then(|v| v.as_u64()),
            Some(530)
        );
        assert_eq!(
            json.pointer("/top/0/code_editor_paint_perf/us_torture_autoscroll")
                .and_then(|v| v.as_u64()),
            Some(19)
        );
        assert_eq!(
            json.pointer("/top/0/code_editor_paint_perf/us_torture_overlay")
                .and_then(|v| v.as_u64()),
            Some(11)
        );
    }

    #[test]
    fn bundle_stats_summarizes_canvas_paint_widget_hotspots() {
        let bundle = serde_json::json!({
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 10,
                        "tick_id": 10,
                        "debug": {
                            "stats": {
                                "total_time_us": 1000,
                                "layout_time_us": 50,
                                "prepaint_time_us": 10,
                                "paint_time_us": 900,
                                "paint_record_visual_bounds_time_us": 5,
                                "paint_record_visual_bounds_calls": 2,
                                "paint_cache_key_time_us": 15,
                                "paint_cache_hit_check_time_us": 1,
                                "paint_widget_time_us": 400,
                                "paint_observation_record_time_us": 7,
                                "paint_host_widget_observed_models_time_us": 10,
                                "paint_host_widget_observed_models_items": 1,
                                "paint_host_widget_observed_globals_time_us": 11,
                                "paint_host_widget_observed_globals_items": 2,
                                "paint_host_widget_observed_deps_calls": 4,
                                "paint_host_widget_observed_deps_empty_calls": 3,
                                "paint_host_widget_observed_models_non_empty_calls": 1,
                                "paint_host_widget_observed_globals_non_empty_calls": 1,
                                "paint_host_widget_instance_lookup_time_us": 12,
                                "paint_host_widget_instance_lookup_calls": 4
                            },
                            "paint_widget_hotspots": [
                                {
                                    "node": 1,
                                    "element": 11,
                                    "element_kind": "Canvas",
                                    "widget_type": "fret_ui::declarative::host_widget::ElementHostWidget",
                                    "paint_time_us": 300,
                                    "inclusive_time_us": 320,
                                    "exclusive_scene_ops_delta": 30,
                                    "inclusive_scene_ops_delta": 35
                                },
                                {
                                    "node": 2,
                                    "element": 12,
                                    "element_kind": "Flex",
                                    "widget_type": "fret_ui::declarative::host_widget::ElementHostWidget",
                                    "paint_time_us": 50,
                                    "inclusive_time_us": 90,
                                    "exclusive_scene_ops_delta": 0,
                                    "inclusive_scene_ops_delta": 3
                                }
                            ]
                        },
                        "app_snapshot": {
                            "code_editor": {
                                "torture": {
                                    "paint_perf": {
                                        "frame_seq": 1,
                                        "us_total": 100,
                                        "surface_rows_with_rect": 10,
                                        "us_windowed_surface_paint_callback": 250,
                                        "us_windowed_surface_hook": 20,
                                        "us_windowed_surface_row_paint": 180,
                                        "us_windowed_surface_non_row": 60,
                                        "us_windowed_surface_row_callback_gap": 80
                                    }
                                }
                            }
                        }
                    },
                    {
                        "frame_id": 20,
                        "tick_id": 20,
                        "debug": {
                            "stats": {
                                "total_time_us": 2000,
                                "layout_time_us": 50,
                                "prepaint_time_us": 10,
                                "paint_time_us": 1900,
                                "paint_record_visual_bounds_time_us": 6,
                                "paint_record_visual_bounds_calls": 3,
                                "paint_cache_key_time_us": 25,
                                "paint_cache_hit_check_time_us": 2,
                                "paint_widget_time_us": 700,
                                "paint_observation_record_time_us": 8,
                                "paint_host_widget_observed_models_time_us": 20,
                                "paint_host_widget_observed_models_items": 2,
                                "paint_host_widget_observed_globals_time_us": 21,
                                "paint_host_widget_observed_globals_items": 3,
                                "paint_host_widget_observed_deps_calls": 5,
                                "paint_host_widget_observed_deps_empty_calls": 3,
                                "paint_host_widget_observed_models_non_empty_calls": 2,
                                "paint_host_widget_observed_globals_non_empty_calls": 2,
                                "paint_host_widget_instance_lookup_time_us": 22,
                                "paint_host_widget_instance_lookup_calls": 5
                            },
                            "paint_widget_hotspots": [
                                {
                                    "node": 3,
                                    "element": 13,
                                    "element_kind": "Canvas",
                                    "widget_type": "fret_ui::declarative::host_widget::ElementHostWidget",
                                    "paint_time_us": 500,
                                    "inclusive_time_us": 530,
                                    "exclusive_scene_ops_delta": 50,
                                    "inclusive_scene_ops_delta": 55
                                },
                                {
                                    "node": 4,
                                    "element": 14,
                                    "element_kind": "Flex",
                                    "widget_type": "fret_ui::declarative::host_widget::ElementHostWidget",
                                    "paint_time_us": 120,
                                    "inclusive_time_us": 180,
                                    "exclusive_scene_ops_delta": 0,
                                    "inclusive_scene_ops_delta": 4
                                }
                            ]
                        },
                        "app_snapshot": {
                            "code_editor": {
                                "torture": {
                                    "paint_perf": {
                                        "frame_seq": 2,
                                        "us_total": 200,
                                        "surface_rows_with_rect": 10,
                                        "us_windowed_surface_paint_callback": 400,
                                        "us_windowed_surface_hook": 30,
                                        "us_windowed_surface_row_paint": 330,
                                        "us_windowed_surface_non_row": 60,
                                        "us_windowed_surface_row_callback_gap": 130
                                    }
                                }
                            }
                        }
                    },
                    {
                        "frame_id": 30,
                        "tick_id": 30,
                        "debug": {
                            "stats": {
                                "total_time_us": 3000,
                                "layout_time_us": 50,
                                "prepaint_time_us": 10,
                                "paint_time_us": 2900,
                                "paint_record_visual_bounds_time_us": 7,
                                "paint_record_visual_bounds_calls": 4,
                                "paint_cache_key_time_us": 35,
                                "paint_cache_hit_check_time_us": 3,
                                "paint_widget_time_us": 2600,
                                "paint_observation_record_time_us": 9,
                                "paint_host_widget_observed_models_time_us": 30,
                                "paint_host_widget_observed_models_items": 3,
                                "paint_host_widget_observed_globals_time_us": 31,
                                "paint_host_widget_observed_globals_items": 4,
                                "paint_host_widget_observed_deps_calls": 6,
                                "paint_host_widget_observed_deps_empty_calls": 3,
                                "paint_host_widget_observed_models_non_empty_calls": 2,
                                "paint_host_widget_observed_globals_non_empty_calls": 3,
                                "paint_host_widget_instance_lookup_time_us": 32,
                                "paint_host_widget_instance_lookup_calls": 6
                            },
                            "paint_widget_hotspots": [
                                {
                                    "node": 5,
                                    "element": 15,
                                    "element_kind": "Container",
                                    "widget_type": "fret_ui::declarative::host_widget::ElementHostWidget",
                                    "paint_time_us": 1000,
                                    "inclusive_time_us": 1100,
                                    "exclusive_scene_ops_delta": 2,
                                    "inclusive_scene_ops_delta": 8
                                },
                                {
                                    "node": 6,
                                    "element": 16,
                                    "element_kind": "Flex",
                                    "widget_type": "fret_ui::declarative::host_widget::ElementHostWidget",
                                    "paint_time_us": 900,
                                    "inclusive_time_us": 1000,
                                    "exclusive_scene_ops_delta": 0,
                                    "inclusive_scene_ops_delta": 6
                                },
                                {
                                    "node": 7,
                                    "element": 17,
                                    "element_kind": "Text",
                                    "widget_type": "fret_ui::declarative::host_widget::ElementHostWidget",
                                    "paint_time_us": 800,
                                    "inclusive_time_us": 850,
                                    "exclusive_scene_ops_delta": 1,
                                    "inclusive_scene_ops_delta": 1
                                },
                                {
                                    "node": 8,
                                    "element": 18,
                                    "element_kind": "Canvas",
                                    "widget_type": "fret_ui::declarative::host_widget::ElementHostWidget",
                                    "paint_time_us": 700,
                                    "inclusive_time_us": 730,
                                    "exclusive_scene_ops_delta": 70,
                                    "inclusive_scene_ops_delta": 75
                                }
                            ]
                        },
                        "app_snapshot": {
                            "code_editor": {
                                "torture": {
                                    "paint_perf": {
                                        "frame_seq": 3,
                                        "us_total": 300,
                                        "surface_rows_with_rect": 10,
                                        "us_windowed_surface_paint_callback": 600,
                                        "us_windowed_surface_hook": 30,
                                        "us_windowed_surface_row_paint": 530,
                                        "us_windowed_surface_non_row": 90,
                                        "us_windowed_surface_row_callback_gap": 230
                                    }
                                }
                            }
                        }
                    }
                ]
            }]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            3,
            BundleStatsSort::Time,
            BundleStatsOptions { warmup_frames: 0 },
        )
        .expect("bundle stats");

        let top = report.top.first().expect("top row");
        assert_eq!(top.frame_id, 30);
        assert_eq!(top.paint_widget_hotspots.len(), 3);
        assert!(
            top.paint_widget_hotspots
                .iter()
                .all(|h| h.element_kind.as_deref() != Some("Canvas"))
        );

        let json = report.to_json();
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/sampled_top_n_per_frame")
                .and_then(|v| v.as_u64()),
            Some(PAINT_WIDGET_HOTSPOT_SUMMARY_TOP_N as u64)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/frames_with_hotspots")
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/canvas/frames")
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/canvas/exclusive_us/p50")
                .and_then(|v| v.as_u64()),
            Some(500)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/canvas/exclusive_us/p95")
                .and_then(|v| v.as_u64()),
            Some(700)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/canvas/exclusive_scene_ops_delta/p95")
                .and_then(|v| v.as_u64()),
            Some(70)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/canvas/top/paint_time_us")
                .and_then(|v| v.as_u64()),
            Some(700)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/non_canvas/exclusive_us/p95")
                .and_then(|v| v.as_u64()),
            Some(1000)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/non_canvas/sampled_sum_exclusive_us/p95")
                .and_then(|v| v.as_u64()),
            Some(2700)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/gap_to_code_editor_p95/canvas_exclusive_minus_us_total")
                .and_then(|v| v.as_i64()),
            Some(400)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/gap_to_code_editor_p95/canvas_exclusive_minus_windowed_surface_paint_callback")
                .and_then(|v| v.as_i64()),
            Some(100)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/gap_to_code_editor_p95/windowed_surface_paint_callback_minus_us_total")
                .and_then(|v| v.as_i64()),
            Some(300)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/gap_to_code_editor_p95/windowed_surface_row_paint_minus_us_total")
                .and_then(|v| v.as_i64()),
            Some(230)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/gap_to_code_editor_p95/windowed_surface_paint_callback_minus_row_paint")
                .and_then(|v| v.as_i64()),
            Some(70)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/gap_to_code_editor_p95/windowed_surface_paint_callback_minus_row_paint_per_row_ns")
                .and_then(|v| v.as_i64()),
            Some(7_000)
        );
        assert_eq!(
            json.pointer(
                "/paint_widget_hotspot_summary/gap_to_code_editor_p95/windowed_surface_row_callback_gap_per_row_ns"
            )
            .and_then(|v| v.as_i64()),
            Some(23_000)
        );
        assert_eq!(
            json.pointer(
                "/paint_widget_hotspot_summary/code_editor_windowed_surface_p95/row_paint"
            )
            .and_then(|v| v.as_u64()),
            Some(530)
        );
        assert_eq!(
            json.pointer("/paint_widget_hotspot_summary/code_editor_windowed_surface_p95/non_row")
                .and_then(|v| v.as_u64()),
            Some(90)
        );
        assert_eq!(
            json.pointer(
                "/paint_widget_hotspot_summary/code_editor_windowed_surface_p95/row_callback_gap"
            )
            .and_then(|v| v.as_u64()),
            Some(230)
        );
        assert_eq!(
            json.pointer(
                "/paint_widget_hotspot_summary/code_editor_windowed_surface_p95/rows_with_rect"
            )
            .and_then(|v| v.as_u64()),
            Some(10)
        );
        assert_eq!(
            json.pointer("/p50/paint_host_widget_observed_models_time_us")
                .and_then(|v| v.as_u64()),
            Some(20)
        );
        assert_eq!(
            json.pointer("/p95/paint_host_widget_observed_models_time_us")
                .and_then(|v| v.as_u64()),
            Some(30)
        );
        assert_eq!(
            json.pointer("/p95/paint_host_widget_instance_lookup_calls")
                .and_then(|v| v.as_u64()),
            Some(6)
        );
        assert_eq!(
            json.pointer("/p50/paint_cache_key_time_us")
                .and_then(|v| v.as_u64()),
            Some(25)
        );
        assert_eq!(
            json.pointer("/p95/paint_cache_key_time_us")
                .and_then(|v| v.as_u64()),
            Some(35)
        );
        assert_eq!(
            json.pointer("/p95/paint_record_visual_bounds_time_us")
                .and_then(|v| v.as_u64()),
            Some(7)
        );
        assert_eq!(
            json.pointer("/p95/paint_record_visual_bounds_calls")
                .and_then(|v| v.as_u64()),
            Some(4)
        );
        assert_eq!(
            json.pointer("/p95/paint_cache_hit_check_time_us")
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            json.pointer("/p95/paint_observation_record_time_us")
                .and_then(|v| v.as_u64()),
            Some(9)
        );
        assert_eq!(
            json.pointer("/p50/paint_host_widget_observed_deps_calls")
                .and_then(|v| v.as_u64()),
            Some(5)
        );
        assert_eq!(
            json.pointer("/p95/paint_host_widget_observed_deps_empty_calls")
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            json.pointer("/max/paint_host_widget_observed_globals_non_empty_calls")
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            json.pointer("/max/paint_host_widget_instance_lookup_time_us")
                .and_then(|v| v.as_u64()),
            Some(32)
        );
        assert_eq!(
            json.pointer("/max/paint_cache_key_time_us")
                .and_then(|v| v.as_u64()),
            Some(35)
        );
        assert_eq!(
            json.pointer("/max/paint_record_visual_bounds_calls")
                .and_then(|v| v.as_u64()),
            Some(4)
        );
    }

    #[test]
    fn bundle_stats_projects_command_availability_hotspots() {
        let bundle = serde_json::json!({
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 9,
                        "tick_id": 10,
                        "timestamp_unix_ms": 11,
                        "debug": {
                            "stats": {
                                "total_time_us": 5000,
                                "dispatch_time_us": 20,
                                "window_runtime_snapshot_command_availability_time_us": 30,
                                "window_runtime_snapshot_widget_command_count": 1,
                                "window_runtime_snapshot_command_registry_collect_time_us": 10,
                                "window_runtime_snapshot_command_availability_eval_time_us": 20
                            }
                        }
                    },
                    {
                        "frame_id": 10,
                        "tick_id": 11,
                        "timestamp_unix_ms": 12,
                        "debug": {
                            "stats": {
                                "total_time_us": 100,
                                "dispatch_time_us": 20,
                                "window_runtime_snapshot_command_availability_time_us": 900,
                                "window_runtime_snapshot_widget_command_count": 2,
                                "window_runtime_snapshot_command_registry_collect_time_us": 10,
                                "window_runtime_snapshot_command_availability_eval_time_us": 890
                            },
                            "command_availability_hotspots": [
                                {
                                    "command": "editor.save",
                                    "route": "focused_or_default",
                                    "start_node": 7,
                                    "resolved_node": 9,
                                    "outcome": "available",
                                    "elapsed_us": 700,
                                    "start_element": 17,
                                    "start_element_kind": "Button",
                                    "start_element_path": "app/root/save-button",
                                    "resolved_element": 19,
                                    "resolved_element_kind": "Editor",
                                    "resolved_element_path": "app/root/editor"
                                },
                                {
                                    "command": "editor.format",
                                    "route": "action_route_fallback_roots",
                                    "start_node": 3,
                                    "outcome": "not_handled",
                                    "elapsed_us": 200,
                                    "start_element_kind": "WindowRoot"
                                }
                            ]
                        }
                    }
                ]
            }]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            2,
            BundleStatsSort::Time,
            BundleStatsOptions { warmup_frames: 0 },
        )
        .expect("bundle stats");

        let command_row = report
            .top
            .iter()
            .find(|row| row.window_runtime_snapshot_command_availability_time_us == 900)
            .expect("command availability row");
        assert_eq!(command_row.command_availability_hotspots.len(), 2);
        assert_eq!(
            command_row.command_availability_hotspots[0].command,
            "editor.save"
        );
        assert_eq!(command_row.command_availability_hotspots[0].elapsed_us, 700);
        assert_eq!(command_row.command_availability_hotspots[0].start_node, 7);
        assert_eq!(
            command_row.command_availability_hotspots[0].resolved_node,
            Some(9)
        );
        assert_eq!(
            command_row.command_availability_hotspots[0]
                .start_element_path
                .as_deref(),
            Some("app/root/save-button")
        );

        let sorted_report = bundle_stats_from_json_with_options(
            &bundle,
            1,
            BundleStatsSort::CommandAvailability,
            BundleStatsOptions { warmup_frames: 0 },
        )
        .expect("bundle stats sorted by command availability");
        let top = sorted_report.top.first().expect("top row");
        assert_eq!(
            top.window_runtime_snapshot_command_availability_time_us,
            900
        );
        assert_eq!(top.command_availability_hotspots[0].command, "editor.save");

        let json = sorted_report.to_json();
        assert_eq!(
            json.pointer("/top/0/command_availability_hotspots/0/command")
                .and_then(|v| v.as_str()),
            Some("editor.save")
        );
        assert_eq!(
            json.pointer("/top/0/command_availability_hotspots/0/route")
                .and_then(|v| v.as_str()),
            Some("focused_or_default")
        );
        assert_eq!(
            json.pointer("/top/0/command_availability_hotspots/0/start_node")
                .and_then(|v| v.as_u64()),
            Some(7)
        );
        assert_eq!(
            json.pointer("/top/0/command_availability_hotspots/0/resolved_element")
                .and_then(|v| v.as_u64()),
            Some(19)
        );
    }
}
