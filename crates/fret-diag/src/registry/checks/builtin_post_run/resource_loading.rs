use super::{PostRunCheckContext, PostRunCheckEntry};
use crate::diag_run::RunChecks;

pub(super) const ENTRIES: &[PostRunCheckEntry] = &[
    PostRunCheckEntry {
        id: "asset_load_missing_bundle_assets_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_asset_load_missing_bundle_assets_max,
        run: run_asset_load_missing_bundle_assets_max,
    },
    PostRunCheckEntry {
        id: "asset_load_stale_manifest_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_asset_load_stale_manifest_max,
        run: run_asset_load_stale_manifest_max,
    },
    PostRunCheckEntry {
        id: "asset_load_unsupported_file_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_asset_load_unsupported_file_max,
        run: run_asset_load_unsupported_file_max,
    },
    PostRunCheckEntry {
        id: "asset_load_unsupported_url_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_asset_load_unsupported_url_max,
        run: run_asset_load_unsupported_url_max,
    },
    PostRunCheckEntry {
        id: "asset_load_external_reference_unavailable_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_asset_load_external_reference_unavailable_max,
        run: run_asset_load_external_reference_unavailable_max,
    },
    PostRunCheckEntry {
        id: "asset_load_revision_changes_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_asset_load_revision_changes_max,
        run: run_asset_load_revision_changes_max,
    },
    PostRunCheckEntry {
        id: "bundled_font_baseline_source",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_bundled_font_baseline_source,
        run: run_bundled_font_baseline_source,
    },
];

fn should_run_asset_load_missing_bundle_assets_max(checks: &RunChecks) -> bool {
    checks.check_asset_load_missing_bundle_assets_max.is_some()
}

fn run_asset_load_missing_bundle_assets_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_allowed) = checks.check_asset_load_missing_bundle_assets_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_asset_load_missing_bundle_assets_max(
        ctx.bundle_path,
        max_allowed,
        ctx.warmup_frames,
    )
}

fn should_run_asset_load_stale_manifest_max(checks: &RunChecks) -> bool {
    checks.check_asset_load_stale_manifest_max.is_some()
}

fn run_asset_load_stale_manifest_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_allowed) = checks.check_asset_load_stale_manifest_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_asset_load_stale_manifest_max(
        ctx.bundle_path,
        max_allowed,
        ctx.warmup_frames,
    )
}

fn should_run_asset_load_unsupported_file_max(checks: &RunChecks) -> bool {
    checks.check_asset_load_unsupported_file_max.is_some()
}

fn run_asset_load_unsupported_file_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_allowed) = checks.check_asset_load_unsupported_file_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_asset_load_unsupported_file_max(
        ctx.bundle_path,
        max_allowed,
        ctx.warmup_frames,
    )
}

fn should_run_asset_load_unsupported_url_max(checks: &RunChecks) -> bool {
    checks.check_asset_load_unsupported_url_max.is_some()
}

fn run_asset_load_unsupported_url_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_allowed) = checks.check_asset_load_unsupported_url_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_asset_load_unsupported_url_max(
        ctx.bundle_path,
        max_allowed,
        ctx.warmup_frames,
    )
}

fn should_run_asset_load_external_reference_unavailable_max(checks: &RunChecks) -> bool {
    checks
        .check_asset_load_external_reference_unavailable_max
        .is_some()
}

fn run_asset_load_external_reference_unavailable_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_allowed) = checks.check_asset_load_external_reference_unavailable_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_asset_load_external_reference_unavailable_max(
        ctx.bundle_path,
        max_allowed,
        ctx.warmup_frames,
    )
}

fn should_run_asset_load_revision_changes_max(checks: &RunChecks) -> bool {
    checks.check_asset_load_revision_changes_max.is_some()
}

fn run_asset_load_revision_changes_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(max_allowed) = checks.check_asset_load_revision_changes_max else {
        return Ok(());
    };
    crate::stats::check_bundle_for_asset_load_revision_changes_max(
        ctx.bundle_path,
        max_allowed,
        ctx.warmup_frames,
    )
}

fn should_run_bundled_font_baseline_source(checks: &RunChecks) -> bool {
    checks.check_bundled_font_baseline_source.is_some()
}

fn run_bundled_font_baseline_source(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(expected_source) = checks.check_bundled_font_baseline_source.as_deref() else {
        return Ok(());
    };
    crate::stats::check_bundle_for_bundled_font_baseline_source(
        ctx.bundle_path,
        expected_source,
        ctx.warmup_frames,
    )
}
