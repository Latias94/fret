use std::path::{Path, PathBuf};

use crate::artifact_lint::lint_run_artifact_dir;

fn resolve_run_dir_from_out_dir(
    out_dir: &Path,
    script_result_path: &Path,
) -> Result<PathBuf, String> {
    let bytes = std::fs::read(script_result_path).map_err(|e| {
        format!(
            "failed to read script.result.json to resolve latest run_id ({}): {}",
            script_result_path.display(),
            e
        )
    })?;
    let result: fret_diag_protocol::UiScriptResultV1 =
        serde_json::from_slice(&bytes).map_err(|e| {
            format!(
                "script.result.json was not valid UiScriptResultV1 JSON ({}): {}",
                script_result_path.display(),
                e
            )
        })?;
    Ok(out_dir.join(result.run_id.to_string()))
}

fn resolve_artifact_dir_input(
    rest: &[String],
    workspace_root: &Path,
    out_dir: &Path,
    script_result_path: &Path,
) -> Result<PathBuf, String> {
    if rest.is_empty() {
        return resolve_run_dir_from_out_dir(out_dir, script_result_path);
    }
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(&rest[0]));
    if src.is_file() {
        return Ok(src
            .parent()
            .ok_or_else(|| "invalid artifact path (no parent dir)".to_string())?
            .to_path_buf());
    }

    if src.is_dir() {
        let direct = src.join("manifest.json");
        if direct.is_file() {
            return Ok(src);
        }
        let root = src.join("_root").join("manifest.json");
        if root.is_file() {
            return Ok(src.join("_root"));
        }

        let script_result = src.join("script.result.json");
        if script_result.is_file() {
            return resolve_run_dir_from_out_dir(&src, &script_result);
        }
    }

    Ok(src)
}

pub(crate) fn cmd_artifact(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    script_result_path: &Path,
    artifact_lint_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let Some(sub) = rest.first().map(|s| s.as_str()) else {
        return Err(
            "missing artifact subcommand (try: fretboard diag artifact lint [<run_dir|out_dir>])"
                .to_string(),
        );
    };

    match sub {
        "lint" => cmd_artifact_lint(
            &rest[1..],
            workspace_root,
            out_dir,
            script_result_path,
            artifact_lint_out,
            warmup_frames,
            stats_json,
        ),
        other => Err(format!("unknown artifact subcommand: {other}")),
    }
}

fn cmd_artifact_lint(
    rest: &[String],
    workspace_root: &Path,
    out_dir: &Path,
    script_result_path: &Path,
    artifact_lint_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    let dir = resolve_artifact_dir_input(rest, workspace_root, out_dir, script_result_path)?;
    let report = lint_run_artifact_dir(&dir, warmup_frames)?;

    let out = artifact_lint_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| dir.join("artifact.lint.json"));

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(&report.payload).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

    if stats_json {
        println!("{pretty}");
    } else {
        println!("{}", out.display());
    }

    if report.error_issues > 0 {
        std::process::exit(1);
    }
    Ok(())
}
