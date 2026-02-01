use std::time::Duration;

use crate::{
    ClipboardToken, ExternalDropToken, FileDialogToken, ImageUpdateToken, ImageUploadToken,
    TimerToken,
};
use fret_core::{
    AlphaMode, AppWindowId, CursorIcon, ExternalDropReadLimits, FileDialogOptions, ImageColorInfo,
    ImageId, Rect, RectPx, WindowAnchor,
};

use crate::{CommandId, MenuBar};

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Request a window redraw (one-shot).
    ///
    /// This is the lowest-level redraw primitive. Higher-level UI code typically calls
    /// `App::request_redraw` (or `Cx::request_redraw` / `Cx::request_frame`), which eventually
    /// results in this effect being handled by the runner/backend.
    ///
    /// Semantics:
    /// - This is a one-shot request and may be coalesced by the runner or platform compositor.
    /// - This does **not** imply continuous frame progression. If you need to keep repainting
    ///   without input events (animations, progressive rendering), use
    ///   [`Effect::RequestAnimationFrame`] and re-issue it each frame while active.
    Redraw(AppWindowId),
    Window(WindowRequest),
    Command {
        window: Option<AppWindowId>,
        command: CommandId,
    },
    /// Request the application to quit (native runners may exit their event loop).
    ///
    /// Web runners may ignore this request.
    QuitApp,
    /// Show the standard native "About" panel when available.
    ///
    /// Platform mapping:
    /// - macOS: `NSApplication orderFrontStandardAboutPanel:`
    /// - Other platforms: runners may ignore this request.
    ShowAboutPanel,
    /// Hide the application (macOS: `NSApplication hide:`).
    ///
    /// Other platforms may ignore this request.
    HideApp,
    /// Hide all other applications (macOS: `NSApplication hideOtherApplications:`).
    ///
    /// Other platforms may ignore this request.
    HideOtherApps,
    /// Unhide all applications (macOS: `NSApplication unhideAllApplications:`).
    ///
    /// Other platforms may ignore this request.
    UnhideAllApps,
    /// Set the application/window menu bar (native runners may map this to an OS menubar).
    ///
    /// Notes:
    /// - This is a platform integration seam; web runners may ignore it.
    /// - The menu model is data-only (`MenuBar`) and is typically derived from command metadata
    ///   (ADR 0023).
    SetMenuBar {
        window: Option<AppWindowId>,
        menu_bar: MenuBar,
    },
    ClipboardSetText {
        text: String,
    },
    ClipboardGetText {
        window: AppWindowId,
        token: ClipboardToken,
    },
    ExternalDropReadAll {
        window: AppWindowId,
        token: ExternalDropToken,
    },
    ExternalDropReadAllWithLimits {
        window: AppWindowId,
        token: ExternalDropToken,
        limits: ExternalDropReadLimits,
    },
    ExternalDropRelease {
        token: ExternalDropToken,
    },
    OpenUrl {
        url: String,
    },
    FileDialogOpen {
        window: AppWindowId,
        options: FileDialogOptions,
    },
    FileDialogReadAll {
        window: AppWindowId,
        token: FileDialogToken,
    },
    FileDialogReadAllWithLimits {
        window: AppWindowId,
        token: FileDialogToken,
        limits: ExternalDropReadLimits,
    },
    FileDialogRelease {
        token: FileDialogToken,
    },
    /// Add font bytes (TTF/OTF/TTC) to the renderer text system.
    ///
    /// The runner/backend is responsible for applying this to the renderer and triggering any
    /// required invalidation/redraw.
    TextAddFonts {
        fonts: Vec<Vec<u8>>,
    },
    ViewportInput(fret_core::ViewportInputEvent),
    Dock(fret_core::DockOp),
    ImeAllow {
        window: AppWindowId,
        enabled: bool,
    },
    ImeSetCursorArea {
        window: AppWindowId,
        rect: Rect,
    },
    CursorSetIcon {
        window: AppWindowId,
        icon: CursorIcon,
    },
    ImageRegisterRgba8 {
        window: AppWindowId,
        token: ImageUploadToken,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        color_info: ImageColorInfo,
        alpha_mode: AlphaMode,
    },
    ImageUpdateRgba8 {
        window: Option<AppWindowId>,
        token: ImageUpdateToken,
        image: ImageId,
        stream_generation: u64,
        width: u32,
        height: u32,
        update_rect_px: Option<RectPx>,
        bytes_per_row: u32,
        bytes: Vec<u8>,
        color_info: ImageColorInfo,
        alpha_mode: AlphaMode,
    },
    ImageUpdateNv12 {
        window: Option<AppWindowId>,
        token: ImageUpdateToken,
        image: ImageId,
        stream_generation: u64,
        width: u32,
        height: u32,
        update_rect_px: Option<RectPx>,
        y_bytes_per_row: u32,
        y_plane: Vec<u8>,
        uv_bytes_per_row: u32,
        uv_plane: Vec<u8>,
        color_info: ImageColorInfo,
        alpha_mode: AlphaMode,
    },
    ImageUpdateI420 {
        window: Option<AppWindowId>,
        token: ImageUpdateToken,
        image: ImageId,
        stream_generation: u64,
        width: u32,
        height: u32,
        update_rect_px: Option<RectPx>,
        y_bytes_per_row: u32,
        y_plane: Vec<u8>,
        u_bytes_per_row: u32,
        u_plane: Vec<u8>,
        v_bytes_per_row: u32,
        v_plane: Vec<u8>,
        color_info: ImageColorInfo,
        alpha_mode: AlphaMode,
    },
    ImageUnregister {
        image: fret_core::ImageId,
    },
    /// Request the next animation frame for a window.
    ///
    /// Use this for frame-driven updates (animations, progressive rendering) where the UI must
    /// keep repainting even if there are no new input events.
    ///
    /// This is a one-shot request. Runners/backends should schedule a redraw and keep advancing
    /// the frame counter while these requests are being issued.
    ///
    /// Platform mapping:
    /// - Web backends typically map this to `requestAnimationFrame`.
    /// - Desktop backends typically translate this into a "redraw on the next event-loop turn"
    ///   request (and may coalesce multiple requests).
    RequestAnimationFrame(AppWindowId),
    SetTimer {
        window: Option<AppWindowId>,
        token: TimerToken,
        after: Duration,
        repeat: Option<Duration>,
    },
    CancelTimer {
        token: TimerToken,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowRequest {
    Create(CreateWindowRequest),
    Close(AppWindowId),
    SetInnerSize {
        window: AppWindowId,
        size: fret_core::Size,
    },
    Raise {
        window: AppWindowId,
        sender: Option<AppWindowId>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowRole {
    Main,
    Auxiliary,
}

impl Default for WindowRole {
    fn default() -> Self {
        Self::Auxiliary
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarVisibility {
    Show,
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationPolicy {
    Activates,
    NonActivating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowZLevel {
    Normal,
    AlwaysOnTop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowStyleRequest {
    pub taskbar: Option<TaskbarVisibility>,
    pub activation: Option<ActivationPolicy>,
    pub z_level: Option<WindowZLevel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateWindowRequest {
    pub kind: CreateWindowKind,
    pub anchor: Option<WindowAnchor>,
    pub role: WindowRole,
    pub style: WindowStyleRequest,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateWindowKind {
    DockFloating {
        source_window: AppWindowId,
        panel: fret_core::PanelKey,
    },
    DockRestore {
        logical_window_id: String,
    },
}
