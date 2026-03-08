use super::*;
use crate::registry::checks::{CheckRegistry, PostRunCheckContext};

pub(crate) fn apply_post_run_checks(
    bundle_path: Option<&Path>,
    out_dir: &Path,
    checks: &diag_run::RunChecks,
    warmup_frames: u64,
) -> Result<(), String> {
    let registry = CheckRegistry::builtin();
    let bundle_path_for_checks = if registry.wants_bundle_artifact(checks) {
        let bundle_path = bundle_path.ok_or_else(|| {
            "post-run checks require a bundle artifact, but none was provided".to_string()
        })?;

        // Prefer the most recent export directory recorded by the diagnostics runtime.
        //
        // `script.result.json` currently reports the last "auto dump" directory (e.g. `press_key`),
        // but scripts typically emit explicit `capture_bundle` exports that include additional frames
        // after the triggering input. Post-run gates should run on the latest export to avoid
        // sampling before the UI has produced updated semantics.
        //
        // Note: the runtime may update `latest.txt` slightly after writing `script.result.json`.
        // Poll briefly to avoid sampling too early.
        {
            fn parse_leading_ts(name: &str) -> Option<u64> {
                let digits: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
                if digits.is_empty() {
                    return None;
                }
                digits.parse::<u64>().ok()
            }

            fn normalize_bundle_path(p: std::path::PathBuf) -> std::path::PathBuf {
                if p.extension().is_some_and(|ext| ext == "json") {
                    p
                } else {
                    crate::resolve_bundle_artifact_path(&p)
                }
            }

            fn path_ts(p: &std::path::Path) -> Option<u64> {
                let dir = p.parent()?;
                let name = dir.file_name()?.to_string_lossy();
                parse_leading_ts(&name)
            }

            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            let mut best: Option<std::path::PathBuf> = None;

            loop {
                let (from_latest, from_scan) = crate::latest::latest_bundle_dir_candidates(out_dir);
                let from_latest = from_latest.map(normalize_bundle_path);
                let from_scan = from_scan.map(|dir| normalize_bundle_path(dir));

                let candidate = match (from_latest, from_scan) {
                    (Some(a), Some(b)) => match (path_ts(&a), path_ts(&b)) {
                        (Some(ta), Some(tb)) => {
                            if tb >= ta {
                                Some(b)
                            } else {
                                Some(a)
                            }
                        }
                        (None, Some(_)) => Some(b),
                        (Some(_), None) => Some(a),
                        (None, None) => Some(b),
                    },
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (None, None) => None,
                }
                .filter(|p| p.is_file());

                if let Some(path) = candidate {
                    best = Some(path.clone());

                    let is_auto_dump = path
                        .parent()
                        .and_then(|p| p.file_name())
                        .map(|v| v.to_string_lossy().contains("script-step-"))
                        .unwrap_or(false);
                    if !is_auto_dump {
                        break path;
                    }
                }

                if std::time::Instant::now() >= deadline {
                    break best.unwrap_or_else(|| bundle_path.to_path_buf());
                }

                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        }
    } else {
        bundle_path
            .map(Path::to_path_buf)
            .unwrap_or_else(|| out_dir.join("bundle.not_required.json"))
    };

    registry.apply_post_run_checks(
        PostRunCheckContext {
            bundle_path: bundle_path_for_checks.as_path(),
            out_dir,
            warmup_frames,
        },
        checks,
    )?;
    Ok(())
}
