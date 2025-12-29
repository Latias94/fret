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
    pub docking: DockingSettingsV1,
}

impl Default for SettingsFileV1 {
    fn default() -> Self {
        Self {
            settings_version: 1,
            fonts: FontsSettingsV1::default(),
            docking: DockingSettingsV1::default(),
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DockingSettingsV1 {
    pub drag_inversion: DockDragInversionSettingsV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DockDragInversionSettingsV1 {
    pub modifier: DockDragInversionModifierV1,
    pub policy: DockDragInversionPolicyV1,
}

impl Default for DockDragInversionSettingsV1 {
    fn default() -> Self {
        Self {
            modifier: DockDragInversionModifierV1::Shift,
            policy: DockDragInversionPolicyV1::DockByDefault,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockDragInversionModifierV1 {
    None,
    #[default]
    Shift,
    Ctrl,
    Alt,
    AltGr,
    Meta,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockDragInversionPolicyV1 {
    #[default]
    DockByDefault,
    DockOnlyWhenModifier,
}

impl SettingsFileV1 {
    pub fn docking_interaction_settings(&self) -> fret_runtime::DockingInteractionSettings {
        fret_runtime::DockingInteractionSettings {
            drag_inversion: self.docking.drag_inversion.clone().into(),
        }
    }
}

impl From<DockDragInversionSettingsV1> for fret_runtime::DockDragInversionSettings {
    fn from(value: DockDragInversionSettingsV1) -> Self {
        Self {
            modifier: value.modifier.into(),
            policy: value.policy.into(),
        }
    }
}

impl From<DockDragInversionModifierV1> for fret_runtime::DockDragInversionModifier {
    fn from(value: DockDragInversionModifierV1) -> Self {
        match value {
            DockDragInversionModifierV1::None => Self::None,
            DockDragInversionModifierV1::Shift => Self::Shift,
            DockDragInversionModifierV1::Ctrl => Self::Ctrl,
            DockDragInversionModifierV1::Alt => Self::Alt,
            DockDragInversionModifierV1::AltGr => Self::AltGr,
            DockDragInversionModifierV1::Meta => Self::Meta,
        }
    }
}

impl From<DockDragInversionPolicyV1> for fret_runtime::DockDragInversionPolicy {
    fn from(value: DockDragInversionPolicyV1) -> Self {
        match value {
            DockDragInversionPolicyV1::DockByDefault => Self::DockByDefault,
            DockDragInversionPolicyV1::DockOnlyWhenModifier => Self::DockOnlyWhenModifier,
        }
    }
}
