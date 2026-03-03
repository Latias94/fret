use super::{PostRunCheckContext, PostRunCheckEntry};
use crate::diag_run::RunChecks;

pub(super) const ENTRIES: &[PostRunCheckEntry] = &[
    PostRunCheckEntry {
        id: "gc_sweep_liveness",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_gc_sweep_liveness,
        run: run_gc_sweep_liveness,
    },
    PostRunCheckEntry {
        id: "stale_paint_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_stale_paint_test_id,
        run: run_stale_paint_test_id,
    },
    PostRunCheckEntry {
        id: "stale_scene_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_stale_scene_test_id,
        run: run_stale_scene_test_id,
    },
    PostRunCheckEntry {
        id: "idle_no_paint_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_idle_no_paint_min,
        run: run_idle_no_paint_min,
    },
];

fn should_run_gc_sweep_liveness(checks: &RunChecks) -> bool {
    checks.check_gc_sweep_liveness
}

fn run_gc_sweep_liveness(ctx: PostRunCheckContext<'_>, _checks: &RunChecks) -> Result<(), String> {
    crate::stats::check_bundle_for_gc_sweep_liveness(ctx.bundle_path, ctx.warmup_frames)
}

fn should_run_stale_paint_test_id(checks: &RunChecks) -> bool {
    checks.check_stale_paint_test_id.is_some()
}

fn run_stale_paint_test_id(ctx: PostRunCheckContext<'_>, checks: &RunChecks) -> Result<(), String> {
    let Some(test_id) = checks.check_stale_paint_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_bundle_for_stale_paint(
        ctx.bundle_path,
        test_id,
        checks.check_stale_paint_eps,
    )
}

fn should_run_stale_scene_test_id(checks: &RunChecks) -> bool {
    checks.check_stale_scene_test_id.is_some()
}

fn run_stale_scene_test_id(ctx: PostRunCheckContext<'_>, checks: &RunChecks) -> Result<(), String> {
    let Some(test_id) = checks.check_stale_scene_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_bundle_for_stale_scene(
        ctx.bundle_path,
        test_id,
        checks.check_stale_scene_eps,
    )
}

fn should_run_idle_no_paint_min(checks: &RunChecks) -> bool {
    checks.check_idle_no_paint_min.is_some()
}

fn run_idle_no_paint_min(ctx: PostRunCheckContext<'_>, checks: &RunChecks) -> Result<(), String> {
    let Some(min) = checks.check_idle_no_paint_min else {
        return Ok(());
    };
    crate::stats::check_bundle_for_idle_no_paint_min(
        ctx.bundle_path,
        ctx.out_dir,
        min,
        ctx.warmup_frames,
    )
}
