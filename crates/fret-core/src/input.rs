use crate::{
    AppWindowId, ClipboardToken, ExternalDropToken, FileDialogDataEvent, FileDialogSelection,
    ImageId, ImageUpdateToken, ImageUploadToken, RenderTargetId, TimerToken, ViewportMapping,
    WindowLogicalPosition,
    geometry::{Point, Px},
};

pub use keyboard_types::Code as KeyCode;

/// Maps a key code to a lowercase ASCII character for basic typeahead use.
///
/// This intentionally only covers `a-z` and `0-9` to match common Radix-like prefix typeahead
/// behavior. Returns `None` for non-alphanumeric keys.
pub fn keycode_to_ascii_lowercase(key: KeyCode) -> Option<char> {
    use keyboard_types::Code;

    Some(match key {
        Code::KeyA => 'a',
        Code::KeyB => 'b',
        Code::KeyC => 'c',
        Code::KeyD => 'd',
        Code::KeyE => 'e',
        Code::KeyF => 'f',
        Code::KeyG => 'g',
        Code::KeyH => 'h',
        Code::KeyI => 'i',
        Code::KeyJ => 'j',
        Code::KeyK => 'k',
        Code::KeyL => 'l',
        Code::KeyM => 'm',
        Code::KeyN => 'n',
        Code::KeyO => 'o',
        Code::KeyP => 'p',
        Code::KeyQ => 'q',
        Code::KeyR => 'r',
        Code::KeyS => 's',
        Code::KeyT => 't',
        Code::KeyU => 'u',
        Code::KeyV => 'v',
        Code::KeyW => 'w',
        Code::KeyX => 'x',
        Code::KeyY => 'y',
        Code::KeyZ => 'z',
        Code::Digit0 => '0',
        Code::Digit1 => '1',
        Code::Digit2 => '2',
        Code::Digit3 => '3',
        Code::Digit4 => '4',
        Code::Digit5 => '5',
        Code::Digit6 => '6',
        Code::Digit7 => '7',
        Code::Digit8 => '8',
        Code::Digit9 => '9',
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

/// Pointer device classification for unified pointer events.
///
/// This is intentionally small and Radix-aligned: it is used by component-layer policies to
/// distinguish mouse-specific behaviors (e.g. open-on-pointer-down) from touch/pen behaviors
/// (e.g. open-on-click to avoid scroll-to-open).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointerType {
    #[default]
    Mouse,
    Touch,
    Pen,
    Unknown,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    /// Alternate Graphics (AltGr / AltGraph) modifier.
    ///
    /// This is semantically distinct from `ctrl+alt` for editor-grade shortcut matching.
    pub alt_gr: bool,
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
        pointer_type: PointerType,
    },
    Down {
        position: Point,
        button: MouseButton,
        modifiers: Modifiers,
        /// Consecutive click count for this button (1 = single click, 2 = double click, ...).
        ///
        /// This count is provided by the platform runner and only increments for "true clicks"
        /// (press + release without exceeding a small drag threshold).
        click_count: u8,
        pointer_type: PointerType,
    },
    Up {
        position: Point,
        button: MouseButton,
        modifiers: Modifiers,
        /// Consecutive click count for this button (1 = single click, 2 = double click, ...).
        ///
        /// See `PointerEvent::Down.click_count` for the normalization rules.
        click_count: u8,
        pointer_type: PointerType,
    },
    Wheel {
        position: Point,
        delta: Point,
        modifiers: Modifiers,
        pointer_type: PointerType,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExternalDragKind {
    EnterFiles(ExternalDragFiles),
    OverFiles(ExternalDragFiles),
    DropFiles(ExternalDragFiles),
    Leave,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalDragFiles {
    pub token: ExternalDropToken,
    pub files: Vec<ExternalDragFile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalDragFile {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalDragEvent {
    pub position: Point,
    pub kind: ExternalDragKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalDropDataEvent {
    pub token: ExternalDropToken,
    pub files: Vec<ExternalDropFileData>,
    pub errors: Vec<ExternalDropReadError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalDropFileData {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalDropReadError {
    pub name: String,
    pub message: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExternalDropReadLimits {
    pub max_total_bytes: u64,
    pub max_file_bytes: u64,
    pub max_files: usize,
}

impl ExternalDropReadLimits {
    pub fn capped_by(self, cap: ExternalDropReadLimits) -> ExternalDropReadLimits {
        ExternalDropReadLimits {
            max_total_bytes: self.max_total_bytes.min(cap.max_total_bytes),
            max_file_bytes: self.max_file_bytes.min(cap.max_file_bytes),
            max_files: self.max_files.min(cap.max_files),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InternalDragKind {
    Enter,
    Over,
    Drop,
    Leave,
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InternalDragEvent {
    pub position: Point,
    pub kind: InternalDragKind,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Pointer(PointerEvent),
    Timer {
        token: TimerToken,
    },
    Ime(ImeEvent),
    ExternalDrag(ExternalDragEvent),
    ExternalDropData(ExternalDropDataEvent),
    InternalDrag(InternalDragEvent),
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
    /// Sets the current selection (or caret when `anchor == focus`) in UTF-8 byte offsets
    /// within the focused widget's text buffer (ADR 0071).
    ///
    /// This event is primarily intended for accessibility and automation backends.
    SetTextSelection {
        anchor: u32,
        focus: u32,
    },
    /// Clipboard text payload delivered to the focused widget (typically as the result of a paste request).
    ClipboardText {
        token: ClipboardToken,
        text: String,
    },
    /// Clipboard read completed without a text payload (clipboard empty/unavailable/error).
    ClipboardTextUnavailable {
        token: ClipboardToken,
    },
    /// File dialog selection metadata (token + names). Bytes must be requested via effects.
    FileDialogSelection(FileDialogSelection),
    /// File dialog data payload, typically produced by `Effect::FileDialogReadAll`.
    FileDialogData(FileDialogDataEvent),
    /// A file dialog request completed without a selection (user canceled).
    FileDialogCanceled,
    /// Image resource registration completed and produced an `ImageId`.
    ImageRegistered {
        token: ImageUploadToken,
        image: ImageId,
        width: u32,
        height: u32,
    },
    /// Image resource registration failed (e.g. invalid bytes, backend error).
    ImageRegisterFailed {
        token: ImageUploadToken,
        message: String,
    },
    /// Optional acknowledgement that a streaming image update was applied.
    ///
    /// This is intended for debugging/telemetry surfaces and must be capability-gated by the
    /// runner to avoid flooding the event loop during video playback (ADR 0126).
    ImageUpdateApplied {
        token: ImageUpdateToken,
        image: ImageId,
    },
    /// Optional acknowledgement that a streaming image update was dropped.
    ///
    /// See `ImageUpdateApplied` for rationale (ADR 0126).
    ImageUpdateDropped {
        token: ImageUpdateToken,
        image: ImageId,
        reason: ImageUpdateDropReason,
    },
    /// Window close button / OS close request was triggered.
    ///
    /// The runner must not close the window immediately; the app/driver may intercept the request
    /// (e.g. unsaved-changes confirmation) and decide whether to emit `WindowRequest::Close`.
    WindowCloseRequested,
    /// Window focus state changed (focused vs blurred).
    WindowFocusChanged(bool),
    WindowScaleFactorChanged(f32),
    WindowMoved(WindowLogicalPosition),
    WindowResized {
        width: Px,
        height: Px,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageUpdateDropReason {
    Coalesced,
    StagingBudgetExceeded,
    UnknownImage,
    InvalidPayload,
    RendererNotReady,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportInputEvent {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub uv: (f32, f32),
    pub target_px: (u32, u32),
    pub kind: ViewportInputKind,
}

impl ViewportInputEvent {
    pub fn from_mapping_window_point(
        window: AppWindowId,
        target: RenderTargetId,
        mapping: &ViewportMapping,
        position: Point,
        kind: ViewportInputKind,
    ) -> Option<Self> {
        let uv = mapping.window_point_to_uv(position)?;
        let target_px = mapping.window_point_to_target_px(position)?;
        Some(Self {
            window,
            target,
            uv,
            target_px,
            kind,
        })
    }

    pub fn from_mapping_window_point_clamped(
        window: AppWindowId,
        target: RenderTargetId,
        mapping: &ViewportMapping,
        position: Point,
        kind: ViewportInputKind,
    ) -> Self {
        let uv = mapping.window_point_to_uv_clamped(position);
        let target_px = mapping.window_point_to_target_px_clamped(position);
        Self {
            window,
            target,
            uv,
            target_px,
            kind,
        }
    }
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
        /// See `PointerEvent::{Down,Up}.click_count` for normalization rules.
        click_count: u8,
    },
    PointerUp {
        button: MouseButton,
        modifiers: Modifiers,
        /// See `PointerEvent::{Down,Up}.click_count` for normalization rules.
        click_count: u8,
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
