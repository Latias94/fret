use std::path::{Path, PathBuf};

use serde_json::json;

use crate::util::{now_unix_ms, sanitize_for_filename, write_json_value};

pub(crate) const SESSIONS_DIRNAME: &str = "sessions";

pub(crate) fn auto_session_id(unix_ms: u64, pid: u32) -> String {
    format!("{unix_ms}-{pid}")
}

pub(crate) fn sanitize_session_id(raw: &str) -> String {
    sanitize_for_filename(raw, 64, "session")
}

pub(crate) fn session_out_dir(base_out_dir: &Path, session_id: &str) -> PathBuf {
    base_out_dir.join(SESSIONS_DIRNAME).join(session_id)
}

pub(crate) fn write_session_json_best_effort(
    session_out_dir: &Path,
    base_out_dir: &Path,
    session_id: &str,
    diag_subcommand: &str,
    launch_cmd: Option<&[String]>,
) {
    let pid = std::process::id();
    let payload = json!({
        "schema_version": 1,
        "created_unix_ms": now_unix_ms(),
        "pid": pid,
        "session_id": session_id,
        "base_out_dir": base_out_dir.display().to_string(),
        "session_out_dir": session_out_dir.display().to_string(),
        "diag_subcommand": diag_subcommand,
        "launch_cmd": launch_cmd.map(|v| v.join(" ")),
    });
    let _ = write_json_value(&session_out_dir.join("session.json"), &payload);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_session_id_is_stable_for_inputs() {
        assert_eq!(auto_session_id(123, 456), "123-456");
    }

    #[test]
    fn sanitize_session_id_keeps_filename_safe() {
        let id = sanitize_session_id(" hello world / .. ");
        assert!(!id.trim().is_empty());
        assert!(!id.contains(' '));
        assert!(!id.contains('/'));
        assert!(!id.contains('\\'));
        assert!(!id.contains(':'));
    }

    #[test]
    fn session_out_dir_is_nested_under_sessions() {
        let base = Path::new("target/fret-diag");
        let got = session_out_dir(base, "abc");
        assert_eq!(
            got,
            PathBuf::from("target/fret-diag")
                .join("sessions")
                .join("abc")
        );
    }
}
