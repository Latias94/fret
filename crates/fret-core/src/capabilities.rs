use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExternalDragPayloadKind {
    None,
    #[default]
    FilePath,
    FileToken,
    Text,
}

impl ExternalDragPayloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FilePath => "file_path",
            Self::FileToken => "file_token",
            Self::Text => "text",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiCapabilities {
    pub multi_window: bool,
    pub window_tear_off: bool,
    pub cursor_icons: bool,
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
    pub wgpu: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlatformCapabilities {
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
            ui: UiCapabilities {
                multi_window: true,
                window_tear_off: true,
                cursor_icons: true,
            },
            clipboard: ClipboardCapabilities {
                text: true,
                files: false,
            },
            dnd: DndCapabilities {
                external: true,
                external_payload: ExternalDragPayloadKind::FilePath,
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
                wgpu: true,
            },
        }
    }
}

impl PlatformCapabilities {
    pub fn bool_key(&self, key: &str) -> Option<bool> {
        match key {
            "ui.multi_window" => Some(self.ui.multi_window),
            "ui.window_tear_off" => Some(self.ui.window_tear_off),
            "ui.cursor_icons" => Some(self.ui.cursor_icons),
            "clipboard.text" => Some(self.clipboard.text),
            "clipboard.files" => Some(self.clipboard.files),
            "dnd.external" => Some(self.dnd.external),
            "ime" | "ime.enabled" => Some(self.ime.enabled),
            "ime.set_cursor_area" => Some(self.ime.set_cursor_area),
            "fs.real_paths" => Some(self.fs.real_paths),
            "fs.file_dialogs" => Some(self.fs.file_dialogs),
            "shell.open_url" => Some(self.shell.open_url),
            "gfx.webgpu" => Some(self.gfx.webgpu),
            "gfx.wgpu" => Some(self.gfx.wgpu),
            _ => None,
        }
    }

    pub fn str_key(&self, key: &str) -> Option<&'static str> {
        match key {
            "dnd.external_payload" => Some(self.dnd.external_payload.as_str()),
            _ => None,
        }
    }
}
