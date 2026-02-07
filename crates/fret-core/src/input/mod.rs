use crate::{
    ClipboardToken, ExternalDropToken, FileDialogDataEvent, FileDialogSelection, ImageId,
    ImageUpdateToken, ImageUploadToken, PointerId, Rect, TimerToken, WindowLogicalPosition,
    geometry::{Point, Px},
};

mod keyboard;
pub use keyboard::{KeyCode, keycode_to_ascii_lowercase};

mod viewport;
pub use viewport::{ViewportInputEvent, ViewportInputGeometry, ViewportInputKind};

#[cfg(test)]
mod viewport_input_event_tests;

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
    /// Delete text surrounding the cursor or selection.
    ///
    /// This event does not affect the preedit string. See winit's `Ime::DeleteSurrounding` docs.
    ///
    /// Offsets are expressed in UTF-8 bytes.
    DeleteSurrounding {
        before_bytes: usize,
        after_bytes: usize,
    },
}

/// Debug snapshot for the wasm textarea IME bridge (ADR 0195).
///
/// This is intended for diagnostics/harness views and is not a normative contract surface.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebImeBridgeDebugSnapshot {
    pub enabled: bool,
    pub composing: bool,
    pub suppress_next_input: bool,

    /// Whether the hidden textarea is currently the document's active element (focused).
    ///
    /// This is best-effort and intended for diagnosing browser "user activation" restrictions where
    /// `focus()` calls can be ignored.
    pub textarea_has_focus: Option<bool>,
    /// Tag name of `document.activeElement` when available (e.g. `"TEXTAREA"`, `"CANVAS"`).
    pub active_element_tag: Option<String>,

    /// Where the hidden textarea is positioned.
    ///
    /// This is intentionally stringly-typed to keep the snapshot portable across runners.
    /// Expected values: `"absolute"`, `"fixed"`, or `None` if not initialized.
    pub position_mode: Option<String>,
    /// Describes what the textarea is mounted into.
    ///
    /// Expected values: `"overlay"`, `"mount"`, `"body"`, or `None` if not initialized.
    pub mount_kind: Option<String>,
    /// Device pixel ratio at the time the bridge was initialized or last updated.
    pub device_pixel_ratio: Option<f64>,

    /// Debug-only textarea metrics (DOM-reported).
    ///
    /// These help diagnose candidate UI jitter and unexpected wrapping/scrolling behaviors across
    /// browsers and IMEs. Units are CSS pixels unless otherwise noted.
    pub textarea_value_chars: Option<usize>,
    pub textarea_selection_start_utf16: Option<u32>,
    pub textarea_selection_end_utf16: Option<u32>,
    pub textarea_client_width_px: Option<i32>,
    pub textarea_client_height_px: Option<i32>,
    pub textarea_scroll_width_px: Option<i32>,
    pub textarea_scroll_height_px: Option<i32>,

    pub last_input_type: Option<String>,
    pub last_beforeinput_data: Option<String>,
    pub last_input_data: Option<String>,

    pub last_key_code: Option<KeyCode>,
    pub last_cursor_area: Option<Rect>,
    /// Where the hidden textarea is anchored (CSS px, relative to its positioning context).
    ///
    /// This is derived from `last_cursor_area` by the web runner and helps diagnose candidate UI
    /// offsets (e.g. top-left vs center anchoring).
    pub last_cursor_anchor_px: Option<(f32, f32)>,

    /// Truncated preedit text observed during `compositionupdate`.
    pub last_preedit_text: Option<String>,
    /// Preedit cursor range in UTF-16 code units (begin, end) as reported by the textarea.
    pub last_preedit_cursor_utf16: Option<(u32, u32)>,
    /// Truncated committed text observed during `compositionend` or `input`.
    pub last_commit_text: Option<String>,

    /// Recent IME-related DOM events (debug-only ring buffer).
    ///
    /// Intended to help diagnose ordering differences across browsers/IMEs.
    pub recent_events: Vec<String>,

    pub beforeinput_seen: u64,
    pub input_seen: u64,
    pub suppressed_input_seen: u64,
    pub composition_start_seen: u64,
    pub composition_update_seen: u64,
    pub composition_end_seen: u64,
    pub cursor_area_set_seen: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent {
    Move {
        pointer_id: PointerId,
        position: Point,
        buttons: MouseButtons,
        modifiers: Modifiers,
        pointer_type: PointerType,
    },
    Down {
        pointer_id: PointerId,
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
        pointer_id: PointerId,
        position: Point,
        button: MouseButton,
        modifiers: Modifiers,
        /// Whether this pointer-up completes a "true click" (press + release without exceeding
        /// the runner's click slop threshold).
        ///
        /// This signal is computed by the platform runner and is intentionally separate from
        /// `click_count`: `click_count` can remain stable even when a press turns into a drag.
        is_click: bool,
        /// Consecutive click count for this button (1 = single click, 2 = double click, ...).
        ///
        /// See `PointerEvent::Down.click_count` for the normalization rules.
        click_count: u8,
        pointer_type: PointerType,
    },
    Wheel {
        pointer_id: PointerId,
        position: Point,
        delta: Point,
        modifiers: Modifiers,
        pointer_type: PointerType,
    },
    /// Two-finger pinch gesture, typically produced by touchpads (and some touch platforms).
    ///
    /// `delta` is positive for magnification (zoom in) and negative for shrinking (zoom out).
    /// This value may be NaN depending on the platform backend; callers should guard accordingly.
    PinchGesture {
        pointer_id: PointerId,
        position: Point,
        delta: f32,
        modifiers: Modifiers,
        pointer_type: PointerType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerCancelReason {
    /// The pointer left the window (e.g. cursor left the window, or touch tracking was canceled).
    LeftWindow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerCancelEvent {
    pub pointer_id: PointerId,
    /// When provided by the platform, this is the last known pointer position (logical pixels).
    pub position: Option<Point>,
    pub buttons: MouseButtons,
    pub modifiers: Modifiers,
    pub pointer_type: PointerType,
    pub reason: PointerCancelReason,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InternalDragKind {
    Enter,
    Over,
    Drop,
    Leave,
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InternalDragEvent {
    pub pointer_id: PointerId,
    pub position: Point,
    pub kind: InternalDragKind,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Pointer(PointerEvent),
    PointerCancel(PointerCancelEvent),
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
    /// Linux primary selection text payload delivered to the focused widget.
    ///
    /// This typically originates from middle-click paste when primary selection is enabled.
    PrimarySelectionText {
        token: ClipboardToken,
        text: String,
    },
    /// Primary selection read completed without a text payload (unavailable/empty/error).
    PrimarySelectionTextUnavailable {
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}
