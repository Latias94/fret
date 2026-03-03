pub const EXEC_BACKGROUND_WORK: &str = "exec.background_work";
pub const EXEC_WAKE: &str = "exec.wake";
pub const EXEC_TIMERS: &str = "exec.timers";

pub const UI_MULTI_WINDOW: &str = "ui.multi_window";
pub const UI_WINDOW_TEAR_OFF: &str = "ui.window_tear_off";
pub const UI_CURSOR_ICONS: &str = "ui.cursor_icons";

pub const UI_WINDOW_HOVER_DETECTION: &str = "ui.window_hover_detection";
pub const UI_WINDOW_SET_OUTER_POSITION: &str = "ui.window_set_outer_position";
pub const UI_WINDOW_Z_LEVEL: &str = "ui.window_z_level";

// Window styles / utility windows (ADR 0139).
pub const UI_WINDOW_DECORATIONS: &str = "ui.window.decorations";
pub const UI_WINDOW_RESIZABLE: &str = "ui.window.resizable";
pub const UI_WINDOW_TRANSPARENT: &str = "ui.window.transparent";
pub const UI_WINDOW_ALWAYS_ON_TOP: &str = "ui.window.always_on_top";
pub const UI_WINDOW_SKIP_TASKBAR: &str = "ui.window.skip_taskbar";
pub const UI_WINDOW_NON_ACTIVATING: &str = "ui.window.non_activating";
pub const UI_WINDOW_MOUSE_PASSTHROUGH: &str = "ui.window.mouse_passthrough";
pub const UI_WINDOW_SET_VISIBLE: &str = "ui.window.set_visible";
pub const UI_WINDOW_BEGIN_DRAG: &str = "ui.window.begin_drag";
pub const UI_WINDOW_BEGIN_RESIZE: &str = "ui.window.begin_resize";

// Background materials (ADR 0310).
pub const UI_WINDOW_BACKGROUND_MATERIAL_SYSTEM_DEFAULT: &str =
    "ui.window.background_material.system_default";
pub const UI_WINDOW_BACKGROUND_MATERIAL_MICA: &str = "ui.window.background_material.mica";
pub const UI_WINDOW_BACKGROUND_MATERIAL_ACRYLIC: &str = "ui.window.background_material.acrylic";
pub const UI_WINDOW_BACKGROUND_MATERIAL_VIBRANCY: &str = "ui.window.background_material.vibrancy";

// Non-portable escape hatch (ADR 0139).
pub const UI_NATIVE_WINDOW_HANDLE: &str = "ui.native_window_handle";

pub const CLIPBOARD_TEXT: &str = "clipboard.text";
pub const CLIPBOARD_TEXT_READ: &str = "clipboard.text_read";
pub const CLIPBOARD_TEXT_WRITE: &str = "clipboard.text_write";
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
pub const SHELL_SHARE_SHEET: &str = "shell.share_sheet";
pub const SHELL_INCOMING_OPEN: &str = "shell.incoming_open";

pub const GFX_WEBGPU: &str = "gfx.webgpu";
/// Indicates that a native GPU rendering backend is available.
///
/// This is intentionally not named after a Rust crate (e.g. `wgpu`) so the contract remains
/// portable across backends and future implementations.
pub const GFX_NATIVE_GPU: &str = "gfx.native_gpu";
