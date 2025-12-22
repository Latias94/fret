use fret_core::{KeyCode, Modifiers, PlatformCapabilities};

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

#[derive(Debug, Clone, Default)]
pub struct InputContext {
    pub platform: Platform,
    pub caps: PlatformCapabilities,
    pub ui_has_modal: bool,
    pub focus_is_text_input: bool,
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

    let key = key_label(chord.key).to_string();
    if parts.is_empty() {
        return key;
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

fn key_label(key: KeyCode) -> &'static str {
    match key {
        KeyCode::Escape => "Esc",
        KeyCode::Enter => "Enter",
        KeyCode::Tab => "Tab",
        KeyCode::Backspace => "Backspace",
        KeyCode::Space => "Space",

        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",

        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::Insert => "Insert",
        KeyCode::Delete => "Delete",

        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",

        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",

        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        KeyCode::BracketLeft => "[",
        KeyCode::BracketRight => "]",
        KeyCode::Backslash => "\\",
        KeyCode::Semicolon => ";",
        KeyCode::Quote => "'",
        KeyCode::Backquote => "`",
        KeyCode::Comma => ",",
        KeyCode::Period => ".",
        KeyCode::Slash => "/",

        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",

        _ => "Unknown",
    }
}
