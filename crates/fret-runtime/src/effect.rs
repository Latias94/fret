use std::time::Duration;

use crate::window_chrome::WindowResizeDirection;
use crate::window_style::{WindowRole, WindowStyleRequest};
use crate::{
    ClipboardToken, ExternalDropToken, FileDialogToken, ImageUpdateToken, ImageUploadToken,
    IncomingOpenToken, ShareSheetToken, TimerToken,
};
use fret_core::{
    AlphaMode, AppWindowId, CursorIcon, Edges, Event, ExternalDropReadLimits, FileDialogOptions,
    ImageColorInfo, ImageId, Rect, RectPx, WindowAnchor,
};

use crate::{CommandId, MenuBar};

#[derive(Debug, Clone, PartialEq)]
pub enum DiagIncomingOpenItem {
    File {
        name: String,
        bytes: Vec<u8>,
        media_type: Option<String>,
    },
    Text {
        text: String,
        media_type: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
/// Effects emitted by the portable runtime surface.
///
/// Effects are collected by the host (e.g. `fret-app::App`) and are expected to be handled by a
/// runner/backend integration layer (native or web).
///
/// ## Completion events (runner contract)
///
/// Many effects represent an *asynchronous* request to the platform and are completed later by a
/// corresponding [`fret_core::Event`]. Runners/backends should treat these as best-effort.
///
/// Common mappings:
///
/// - `ClipboardReadText { token, .. }` → `fret_core::Event::ClipboardReadText { token, .. }` or
///   `fret_core::Event::ClipboardReadFailed { token, .. }`
/// - `PrimarySelectionGetText { token, .. }` → `fret_core::Event::PrimarySelectionText { token, .. }`
///   or `fret_core::Event::PrimarySelectionTextUnavailable { token, .. }`
/// - `ShareSheetShow { token, .. }` → `fret_core::Event::ShareSheetCompleted { token, .. }`
/// - `FileDialogOpen { .. }` → `fret_core::Event::FileDialogSelection(..)` or
///   `fret_core::Event::FileDialogCanceled`
/// - `FileDialogReadAll { token, .. }` → `fret_core::Event::FileDialogData(..)`
/// - `IncomingOpenReadAll { token, .. }` → `fret_core::Event::IncomingOpenData(..)` or
///   `fret_core::Event::IncomingOpenUnavailable { token, .. }`
/// - `SetTimer { token, .. }` → `fret_core::Event::Timer { token }`
/// - `ImageRegister* { token, .. }` → `fret_core::Event::ImageRegistered { token, .. }` or
///   `fret_core::Event::ImageRegisterFailed { token, .. }`
/// - `ImageUpdate* { token, .. }` → optionally `fret_core::Event::ImageUpdateApplied { token, .. }`
///   or `fret_core::Event::ImageUpdateDropped { token, .. }` when the runner supports these acks
///   (capability-gated to avoid flooding the event loop).
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
    /// Requests writing platform clipboard text (best-effort).
    ClipboardWriteText {
        window: AppWindowId,
        token: ClipboardToken,
        text: String,
    },
    /// Requests reading platform clipboard text (best-effort).
    ///
    /// Runners/backends should eventually complete this request by emitting a corresponding event
    /// carrying `token` (see `ClipboardToken` contract in `fret-core`).
    ClipboardReadText {
        window: AppWindowId,
        token: ClipboardToken,
    },
    /// Set Linux primary selection text (copy-on-select).
    ///
    /// This is intentionally separate from `ClipboardWriteText` so selecting text does not
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
    /// Requests opening a URL using the platform's default handler (best-effort).
    ///
    /// Callers should ensure the URL is safe/expected. Component-layer helpers may apply
    /// additional policies (e.g. `rel="noreferrer"`).
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
    /// Opens a platform-native file dialog (best-effort).
    ///
    /// Runners/backends typically respond by delivering one of:
    /// - `fret_core::Event::FileDialogSelection` (token + names), followed by
    ///   `Effect::FileDialogReadAll` to obtain bytes, or
    /// - `fret_core::Event::FileDialogCanceled` if the user cancels.
    FileDialogOpen {
        window: AppWindowId,
        options: FileDialogOptions,
    },
    /// Requests reading all selected file bytes for a previously opened file dialog.
    FileDialogReadAll {
        window: AppWindowId,
        token: FileDialogToken,
    },
    FileDialogReadAllWithLimits {
        window: AppWindowId,
        token: FileDialogToken,
        limits: ExternalDropReadLimits,
    },
    /// Releases runner-owned resources associated with a file dialog token (best-effort).
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
    /// Releases runner-owned resources associated with an incoming-open token (best-effort).
    IncomingOpenRelease {
        token: IncomingOpenToken,
    },
    /// Diagnostics-only “incoming open” injection (best-effort).
    ///
    /// This simulates mobile-style share-target / open-in flows in CI by injecting an
    /// `Event::IncomingOpenRequest` carrying tokenized items.
    ///
    /// Runners SHOULD:
    ///
    /// - allocate an `IncomingOpenToken`,
    /// - enqueue/deliver `Event::IncomingOpenRequest { token, items }`,
    /// - and retain the injected payload behind the token so subsequent reads can succeed.
    ///
    /// Notes:
    ///
    /// - This is intended for diagnostics/scripts only; real incoming-open requests originate from
    ///   the OS.
    /// - Payload bytes are diagnostic fixtures; they are not intended to model platform handles.
    DiagIncomingOpenInject {
        window: AppWindowId,
        items: Vec<DiagIncomingOpenItem>,
    },
    /// Diagnostics-only synthetic event injection (best-effort).
    ///
    /// This lets tooling deliver an already-constructed runtime event to a specific window even
    /// when that window is not the one currently producing render callbacks.
    DiagInjectEvent {
        window: AppWindowId,
        event: Event,
    },
    /// Diagnostics-only clipboard override to simulate mobile privacy/user-activation denial paths.
    ///
    /// Notes:
    /// - Runners SHOULD treat this as a best-effort toggle and default to `enabled=false`.
    /// - When enabled, clipboard reads (`ClipboardReadText`, `PrimarySelectionGetText`) SHOULD
    ///   complete as unavailable rather than attempting platform access.
    /// - Clipboard writes (`ClipboardWriteText`) SHOULD complete with a failed outcome rather than
    ///   attempting platform access.
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
    /// Requests a timer callback to be delivered as `fret_core::Event::Timer` (best-effort).
    SetTimer {
        window: Option<AppWindowId>,
        token: TimerToken,
        after: Duration,
        repeat: Option<Duration>,
    },
    /// Cancels a previously requested timer (best-effort).
    CancelTimer {
        token: TimerToken,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowRequest {
    Create(CreateWindowRequest),
    Close(AppWindowId),
    /// Request showing or hiding an OS window without destroying it (best-effort).
    SetVisible {
        window: AppWindowId,
        visible: bool,
    },
    SetInnerSize {
        window: AppWindowId,
        size: fret_core::Size,
    },
    /// Request moving the OS window to a screen-space logical position (ADR 0017).
    ///
    /// Runners should treat this as best-effort and may clamp/deny the request based on platform
    /// constraints and user settings.
    SetOuterPosition {
        window: AppWindowId,
        position: fret_core::WindowLogicalPosition,
    },
    Raise {
        window: AppWindowId,
        sender: Option<AppWindowId>,
    },
    /// Begin an OS-native interactive window drag (best-effort).
    BeginDrag {
        window: AppWindowId,
    },
    /// Begin an OS-native interactive window resize (best-effort).
    BeginResize {
        window: AppWindowId,
        direction: WindowResizeDirection,
    },
    /// Best-effort request to update OS window style facets at runtime.
    ///
    /// Semantics:
    /// - This is a patch request: each `Some(...)` field updates that facet, `None` leaves it
    ///   unchanged.
    /// - Runners may ignore unsupported facets based on platform constraints.
    SetStyle {
        window: AppWindowId,
        style: WindowStyleRequest,
    },
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
