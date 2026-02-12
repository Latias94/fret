use super::keys;
use super::{
    ClipboardCapabilities, DndCapabilities, ExecCapabilities, FsCapabilities, GfxCapabilities,
    ImeCapabilities, ShellCapabilities, UiCapabilities,
};
use super::{
    ExternalDragPayloadKind, ExternalDragPositionQuality, WindowHoverDetectionQuality,
    WindowSetOuterPositionQuality, WindowZLevelQuality,
};

use serde::{Deserialize, Serialize};

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
                primary_text: cfg!(all(
                    unix,
                    not(any(
                        target_os = "macos",
                        target_os = "android",
                        target_os = "emscripten",
                        target_arch = "wasm32"
                    ))
                )),
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
            shell: ShellCapabilities {
                open_url: true,
                share_sheet: false,
                incoming_open: false,
            },
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
            keys::CLIPBOARD_PRIMARY_TEXT => Some(self.clipboard.primary_text),
            keys::DND_EXTERNAL => Some(self.dnd.external),
            keys::IME | keys::IME_ENABLED => Some(self.ime.enabled),
            keys::IME_SET_CURSOR_AREA => Some(self.ime.set_cursor_area),
            keys::FS_REAL_PATHS => Some(self.fs.real_paths),
            keys::FS_FILE_DIALOGS => Some(self.fs.file_dialogs),
            keys::SHELL_OPEN_URL => Some(self.shell.open_url),
            keys::SHELL_SHARE_SHEET => Some(self.shell.share_sheet),
            keys::SHELL_INCOMING_OPEN => Some(self.shell.incoming_open),
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
