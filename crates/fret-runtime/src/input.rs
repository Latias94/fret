use crate::PlatformCapabilities;
use crate::WindowInputArbitrationSnapshot;
use crate::WindowPointerOcclusion;
use fret_core::{KeyCode, Modifiers};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Macos,
    Windows,
    Linux,
    Web,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        return Self::Macos;
        #[cfg(target_os = "windows")]
        return Self::Windows;
        #[cfg(all(unix, not(target_os = "macos")))]
        return Self::Linux;
        #[cfg(target_arch = "wasm32")]
        return Self::Web;
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Web => "web",
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::current()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextBoundaryMode {
    #[default]
    UnicodeWord,
    Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputContext {
    pub platform: Platform,
    pub caps: PlatformCapabilities,
    pub ui_has_modal: bool,
    /// Window-level input arbitration snapshot for the current dispatch pass.
    ///
    /// When present, this allows policy-heavy ecosystem layers to observe modal/capture/occlusion
    /// state without reaching into global services (and keeps the snapshot consistent within a
    /// single dispatch).
    pub window_arbitration: Option<WindowInputArbitrationSnapshot>,
    pub focus_is_text_input: bool,
    pub text_boundary_mode: TextBoundaryMode,
    pub edit_can_undo: bool,
    pub edit_can_redo: bool,
    pub dispatch_phase: InputDispatchPhase,
}

impl InputContext {
    pub fn fallback(platform: Platform, caps: PlatformCapabilities) -> Self {
        Self {
            platform,
            caps,
            ..Default::default()
        }
    }
}

impl Default for InputContext {
    fn default() -> Self {
        Self {
            platform: Platform::current(),
            caps: PlatformCapabilities::default(),
            ui_has_modal: false,
            window_arbitration: None,
            focus_is_text_input: false,
            text_boundary_mode: TextBoundaryMode::UnicodeWord,
            edit_can_undo: true,
            edit_can_redo: true,
            dispatch_phase: InputDispatchPhase::Bubble,
        }
    }
}

impl InputContext {
    pub fn window_arbitration(&self) -> Option<WindowInputArbitrationSnapshot> {
        self.window_arbitration
    }

    pub fn window_pointer_occlusion(&self) -> WindowPointerOcclusion {
        self.window_arbitration
            .map(|snapshot| snapshot.pointer_occlusion)
            .unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputDispatchPhase {
    #[default]
    Bubble,
    Preview,
    Capture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefaultAction {
    FocusOnPointerDown,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DefaultActionSet(u32);

impl DefaultActionSet {
    pub fn insert(&mut self, action: DefaultAction) {
        self.0 |= 1 << (action as u32);
    }

    pub fn contains(self, action: DefaultAction) -> bool {
        (self.0 & (1 << (action as u32))) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: KeyCode,
    pub mods: Modifiers,
}

impl KeyChord {
    pub fn new(key: KeyCode, mods: Modifiers) -> Self {
        Self { key, mods }
    }
}

pub fn format_chord(platform: Platform, chord: KeyChord) -> String {
    let mut parts: Vec<&'static str> = Vec::new();

    match platform {
        Platform::Macos => {
            if chord.mods.alt_gr {
                parts.push("AltGr");
            }
            if chord.mods.meta {
                parts.push("Cmd");
            }
            if chord.mods.ctrl {
                parts.push("Ctrl");
            }
            if chord.mods.alt {
                parts.push("Alt");
            }
            if chord.mods.shift {
                parts.push("Shift");
            }
        }
        Platform::Windows | Platform::Linux | Platform::Web => {
            if chord.mods.ctrl {
                parts.push("Ctrl");
            }
            if chord.mods.alt_gr {
                parts.push("AltGr");
            }
            if chord.mods.alt {
                parts.push("Alt");
            }
            if chord.mods.shift {
                parts.push("Shift");
            }
            if chord.mods.meta {
                parts.push("Meta");
            }
        }
    }

    let key = key_label(chord.key);
    if parts.is_empty() {
        return key.into_owned();
    }
    format!("{}+{}", parts.join("+"), key)
}

pub fn format_sequence(platform: Platform, sequence: &[KeyChord]) -> String {
    let mut out = String::new();
    for (index, chord) in sequence.iter().copied().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        out.push_str(&format_chord(platform, chord));
    }
    out
}

fn key_label(key: KeyCode) -> Cow<'static, str> {
    match key {
        KeyCode::Escape => Cow::Borrowed("Esc"),
        KeyCode::Enter => Cow::Borrowed("Enter"),
        KeyCode::Tab => Cow::Borrowed("Tab"),
        KeyCode::Backspace => Cow::Borrowed("Backspace"),
        KeyCode::Space => Cow::Borrowed("Space"),

        KeyCode::ArrowUp => Cow::Borrowed("Up"),
        KeyCode::ArrowDown => Cow::Borrowed("Down"),
        KeyCode::ArrowLeft => Cow::Borrowed("Left"),
        KeyCode::ArrowRight => Cow::Borrowed("Right"),

        KeyCode::Home => Cow::Borrowed("Home"),
        KeyCode::End => Cow::Borrowed("End"),
        KeyCode::PageUp => Cow::Borrowed("PageUp"),
        KeyCode::PageDown => Cow::Borrowed("PageDown"),
        KeyCode::Insert => Cow::Borrowed("Insert"),
        KeyCode::Delete => Cow::Borrowed("Delete"),

        KeyCode::CapsLock => Cow::Borrowed("CapsLock"),

        KeyCode::ShiftLeft | KeyCode::ShiftRight => Cow::Borrowed("Shift"),
        KeyCode::ControlLeft | KeyCode::ControlRight => Cow::Borrowed("Ctrl"),
        KeyCode::AltLeft | KeyCode::AltRight => Cow::Borrowed("Alt"),
        KeyCode::MetaLeft | KeyCode::MetaRight => Cow::Borrowed("Super"),

        KeyCode::Digit0 => Cow::Borrowed("0"),
        KeyCode::Digit1 => Cow::Borrowed("1"),
        KeyCode::Digit2 => Cow::Borrowed("2"),
        KeyCode::Digit3 => Cow::Borrowed("3"),
        KeyCode::Digit4 => Cow::Borrowed("4"),
        KeyCode::Digit5 => Cow::Borrowed("5"),
        KeyCode::Digit6 => Cow::Borrowed("6"),
        KeyCode::Digit7 => Cow::Borrowed("7"),
        KeyCode::Digit8 => Cow::Borrowed("8"),
        KeyCode::Digit9 => Cow::Borrowed("9"),

        KeyCode::KeyA => Cow::Borrowed("A"),
        KeyCode::KeyB => Cow::Borrowed("B"),
        KeyCode::KeyC => Cow::Borrowed("C"),
        KeyCode::KeyD => Cow::Borrowed("D"),
        KeyCode::KeyE => Cow::Borrowed("E"),
        KeyCode::KeyF => Cow::Borrowed("F"),
        KeyCode::KeyG => Cow::Borrowed("G"),
        KeyCode::KeyH => Cow::Borrowed("H"),
        KeyCode::KeyI => Cow::Borrowed("I"),
        KeyCode::KeyJ => Cow::Borrowed("J"),
        KeyCode::KeyK => Cow::Borrowed("K"),
        KeyCode::KeyL => Cow::Borrowed("L"),
        KeyCode::KeyM => Cow::Borrowed("M"),
        KeyCode::KeyN => Cow::Borrowed("N"),
        KeyCode::KeyO => Cow::Borrowed("O"),
        KeyCode::KeyP => Cow::Borrowed("P"),
        KeyCode::KeyQ => Cow::Borrowed("Q"),
        KeyCode::KeyR => Cow::Borrowed("R"),
        KeyCode::KeyS => Cow::Borrowed("S"),
        KeyCode::KeyT => Cow::Borrowed("T"),
        KeyCode::KeyU => Cow::Borrowed("U"),
        KeyCode::KeyV => Cow::Borrowed("V"),
        KeyCode::KeyW => Cow::Borrowed("W"),
        KeyCode::KeyX => Cow::Borrowed("X"),
        KeyCode::KeyY => Cow::Borrowed("Y"),
        KeyCode::KeyZ => Cow::Borrowed("Z"),

        KeyCode::Minus => Cow::Borrowed("-"),
        KeyCode::Equal => Cow::Borrowed("="),
        KeyCode::BracketLeft => Cow::Borrowed("["),
        KeyCode::BracketRight => Cow::Borrowed("]"),
        KeyCode::Backslash => Cow::Borrowed("\\"),
        KeyCode::Semicolon => Cow::Borrowed(";"),
        KeyCode::Quote => Cow::Borrowed("'"),
        KeyCode::Backquote => Cow::Borrowed("`"),
        KeyCode::Comma => Cow::Borrowed(","),
        KeyCode::Period => Cow::Borrowed("."),
        KeyCode::Slash => Cow::Borrowed("/"),

        KeyCode::F1 => Cow::Borrowed("F1"),
        KeyCode::F2 => Cow::Borrowed("F2"),
        KeyCode::F3 => Cow::Borrowed("F3"),
        KeyCode::F4 => Cow::Borrowed("F4"),
        KeyCode::F5 => Cow::Borrowed("F5"),
        KeyCode::F6 => Cow::Borrowed("F6"),
        KeyCode::F7 => Cow::Borrowed("F7"),
        KeyCode::F8 => Cow::Borrowed("F8"),
        KeyCode::F9 => Cow::Borrowed("F9"),
        KeyCode::F10 => Cow::Borrowed("F10"),
        KeyCode::F11 => Cow::Borrowed("F11"),
        KeyCode::F12 => Cow::Borrowed("F12"),
        KeyCode::F13 => Cow::Borrowed("F13"),
        KeyCode::F14 => Cow::Borrowed("F14"),
        KeyCode::F15 => Cow::Borrowed("F15"),
        KeyCode::F16 => Cow::Borrowed("F16"),
        KeyCode::F17 => Cow::Borrowed("F17"),
        KeyCode::F18 => Cow::Borrowed("F18"),
        KeyCode::F19 => Cow::Borrowed("F19"),
        KeyCode::F20 => Cow::Borrowed("F20"),
        KeyCode::F21 => Cow::Borrowed("F21"),
        KeyCode::F22 => Cow::Borrowed("F22"),
        KeyCode::F23 => Cow::Borrowed("F23"),
        KeyCode::F24 => Cow::Borrowed("F24"),

        KeyCode::Numpad0 => Cow::Borrowed("Num0"),
        KeyCode::Numpad1 => Cow::Borrowed("Num1"),
        KeyCode::Numpad2 => Cow::Borrowed("Num2"),
        KeyCode::Numpad3 => Cow::Borrowed("Num3"),
        KeyCode::Numpad4 => Cow::Borrowed("Num4"),
        KeyCode::Numpad5 => Cow::Borrowed("Num5"),
        KeyCode::Numpad6 => Cow::Borrowed("Num6"),
        KeyCode::Numpad7 => Cow::Borrowed("Num7"),
        KeyCode::Numpad8 => Cow::Borrowed("Num8"),
        KeyCode::Numpad9 => Cow::Borrowed("Num9"),
        KeyCode::NumpadAdd => Cow::Borrowed("Num+"),
        KeyCode::NumpadSubtract => Cow::Borrowed("Num-"),
        KeyCode::NumpadMultiply => Cow::Borrowed("Num*"),
        KeyCode::NumpadDivide => Cow::Borrowed("Num/"),
        KeyCode::NumpadDecimal => Cow::Borrowed("Num."),
        KeyCode::NumpadEnter => Cow::Borrowed("NumEnter"),

        other => Cow::Owned(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_chord_macos_orders_modifiers_and_formats_cmd() {
        let chord = KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                meta: true,
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(format_chord(Platform::Macos, chord), "Cmd+Shift+P");
    }

    #[test]
    fn format_chord_windows_orders_modifiers_and_formats_ctrl() {
        let chord = KeyChord::new(
            KeyCode::KeyP,
            Modifiers {
                ctrl: true,
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(format_chord(Platform::Windows, chord), "Ctrl+Shift+P");
    }

    #[test]
    fn format_chord_falls_back_to_code_token_for_unhandled_keys() {
        let chord = KeyChord::new(KeyCode::PrintScreen, Modifiers::default());
        assert_eq!(format_chord(Platform::Windows, chord), "PrintScreen");
    }
}
