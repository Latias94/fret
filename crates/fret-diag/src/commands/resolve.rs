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

pub(crate) fn maybe_resolve_base_or_session_out_dir_to_latest_bundle_dir(path: &Path) -> PathBuf {
    if !path.is_dir() {
        return path.to_path_buf();
    }
    if crate::resolve_bundle_artifact_path_no_materialize(path).is_some() {
        // Already a bundle export dir (or a packed `_root` bundle dir).
        return path.to_path_buf();
    }
    if !(looks_like_diag_session_root(path) || path.join(crate::session::SESSIONS_DIRNAME).is_dir())
    {
        // Not a diagnostics out dir; treat as user-provided bundle dir input.
        return path.to_path_buf();
    }
    if let Ok((bundle_dir, _session_id, _source)) =
        resolve_latest_bundle_dir_from_base_or_session_out_dir(path, None)
    {
        return bundle_dir;
    }
    path.to_path_buf()
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
}
