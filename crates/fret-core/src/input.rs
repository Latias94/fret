use crate::{
    AppWindowId, RenderTargetId,
    geometry::{Point, Px},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Unknown,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
    Preedit { text: String, cursor: Option<usize> },
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
pub enum Event {
    Pointer(PointerEvent),
    Ime(ImeEvent),
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
    WindowScaleFactorChanged(f32),
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
