use serde::{Deserialize, Serialize};

pub mod keys {
    pub const EXEC_BACKGROUND_WORK: &str = "exec.background_work";
    pub const EXEC_WAKE: &str = "exec.wake";
    pub const EXEC_TIMERS: &str = "exec.timers";

    pub const UI_MULTI_WINDOW: &str = "ui.multi_window";
    pub const UI_WINDOW_TEAR_OFF: &str = "ui.window_tear_off";
    pub const UI_CURSOR_ICONS: &str = "ui.cursor_icons";

    pub const UI_WINDOW_HOVER_DETECTION: &str = "ui.window_hover_detection";
    pub const UI_WINDOW_SET_OUTER_POSITION: &str = "ui.window_set_outer_position";
    pub const UI_WINDOW_Z_LEVEL: &str = "ui.window_z_level";

    pub const CLIPBOARD_TEXT: &str = "clipboard.text";
    pub const CLIPBOARD_FILES: &str = "clipboard.files";

    pub const DND_EXTERNAL: &str = "dnd.external";
    pub const DND_EXTERNAL_PAYLOAD: &str = "dnd.external_payload";
    /// Indicates the quality of pointer position updates during external OS drag sessions.
    ///
    /// This is intentionally a capability (not a widget behavior fork): components may use it
    /// to decide whether "drag hover" UX is reliable (e.g. highlight drop targets) or should be
    /// treated as best-effort / drop-only.
    pub const DND_EXTERNAL_POSITION: &str = "dnd.external_position";

    pub const IME: &str = "ime";
    pub const IME_ENABLED: &str = "ime.enabled";
    pub const IME_SET_CURSOR_AREA: &str = "ime.set_cursor_area";

    pub const FS_REAL_PATHS: &str = "fs.real_paths";
    pub const FS_FILE_DIALOGS: &str = "fs.file_dialogs";

    pub const SHELL_OPEN_URL: &str = "shell.open_url";

    pub const GFX_WEBGPU: &str = "gfx.webgpu";
    /// Indicates that a native GPU rendering backend is available.
    ///
    /// This is intentionally not named after a Rust crate (e.g. `wgpu`) so the contract remains
    /// portable across backends and future implementations.
    pub const GFX_NATIVE_GPU: &str = "gfx.native_gpu";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityValueKind {
    Bool,
    Str,
}

pub const KNOWN_BOOL_CAPABILITY_KEYS: &[&str] = &[
    keys::UI_MULTI_WINDOW,
    keys::UI_WINDOW_TEAR_OFF,
    keys::UI_CURSOR_ICONS,
    keys::CLIPBOARD_TEXT,
    keys::CLIPBOARD_FILES,
    keys::DND_EXTERNAL,
    keys::IME,
    keys::IME_ENABLED,
    keys::IME_SET_CURSOR_AREA,
    keys::FS_REAL_PATHS,
    keys::FS_FILE_DIALOGS,
    keys::SHELL_OPEN_URL,
    keys::GFX_WEBGPU,
    keys::GFX_NATIVE_GPU,
];

pub const KNOWN_STR_CAPABILITY_KEYS: &[&str] = &[
    keys::EXEC_BACKGROUND_WORK,
    keys::EXEC_WAKE,
    keys::EXEC_TIMERS,
    keys::UI_WINDOW_HOVER_DETECTION,
    keys::UI_WINDOW_SET_OUTER_POSITION,
    keys::UI_WINDOW_Z_LEVEL,
    keys::DND_EXTERNAL_PAYLOAD,
    keys::DND_EXTERNAL_POSITION,
];

pub fn capability_key_kind(key: &str) -> Option<CapabilityValueKind> {
    if KNOWN_BOOL_CAPABILITY_KEYS.contains(&key) {
        return Some(CapabilityValueKind::Bool);
    }
    if KNOWN_STR_CAPABILITY_KEYS.contains(&key) {
        return Some(CapabilityValueKind::Str);
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExternalDragPayloadKind {
    None,
    FileToken,
    #[default]
    Text,
}

impl ExternalDragPayloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FileToken => "file_token",
            Self::Text => "text",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecBackgroundWork {
    #[default]
    Threads,
    Cooperative,
    None,
}

impl ExecBackgroundWork {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Cooperative => "cooperative",
            Self::Threads => "threads",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use ExecBackgroundWork::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (Cooperative, Cooperative | Threads) => Cooperative,
            (Threads, Threads) => Threads,
            (Threads, Cooperative) => Cooperative,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecWake {
    #[default]
    Reliable,
    BestEffort,
    None,
}

impl ExecWake {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use ExecWake::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecTimers {
    #[default]
    Reliable,
    BestEffort,
    None,
}

impl ExecTimers {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use ExecTimers::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExecCapabilities {
    pub background_work: ExecBackgroundWork,
    pub wake: ExecWake,
    pub timers: ExecTimers,
}

/// Quality of cursor/position updates during external OS drag sessions (e.g. file drag hover).
///
/// This is used to express *degradation*, not just availability:
/// a backend may support external drops but not provide reliable per-frame hover coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExternalDragPositionQuality {
    /// External drag is unsupported (or position updates are unavailable).
    None,
    /// The backend provides external drag events, but pointer positions are best-effort / may be
    /// stale or missing (e.g. macOS winit file DnD hover limitations).
    BestEffort,
    /// The backend provides reliable pointer position updates during external drag hover.
    #[default]
    Continuous,
}

impl ExternalDragPositionQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Continuous => "continuous",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use ExternalDragPositionQuality::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Continuous) => BestEffort,
            (Continuous, Continuous) => Continuous,
            (Continuous, BestEffort) => BestEffort,
        }
    }
}

/// Windowing quality signal: whether the backend can reliably determine which window is under the
/// cursor.
///
/// This is used as a degradation signal for editor-grade multi-window UX (e.g. docking tear-off
/// hover target selection under overlap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowHoverDetectionQuality {
    /// The backend cannot reliably determine window-under-cursor (or cannot provide global cursor
    /// position updates needed to infer it).
    None,
    /// Best-effort: selection may be stale/missing or ambiguous under overlap.
    BestEffort,
    /// Reliable enough for editor-grade hover selection.
    #[default]
    Reliable,
}

impl WindowHoverDetectionQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use WindowHoverDetectionQuality::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}

/// Windowing quality signal: whether programmatic window movement via outer-position requests is
/// reliable enough for "follow cursor" UX.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowSetOuterPositionQuality {
    None,
    BestEffort,
    #[default]
    Reliable,
}

impl WindowSetOuterPositionQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use WindowSetOuterPositionQuality::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}

/// Windowing quality signal: whether OS z-level requests (e.g. AlwaysOnTop during drags) behave
/// predictably.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowZLevelQuality {
    None,
    BestEffort,
    #[default]
    Reliable,
}

impl WindowZLevelQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use WindowZLevelQuality::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}

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
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GfxCapabilities {
    pub webgpu: bool,
    pub native_gpu: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlatformCapabilities {
    pub exec: ExecCapabilities,
    pub ui: UiCapabilities,
    pub clipboard: ClipboardCapabilities,
    pub dnd: DndCapabilities,
    pub ime: ImeCapabilities,
    pub fs: FsCapabilities,
    pub shell: ShellCapabilities,
    pub gfx: GfxCapabilities,
}

impl Default for PlatformCapabilities {
    fn default() -> Self {
        Self {
            exec: ExecCapabilities::default(),
            ui: UiCapabilities {
                multi_window: true,
                window_tear_off: true,
                cursor_icons: true,
                window_hover_detection: WindowHoverDetectionQuality::Reliable,
                window_set_outer_position: WindowSetOuterPositionQuality::Reliable,
                window_z_level: WindowZLevelQuality::Reliable,
            },
            clipboard: ClipboardCapabilities {
                text: true,
                files: false,
            },
            dnd: DndCapabilities {
                external: true,
                external_payload: ExternalDragPayloadKind::FileToken,
                external_position: ExternalDragPositionQuality::Continuous,
            },
            ime: ImeCapabilities {
                enabled: true,
                set_cursor_area: true,
            },
            fs: FsCapabilities {
                real_paths: true,
                file_dialogs: true,
            },
            shell: ShellCapabilities { open_url: true },
            gfx: GfxCapabilities {
                webgpu: false,
                native_gpu: true,
            },
        }
    }
}

impl PlatformCapabilities {
    pub fn bool_key(&self, key: &str) -> Option<bool> {
        match key {
            keys::UI_MULTI_WINDOW => Some(self.ui.multi_window),
            keys::UI_WINDOW_TEAR_OFF => Some(self.ui.window_tear_off),
            keys::UI_CURSOR_ICONS => Some(self.ui.cursor_icons),
            keys::CLIPBOARD_TEXT => Some(self.clipboard.text),
            keys::CLIPBOARD_FILES => Some(self.clipboard.files),
            keys::DND_EXTERNAL => Some(self.dnd.external),
            keys::IME | keys::IME_ENABLED => Some(self.ime.enabled),
            keys::IME_SET_CURSOR_AREA => Some(self.ime.set_cursor_area),
            keys::FS_REAL_PATHS => Some(self.fs.real_paths),
            keys::FS_FILE_DIALOGS => Some(self.fs.file_dialogs),
            keys::SHELL_OPEN_URL => Some(self.shell.open_url),
            keys::GFX_WEBGPU => Some(self.gfx.webgpu),
            keys::GFX_NATIVE_GPU => Some(self.gfx.native_gpu),
            _ => None,
        }
    }

    pub fn str_key(&self, key: &str) -> Option<&'static str> {
        match key {
            keys::EXEC_BACKGROUND_WORK => Some(self.exec.background_work.as_str()),
            keys::EXEC_WAKE => Some(self.exec.wake.as_str()),
            keys::EXEC_TIMERS => Some(self.exec.timers.as_str()),
            keys::UI_WINDOW_HOVER_DETECTION => Some(self.ui.window_hover_detection.as_str()),
            keys::UI_WINDOW_SET_OUTER_POSITION => Some(self.ui.window_set_outer_position.as_str()),
            keys::UI_WINDOW_Z_LEVEL => Some(self.ui.window_z_level.as_str()),
            keys::DND_EXTERNAL_PAYLOAD => Some(self.dnd.external_payload.as_str()),
            keys::DND_EXTERNAL_POSITION => Some(self.dnd.external_position.as_str()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_key_kind_matches_platform_capabilities_accessors() {
        let caps = PlatformCapabilities::default();

        for &key in KNOWN_BOOL_CAPABILITY_KEYS {
            assert!(caps.bool_key(key).is_some(), "bool_key must accept {key}");
            assert_eq!(capability_key_kind(key), Some(CapabilityValueKind::Bool));
        }

        for &key in KNOWN_STR_CAPABILITY_KEYS {
            assert!(caps.str_key(key).is_some(), "str_key must accept {key}");
            assert_eq!(capability_key_kind(key), Some(CapabilityValueKind::Str));
        }

        assert_eq!(capability_key_kind("does.not.exist"), None);
        assert_eq!(caps.bool_key("does.not.exist"), None);
        assert_eq!(caps.str_key("does.not.exist"), None);
    }
}
