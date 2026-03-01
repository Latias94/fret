use std::path::{Path, PathBuf};

use super::resolve;

pub(crate) fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
}

pub(crate) fn resolve_bundle_artifact_path_or_latest(
    bundle_arg: Option<&str>,
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    if let Some(s) = bundle_arg {
        let src = crate::resolve_path(workspace_root, PathBuf::from(s));
        let src = resolve::maybe_resolve_base_or_session_out_dir_to_latest_bundle_dir(&src);
        return Ok(crate::resolve_bundle_artifact_path(&src));
    }
    let latest = resolve_latest_bundle_dir_path(out_dir)?;
    Ok(crate::resolve_bundle_artifact_path(&latest))
}

pub(crate) fn resolve_latest_bundle_dir_path(out_dir: &Path) -> Result<PathBuf, String> {
    if let Ok((dir, _session_id, _source)) =
        resolve::resolve_latest_bundle_dir_from_base_or_session_out_dir(out_dir, None)
    {
        return Ok(dir);
    }
    crate::latest::resolve_latest_bundle_dir_path(out_dir)
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
        let payload = fret_diag_protocol::UiScriptResultV1 {
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
    fn resolve_latest_bundle_dir_path_accepts_base_dir_with_sessions() {
        let base = make_temp_dir("fret-diag-args-latest-base");

        let s1 = crate::session::session_out_dir(&base, "100-1");
        let s2 = crate::session::session_out_dir(&base, "200-1");
        std::fs::create_dir_all(&s1).expect("create session 1");
        std::fs::create_dir_all(&s2).expect("create session 2");

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

        let got = resolve_latest_bundle_dir_path(&base).expect("resolve latest for base");
        assert_eq!(got, bundle_dir);
    }

    #[test]
    fn resolve_bundle_artifact_path_accepts_base_dir_with_sessions() {
        let base = make_temp_dir("fret-diag-args-bundle-artifact-base");

        let s1 = crate::session::session_out_dir(&base, "100-1");
        let s2 = crate::session::session_out_dir(&base, "200-1");
        std::fs::create_dir_all(&s1).expect("create session 1");
        std::fs::create_dir_all(&s2).expect("create session 2");

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
        std::fs::write(bundle_dir.join("bundle.schema2.json"), b"{}").expect("write bundle");

        let base_str = base.to_string_lossy().to_string();
        let got = resolve_bundle_artifact_path_or_latest(
            Some(base_str.as_str()),
            Path::new("."),
            Path::new("."),
        )
        .expect("resolve bundle artifact");
        assert_eq!(got, bundle_dir.join("bundle.schema2.json"));
    }
}
