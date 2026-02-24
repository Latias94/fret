use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) fn resolve_path(workspace_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    }
}

fn normalize_host_path_separators(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(path.to_string_lossy().replace('/', "\\"))
    }
    #[cfg(not(windows))]
    {
        path
    }
}

pub(crate) fn expand_script_inputs(
    workspace_root: &Path,
    inputs: &[String],
) -> Result<Vec<PathBuf>, String> {
    let mut set: BTreeSet<PathBuf> = BTreeSet::new();

    for input in inputs {
        let resolved = resolve_path(workspace_root, PathBuf::from(input));

        // Directory input: treat as recursive `**/*.json` to support suite-like workflows.
        if resolved.is_dir() {
            let mut pattern = resolved.to_string_lossy().to_string();
            pattern = pattern.replace('\\', "/");
            if !pattern.ends_with('/') {
                pattern.push('/');
            }
            pattern.push_str("**/*.json");

            let mut any = false;
            for entry in glob::glob(&pattern).map_err(|e| e.to_string())? {
                let path = entry.map_err(|e| e.to_string())?;
                set.insert(normalize_host_path_separators(path));
                any = true;
            }
            if !any {
                return Err(format!(
                    "script input matched no files: {input} ({pattern})"
                ));
            }
            continue;
        }

        // Wildcard input: expand via glob. (PowerShell doesn't always expand globs for child args.)
        if input.contains('*') || input.contains('?') || input.contains('[') {
            let mut pattern = resolved.to_string_lossy().to_string();
            pattern = pattern.replace('\\', "/");

            let mut any = false;
            for entry in glob::glob(&pattern).map_err(|e| e.to_string())? {
                let path = entry.map_err(|e| e.to_string())?;
                set.insert(normalize_host_path_separators(path));
                any = true;
            }
            if !any {
                return Err(format!(
                    "script input matched no files: {input} ({pattern})"
                ));
            }
            continue;
        }

        set.insert(resolved);
    }

    Ok(set.into_iter().collect())
}

pub(crate) fn resolve_bundle_root_dir(path: &Path) -> Result<PathBuf, String> {
    if path.is_dir() {
        return Ok(path.to_path_buf());
    }
    let Some(parent) = path.parent() else {
        return Err(format!("invalid bundle path: {}", path.display()));
    };
    Ok(parent.to_path_buf())
}

pub(crate) fn default_pack_out_path(out_dir: &Path, bundle_dir: &Path) -> PathBuf {
    let name = bundle_dir
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");
    if bundle_dir.starts_with(out_dir) {
        out_dir.join("share").join(format!("{name}.zip"))
    } else {
        bundle_dir.with_extension("zip")
    }
}

pub(crate) fn default_triage_out_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("triage.json")
}

pub(crate) fn default_lint_out_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("check.lint.json")
}

pub(crate) fn default_meta_out_path(bundle_path: &Path) -> PathBuf {
    crate::bundle_index::default_bundle_meta_path(bundle_path)
}

pub(crate) fn default_test_ids_out_path(bundle_path: &Path) -> PathBuf {
    crate::bundle_index::default_test_ids_path(bundle_path)
}

pub(crate) fn record_tooling_artifact_integrity_failure_for_dir(dir: &Path, err: &str) {
    use fret_diag_protocol::UiScriptResultV1;

    let reason_code = "tooling.artifact.integrity.failed";
    let kind = "tooling_artifact_integrity_failed";

    let direct = dir.join("script.result.json");
    let from_parent = dir
        .parent()
        .map(|p| p.join("script.result.json"))
        .unwrap_or_else(|| direct.clone());
    let script_result_path = if direct.is_file() {
        direct
    } else if from_parent.is_file() {
        from_parent
    } else {
        return;
    };

    let bytes = std::fs::read(&script_result_path).ok();
    let parsed = bytes
        .as_deref()
        .and_then(|b| serde_json::from_slice::<UiScriptResultV1>(b).ok());
    let Some(parsed) = parsed else {
        return;
    };

    let mut out_dir = script_result_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| dir.to_path_buf());
    if let Some(parent) = script_result_path.parent()
        && parent
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s == parsed.run_id.to_string())
            .unwrap_or(false)
    {
        out_dir = parent.parent().unwrap_or(parent).to_path_buf();
    }

    crate::tooling_failures::mark_existing_script_result_tooling_failure(
        &out_dir,
        &script_result_path,
        reason_code,
        err,
        kind,
        Some(format!("dir={}", dir.display())),
    );
}

pub(crate) fn resolve_bundle_artifact_path(path: &Path) -> PathBuf {
    if !path.is_dir() {
        return path.to_path_buf();
    }

    let direct_v2 = path.join("bundle.schema2.json");
    if direct_v2.is_file() {
        return direct_v2;
    }

    let direct = path.join("bundle.json");
    if direct.is_file() {
        return direct;
    }

    let root_v2 = path.join("_root").join("bundle.schema2.json");
    if root_v2.is_file() {
        return root_v2;
    }

    let root = path.join("_root").join("bundle.json");
    if root.is_file() {
        return root;
    }

    match crate::run_artifacts::materialize_bundle_json_from_manifest_chunks_if_missing(path) {
        Ok(Some(v2)) if v2.is_file() => {
            return v2;
        }
        Ok(_) => {}
        Err(err) => {
            record_tooling_artifact_integrity_failure_for_dir(
                path,
                &format!("failed to materialize bundle.json from chunks: {err}"),
            );
        }
    }

    if let Some(run_id) = crate::util::read_script_result_run_id(&path.join("script.result.json")) {
        let run_id_dir = crate::run_artifacts::run_id_artifact_dir(path, run_id);
        let run_id_schema2 = run_id_dir.join("bundle.schema2.json");
        if run_id_schema2.is_file() {
            return run_id_schema2;
        }

        let run_id_bundle = run_id_dir.join("bundle.json");
        if run_id_bundle.is_file() {
            return run_id_bundle;
        }
        match crate::run_artifacts::materialize_run_id_bundle_json_from_chunks_if_missing(
            path, run_id,
        ) {
            Ok(Some(v2)) if v2.is_file() => {
                return v2;
            }
            Ok(_) => {}
            Err(err) => {
                record_tooling_artifact_integrity_failure_for_dir(
                    &crate::run_artifacts::run_id_artifact_dir(path, run_id),
                    &format!("failed to materialize bundle.json from chunks: {err}"),
                );
            }
        }
    }

    if let Some(dir) = crate::compare::read_latest_pointer(path)
        .or_else(|| crate::compare::find_latest_export_dir(path))
    {
        let nested_v2 = dir.join("bundle.schema2.json");
        if nested_v2.is_file() {
            return nested_v2;
        }
        let nested = dir.join("bundle.json");
        if nested.is_file() {
            return nested;
        }
    }

    direct
}

pub(crate) fn wait_for_bundle_artifact_from_script_result(
    out_dir: &Path,
    result: &crate::stats::ScriptResultSummary,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    use std::time::{Duration, Instant};

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.clamp(250, 5_000));
    while Instant::now() < deadline {
        let run_id_dir = crate::run_artifacts::run_id_artifact_dir(out_dir, result.run_id);
        let run_id_schema2 = run_id_dir.join("bundle.schema2.json");
        if run_id_schema2.is_file() {
            return Some(run_id_schema2);
        }
        let run_id_bundle_json = run_id_dir.join("bundle.json");
        if run_id_bundle_json.is_file() {
            return Some(run_id_bundle_json);
        }

        let dir = result
            .last_bundle_dir
            .as_deref()
            .and_then(|s| (!s.trim().is_empty()).then_some(s.trim()))
            .map(PathBuf::from)
            .map(|p| if p.is_absolute() { p } else { out_dir.join(p) })
            .or_else(|| crate::compare::read_latest_pointer(out_dir))
            .or_else(|| crate::compare::find_latest_export_dir(out_dir));
        if let Some(dir) = dir {
            let bundle_path = resolve_bundle_artifact_path(&dir);
            if bundle_path.is_file() {
                return Some(bundle_path);
            }
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
    None
}

pub(crate) fn wait_for_bundle_artifact_in_dir(
    bundle_dir: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    use std::time::{Duration, Instant};

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.clamp(250, 5_000));
    let bundle_path = resolve_bundle_artifact_path(bundle_dir);
    while Instant::now() < deadline {
        if bundle_path.is_file() {
            return Some(bundle_path.clone());
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
    None
}
