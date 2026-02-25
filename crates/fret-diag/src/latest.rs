use std::path::{Path, PathBuf};

pub(crate) fn latest_bundle_dir_candidates(out_dir: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    (
        crate::read_latest_pointer(out_dir),
        crate::find_latest_export_dir(out_dir),
    )
}

pub(crate) fn latest_bundle_dir_path_opt(out_dir: &Path) -> Option<PathBuf> {
    crate::read_latest_pointer(out_dir).or_else(|| crate::find_latest_export_dir(out_dir))
}

pub(crate) fn resolve_latest_bundle_dir_path(out_dir: &Path) -> Result<PathBuf, String> {
    latest_bundle_dir_path_opt(out_dir)
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))
}
