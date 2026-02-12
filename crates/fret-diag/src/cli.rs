use std::path::PathBuf;

pub(crate) fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("failed to locate workspace root (Cargo.toml not found in ancestors)".to_string())
}
