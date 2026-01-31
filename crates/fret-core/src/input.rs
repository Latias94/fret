use crate::{
    AppWindowId, ClipboardToken, ExternalDropToken, FileDialogDataEvent, FileDialogSelection,
    ImageId, ImageUpdateToken, ImageUploadToken, PointerId, Rect, RenderTargetId, TimerToken,
    ViewportFit, ViewportMapping, WindowLogicalPosition,
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

    pub last_input_type: Option<String>,
    pub last_beforeinput_data: Option<String>,
    pub last_input_data: Option<String>,

    pub last_key_code: Option<KeyCode>,
    pub last_cursor_area: Option<Rect>,

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportInputGeometry {
    /// The viewport widget bounds in window-local logical pixels (ADR 0017).
    pub content_rect_px: Rect,
    /// The mapped draw rect in window-local logical pixels after applying the viewport `fit`.
    pub draw_rect_px: Rect,
    /// The backing render target size in physical pixels.
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    /// Pixels-per-point (a.k.a. window scale factor) used to convert logical px → physical px.
    pub pixels_per_point: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportInputEvent {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub pointer_id: PointerId,
    pub pointer_type: PointerType,
    pub geometry: ViewportInputGeometry,
    /// Cursor position in window-local logical pixels (ADR 0017).
    pub cursor_px: Point,
    pub uv: (f32, f32),
    pub target_px: (u32, u32),
    pub kind: ViewportInputKind,
}

impl ViewportInputEvent {
    /// Returns the scale from window-local logical pixels ("screen px") to render-target pixels.
    ///
    /// This is derived from `self.geometry.draw_rect_px` (logical pixels) and the backing render
    /// target size `self.geometry.target_px_size` (physical pixels).
    ///
    /// For `ViewportFit::Contain`/`Cover` this is uniform; for `ViewportFit::Stretch` the mapping
    /// is non-uniform, so this returns the smaller axis scale as a conservative approximation for
    /// isotropic thresholds (hit radii, click distances).
    pub fn target_px_per_screen_px(&self) -> Option<f32> {
        let (tw, th) = self.geometry.target_px_size;
        let tw = tw.max(1) as f32;
        let th = th.max(1) as f32;

        let rect = self.geometry.draw_rect_px;
        let dw = rect.size.width.0.max(0.0);
        let dh = rect.size.height.0.max(0.0);
        if dw <= 0.0 || dh <= 0.0 || !dw.is_finite() || !dh.is_finite() {
            return None;
        }

        let sx = tw / dw;
        let sy = th / dh;
        let s = sx.min(sy);
        (s.is_finite() && s > 0.0).then_some(s)
    }

    /// Computes the cursor position in the viewport render target's pixel space (float).
    ///
    /// - Input `self.cursor_px` is in window-local logical pixels (ADR 0017).
    /// - The mapping uses `self.geometry.draw_rect_px` (logical pixels) as the area that maps to
    ///   the full render target.
    /// - Output is expressed in physical target pixels (`self.geometry.target_px_size`).
    ///
    /// This is useful for editor tooling that operates directly on render-target pixel buffers.
    /// Prefer this over reconstructing target coordinates from `uv * target_px_size` because `uv`
    /// and `target_px` may be clamped when pointer capture is active.
    pub fn cursor_target_px_f32(&self) -> Option<(f32, f32)> {
        let (tw, th) = self.geometry.target_px_size;
        let tw = tw.max(1) as f32;
        let th = th.max(1) as f32;

        let rect = self.geometry.draw_rect_px;
        let dw = rect.size.width.0.max(0.0);
        let dh = rect.size.height.0.max(0.0);
        if dw <= 0.0 || dh <= 0.0 || !dw.is_finite() || !dh.is_finite() {
            return None;
        }

        let uv_x = (self.cursor_px.x.0 - rect.origin.x.0) / dw;
        let uv_y = (self.cursor_px.y.0 - rect.origin.y.0) / dh;
        Some((uv_x * tw, uv_y * th))
    }

    /// Like [`Self::cursor_target_px_f32`], but clamps the resulting coordinates to the render
    /// target bounds.
    pub fn cursor_target_px_f32_clamped(&self) -> (f32, f32) {
        let (tw, th) = self.geometry.target_px_size;
        let tw = tw.max(1) as f32;
        let th = th.max(1) as f32;

        let Some((x, y)) = self.cursor_target_px_f32() else {
            return (self.target_px.0 as f32, self.target_px.1 as f32);
        };
        (x.clamp(0.0, tw), y.clamp(0.0, th))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_mapping_window_point(
        window: AppWindowId,
        target: RenderTargetId,
        mapping: &ViewportMapping,
        pixels_per_point: f32,
        pointer_id: PointerId,
        pointer_type: PointerType,
        position: Point,
        kind: ViewportInputKind,
    ) -> Option<Self> {
        let mapped = mapping.map();
        let uv = mapping.window_point_to_uv(position)?;
        let target_px = mapping.window_point_to_target_px(position)?;
        Some(Self {
            window,
            target,
            pointer_id,
            pointer_type,
            geometry: ViewportInputGeometry {
                content_rect_px: mapping.content_rect,
                draw_rect_px: mapped.draw_rect,
                target_px_size: mapping.target_px_size,
                fit: mapping.fit,
                pixels_per_point,
            },
            cursor_px: position,
            uv,
            target_px,
            kind,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_mapping_window_point_clamped(
        window: AppWindowId,
        target: RenderTargetId,
        mapping: &ViewportMapping,
        pixels_per_point: f32,
        pointer_id: PointerId,
        pointer_type: PointerType,
        position: Point,
        kind: ViewportInputKind,
    ) -> Self {
        let mapped = mapping.map();
        let uv = mapping.window_point_to_uv_clamped(position);
        let target_px = mapping.window_point_to_target_px_clamped(position);
        Self {
            window,
            target,
            pointer_id,
            pointer_type,
            geometry: ViewportInputGeometry {
                content_rect_px: mapping.content_rect,
                draw_rect_px: mapped.draw_rect,
                target_px_size: mapping.target_px_size,
                fit: mapping.fit,
                pixels_per_point,
            },
            cursor_px: position,
            uv,
            target_px,
            kind,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_mapping_window_point_maybe_clamped(
        window: AppWindowId,
        target: RenderTargetId,
        mapping: &ViewportMapping,
        pixels_per_point: f32,
        pointer_id: PointerId,
        pointer_type: PointerType,
        position: Point,
        kind: ViewportInputKind,
        clamped: bool,
    ) -> Option<Self> {
        if clamped {
            Some(Self::from_mapping_window_point_clamped(
                window,
                target,
                mapping,
                pixels_per_point,
                pointer_id,
                pointer_type,
                position,
                kind,
            ))
        } else {
            Self::from_mapping_window_point(
                window,
                target,
                mapping,
                pixels_per_point,
                pointer_id,
                pointer_type,
                position,
                kind,
            )
        }
    }
}

#[cfg(test)]
mod viewport_input_event_tests {
    use super::*;
    use crate::geometry::{Point, Px, Rect, Size};

    fn dummy_event(cursor: Point) -> ViewportInputEvent {
        ViewportInputEvent {
            window: AppWindowId::default(),
            target: RenderTargetId::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
            geometry: ViewportInputGeometry {
                content_rect_px: Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(200.0), Px(100.0)),
                ),
                draw_rect_px: Rect::new(
                    Point::new(Px(50.0), Px(25.0)),
                    Size::new(Px(100.0), Px(50.0)),
                ),
                target_px_size: (1000, 500),
                fit: ViewportFit::Contain,
                pixels_per_point: 2.0,
            },
            cursor_px: cursor,
            uv: (0.0, 0.0),
            target_px: (0, 0),
            kind: ViewportInputKind::PointerMove {
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            },
        }
    }

    #[test]
    fn target_px_per_screen_px_matches_draw_rect_mapping() {
        let event = dummy_event(Point::new(Px(0.0), Px(0.0)));
        let scale = event.target_px_per_screen_px().unwrap();
        assert!((scale - 10.0).abs() < 1e-3);
    }

    #[test]
    fn cursor_target_px_maps_draw_rect_origin_to_zero() {
        let event = dummy_event(Point::new(Px(50.0), Px(25.0)));
        let (x, y) = event.cursor_target_px_f32().unwrap();
        assert!(((x - 0.0).powi(2) + (y - 0.0).powi(2)).sqrt() < 1e-3);
    }

    #[test]
    fn cursor_target_px_maps_draw_rect_max_to_target_size() {
        let event = dummy_event(Point::new(Px(150.0), Px(75.0)));
        let (x, y) = event.cursor_target_px_f32().unwrap();
        assert!(((x - 1000.0).powi(2) + (y - 500.0).powi(2)).sqrt() < 1e-3);
    }

    #[test]
    fn cursor_target_px_clamped_caps_outside_values() {
        let event = dummy_event(Point::new(Px(200.0), Px(125.0)));
        let (x, y) = event.cursor_target_px_f32_clamped();
        assert_eq!((x, y), (1000.0, 500.0));
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
        /// Whether this pointer-up completes a "true click".
        ///
        /// See `PointerEvent::Up.is_click` for normalization rules.
        is_click: bool,
        /// See `PointerEvent::{Down,Up}.click_count` for normalization rules.
        click_count: u8,
    },
    PointerCancel {
        buttons: MouseButtons,
        modifiers: Modifiers,
        reason: PointerCancelReason,
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
