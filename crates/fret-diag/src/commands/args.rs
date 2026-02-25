use std::path::{Path, PathBuf};

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
        return Ok(crate::resolve_bundle_artifact_path(&src));
    }
    let latest = crate::latest::resolve_latest_bundle_dir_path(out_dir)?;
    Ok(crate::resolve_bundle_artifact_path(&latest))
}

pub(crate) fn resolve_latest_bundle_dir_path(out_dir: &Path) -> Result<PathBuf, String> {
    crate::latest::resolve_latest_bundle_dir_path(out_dir)
}
