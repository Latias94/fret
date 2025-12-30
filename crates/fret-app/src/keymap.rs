use std::path::Path;

pub use fret_runtime::keymap::*;

/// Loads a keymap file from disk and parses it into a `Keymap`.
///
/// Note: `fret-runtime` is intentionally IO-free; file IO lives at the app/runner boundary.
pub fn keymap_from_file(path: &Path) -> Result<Keymap, KeymapError> {
    let bytes = std::fs::read(path).map_err(|source| KeymapError::ReadFailed { source })?;
    Keymap::from_bytes(&bytes)
}
