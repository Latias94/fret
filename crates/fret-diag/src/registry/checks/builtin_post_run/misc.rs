use super::{PostRunCheckContext, PostRunCheckEntry};
use crate::diag_run::RunChecks;

pub(super) const ENTRIES: &[PostRunCheckEntry] = &[
    PostRunCheckEntry {
        id: "notify_hotspot_file_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_notify_hotspot_file_max,
        run: run_notify_hotspot_file_max,
    },
    PostRunCheckEntry {
        id: "triage_hint_absent_codes",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_triage_hint_absent_codes,
        run: run_triage_hint_absent_codes,
    },
    PostRunCheckEntry {
        id: "pixels_changed_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: true,
        should_run: should_run_pixels_changed_test_id,
        run: run_pixels_changed_test_id,
    },
    PostRunCheckEntry {
        id: "pixels_unchanged_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: true,
        should_run: should_run_pixels_unchanged_test_id,
        run: run_pixels_unchanged_test_id,
    },
];

fn should_run_notify_hotspot_file_max(checks: &RunChecks) -> bool {
    !checks.check_notify_hotspot_file_max.is_empty()
}

fn run_notify_hotspot_file_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    for (file, max) in checks.check_notify_hotspot_file_max.iter() {
        crate::stats::check_bundle_for_notify_hotspot_file_max(
            ctx.bundle_path,
            file.as_str(),
            *max,
            ctx.warmup_frames,
        )?;
    }
    Ok(())
}

fn should_run_triage_hint_absent_codes(checks: &RunChecks) -> bool {
    !checks.check_triage_hint_absent_codes.is_empty()
}

fn run_triage_hint_absent_codes(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let sort = crate::BundleStatsSort::Invalidation;
    let report = crate::bundle_stats_from_path(
        ctx.bundle_path,
        1,
        sort,
        crate::BundleStatsOptions {
            warmup_frames: ctx.warmup_frames,
        },
    )?;
    let triage = crate::triage_json_from_stats(ctx.bundle_path, &report, sort, ctx.warmup_frames);
    let present_codes: Vec<String> = triage
        .get("hints")
        .and_then(|v| v.as_array())
        .map(|hints| {
            hints
                .iter()
                .filter_map(|h| {
                    h.get("code")
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    let mut violations: Vec<String> = Vec::new();
    for code in checks.check_triage_hint_absent_codes.iter() {
        if present_codes.iter().any(|c| c == code) {
            violations.push(code.clone());
        }
    }
    if !violations.is_empty() {
        return Err(format!(
            "triage hint(s) present but forbidden by --check-triage-hint-absent: {}\n\
 bundle={}\n\
 present_hints={}",
            violations.join(", "),
            ctx.bundle_path.display(),
            present_codes.join(", ")
        ));
    }

    Ok(())
}

fn should_run_pixels_changed_test_id(checks: &RunChecks) -> bool {
    checks.check_pixels_changed_test_id.is_some()
}

fn run_pixels_changed_test_id(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(test_id) = checks.check_pixels_changed_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_out_dir_for_pixels_changed(ctx.out_dir, test_id, ctx.warmup_frames)
}

fn should_run_pixels_unchanged_test_id(checks: &RunChecks) -> bool {
    checks.check_pixels_unchanged_test_id.is_some()
}

fn run_pixels_unchanged_test_id(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(test_id) = checks.check_pixels_unchanged_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_out_dir_for_pixels_unchanged(ctx.out_dir, test_id, ctx.warmup_frames)
}
