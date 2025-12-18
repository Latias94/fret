use crate::{
    AppWindowId, RenderTargetId, TimerToken,
    geometry::{Point, Px},
};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Unknown,
    Escape,
    Enter,
    Tab,
    Backspace,
    Space,

    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,

    CapsLock,

    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    SuperLeft,
    SuperRight,

    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,

    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    Semicolon,
    Quote,
    Backquote,
    Comma,
    Period,
    Slash,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadDecimal,
    NumpadEnter,
}

impl KeyCode {
    pub fn from_token(token: &str) -> Option<Self> {
        Some(match token {
            "Unknown" => Self::Unknown,
            "Escape" => Self::Escape,
            "Enter" => Self::Enter,
            "Tab" => Self::Tab,
            "Backspace" => Self::Backspace,
            "Space" => Self::Space,

            "ArrowUp" => Self::ArrowUp,
            "ArrowDown" => Self::ArrowDown,
            "ArrowLeft" => Self::ArrowLeft,
            "ArrowRight" => Self::ArrowRight,

            "Home" => Self::Home,
            "End" => Self::End,
            "PageUp" => Self::PageUp,
            "PageDown" => Self::PageDown,
            "Insert" => Self::Insert,
            "Delete" => Self::Delete,

            "CapsLock" => Self::CapsLock,

            "ShiftLeft" => Self::ShiftLeft,
            "ShiftRight" => Self::ShiftRight,
            "ControlLeft" => Self::ControlLeft,
            "ControlRight" => Self::ControlRight,
            "AltLeft" => Self::AltLeft,
            "AltRight" => Self::AltRight,
            "SuperLeft" => Self::SuperLeft,
            "SuperRight" => Self::SuperRight,

            "Digit0" => Self::Digit0,
            "Digit1" => Self::Digit1,
            "Digit2" => Self::Digit2,
            "Digit3" => Self::Digit3,
            "Digit4" => Self::Digit4,
            "Digit5" => Self::Digit5,
            "Digit6" => Self::Digit6,
            "Digit7" => Self::Digit7,
            "Digit8" => Self::Digit8,
            "Digit9" => Self::Digit9,

            "KeyA" => Self::KeyA,
            "KeyB" => Self::KeyB,
            "KeyC" => Self::KeyC,
            "KeyD" => Self::KeyD,
            "KeyE" => Self::KeyE,
            "KeyF" => Self::KeyF,
            "KeyG" => Self::KeyG,
            "KeyH" => Self::KeyH,
            "KeyI" => Self::KeyI,
            "KeyJ" => Self::KeyJ,
            "KeyK" => Self::KeyK,
            "KeyL" => Self::KeyL,
            "KeyM" => Self::KeyM,
            "KeyN" => Self::KeyN,
            "KeyO" => Self::KeyO,
            "KeyP" => Self::KeyP,
            "KeyQ" => Self::KeyQ,
            "KeyR" => Self::KeyR,
            "KeyS" => Self::KeyS,
            "KeyT" => Self::KeyT,
            "KeyU" => Self::KeyU,
            "KeyV" => Self::KeyV,
            "KeyW" => Self::KeyW,
            "KeyX" => Self::KeyX,
            "KeyY" => Self::KeyY,
            "KeyZ" => Self::KeyZ,

            "Minus" => Self::Minus,
            "Equal" => Self::Equal,
            "BracketLeft" => Self::BracketLeft,
            "BracketRight" => Self::BracketRight,
            "Backslash" => Self::Backslash,
            "Semicolon" => Self::Semicolon,
            "Quote" => Self::Quote,
            "Backquote" => Self::Backquote,
            "Comma" => Self::Comma,
            "Period" => Self::Period,
            "Slash" => Self::Slash,

            "F1" => Self::F1,
            "F2" => Self::F2,
            "F3" => Self::F3,
            "F4" => Self::F4,
            "F5" => Self::F5,
            "F6" => Self::F6,
            "F7" => Self::F7,
            "F8" => Self::F8,
            "F9" => Self::F9,
            "F10" => Self::F10,
            "F11" => Self::F11,
            "F12" => Self::F12,

            "Numpad0" => Self::Numpad0,
            "Numpad1" => Self::Numpad1,
            "Numpad2" => Self::Numpad2,
            "Numpad3" => Self::Numpad3,
            "Numpad4" => Self::Numpad4,
            "Numpad5" => Self::Numpad5,
            "Numpad6" => Self::Numpad6,
            "Numpad7" => Self::Numpad7,
            "Numpad8" => Self::Numpad8,
            "Numpad9" => Self::Numpad9,
            "NumpadAdd" => Self::NumpadAdd,
            "NumpadSubtract" => Self::NumpadSubtract,
            "NumpadMultiply" => Self::NumpadMultiply,
            "NumpadDivide" => Self::NumpadDivide,
            "NumpadDecimal" => Self::NumpadDecimal,
            "NumpadEnter" => Self::NumpadEnter,

            _ => return None,
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImeEvent {
    Enabled,
    Disabled,
    Commit(String),
    /// `cursor` is a byte-indexed range in the preedit string (begin, end).
    /// When `None`, the cursor should be hidden.
    Preedit {
        text: String,
        cursor: Option<(usize, usize)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent {
    Move {
        position: Point,
        buttons: MouseButtons,
        modifiers: Modifiers,
    },
    Down {
        position: Point,
        button: MouseButton,
        modifiers: Modifiers,
    },
    Up {
        position: Point,
        button: MouseButton,
        modifiers: Modifiers,
    },
    Wheel {
        position: Point,
        delta: Point,
        modifiers: Modifiers,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExternalDragKind {
    EnterFiles(Vec<PathBuf>),
    OverFiles(Vec<PathBuf>),
    DropFiles(Vec<PathBuf>),
    Leave,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalDragEvent {
    pub position: Point,
    pub kind: ExternalDragKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Pointer(PointerEvent),
    Timer {
        token: TimerToken,
    },
    Ime(ImeEvent),
    ExternalDrag(ExternalDragEvent),
    KeyDown {
        key: KeyCode,
        modifiers: Modifiers,
        repeat: bool,
    },
    KeyUp {
        key: KeyCode,
        modifiers: Modifiers,
    },
    TextInput(String),
    /// Clipboard text payload delivered to the focused widget (typically as the result of a paste request).
    ClipboardText(String),
    WindowScaleFactorChanged(f32),
    WindowMoved {
        x: i32,
        y: i32,
    },
    WindowResized {
        width: Px,
        height: Px,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportInputEvent {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub uv: (f32, f32),
    pub target_px: (u32, u32),
    pub kind: ViewportInputKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportInputKind {
    PointerMove {
        buttons: MouseButtons,
        modifiers: Modifiers,
    },
    PointerDown {
        button: MouseButton,
        modifiers: Modifiers,
    },
    PointerUp {
        button: MouseButton,
        modifiers: Modifiers,
    },
    Wheel {
        delta: Point,
        modifiers: Modifiers,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}
