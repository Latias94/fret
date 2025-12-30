use fret_core::DockLayout;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DockLayoutError {
    #[error("failed to read dock layout file: {path}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse dock layout file JSON: {path}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    #[error("failed to write dock layout file: {path}")]
    Write {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to serialize dock layout JSON: {path}")]
    Serialize {
        path: String,
        source: serde_json::Error,
    },
}

/// Dock layout persistence file.
///
/// This is intentionally a thin wrapper around `fret_core::DockLayout` serialization, and is
/// designed to live at the app layer (ADR 0013 / ADR 0014 / ADR 0017).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockLayoutFileV1 {
    pub layout: DockLayout,
}

impl DockLayoutFileV1 {
    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, DockLayoutError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| DockLayoutError::Read {
            path: path.display().to_string(),
            source,
        })?;

        // Backward compatibility: accept both the wrapped `{ "layout": ... }` form and a plain
        // `DockLayout` root object (historical demo format).
        match serde_json::from_slice::<Self>(&bytes) {
            Ok(v) => Ok(v),
            Err(new_err) => match serde_json::from_slice::<DockLayout>(&bytes) {
                Ok(layout) => Ok(Self { layout }),
                Err(_old_err) => Err(DockLayoutError::Parse {
                    path: path.display().to_string(),
                    source: new_err,
                }),
            },
        }
    }

    pub fn load_json_if_exists(path: impl AsRef<Path>) -> Result<Option<Self>, DockLayoutError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }
        Self::load_json(path).map(Some)
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), DockLayoutError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| DockLayoutError::Write {
                path: path.display().to_string(),
                source,
            })?;
        }

        let bytes =
            serde_json::to_vec_pretty(self).map_err(|source| DockLayoutError::Serialize {
                path: path.display().to_string(),
                source,
            })?;
        std::fs::write(path, bytes).map_err(|source| DockLayoutError::Write {
            path: path.display().to_string(),
            source,
        })
    }
}
