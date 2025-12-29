use std::time::Duration;

use fret_core::{AppWindowId, CursorIcon, Rect, TimerToken, WindowAnchor};

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
    },
    ExternalDropReadAll {
        window: AppWindowId,
        token: fret_core::ExternalDropToken,
    },
    ExternalDropRelease {
        token: fret_core::ExternalDropToken,
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
