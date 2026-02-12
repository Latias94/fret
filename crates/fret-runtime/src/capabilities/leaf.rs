use super::{
    ExternalDragPayloadKind, ExternalDragPositionQuality, WindowHoverDetectionQuality,
    WindowSetOuterPositionQuality, WindowZLevelQuality,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiCapabilities {
    pub multi_window: bool,
    pub window_tear_off: bool,
    pub cursor_icons: bool,

    pub window_hover_detection: WindowHoverDetectionQuality,
    pub window_set_outer_position: WindowSetOuterPositionQuality,
    pub window_z_level: WindowZLevelQuality,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipboardCapabilities {
    pub text: bool,
    pub files: bool,
    /// Linux/X11/Wayland primary selection text support.
    ///
    /// This is intentionally modeled as a capability separate from `clipboard.text`:
    /// on Linux, primary selection is typically used as "copy-on-select" + middle-click paste,
    /// and should not overwrite the explicit clipboard used by `Ctrl+C` / `edit.copy`.
    pub primary_text: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DndCapabilities {
    pub external: bool,
    pub external_payload: ExternalDragPayloadKind,
    pub external_position: ExternalDragPositionQuality,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ImeCapabilities {
    pub enabled: bool,
    pub set_cursor_area: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FsCapabilities {
    pub real_paths: bool,
    pub file_dialogs: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellCapabilities {
    pub open_url: bool,
    pub share_sheet: bool,
    pub incoming_open: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GfxCapabilities {
    pub webgpu: bool,
    pub native_gpu: bool,
}
