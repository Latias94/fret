use super::{PostRunCheckContext, PostRunCheckEntry};
use crate::diag_run::RunChecks;

pub(super) const ENTRIES: &[PostRunCheckEntry] = &[PostRunCheckEntry {
    id: "ui_gallery_web_ime_bridge_enabled",
    requires_bundle_artifact: true,
    requires_screenshots: false,
    should_run: should_run_ui_gallery_web_ime_bridge_enabled,
    run: run_ui_gallery_web_ime_bridge_enabled,
}];

fn should_run_ui_gallery_web_ime_bridge_enabled(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_web_ime_bridge_enabled
}

fn run_ui_gallery_web_ime_bridge_enabled(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_web_ime_bridge_enabled(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}
