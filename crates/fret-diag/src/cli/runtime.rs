use std::path::{Path, PathBuf};

use crate::{ResolvedRunContext, ResolvedScriptPaths, resolve_path, session, util::now_unix_ms};

#[derive(Debug, Clone, Default)]
pub(crate) struct DiagPathOverrides {
    pub out_dir: Option<PathBuf>,
    pub trigger_path: Option<PathBuf>,
    pub script_path: Option<PathBuf>,
    pub script_trigger_path: Option<PathBuf>,
    pub script_result_path: Option<PathBuf>,
    pub script_result_trigger_path: Option<PathBuf>,
    pub pick_trigger_path: Option<PathBuf>,
    pub pick_result_path: Option<PathBuf>,
    pub pick_result_trigger_path: Option<PathBuf>,
    pub pick_script_out: Option<PathBuf>,
    pub inspect_path: Option<PathBuf>,
    pub inspect_trigger_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolveDiagCliPathsRequest<'a> {
    pub workspace_root: &'a Path,
    pub sub: &'a str,
    pub launch: Option<&'a [String]>,
    pub session_auto: bool,
    pub session_id: Option<String>,
    pub overrides: DiagPathOverrides,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedDiagCliPaths {
    pub resolved_out_dir: PathBuf,
    pub resolved_run_context: ResolvedRunContext,
    pub resolved_pick_trigger_path: PathBuf,
    pub resolved_pick_result_path: PathBuf,
    pub resolved_pick_result_trigger_path: PathBuf,
    pub resolved_pick_script_out: PathBuf,
    pub resolved_inspect_path: PathBuf,
    pub resolved_inspect_trigger_path: PathBuf,
}

fn resolve_diag_path_override(
    workspace_root: &Path,
    session_enabled: bool,
    override_path: Option<PathBuf>,
    env_var: Option<&str>,
    default_path: impl FnOnce() -> PathBuf,
) -> PathBuf {
    let raw = if session_enabled {
        override_path.unwrap_or_else(default_path)
    } else {
        override_path
            .or_else(|| {
                env_var.and_then(|name| {
                    std::env::var_os(name)
                        .filter(|v| !v.is_empty())
                        .map(PathBuf::from)
                })
            })
            .unwrap_or_else(default_path)
    };
    resolve_path(workspace_root, raw)
}

pub(crate) fn resolve_diag_cli_paths(
    request: ResolveDiagCliPathsRequest<'_>,
) -> Result<ResolvedDiagCliPaths, String> {
    let ResolveDiagCliPathsRequest {
        workspace_root,
        sub,
        launch,
        session_auto,
        session_id,
        overrides,
    } = request;
    let session_enabled = session_auto || session_id.is_some();

    let DiagPathOverrides {
        out_dir,
        trigger_path,
        script_path,
        script_trigger_path,
        script_result_path,
        script_result_trigger_path,
        pick_trigger_path,
        pick_result_path,
        pick_result_trigger_path,
        pick_script_out,
        inspect_path,
        inspect_trigger_path,
    } = overrides;

    let resolved_base_out_dir = {
        let raw = out_dir
            .or_else(|| {
                std::env::var_os("FRET_DIAG_DIR")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| PathBuf::from("target").join("fret-diag"));
        resolve_path(workspace_root, raw)
    };

    if session_enabled {
        if launch.is_none() {
            return Err(
                "--session-auto/--session requires --launch (tool-launched only)".to_string(),
            );
        }

        let mut overrides: Vec<&'static str> = Vec::new();
        if trigger_path.is_some() {
            overrides.push("--trigger-path");
        }
        if script_path.is_some() {
            overrides.push("--script-path");
        }
        if script_trigger_path.is_some() {
            overrides.push("--script-trigger-path");
        }
        if script_result_path.is_some() {
            overrides.push("--script-result-path");
        }
        if script_result_trigger_path.is_some() {
            overrides.push("--script-result-trigger-path");
        }
        if pick_trigger_path.is_some() {
            overrides.push("--pick-trigger-path");
        }
        if pick_result_path.is_some() {
            overrides.push("--pick-result-path");
        }
        if pick_result_trigger_path.is_some() {
            overrides.push("--pick-result-trigger-path");
        }
        if inspect_path.is_some() {
            overrides.push("--inspect-path");
        }
        if inspect_trigger_path.is_some() {
            overrides.push("--inspect-trigger-path");
        }
        if !overrides.is_empty() {
            return Err(format!(
                "--session-auto/--session is not compatible with explicit path overrides ({})",
                overrides.join(", ")
            ));
        }
    }

    let resolved_out_dir = if session_enabled {
        let raw_id = session_id
            .unwrap_or_else(|| session::auto_session_id(now_unix_ms(), std::process::id()));
        let sid = session::sanitize_session_id(&raw_id);
        let out = session::session_out_dir(&resolved_base_out_dir, &sid);
        session::write_session_json_best_effort(&out, &resolved_base_out_dir, &sid, sub, launch);
        eprintln!(
            "diag session: base_out_dir={} session_id={} out_dir={}",
            resolved_base_out_dir.display(),
            sid,
            out.display()
        );
        out
    } else {
        resolved_base_out_dir.clone()
    };

    if launch.is_some() && !session_enabled {
        let sessions_root = resolved_out_dir.join(session::SESSIONS_DIRNAME);
        if sessions_root.is_dir() {
            eprintln!(
                "warning: diag --launch is writing control-plane files directly under a base dir that contains `sessions/`.\n\
  base_out_dir: {}\n\
  out_dir: {}\n\
  hint: prefer `--session-auto` (or `--session <id>`) to isolate concurrent runs under `{}`\n\
  example:\n\
    cargo run -p fretboard-dev -- diag {} --dir {} --session-auto --launch -- <cmd...>",
                resolved_base_out_dir.display(),
                resolved_out_dir.display(),
                sessions_root.display(),
                sub,
                resolved_base_out_dir.display(),
            );
        }
    }

    let resolved_trigger_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        trigger_path,
        Some("FRET_DIAG_TRIGGER_PATH"),
        || resolved_out_dir.join("trigger.touch"),
    );
    let resolved_ready_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        None,
        Some("FRET_DIAG_READY_PATH"),
        || resolved_out_dir.join("ready.touch"),
    );
    let resolved_exit_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        None,
        Some("FRET_DIAG_EXIT_PATH"),
        || resolved_out_dir.join("exit.touch"),
    );
    let resolved_script_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        script_path,
        Some("FRET_DIAG_SCRIPT_PATH"),
        || resolved_out_dir.join("script.json"),
    );
    let resolved_script_trigger_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        script_trigger_path,
        Some("FRET_DIAG_SCRIPT_TRIGGER_PATH"),
        || resolved_out_dir.join("script.touch"),
    );
    let resolved_script_result_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        script_result_path,
        Some("FRET_DIAG_SCRIPT_RESULT_PATH"),
        || resolved_out_dir.join("script.result.json"),
    );
    let resolved_script_result_trigger_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        script_result_trigger_path,
        Some("FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH"),
        || resolved_out_dir.join("script.result.touch"),
    );
    let resolved_pick_trigger_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        pick_trigger_path,
        Some("FRET_DIAG_PICK_TRIGGER_PATH"),
        || resolved_out_dir.join("pick.touch"),
    );
    let resolved_pick_result_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        pick_result_path,
        Some("FRET_DIAG_PICK_RESULT_PATH"),
        || resolved_out_dir.join("pick.result.json"),
    );
    let resolved_pick_result_trigger_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        pick_result_trigger_path,
        Some("FRET_DIAG_PICK_RESULT_TRIGGER_PATH"),
        || resolved_out_dir.join("pick.result.touch"),
    );
    let resolved_pick_script_out = resolve_path(
        workspace_root,
        pick_script_out.unwrap_or_else(|| resolved_out_dir.join("picked.script.json")),
    );
    let resolved_inspect_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        inspect_path,
        Some("FRET_DIAG_INSPECT_PATH"),
        || resolved_out_dir.join("inspect.json"),
    );
    let resolved_inspect_trigger_path = resolve_diag_path_override(
        workspace_root,
        session_enabled,
        inspect_trigger_path,
        Some("FRET_DIAG_INSPECT_TRIGGER_PATH"),
        || resolved_out_dir.join("inspect.touch"),
    );

    let resolved_paths = ResolvedScriptPaths {
        out_dir: resolved_out_dir.clone(),
        trigger_path: resolved_trigger_path.clone(),
        ready_path: resolved_ready_path,
        exit_path: resolved_exit_path,
        script_path: resolved_script_path.clone(),
        script_trigger_path: resolved_script_trigger_path.clone(),
        script_result_path: resolved_script_result_path.clone(),
        script_result_trigger_path: resolved_script_result_trigger_path.clone(),
    };
    let mut fs_transport_cfg = resolved_paths.launch_fs_transport_cfg();
    fs_transport_cfg.trigger_path = resolved_trigger_path;
    fs_transport_cfg.pick_trigger_path = resolved_pick_trigger_path.clone();
    fs_transport_cfg.pick_result_path = resolved_pick_result_path.clone();
    fs_transport_cfg.pick_result_trigger_path = resolved_pick_result_trigger_path.clone();
    fs_transport_cfg.inspect_path = resolved_inspect_path.clone();
    fs_transport_cfg.inspect_trigger_path = resolved_inspect_trigger_path.clone();

    Ok(ResolvedDiagCliPaths {
        resolved_out_dir,
        resolved_run_context: ResolvedRunContext {
            paths: resolved_paths,
            fs_transport_cfg,
        },
        resolved_pick_trigger_path,
        resolved_pick_result_path,
        resolved_pick_result_trigger_path,
        resolved_pick_script_out,
        resolved_inspect_path,
        resolved_inspect_trigger_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-cli-runtime-{label}-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp dir");
        root
    }

    #[test]
    fn resolve_diag_cli_paths_rejects_session_with_explicit_path_overrides() {
        let workspace_root = make_temp_dir("session-conflict");
        let err = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
            workspace_root: &workspace_root,
            sub: "run",
            launch: Some(&["cargo".to_string()]),
            session_auto: true,
            session_id: None,
            overrides: DiagPathOverrides {
                trigger_path: Some(PathBuf::from("custom.trigger")),
                ..DiagPathOverrides::default()
            },
        })
        .expect_err("session mode should reject explicit path overrides");

        assert_eq!(
            err,
            "--session-auto/--session is not compatible with explicit path overrides (--trigger-path)"
        );
    }

    #[test]
    fn resolve_diag_cli_paths_places_tool_launched_sessions_under_sessions_dir() {
        let workspace_root = make_temp_dir("session-layout");
        let launch_argv = ["cargo".to_string(), "run".to_string()];
        let resolved = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
            workspace_root: &workspace_root,
            sub: "suite",
            launch: Some(&launch_argv),
            session_auto: false,
            session_id: Some("smoke".to_string()),
            overrides: DiagPathOverrides {
                out_dir: Some(PathBuf::from("target/fret-diag")),
                ..DiagPathOverrides::default()
            },
        })
        .expect("resolve diag cli paths");

        assert!(
            resolved
                .resolved_out_dir
                .ends_with("target/fret-diag/sessions/smoke")
        );
        assert_eq!(
            resolved
                .resolved_run_context
                .paths
                .script_path
                .file_name()
                .and_then(|name| name.to_str()),
            Some("script.json")
        );
        assert_eq!(
            resolved
                .resolved_pick_script_out
                .file_name()
                .and_then(|name| name.to_str()),
            Some("picked.script.json")
        );
    }
}
