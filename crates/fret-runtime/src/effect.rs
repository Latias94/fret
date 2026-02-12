use std::time::Duration;

use crate::{
    ClipboardToken, ExternalDropToken, FileDialogToken, ImageUpdateToken, ImageUploadToken,
    IncomingOpenToken, ShareSheetToken, TimerToken,
};
use fret_core::{
    AlphaMode, AppWindowId, CursorIcon, Edges, ExternalDropReadLimits, FileDialogOptions,
    ImageColorInfo, ImageId, Rect, RectPx, WindowAnchor,
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
    /// Set Linux primary selection text (copy-on-select).
    ///
    /// This is intentionally separate from `ClipboardSetText` so selecting text does not
    /// overwrite the explicit clipboard used by `Ctrl+C` / `edit.copy`.
    PrimarySelectionSetText {
        text: String,
    },
    /// Read Linux primary selection text (middle-click paste).
    PrimarySelectionGetText {
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
        target: Option<String>,
        rel: Option<String>,
    },
    /// Show the platform-native share sheet (best-effort).
    ShareSheetShow {
        window: AppWindowId,
        token: ShareSheetToken,
        items: Vec<fret_core::ShareItem>,
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
    /// Read all data associated with an incoming-open token (best-effort).
    IncomingOpenReadAll {
        window: AppWindowId,
        token: IncomingOpenToken,
    },
    IncomingOpenReadAllWithLimits {
        window: AppWindowId,
        token: IncomingOpenToken,
        limits: ExternalDropReadLimits,
    },
    IncomingOpenRelease {
        token: IncomingOpenToken,
    },
    /// Diagnostics-only clipboard override to simulate mobile privacy/user-activation denial paths.
    ///
    /// Notes:
    /// - Runners SHOULD treat this as a best-effort toggle and default to `enabled=false`.
    /// - When enabled, clipboard reads (`ClipboardGetText`, `PrimarySelectionGetText`) SHOULD
    ///   complete as unavailable rather than attempting platform access.
    DiagClipboardForceUnavailable {
        window: AppWindowId,
        enabled: bool,
    },
    /// Add font bytes (TTF/OTF/TTC) to the renderer text system.
    ///
    /// The runner/backend is responsible for applying this to the renderer and triggering any
    /// required invalidation/redraw.
    TextAddFonts {
        fonts: Vec<Vec<u8>>,
    },
    /// Request a best-effort rescan of system-installed fonts (native-only).
    ///
    /// Web/WASM runners should ignore this effect, as they cannot access system font databases.
    ///
    /// Semantics:
    /// - This is an explicit, user-initiated refresh hook (ADR 0258).
    /// - Runners should re-enumerate the font catalog and republish `FontCatalogMetadata` if
    ///   changes are observed.
    /// - Runners should also bump renderer text invalidation keys (e.g. `TextFontStackKey`) so
    ///   cached shaping/rasterization results cannot be reused after a rescan attempt.
    TextRescanSystemFonts,
    ViewportInput(fret_core::ViewportInputEvent),
    Dock(fret_core::DockOp),
    ImeAllow {
        window: AppWindowId,
        enabled: bool,
    },
    /// Best-effort request to show/hide the platform virtual keyboard.
    ///
    /// Notes:
    /// - This does not replace `Effect::ImeAllow`, which remains the source of truth for whether
    ///   the focused widget is a text input.
    /// - Some platforms (notably Android) may require this request to be issued within a
    ///   user-activation turn (direct input event handling), otherwise it may be ignored.
    ImeRequestVirtualKeyboard {
        window: AppWindowId,
        visible: bool,
    },
    ImeSetCursorArea {
        window: AppWindowId,
        rect: Rect,
    },
    /// Override window insets in `WindowMetricsService` (safe area / occlusion).
    ///
    /// This is primarily used by diagnostics/scripted repros to simulate keyboard occlusion on
    /// platforms where the real OS insets are not available in CI.
    ///
    /// Semantics:
    /// - `None` means "no change".
    /// - `Some(None)` clears the insets but still marks them as "known".
    /// - `Some(Some(v))` sets the insets to `v`.
    WindowMetricsSetInsets {
        window: AppWindowId,
        safe_area_insets: Option<Option<Edges>>,
        occlusion_insets: Option<Option<Edges>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowRole {
    Main,
    #[default]
    Auxiliary,
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
