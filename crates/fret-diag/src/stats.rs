use std::borrow::Cow;
use std::path::{Path, PathBuf};

mod bundle_stats_snapshot;
mod bundle_stats_sort;
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
pub(super) use gc_gates::check_bundle_for_gc_sweep_liveness;
pub(super) use hover_layout_checks::check_report_for_hover_layout_invalidations;
pub(super) use interaction_gates::{
    check_bundle_for_dock_drag_min, check_bundle_for_viewport_capture_min,
    check_bundle_for_viewport_input_min,
};
pub(super) use notify_gates::check_bundle_for_notify_hotspot_file_max;
pub(super) use overlay_gates::check_bundle_for_overlay_synthesis_min;
pub(super) use retained_vlist_gates::{
    check_bundle_for_retained_vlist_attach_detach_max,
    check_bundle_for_retained_vlist_keep_alive_budget,
    check_bundle_for_retained_vlist_keep_alive_reuse_min,
    check_bundle_for_retained_vlist_reconcile_no_notify_min,
};
pub(super) use stale::{
    check_bundle_for_semantics_changed_repainted, check_bundle_for_stale_paint,
    check_bundle_for_stale_scene,
};
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

include!("stats/bundle_stats_compute.inc.rs");

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
