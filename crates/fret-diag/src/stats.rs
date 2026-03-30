use std::borrow::Cow;
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
    SemanticsIndex, format_text_prepare_reasons, snapshot_global_change_hotspots,
    snapshot_global_change_unobserved, snapshot_layout_engine_solves, snapshot_layout_hotspots,
    snapshot_lookup_semantics, snapshot_model_change_hotspots, snapshot_model_change_unobserved,
    snapshot_paint_text_prepare_hotspots, snapshot_paint_widget_hotspots,
    snapshot_widget_measure_hotspots,
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

pub(super) fn bundle_stats_from_path(
    bundle_path: &Path,
    top: usize,
    sort: BundleStatsSort,
    opts: BundleStatsOptions,
) -> Result<BundleStatsReport, String> {
    const MAX_MATERIALIZED_BUNDLE_BYTES: u64 = 64 * 1024 * 1024;
    let file_len = std::fs::metadata(bundle_path)
        .map(|m| m.len())
        .unwrap_or(MAX_MATERIALIZED_BUNDLE_BYTES + 1);
    if file_len > MAX_MATERIALIZED_BUNDLE_BYTES {
        return bundle_stats_from_frames_index(bundle_path, top, sort, opts.warmup_frames).map_err(
            |err| {
                format!(
                    "{err}\n\
  bundle: {} ({} MiB)\n\
  hint: prefer schema2 + sidecars + lite triage:\n\
    - fretboard diag doctor --fix-schema2 <bundle_dir> --warmup-frames {}\n\
    - fretboard diag index <bundle_dir> --warmup-frames {}\n\
    - fretboard diag triage --lite <bundle_dir> --warmup-frames {}",
                    bundle_path.display(),
                    file_len / (1024 * 1024),
                    opts.warmup_frames,
                    opts.warmup_frames,
                    opts.warmup_frames
                )
            },
        );
    }
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    bundle_stats_from_json_with_options(&bundle, top, sort, opts)
}

include!("stats/bundle_stats_compute.inc.rs");

fn bundle_stats_from_frames_index(
    bundle_path: &Path,
    top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
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
        out.snapshots_considered = out
            .snapshots_considered
            .saturating_add(rows_arr.len().min(u32::MAX as usize) as u32);

        let skipped = snapshots_total.saturating_sub(rows_arr.len() as u64);
        out.snapshots_skipped_warmup = out
            .snapshots_skipped_warmup
            .saturating_add(skipped.min(u32::MAX as u64) as u32);

        for row in rows_arr {
            let Some(row) = row.as_array() else {
                continue;
            };

            let frame_id = row_u64(row, idx_frame_id).unwrap_or(0);
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
