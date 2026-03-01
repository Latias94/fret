use std::path::Path;

use serde_json::json;

pub(crate) fn tooling_warnings_for_bundle_dir(bundle_dir: &Path) -> Vec<serde_json::Value> {
    let mut out: Vec<serde_json::Value> = Vec::new();
    if let Some(w) = concurrency_warning_for_bundle_dir(bundle_dir) {
        out.push(w);
    }
    out
}

fn find_nearest_base_out_dir_containing_sessions_dir(from: &Path) -> Option<&Path> {
    let mut cur: Option<&Path> = Some(from);
    while let Some(p) = cur {
        if p.join(crate::session::SESSIONS_DIRNAME).is_dir() {
            return Some(p);
        }
        cur = p.parent();
    }
    None
}

fn concurrency_warning_for_bundle_dir(bundle_dir: &Path) -> Option<serde_json::Value> {
    let base_out_dir = find_nearest_base_out_dir_containing_sessions_dir(bundle_dir)?;
    let sessions_root = base_out_dir.join(crate::session::SESSIONS_DIRNAME);
    if bundle_dir.starts_with(&sessions_root) {
        return None;
    }

    Some(json!({
        "severity": "warning",
        "code": "diag.concurrency.base_dir_contains_sessions_but_bundle_not_in_session",
        "message": "the bundle directory is under a base diagnostics dir that contains `sessions/`, but the bundle is not inside a session root; this usually means a tool-launched run wrote control-plane files at the base root (concurrency footgun for multiple terminals/AI agents)",
        "bundle_dir": bundle_dir.display().to_string(),
        "base_out_dir": base_out_dir.display().to_string(),
        "sessions_root": sessions_root.display().to_string(),
        "hint": "prefer: fretboard diag <run|suite|repro|perf|repeat|matrix> --dir <base_out_dir> --session-auto --launch -- <cmd...>",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_when_bundle_is_under_base_with_sessions_but_not_in_session_root() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-tooling-warnings-base-sessions-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(crate::session::SESSIONS_DIRNAME))
            .expect("create sessions dir");

        let bundle_dir = root.join("123-bundle");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");

        let warnings = tooling_warnings_for_bundle_dir(&bundle_dir);
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings[0]
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "diag.concurrency.base_dir_contains_sessions_but_bundle_not_in_session"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn does_not_warn_when_bundle_is_under_sessions_root() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-tooling-warnings-session-root-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let sessions_root = root.join(crate::session::SESSIONS_DIRNAME);
        std::fs::create_dir_all(&sessions_root).expect("create sessions dir");

        let bundle_dir = sessions_root.join("111-1").join("123-bundle");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");

        let warnings = tooling_warnings_for_bundle_dir(&bundle_dir);
        assert!(warnings.is_empty(), "expected no warning");

        let _ = std::fs::remove_dir_all(&root);
    }
}
