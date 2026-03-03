use super::{PostRunCheckContext, PostRunCheckEntry};
use crate::diag_run::RunChecks;

pub(super) const ENTRIES: &[PostRunCheckEntry] = &[
    PostRunCheckEntry {
        id: "wheel_scroll_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_wheel_scroll_test_id,
        run: run_wheel_scroll_test_id,
    },
    PostRunCheckEntry {
        id: "wheel_scroll_hit_changes_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_wheel_scroll_hit_changes_test_id,
        run: run_wheel_scroll_hit_changes_test_id,
    },
    PostRunCheckEntry {
        id: "wheel_events_max_per_frame",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_wheel_events_max_per_frame,
        run: run_wheel_events_max_per_frame,
    },
    PostRunCheckEntry {
        id: "prepaint_actions_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_prepaint_actions_min,
        run: run_prepaint_actions_min,
    },
    PostRunCheckEntry {
        id: "chart_sampling_window_shifts_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_chart_sampling_window_shifts_min,
        run: run_chart_sampling_window_shifts_min,
    },
    PostRunCheckEntry {
        id: "node_graph_cull_window_shifts_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_node_graph_cull_window_shifts_min,
        run: run_node_graph_cull_window_shifts_min,
    },
    PostRunCheckEntry {
        id: "node_graph_cull_window_shifts_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_node_graph_cull_window_shifts_max,
        run: run_node_graph_cull_window_shifts_max,
    },
    PostRunCheckEntry {
        id: "vlist_visible_range_refreshes_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_vlist_visible_range_refreshes_min,
        run: run_vlist_visible_range_refreshes_min,
    },
    PostRunCheckEntry {
        id: "vlist_visible_range_refreshes_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_vlist_visible_range_refreshes_max,
        run: run_vlist_visible_range_refreshes_max,
    },
    PostRunCheckEntry {
        id: "vlist_window_shifts_explainable",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_vlist_window_shifts_explainable,
        run: run_vlist_window_shifts_explainable,
    },
    PostRunCheckEntry {
        id: "vlist_window_shifts_have_prepaint_actions",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_vlist_window_shifts_have_prepaint_actions,
        run: run_vlist_window_shifts_have_prepaint_actions,
    },
    PostRunCheckEntry {
        id: "vlist_window_shifts_non_retained_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_vlist_window_shifts_non_retained_max,
        run: run_vlist_window_shifts_non_retained_max,
    },
    PostRunCheckEntry {
        id: "vlist_window_shifts_prefetch_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_vlist_window_shifts_prefetch_max,
        run: run_vlist_window_shifts_prefetch_max,
    },
    PostRunCheckEntry {
        id: "vlist_window_shifts_escape_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_vlist_window_shifts_escape_max,
        run: run_vlist_window_shifts_escape_max,
    },
    PostRunCheckEntry {
        id: "vlist_policy_key_stable",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_vlist_policy_key_stable,
        run: run_vlist_policy_key_stable,
    },
    PostRunCheckEntry {
        id: "windowed_rows_offset_changes_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_windowed_rows_offset_changes_min,
        run: run_windowed_rows_offset_changes_min,
    },
    PostRunCheckEntry {
        id: "windowed_rows_visible_start_changes_repainted",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_windowed_rows_visible_start_changes_repainted,
        run: run_windowed_rows_visible_start_changes_repainted,
    },
    PostRunCheckEntry {
        id: "layout_fast_path_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_layout_fast_path_min,
        run: run_layout_fast_path_min,
    },
    PostRunCheckEntry {
        id: "drag_cache_root_paint_only_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_drag_cache_root_paint_only_test_id,
        run: run_drag_cache_root_paint_only_test_id,
    },
    PostRunCheckEntry {
        id: "hover_layout_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_hover_layout_max,
        run: run_hover_layout_max,
    },
    PostRunCheckEntry {
        id: "view_cache_reuse_stable_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_view_cache_reuse_stable_min,
        run: run_view_cache_reuse_stable_min,
    },
    PostRunCheckEntry {
        id: "view_cache_reuse_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_view_cache_reuse_min,
        run: run_view_cache_reuse_min,
    },
    PostRunCheckEntry {
        id: "overlay_synthesis_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_overlay_synthesis_min,
        run: run_overlay_synthesis_min,
    },
    PostRunCheckEntry {
        id: "viewport_input_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_viewport_input_min,
        run: run_viewport_input_min,
    },
    PostRunCheckEntry {
        id: "dock_drag_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_dock_drag_min,
        run: run_dock_drag_min,
    },
    PostRunCheckEntry {
        id: "viewport_capture_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_viewport_capture_min,
        run: run_viewport_capture_min,
    },
    PostRunCheckEntry {
        id: "retained_vlist_reconcile_no_notify_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_retained_vlist_reconcile_no_notify_min,
        run: run_retained_vlist_reconcile_no_notify_min,
    },
    PostRunCheckEntry {
        id: "retained_vlist_attach_detach_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_retained_vlist_attach_detach_max,
        run: run_retained_vlist_attach_detach_max,
    },
    PostRunCheckEntry {
        id: "retained_vlist_keep_alive_reuse_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_retained_vlist_keep_alive_reuse_min,
        run: run_retained_vlist_keep_alive_reuse_min,
    },
    PostRunCheckEntry {
        id: "retained_vlist_keep_alive_budget",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_retained_vlist_keep_alive_budget,
        run: run_retained_vlist_keep_alive_budget,
    },
];

fn should_run_wheel_scroll_test_id(checks: &RunChecks) -> bool {
    checks.check_wheel_scroll_test_id.is_some()
}

fn run_wheel_scroll_test_id(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(test_id) = checks.check_wheel_scroll_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_bundle_for_wheel_scroll(ctx.bundle_path, test_id, ctx.warmup_frames)
}

fn should_run_wheel_scroll_hit_changes_test_id(checks: &RunChecks) -> bool {
    checks.check_wheel_scroll_hit_changes_test_id.is_some()
}

fn run_wheel_scroll_hit_changes_test_id(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(test_id) = checks.check_wheel_scroll_hit_changes_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_bundle_for_wheel_scroll_hit_changes(
        ctx.bundle_path,
        test_id,
        ctx.warmup_frames,
    )
}

fn should_run_wheel_events_max_per_frame(checks: &RunChecks) -> bool {
    checks.check_wheel_events_max_per_frame.is_some()
}

fn run_wheel_events_max_per_frame(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_per_frame) = checks.check_wheel_events_max_per_frame else {
        return Ok(());
    };
    crate::stats::check_bundle_for_wheel_events_max_per_frame(
        ctx.bundle_path,
        ctx.out_dir,
        max_per_frame,
    )
}

fn should_run_prepaint_actions_min(checks: &RunChecks) -> bool {
    checks.check_prepaint_actions_min.is_some()
}

fn run_prepaint_actions_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min) = checks.check_prepaint_actions_min else {
        return Ok(());
    };
    crate::stats::check_bundle_for_prepaint_actions_min(
        ctx.bundle_path,
        ctx.out_dir,
        min,
        ctx.warmup_frames,
    )
}

fn should_run_chart_sampling_window_shifts_min(checks: &RunChecks) -> bool {
    checks.check_chart_sampling_window_shifts_min.is_some()
}

fn run_chart_sampling_window_shifts_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min) = checks.check_chart_sampling_window_shifts_min else {
        return Ok(());
    };
    crate::stats::check_bundle_for_chart_sampling_window_shifts_min(
        ctx.bundle_path,
        ctx.out_dir,
        min,
        ctx.warmup_frames,
    )
}

fn should_run_node_graph_cull_window_shifts_min(checks: &RunChecks) -> bool {
    checks.check_node_graph_cull_window_shifts_min.is_some()
}

fn run_node_graph_cull_window_shifts_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min) = checks.check_node_graph_cull_window_shifts_min else {
        return Ok(());
    };
    crate::stats::check_bundle_for_node_graph_cull_window_shifts_min(
        ctx.bundle_path,
        ctx.out_dir,
        min,
        ctx.warmup_frames,
    )
}

fn should_run_node_graph_cull_window_shifts_max(checks: &RunChecks) -> bool {
    checks.check_node_graph_cull_window_shifts_max.is_some()
}

fn run_node_graph_cull_window_shifts_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max) = checks.check_node_graph_cull_window_shifts_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_node_graph_cull_window_shifts_max(
        ctx.bundle_path,
        ctx.out_dir,
        max,
        ctx.warmup_frames,
    )
}

fn should_run_vlist_visible_range_refreshes_min(checks: &RunChecks) -> bool {
    checks.check_vlist_visible_range_refreshes_min.is_some()
}

fn run_vlist_visible_range_refreshes_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min_total_refreshes) = checks.check_vlist_visible_range_refreshes_min else {
        return Ok(());
    };
    crate::stats::check_bundle_for_vlist_visible_range_refreshes_min(
        ctx.bundle_path,
        ctx.out_dir,
        min_total_refreshes,
        ctx.warmup_frames,
    )
}

fn should_run_vlist_visible_range_refreshes_max(checks: &RunChecks) -> bool {
    checks.check_vlist_visible_range_refreshes_max.is_some()
}

fn run_vlist_visible_range_refreshes_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_total_refreshes) = checks.check_vlist_visible_range_refreshes_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_vlist_visible_range_refreshes_max(
        ctx.bundle_path,
        ctx.out_dir,
        max_total_refreshes,
        ctx.warmup_frames,
    )
}

fn should_run_vlist_window_shifts_explainable(checks: &RunChecks) -> bool {
    checks.check_vlist_window_shifts_explainable
}

fn run_vlist_window_shifts_explainable(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_vlist_window_shifts_explainable(
        ctx.bundle_path,
        ctx.out_dir,
        ctx.warmup_frames,
    )
}

fn should_run_vlist_window_shifts_have_prepaint_actions(checks: &RunChecks) -> bool {
    checks.check_vlist_window_shifts_have_prepaint_actions
}

fn run_vlist_window_shifts_have_prepaint_actions(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_vlist_window_shifts_have_prepaint_actions(
        ctx.bundle_path,
        ctx.out_dir,
        ctx.warmup_frames,
    )
}

fn should_run_vlist_window_shifts_non_retained_max(checks: &RunChecks) -> bool {
    checks.check_vlist_window_shifts_non_retained_max.is_some()
}

fn run_vlist_window_shifts_non_retained_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_total_non_retained_shifts) = checks.check_vlist_window_shifts_non_retained_max
    else {
        return Ok(());
    };
    crate::stats::check_bundle_for_vlist_window_shifts_non_retained_max(
        ctx.bundle_path,
        ctx.out_dir,
        max_total_non_retained_shifts,
        ctx.warmup_frames,
    )
}

fn should_run_vlist_window_shifts_prefetch_max(checks: &RunChecks) -> bool {
    checks.check_vlist_window_shifts_prefetch_max.is_some()
}

fn run_vlist_window_shifts_prefetch_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_total_prefetch_shifts) = checks.check_vlist_window_shifts_prefetch_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_vlist_window_shifts_kind_max(
        ctx.bundle_path,
        ctx.out_dir,
        "prefetch",
        max_total_prefetch_shifts,
        ctx.warmup_frames,
    )
}

fn should_run_vlist_window_shifts_escape_max(checks: &RunChecks) -> bool {
    checks.check_vlist_window_shifts_escape_max.is_some()
}

fn run_vlist_window_shifts_escape_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_total_escape_shifts) = checks.check_vlist_window_shifts_escape_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_vlist_window_shifts_kind_max(
        ctx.bundle_path,
        ctx.out_dir,
        "escape",
        max_total_escape_shifts,
        ctx.warmup_frames,
    )
}

fn should_run_vlist_policy_key_stable(checks: &RunChecks) -> bool {
    checks.check_vlist_policy_key_stable
}

fn run_vlist_policy_key_stable(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_vlist_policy_key_stable(
        ctx.bundle_path,
        ctx.out_dir,
        ctx.warmup_frames,
    )
}

fn should_run_windowed_rows_offset_changes_min(checks: &RunChecks) -> bool {
    checks.check_windowed_rows_offset_changes_min.is_some()
}

fn run_windowed_rows_offset_changes_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min_total_offset_changes) = checks.check_windowed_rows_offset_changes_min else {
        return Ok(());
    };
    crate::stats::check_bundle_for_windowed_rows_offset_changes_min(
        ctx.bundle_path,
        ctx.out_dir,
        min_total_offset_changes,
        ctx.warmup_frames,
        checks.check_windowed_rows_offset_changes_eps,
    )
}

fn should_run_windowed_rows_visible_start_changes_repainted(checks: &RunChecks) -> bool {
    checks.check_windowed_rows_visible_start_changes_repainted
}

fn run_windowed_rows_visible_start_changes_repainted(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_windowed_rows_visible_start_changes_repainted(
        ctx.bundle_path,
        ctx.out_dir,
        ctx.warmup_frames,
    )
}

fn should_run_layout_fast_path_min(checks: &RunChecks) -> bool {
    checks.check_layout_fast_path_min.is_some()
}

fn run_layout_fast_path_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min_frames) = checks.check_layout_fast_path_min else {
        return Ok(());
    };
    crate::stats::check_bundle_for_layout_fast_path_min(
        ctx.bundle_path,
        ctx.out_dir,
        min_frames,
        ctx.warmup_frames,
    )
}

fn should_run_drag_cache_root_paint_only_test_id(checks: &RunChecks) -> bool {
    checks.check_drag_cache_root_paint_only_test_id.is_some()
}

fn run_drag_cache_root_paint_only_test_id(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(test_id) = checks.check_drag_cache_root_paint_only_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_bundle_for_drag_cache_root_paint_only(
        ctx.bundle_path,
        test_id,
        ctx.warmup_frames,
    )
}

fn should_run_hover_layout_max(checks: &RunChecks) -> bool {
    checks.check_hover_layout_max.is_some()
}

fn run_hover_layout_max(ctx: PostRunCheckContext<'_>, checks: &RunChecks) -> Result<(), String> {
    let Some(max_allowed) = checks.check_hover_layout_max else {
        return Ok(());
    };
    let report = crate::bundle_stats_from_path(
        ctx.bundle_path,
        1,
        crate::BundleStatsSort::Invalidation,
        crate::BundleStatsOptions {
            warmup_frames: ctx.warmup_frames,
        },
    )?;
    crate::check_report_for_hover_layout_invalidations(&report, max_allowed)
}

fn should_run_view_cache_reuse_stable_min(checks: &RunChecks) -> bool {
    checks.check_view_cache_reuse_stable_min.is_some()
}

fn run_view_cache_reuse_stable_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min) = checks.check_view_cache_reuse_stable_min else {
        return Ok(());
    };
    if min == 0 {
        return Ok(());
    }
    crate::stats::check_bundle_for_view_cache_reuse_stable_min(
        ctx.bundle_path,
        ctx.out_dir,
        min,
        ctx.warmup_frames,
    )
}

fn should_run_view_cache_reuse_min(checks: &RunChecks) -> bool {
    checks.check_view_cache_reuse_min.is_some()
}

fn run_view_cache_reuse_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min) = checks.check_view_cache_reuse_min else {
        return Ok(());
    };
    if min == 0 {
        return Ok(());
    }
    crate::stats::check_bundle_for_view_cache_reuse_min(ctx.bundle_path, min, ctx.warmup_frames)
}

fn should_run_overlay_synthesis_min(checks: &RunChecks) -> bool {
    checks.check_overlay_synthesis_min.is_some()
}

fn run_overlay_synthesis_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min) = checks.check_overlay_synthesis_min else {
        return Ok(());
    };
    if min == 0 {
        return Ok(());
    }
    crate::stats::check_bundle_for_overlay_synthesis_min(ctx.bundle_path, min, ctx.warmup_frames)
}

fn should_run_viewport_input_min(checks: &RunChecks) -> bool {
    checks.check_viewport_input_min.is_some()
}

fn run_viewport_input_min(ctx: PostRunCheckContext<'_>, checks: &RunChecks) -> Result<(), String> {
    let Some(min) = checks.check_viewport_input_min else {
        return Ok(());
    };
    if min == 0 {
        return Ok(());
    }
    crate::stats::check_bundle_for_viewport_input_min(ctx.bundle_path, min, ctx.warmup_frames)
}

fn should_run_dock_drag_min(checks: &RunChecks) -> bool {
    checks.check_dock_drag_min.is_some()
}

fn run_dock_drag_min(ctx: PostRunCheckContext<'_>, checks: &RunChecks) -> Result<(), String> {
    let Some(min) = checks.check_dock_drag_min else {
        return Ok(());
    };
    if min == 0 {
        return Ok(());
    }
    crate::stats::check_bundle_for_dock_drag_min(ctx.bundle_path, min, ctx.warmup_frames)
}

fn should_run_viewport_capture_min(checks: &RunChecks) -> bool {
    checks.check_viewport_capture_min.is_some()
}

fn run_viewport_capture_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min) = checks.check_viewport_capture_min else {
        return Ok(());
    };
    if min == 0 {
        return Ok(());
    }
    crate::stats::check_bundle_for_viewport_capture_min(ctx.bundle_path, min, ctx.warmup_frames)
}

fn should_run_retained_vlist_reconcile_no_notify_min(checks: &RunChecks) -> bool {
    checks
        .check_retained_vlist_reconcile_no_notify_min
        .is_some()
}

fn run_retained_vlist_reconcile_no_notify_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min) = checks.check_retained_vlist_reconcile_no_notify_min else {
        return Ok(());
    };
    if min == 0 {
        return Ok(());
    }
    crate::stats::check_bundle_for_retained_vlist_reconcile_no_notify_min(
        ctx.bundle_path,
        min,
        ctx.warmup_frames,
    )
}

fn should_run_retained_vlist_attach_detach_max(checks: &RunChecks) -> bool {
    checks.check_retained_vlist_attach_detach_max.is_some()
}

fn run_retained_vlist_attach_detach_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_delta) = checks.check_retained_vlist_attach_detach_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_retained_vlist_attach_detach_max(
        ctx.bundle_path,
        max_delta,
        ctx.warmup_frames,
    )
}

fn should_run_retained_vlist_keep_alive_reuse_min(checks: &RunChecks) -> bool {
    checks.check_retained_vlist_keep_alive_reuse_min.is_some()
}

fn run_retained_vlist_keep_alive_reuse_min(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(min) = checks.check_retained_vlist_keep_alive_reuse_min else {
        return Ok(());
    };
    if min == 0 {
        return Ok(());
    }
    crate::stats::check_bundle_for_retained_vlist_keep_alive_reuse_min(
        ctx.bundle_path,
        min,
        ctx.warmup_frames,
    )
}

fn should_run_retained_vlist_keep_alive_budget(checks: &RunChecks) -> bool {
    checks.check_retained_vlist_keep_alive_budget.is_some()
}

fn run_retained_vlist_keep_alive_budget(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some((min_max_pool_len_after, max_total_evicted_items)) =
        checks.check_retained_vlist_keep_alive_budget
    else {
        return Ok(());
    };
    crate::stats::check_bundle_for_retained_vlist_keep_alive_budget(
        ctx.bundle_path,
        min_max_pool_len_after,
        max_total_evicted_items,
        ctx.warmup_frames,
    )
}
