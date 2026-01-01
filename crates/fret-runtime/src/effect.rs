use std::time::Duration;

use crate::{ClipboardToken, ExternalDropToken, FileDialogToken, TimerToken};
use fret_core::{
    AppWindowId, CursorIcon, ExternalDropReadLimits, FileDialogOptions, Rect, WindowAnchor,
};

use crate::CommandId;

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Redraw(AppWindowId),
    Window(WindowRequest),
    Command {
        window: Option<AppWindowId>,
        command: CommandId,
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
    Raise {
        window: AppWindowId,
        sender: Option<AppWindowId>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateWindowRequest {
    pub kind: CreateWindowKind,
    pub anchor: Option<WindowAnchor>,
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
