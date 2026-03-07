use std::path::{Path, PathBuf};

use fret_diag_protocol::UiScriptResultV1;

#[derive(Debug, Clone)]
struct ResolvedSessionOutDir {
    base_out_dir: PathBuf,
    out_dir: PathBuf,
    session_id: Option<String>,
}

#[derive(Debug, Clone)]
struct ResolveLatestOutput {
    base_out_dir: PathBuf,
    out_dir: PathBuf,
    session_id: Option<String>,
    latest_run_id: Option<u64>,
    latest_run_dir: Option<PathBuf>,
    latest_bundle_dir: Option<PathBuf>,
    latest_bundle_dir_source: Option<&'static str>,
    latest_bundle_artifact: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedBundleInput {
    pub bundle_dir: PathBuf,
    #[allow(dead_code)]
    pub bundle_artifact: PathBuf,
    pub artifacts_root: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedBundleRef {
    pub bundle_dir: PathBuf,
    pub bundle_artifact: PathBuf,
}

pub(crate) fn cmd_resolve(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let Some(kind) = rest.first().map(|s| s.as_str()) else {
        return Err("missing resolve target (try: fretboard diag resolve latest)".to_string());
    };

    match kind {
        "latest" => cmd_resolve_latest(&rest[1..], workspace_root, out_dir, json),
        other => Err(format!("unknown diag resolve target: {other}")),
    }
}

fn cmd_resolve_latest(
    rest: &[String],
    workspace_root: &Path,
    base_out_dir: &Path,
    json: bool,
) -> Result<(), String> {
    let mut within_session: Option<String> = None;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--within-session" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --within-session".to_string());
                };
                within_session = Some(v.to_string());
                i += 2;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag resolve latest flag: {other}"));
            }
            other => {
                return Err(format!("unexpected argument: {other}"));
            }
        }
    }

    let base_out_dir = crate::resolve_path(workspace_root, base_out_dir.to_path_buf());
    let resolved = resolve_session_out_dir_for_base_dir(&base_out_dir, within_session.as_deref())?;
    let out = resolve_latest_for_out_dir(&resolved)?;

    if json {
        let payload = serde_json::json!({
            "schema_version": 1,
            "base_out_dir": out.base_out_dir.display().to_string(),
            "out_dir": out.out_dir.display().to_string(),
            "session_id": out.session_id,
            "latest_run_id": out.latest_run_id,
            "latest_run_dir": out.latest_run_dir.as_ref().map(|p| p.display().to_string()),
            "latest_bundle_dir": out.latest_bundle_dir.as_ref().map(|p| p.display().to_string()),
            "latest_bundle_dir_source": out.latest_bundle_dir_source,
            "latest_bundle_artifact": out.latest_bundle_artifact.as_ref().map(|p| p.display().to_string()),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    println!("resolve_latest:");
    println!("  base_out_dir: {}", out.base_out_dir.display());
    println!("  out_dir: {}", out.out_dir.display());
    if let Some(id) = out.session_id.as_deref() {
        println!("  session_id: {}", id);
    }
    match out.latest_run_id {
        Some(id) => println!("  latest_run_id: {}", id),
        None => println!("  latest_run_id: null"),
    }
    match out.latest_run_dir.as_deref() {
        Some(p) => println!("  latest_run_dir: {}", p.display()),
        None => println!("  latest_run_dir: null"),
    }
    match out.latest_bundle_dir.as_deref() {
        Some(p) => println!("  latest_bundle_dir: {}", p.display()),
        None => println!("  latest_bundle_dir: null"),
    }
    match out.latest_bundle_dir_source {
        Some(s) => println!("  latest_bundle_dir_source: {}", s),
        None => println!("  latest_bundle_dir_source: null"),
    }
    match out.latest_bundle_artifact.as_deref() {
        Some(p) => println!("  latest_bundle_artifact: {}", p.display()),
        None => println!("  latest_bundle_artifact: null"),
    }

    Ok(())
}

pub(crate) fn looks_like_diag_session_root(dir: &Path) -> bool {
    dir.join("script.result.json").is_file()
        || dir.join("latest.txt").is_file()
        || dir.join("diag.config.json").is_file()
        || dir.join("trigger.touch").is_file()
        || dir.join("ready.touch").is_file()
        || dir.join("exit.touch").is_file()
}

pub(crate) fn find_nearest_script_result_json_preferring_evidence(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    let mut first_found: Option<PathBuf> = None;
    for _ in 0..10 {
        let Some(dir) = cur else { break };
        let direct = dir.join("script.result.json");
        if direct.is_file() {
            if first_found.is_none() {
                first_found = Some(direct.clone());
            }
            if let Some(result) = try_read_script_result_v1(&direct)
                && result.evidence.is_some()
            {
                return Some(direct);
            }
        }
        cur = dir.parent();
    }
    first_found
}

pub(crate) fn resolve_base_or_session_out_dir_to_latest_bundle_dir_or_err(
    path: &Path,
) -> Result<PathBuf, String> {
    if !path.is_dir() {
        return Ok(path.to_path_buf());
    }
    if crate::resolve_bundle_artifact_path_no_materialize(path).is_some() {
        // Already a bundle export dir (or a packed `_root` bundle dir).
        return Ok(path.to_path_buf());
    }
    if !(looks_like_diag_session_root(path) || path.join(crate::session::SESSIONS_DIRNAME).is_dir())
    {
        // Not a diagnostics out dir; treat as user-provided bundle dir input.
        return Ok(path.to_path_buf());
    }

    let (bundle_dir, _session_id, _source) =
        resolve_latest_bundle_dir_from_base_or_session_out_dir(path, None)?;
    Ok(bundle_dir)
}

pub(crate) fn resolve_base_or_session_out_dir_to_latest_bundle_dir_or_self(path: &Path) -> PathBuf {
    resolve_base_or_session_out_dir_to_latest_bundle_dir_or_err(path)
        .unwrap_or_else(|_| path.to_path_buf())
}

pub(crate) fn resolve_script_result_json_path_or_latest(
    src: Option<&Path>,
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    let Some(src) = src else {
        let bundle_path =
            super::args::resolve_bundle_artifact_path_or_latest(None, workspace_root, out_dir)?;
        let start = bundle_path.parent().unwrap_or_else(|| Path::new("."));
        return find_nearest_script_result_json_preferring_evidence(start).ok_or_else(|| {
            format!(
                "failed to locate script.result.json near latest bundle: {}",
                bundle_path.display()
            )
        });
    };

    let start = if src.is_dir() {
        let direct = src.join("script.result.json");
        if direct.is_file() {
            src.to_path_buf()
        } else {
            resolve_base_or_session_out_dir_to_latest_bundle_dir_or_self(src)
        }
    } else if src.is_file()
        && src
            .file_name()
            .is_some_and(|s| s.eq_ignore_ascii_case("script.result.json"))
    {
        src.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
    } else {
        src.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
    };

    find_nearest_script_result_json_preferring_evidence(&start).ok_or_else(|| {
        format!(
            "failed to locate script.result.json near: {}\n\
hint: pass a diagnostics out dir (or bundle dir) that contains script.result.json",
            src.display()
        )
    })
}

pub(crate) fn resolve_bundle_input_or_latest(
    src: &Path,
    default_out_dir: &Path,
) -> Result<ResolvedBundleInput, String> {
    let source_path = if src.as_os_str().is_empty() {
        super::args::resolve_latest_bundle_dir_path(default_out_dir).map_err(|_err| {
            format!(
                "no diagnostics bundle found under {}",
                default_out_dir.display()
            )
        })?
    } else {
        src.to_path_buf()
    };

    let source_path = resolve_base_or_session_out_dir_to_latest_bundle_dir_or_err(&source_path)?;
    let bundle_dir = crate::resolve_bundle_root_dir(&source_path)?;
    let bundle_artifact = crate::resolve_bundle_artifact_path(&bundle_dir);
    let artifacts_root = if bundle_dir.starts_with(default_out_dir) {
        default_out_dir.to_path_buf()
    } else {
        bundle_dir.parent().unwrap_or(default_out_dir).to_path_buf()
    };

    Ok(ResolvedBundleInput {
        bundle_dir,
        bundle_artifact,
        artifacts_root,
    })
}

pub(crate) fn resolve_bundle_ref(src: &Path) -> Result<ResolvedBundleRef, String> {
    let src = resolve_base_or_session_out_dir_to_latest_bundle_dir_or_err(src)?;
    let bundle_dir = crate::resolve_bundle_root_dir(&src)?;
    let bundle_artifact = crate::resolve_bundle_artifact_path(&bundle_dir);
    Ok(ResolvedBundleRef {
        bundle_dir,
        bundle_artifact,
    })
}

fn resolve_session_out_dir_for_base_dir(
    base_out_dir: &Path,
    within_session: Option<&str>,
) -> Result<ResolvedSessionOutDir, String> {
    let sessions_root = base_out_dir.join(crate::session::SESSIONS_DIRNAME);
    let has_sessions_root = sessions_root.is_dir();

    // If the directory already looks like a session root, treat it as such even if it contains
    // `sessions/` (defensive against nested layouts).
    if looks_like_diag_session_root(base_out_dir) || !has_sessions_root {
        return Ok(ResolvedSessionOutDir {
            base_out_dir: base_out_dir.to_path_buf(),
            out_dir: base_out_dir.to_path_buf(),
            session_id: None,
        });
    }

    let want = within_session.unwrap_or("latest").trim();
    let sid = if want.is_empty() || want == "latest" {
        let sessions = crate::session::collect_sessions(base_out_dir)?;
        let Some(first) = sessions.first() else {
            return Err(format!(
                "no sessions found under base_out_dir: {}\n\
hint: list sessions via `fretboard diag list sessions --dir {}`",
                base_out_dir.display(),
                base_out_dir.display()
            ));
        };
        first.session_id.clone()
    } else {
        crate::session::sanitize_session_id(want)
    };

    let out_dir = crate::session::session_out_dir(base_out_dir, &sid);
    if !out_dir.is_dir() {
        return Err(format!(
            "session directory does not exist: {}\n\
hint: list sessions via `fretboard diag list sessions --dir {}`",
            out_dir.display(),
            base_out_dir.display()
        ));
    }

    Ok(ResolvedSessionOutDir {
        base_out_dir: base_out_dir.to_path_buf(),
        out_dir,
        session_id: Some(sid),
    })
}

fn resolve_latest_for_out_dir(
    session: &ResolvedSessionOutDir,
) -> Result<ResolveLatestOutput, String> {
    let out_dir = &session.out_dir;

    let script_result = try_read_script_result_v1(&out_dir.join("script.result.json"));
    let latest_run_id = script_result.as_ref().map(|r| r.run_id).filter(|v| *v != 0);
    let latest_run_dir = latest_run_id.and_then(|id| {
        let dir = out_dir.join(id.to_string());
        dir.is_dir().then_some(dir)
    });

    let (latest_bundle_dir, latest_bundle_dir_source) =
        resolve_latest_bundle_dir_for_out_dir(out_dir, script_result.as_ref());
    let latest_bundle_artifact = latest_bundle_dir
        .as_deref()
        .map(crate::resolve_bundle_artifact_path);

    Ok(ResolveLatestOutput {
        base_out_dir: session.base_out_dir.clone(),
        out_dir: out_dir.clone(),
        session_id: session.session_id.clone(),
        latest_run_id,
        latest_run_dir,
        latest_bundle_dir,
        latest_bundle_dir_source,
        latest_bundle_artifact: latest_bundle_artifact.filter(|p| p.is_file()),
    })
}

pub(crate) fn try_read_script_result_v1(path: &Path) -> Option<UiScriptResultV1> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice::<UiScriptResultV1>(&bytes).ok()
}

pub(crate) fn resolve_latest_bundle_dir_for_out_dir(
    out_dir: &Path,
    script_result: Option<&UiScriptResultV1>,
) -> (Option<PathBuf>, Option<&'static str>) {
    if let Some(script) = script_result {
        if let Some(dir) = script
            .last_bundle_dir
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            let p = PathBuf::from(dir);
            let candidate = if p.is_absolute() { p } else { out_dir.join(p) };
            if candidate.is_dir() {
                return (Some(candidate), Some("script.result.json:last_bundle_dir"));
            }
        }
    }

    if let Some(dir) = crate::latest::latest_bundle_dir_path_opt(out_dir) {
        return (Some(dir), Some("latest.txt_or_scan"));
    }

    (None, None)
}

pub(crate) fn resolve_latest_bundle_dir_from_base_or_session_out_dir(
    base_or_session_out_dir: &Path,
    within_session: Option<&str>,
) -> Result<(PathBuf, Option<String>, &'static str), String> {
    let resolved = resolve_session_out_dir_for_base_dir(base_or_session_out_dir, within_session)?;
    let script_result = try_read_script_result_v1(&resolved.out_dir.join("script.result.json"));
    let (dir, source) =
        resolve_latest_bundle_dir_for_out_dir(&resolved.out_dir, script_result.as_ref());
    let Some(dir) = dir else {
        return Err(format!(
            "no diagnostics bundle found under {}",
            resolved.out_dir.display()
        ));
    };
    Ok((dir, resolved.session_id, source.unwrap_or("unknown")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("{prefix}-{}", crate::util::now_unix_ms()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_script_result(out_dir: &Path, run_id: u64, last_bundle_dir: &str) {
        let payload = UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: 1,
            window: None,
            stage: fret_diag_protocol::UiScriptStageV1::Passed,
            step_index: None,
            reason_code: Some("ok".to_string()),
            reason: None,
            evidence: None,
            last_bundle_dir: Some(last_bundle_dir.to_string()),
            last_bundle_artifact: None,
        };
        let bytes = serde_json::to_vec_pretty(&payload).expect("serialize script.result");
        std::fs::write(out_dir.join("script.result.json"), bytes).expect("write script.result");
    }

    #[test]
    fn resolve_latest_bundle_dir_prefers_script_result() {
        let root = make_temp_dir("fret-diag-resolve-latest-script-result");
        let bundle_dir = root.join("123-ui-gallery");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        write_script_result(&root, 123, "123-ui-gallery");

        let script = try_read_script_result_v1(&root.join("script.result.json"));
        let (dir, source) = resolve_latest_bundle_dir_for_out_dir(&root, script.as_ref());
        assert_eq!(dir.as_deref(), Some(bundle_dir.as_path()));
        assert_eq!(source, Some("script.result.json:last_bundle_dir"));
    }

    #[test]
    fn resolve_latest_bundle_dir_falls_back_to_latest_txt() {
        let root = make_temp_dir("fret-diag-resolve-latest-latest-txt");
        let bundle_dir = root.join("777-bundle");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(root.join("latest.txt"), b"777-bundle").expect("write latest.txt");

        let (dir, source) = resolve_latest_bundle_dir_for_out_dir(&root, None);
        assert_eq!(dir.as_deref(), Some(bundle_dir.as_path()));
        assert_eq!(source, Some("latest.txt_or_scan"));
    }

    #[test]
    fn resolve_latest_from_base_dir_picks_latest_session_by_default() {
        let base = make_temp_dir("fret-diag-resolve-latest-base");

        let s1 = crate::session::session_out_dir(&base, "100-1");
        let s2 = crate::session::session_out_dir(&base, "200-1");
        std::fs::create_dir_all(&s1).expect("create session 1");
        std::fs::create_dir_all(&s2).expect("create session 2");

        // Make session 2 newer via session.json.
        let _ = crate::util::write_json_value(
            &s1.join("session.json"),
            &serde_json::json!({"schema_version":1,"created_unix_ms":100,"pid":1,"session_id":"100-1"}),
        );
        let _ = crate::util::write_json_value(
            &s2.join("session.json"),
            &serde_json::json!({"schema_version":1,"created_unix_ms":200,"pid":1,"session_id":"200-1"}),
        );

        let bundle_dir = s2.join("999-bundle");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        write_script_result(&s2, 999, "999-bundle");

        let (dir, sid, source) =
            resolve_latest_bundle_dir_from_base_or_session_out_dir(&base, None)
                .expect("resolve latest from base");
        assert_eq!(sid.as_deref(), Some("200-1"));
        assert_eq!(dir, bundle_dir);
        assert_eq!(source, "script.result.json:last_bundle_dir");
    }

    #[test]
    fn resolve_bundle_input_or_latest_returns_bundle_artifact_and_root() {
        let root = make_temp_dir("fret-diag-resolve-bundle-input");
        let bundle_dir = root.join("123-bundle");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.meta.json"), b"{}" as &[u8])
            .expect("write bundle.meta.json");

        let resolved = resolve_bundle_input_or_latest(&bundle_dir, &root).expect("resolve input");
        assert_eq!(resolved.bundle_dir, bundle_dir);
        assert_eq!(resolved.bundle_artifact, bundle_dir.join("bundle.json"));
        assert_eq!(resolved.artifacts_root, root);
    }
}
