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

const BUILTIN_POST_RUN_CHECKS: &[PostRunCheckEntry] = &[PostRunCheckEntry {
    id: "gc_sweep_liveness",
    should_run: should_run_gc_sweep_liveness,
    run: run_gc_sweep_liveness,
}];

fn should_run_gc_sweep_liveness(checks: &RunChecks) -> bool {
    checks.check_gc_sweep_liveness
}

fn run_gc_sweep_liveness(ctx: PostRunCheckContext<'_>, _checks: &RunChecks) -> Result<(), String> {
    crate::stats::check_bundle_for_gc_sweep_liveness(ctx.bundle_path, ctx.warmup_frames)
}
