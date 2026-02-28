use std::path::{Path, PathBuf};

use serde_json::json;

use crate::util::{now_unix_ms, sanitize_for_filename, write_json_value};

pub(crate) const SESSIONS_DIRNAME: &str = "sessions";

#[derive(Debug, Clone)]
pub(crate) struct SessionInfo {
    pub session_id: String,
    pub session_dir: PathBuf,
    pub created_unix_ms: Option<u64>,
    pub pid: Option<u64>,
    pub diag_subcommand: Option<String>,
}

pub(crate) fn auto_session_id(unix_ms: u64, pid: u32) -> String {
    format!("{unix_ms}-{pid}")
}

pub(crate) fn sanitize_session_id(raw: &str) -> String {
    sanitize_for_filename(raw, 64, "session")
}

pub(crate) fn session_out_dir(base_out_dir: &Path, session_id: &str) -> PathBuf {
    base_out_dir.join(SESSIONS_DIRNAME).join(session_id)
}

pub(crate) fn collect_sessions(base_out_dir: &Path) -> Result<Vec<SessionInfo>, String> {
    let sessions_root = base_out_dir.join(SESSIONS_DIRNAME);
    if !sessions_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut out: Vec<SessionInfo> = Vec::new();
    let iter = std::fs::read_dir(&sessions_root).map_err(|e| e.to_string())?;
    for entry in iter.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let session_id = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("session")
            .to_string();

        let mut created_unix_ms: Option<u64> = None;
        let mut pid: Option<u64> = None;
        let mut diag_subcommand: Option<String> = None;

        let session_json = path.join("session.json");
        if let Ok(bytes) = std::fs::read(&session_json)
            && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes)
        {
            created_unix_ms = v.get("created_unix_ms").and_then(|v| v.as_u64());
            pid = v.get("pid").and_then(|v| v.as_u64());
            diag_subcommand = v
                .get("diag_subcommand")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        } else if let Some((prefix, _)) = session_id.split_once('-') {
            created_unix_ms = prefix.trim().parse::<u64>().ok();
        }

        out.push(SessionInfo {
            session_id,
            session_dir: path,
            created_unix_ms,
            pid,
            diag_subcommand,
        });
    }

    out.sort_by(|a, b| {
        b.created_unix_ms
            .unwrap_or(0)
            .cmp(&a.created_unix_ms.unwrap_or(0))
            .then_with(|| a.session_id.cmp(&b.session_id))
    });

    Ok(out)
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
