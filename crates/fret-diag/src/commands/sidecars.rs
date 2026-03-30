use std::fmt;
use std::path::{Path, PathBuf};

use serde_json::Value;

#[derive(Debug, Clone)]
pub(crate) enum SidecarReadError {
    Io(String),
    Json(String),
    KindMismatch {
        expected: String,
        got: Option<String>,
    },
    SchemaVersionMismatch {
        expected: u64,
        got: Option<u64>,
    },
    WarmupFramesMismatch {
        expected: u64,
        got: Option<u64>,
    },
}

impl fmt::Display for SidecarReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json parse error: {err}"),
            Self::KindMismatch { expected, got } => {
                write!(f, "kind mismatch (expected={expected} got={:?})", got)
            }
            Self::SchemaVersionMismatch { expected, got } => write!(
                f,
                "schema_version mismatch (expected={expected} got={:?})",
                got
            ),
            Self::WarmupFramesMismatch { expected, got } => write!(
                f,
                "warmup_frames mismatch (expected={expected} got={:?})",
                got
            ),
        }
    }
}

pub(crate) fn read_sidecar_json_v1(
    path: &Path,
    kind: &str,
    warmup_frames: u64,
) -> Result<Value, SidecarReadError> {
    let bytes = std::fs::read(path).map_err(|e| SidecarReadError::Io(e.to_string()))?;
    let v: Value =
        serde_json::from_slice(&bytes).map_err(|e| SidecarReadError::Json(e.to_string()))?;

    let got_kind = v
        .get("kind")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    if got_kind.as_deref() != Some(kind) {
        return Err(SidecarReadError::KindMismatch {
            expected: kind.to_string(),
            got: got_kind,
        });
    }

    let got_schema = v.get("schema_version").and_then(|v| v.as_u64());
    if got_schema != Some(1) {
        return Err(SidecarReadError::SchemaVersionMismatch {
            expected: 1,
            got: got_schema,
        });
    }

    let got_warmup = v.get("warmup_frames").and_then(|v| v.as_u64());
    if got_warmup != Some(warmup_frames) {
        return Err(SidecarReadError::WarmupFramesMismatch {
            expected: warmup_frames,
            got: got_warmup,
        });
    }

    Ok(v)
}

pub(crate) fn try_read_sidecar_json_v1(
    path: &Path,
    kind: &str,
    warmup_frames: u64,
) -> Option<Value> {
    read_sidecar_json_v1(path, kind, warmup_frames).ok()
}

pub(crate) fn adjacent_bundle_path_for_sidecar(path: &Path) -> Option<PathBuf> {
    let parent = path.parent()?;

    // Common layouts:
    // - <bundle_dir>/<bundle artifact> + <bundle_dir>/bundle.index.json
    // - <bundle_dir>/_root/<bundle artifact> (packed zip extraction)
    let bundle = crate::resolve_bundle_artifact_path(parent);
    if bundle.is_file() {
        return Some(bundle);
    }

    // Best-effort: if the sidecar is under `_root/`, also try the bundle dir.
    if parent.file_name().and_then(|s| s.to_str()) == Some("_root")
        && let Some(grandparent) = parent.parent()
    {
        let bundle = crate::resolve_bundle_artifact_path(grandparent);
        if bundle.is_file() {
            return Some(bundle);
        }
    }

    None
}
