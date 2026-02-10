use super::keys;

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
    keys::CLIPBOARD_PRIMARY_TEXT,
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
