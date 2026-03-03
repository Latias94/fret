use super::{PostRunCheckContext, PostRunCheckEntry};
use crate::diag_run::RunChecks;

pub(super) const ENTRIES: &[PostRunCheckEntry] = &[PostRunCheckEntry {
    id: "semantics_changed_repainted",
    requires_bundle_artifact: true,
    requires_screenshots: false,
    should_run: should_run_semantics_changed_repainted,
    run: run_semantics_changed_repainted,
}];

fn should_run_semantics_changed_repainted(checks: &RunChecks) -> bool {
    checks.check_semantics_changed_repainted
}

fn run_semantics_changed_repainted(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_semantics_changed_repainted(
        ctx.bundle_path,
        ctx.warmup_frames,
        checks.dump_semantics_changed_repainted_json,
    )
}
