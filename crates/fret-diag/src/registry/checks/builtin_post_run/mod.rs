//! Builtin post-run check registry.
//!
//! This module is a tooling-only seam (not a runtime contract). Keep it easy to extend without
//! growing central wiring churn.

use std::path::Path;
use std::sync::OnceLock;

use crate::diag_run::RunChecks;

mod base;
mod engine;
mod misc;
mod ui_gallery;

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
    requires_bundle_artifact: bool,
    requires_screenshots: bool,
    should_run: fn(&RunChecks) -> bool,
    run: fn(PostRunCheckContext<'_>, &RunChecks) -> Result<(), String>,
}

pub(crate) struct CheckRegistry {
    post_run_checks: &'static [PostRunCheckEntry],
}

impl CheckRegistry {
    pub(crate) fn builtin() -> Self {
        Self {
            post_run_checks: builtin_post_run_checks(),
        }
    }

    pub(crate) fn wants_post_run_checks(&self, checks: &RunChecks) -> bool {
        self.post_run_checks
            .iter()
            .any(|entry| (entry.should_run)(checks))
    }

    pub(crate) fn wants_bundle_artifact(&self, checks: &RunChecks) -> bool {
        self.post_run_checks
            .iter()
            .any(|entry| entry.requires_bundle_artifact && (entry.should_run)(checks))
    }

    pub(crate) fn wants_screenshots(&self, checks: &RunChecks) -> bool {
        self.post_run_checks
            .iter()
            .any(|entry| entry.requires_screenshots && (entry.should_run)(checks))
    }

    pub(crate) fn apply_post_run_checks(
        &self,
        ctx: PostRunCheckContext<'_>,
        checks: &RunChecks,
    ) -> Result<(), String> {
        for entry in self.post_run_checks {
            if (entry.should_run)(checks) {
                (entry.run)(ctx, checks)
                    .map_err(|err| format!("post-run check `{}` failed: {}", entry.id, err))?;
            }
        }
        Ok(())
    }
}

fn builtin_post_run_checks() -> &'static [PostRunCheckEntry] {
    static BUILTIN: OnceLock<Vec<PostRunCheckEntry>> = OnceLock::new();
    BUILTIN
        .get_or_init(|| {
            let mut out = Vec::new();
            out.extend_from_slice(base::ENTRIES);
            out.extend_from_slice(ui_gallery::entries());
            out.extend_from_slice(engine::ENTRIES);
            out.extend_from_slice(misc::ENTRIES);
            out
        })
        .as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_post_run_checks_includes_check_id_in_error() {
        fn should_run(_checks: &RunChecks) -> bool {
            true
        }

        fn run(_ctx: PostRunCheckContext<'_>, _checks: &RunChecks) -> Result<(), String> {
            Err("boom".to_string())
        }

        let registry = CheckRegistry {
            post_run_checks: &[PostRunCheckEntry {
                id: "test_check",
                requires_bundle_artifact: false,
                requires_screenshots: false,
                should_run,
                run,
            }],
        };

        let checks = RunChecks::default();

        let ctx = PostRunCheckContext {
            bundle_path: Path::new("bundle.json"),
            out_dir: Path::new("out"),
            warmup_frames: 0,
        };

        let err = registry.apply_post_run_checks(ctx, &checks).unwrap_err();
        assert!(err.contains("test_check"), "{err}");
        assert!(err.contains("boom"), "{err}");
    }
}
