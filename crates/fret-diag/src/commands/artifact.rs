use std::path::{Path, PathBuf};

use crate::artifact_lint::lint_run_artifact_dir;

use super::resolve;

struct ArtifactLintCommandRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    out_dir: &'a Path,
    script_result_path: &'a Path,
    artifact_lint_out: Option<PathBuf>,
}

struct PreparedArtifactLintCommand {
    dir: PathBuf,
    out: PathBuf,
}

fn resolve_run_dir_from_out_dir(
    out_dir: &Path,
    script_result_path: &Path,
) -> Result<PathBuf, String> {
    let result =
        resolve::read_script_result_v1_or_err(script_result_path, "to resolve latest run_id")?;
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

fn default_artifact_lint_out_path(dir: &Path) -> PathBuf {
    dir.join("artifact.lint.json")
}

fn prepare_cmd_artifact_lint(
    request: ArtifactLintCommandRequest<'_>,
) -> Result<PreparedArtifactLintCommand, String> {
    let dir = resolve_artifact_dir_input(
        request.rest,
        request.workspace_root,
        request.out_dir,
        request.script_result_path,
    )?;
    let out = request
        .artifact_lint_out
        .map(|path| crate::resolve_path(request.workspace_root, path))
        .unwrap_or_else(|| default_artifact_lint_out_path(&dir));

    Ok(PreparedArtifactLintCommand { dir, out })
}

fn write_artifact_lint_output(
    out: &Path,
    payload: &serde_json::Value,
    stats_json: bool,
) -> Result<(), String> {
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(payload).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(out, pretty.as_bytes()).map_err(|e| e.to_string())?;

    if stats_json {
        println!("{pretty}");
    } else {
        println!("{}", out.display());
    }
    Ok(())
}

fn artifact_lint_exit_required(error_issues: u64) -> bool {
    error_issues > 0
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
    let PreparedArtifactLintCommand { dir, out } =
        prepare_cmd_artifact_lint(ArtifactLintCommandRequest {
            rest,
            workspace_root,
            out_dir,
            script_result_path,
            artifact_lint_out,
        })?;
    let report = lint_run_artifact_dir(&dir, warmup_frames)?;

    write_artifact_lint_output(&out, &report.payload, stats_json)?;

    if artifact_lint_exit_required(report.error_issues) {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_artifact_lint_out_path_appends_sidecar_name() {
        let out = default_artifact_lint_out_path(Path::new("captures/run-1/_root"));

        assert_eq!(
            out,
            PathBuf::from("captures/run-1/_root/artifact.lint.json")
        );
    }

    #[test]
    fn artifact_lint_exit_required_only_when_errors_exist() {
        assert!(!artifact_lint_exit_required(0));
        assert!(artifact_lint_exit_required(1));
        assert!(artifact_lint_exit_required(2));
    }

    #[test]
    fn resolve_artifact_dir_input_prefers_root_manifest_dir() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-artifact-dir-input-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let artifact_dir = root.join("run");
        std::fs::create_dir_all(artifact_dir.join("_root")).expect("create artifact root dir");
        std::fs::write(artifact_dir.join("_root").join("manifest.json"), b"{}")
            .expect("write manifest");

        let rest = vec![artifact_dir.display().to_string()];
        let resolved = resolve_artifact_dir_input(
            &rest,
            Path::new("."),
            Path::new("unused-out-dir"),
            Path::new("unused-script-result"),
        )
        .expect("resolve artifact dir");

        assert_eq!(resolved, artifact_dir.join("_root"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
