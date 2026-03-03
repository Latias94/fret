//! Internal registries for diag checks.
//!
//! This module starts as a small seam. The long-term goal is to move ad-hoc check wiring
//! (lint/post-run gates/perf hint gates) behind explicit registries so adding checks does not
//! require editing a giant central match statement.
//!
//! NOTE: This is tooling-only; it is not a runtime contract.

use std::path::Path;

use crate::diag_run::RunChecks;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CheckKind {
    Lint,
    Triage,
    Perf,
    Hotspots,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PostRunCheckContext<'a> {
    pub bundle_path: &'a Path,
    #[allow(dead_code)]
    pub out_dir: &'a Path,
    pub warmup_frames: u64,
}

#[derive(Clone, Copy)]
struct PostRunCheckEntry {
    #[allow(dead_code)]
    id: &'static str,
    should_run: fn(&RunChecks) -> bool,
    run: fn(PostRunCheckContext<'_>, &RunChecks) -> Result<(), String>,
}

pub(crate) struct CheckRegistry {
    post_run_checks: &'static [PostRunCheckEntry],
}

impl CheckRegistry {
    pub(crate) fn builtin() -> Self {
        Self {
            post_run_checks: BUILTIN_POST_RUN_CHECKS,
        }
    }

    pub(crate) fn wants_post_run_checks(&self, checks: &RunChecks) -> bool {
        self.post_run_checks
            .iter()
            .any(|entry| (entry.should_run)(checks))
    }

    pub(crate) fn apply_post_run_checks(
        &self,
        ctx: PostRunCheckContext<'_>,
        checks: &RunChecks,
    ) -> Result<(), String> {
        for entry in self.post_run_checks {
            if (entry.should_run)(checks) {
                (entry.run)(ctx, checks)?;
            }
        }
        Ok(())
    }
}

const BUILTIN_POST_RUN_CHECKS: &[PostRunCheckEntry] = &[
    PostRunCheckEntry {
        id: "gc_sweep_liveness",
        should_run: should_run_gc_sweep_liveness,
        run: run_gc_sweep_liveness,
    },
    PostRunCheckEntry {
        id: "notify_hotspot_file_max",
        should_run: should_run_notify_hotspot_file_max,
        run: run_notify_hotspot_file_max,
    },
    PostRunCheckEntry {
        id: "triage_hint_absent_codes",
        should_run: should_run_triage_hint_absent_codes,
        run: run_triage_hint_absent_codes,
    },
    PostRunCheckEntry {
        id: "pixels_changed_test_id",
        should_run: should_run_pixels_changed_test_id,
        run: run_pixels_changed_test_id,
    },
    PostRunCheckEntry {
        id: "pixels_unchanged_test_id",
        should_run: should_run_pixels_unchanged_test_id,
        run: run_pixels_unchanged_test_id,
    },
];

fn should_run_gc_sweep_liveness(checks: &RunChecks) -> bool {
    checks.check_gc_sweep_liveness
}

fn run_gc_sweep_liveness(ctx: PostRunCheckContext<'_>, _checks: &RunChecks) -> Result<(), String> {
    crate::stats::check_bundle_for_gc_sweep_liveness(ctx.bundle_path, ctx.warmup_frames)
}

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
