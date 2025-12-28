use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("failed to read settings file: {path}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse settings file JSON: {path}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
}

/// Project/user settings file (v1).
///
/// This is intentionally small and strongly typed. See ADR 0014.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsFileV1 {
    pub settings_version: u32,
    pub fonts: FontsSettingsV1,
}

impl Default for SettingsFileV1 {
    fn default() -> Self {
        Self {
            settings_version: 1,
            fonts: FontsSettingsV1::default(),
        }
    }
}

impl SettingsFileV1 {
    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, SettingsError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| SettingsError::Read {
            path: path.display().to_string(),
            source,
        })?;
        serde_json::from_slice(&bytes).map_err(|source| SettingsError::Parse {
            path: path.display().to_string(),
            source,
        })
    }

    pub fn load_json_if_exists(path: impl AsRef<Path>) -> Result<Option<Self>, SettingsError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }
        Self::load_json(path).map(Some)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FontsSettingsV1 {
    /// Ordered candidate families to use as the default sans-serif UI font.
    pub ui_sans: Vec<String>,
    /// Ordered candidate families to use as the default serif font.
    pub ui_serif: Vec<String>,
    /// Ordered candidate families to use as the default monospace font.
    pub ui_mono: Vec<String>,
}
