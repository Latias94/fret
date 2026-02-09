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
pub const CLIPBOARD_PRIMARY_TEXT: &str = "clipboard.primary_text";

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
