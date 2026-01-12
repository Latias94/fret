use std::path::Path;

pub use fret_runtime::keymap::*;

#[derive(Debug, thiserror::Error)]
pub enum KeymapFileError {
    #[error("failed to read keymap file: {path}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse keymap file: {path}")]
    Parse { path: String, source: KeymapError },
}

/// Loads a keymap file from disk and parses it into a `Keymap`.
///
/// Note: `fret-runtime` is intentionally IO-free; file IO lives at the app/runner boundary.
pub fn keymap_from_file(path: &Path) -> Result<Keymap, KeymapError> {
    let bytes = std::fs::read(path).map_err(|source| KeymapError::ReadFailed { source })?;
    Keymap::from_bytes(&bytes)
}

pub fn keymap_from_file_if_exists(
    path: impl AsRef<Path>,
) -> Result<Option<Keymap>, KeymapFileError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path).map_err(|source| KeymapFileError::Read {
        path: path.display().to_string(),
        source,
    })?;
    Keymap::from_bytes(&bytes)
        .map(Some)
        .map_err(|source| KeymapFileError::Parse {
            path: path.display().to_string(),
            source,
        })
}
