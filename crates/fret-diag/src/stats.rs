use std::borrow::Cow;
use std::path::{Path, PathBuf};

mod debug_stats_gates;
mod drag_cache_gates;
mod bundle_stats_sort;
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

include!("stats/bundle_stats_compute.inc.rs");

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
