use fret_core::{AppWindowId, Rect};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementCx, UiHost, UiTree};

use crate::declarative::presence::fade_presence;
use crate::headless::presence::PresenceOutput;
use crate::window_overlays;

/// Presence state for an overlay root (mount/paint vs interactive).
///
/// This is intentionally a small, typed wrapper so component code doesn't pass raw `present: bool`
/// around and accidentally conflate it with `open`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayPresence {
    pub present: bool,
    pub interactive: bool,
}

impl OverlayPresence {
    pub fn hidden() -> Self {
        Self {
            present: false,
            interactive: false,
        }
    }

    pub fn instant(open: bool) -> Self {
        Self {
            present: open,
            interactive: open,
        }
    }

    pub fn from_fade(open: bool, presence: PresenceOutput) -> Self {
        Self {
            present: presence.present,
            interactive: open,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    NonModalDismissible,
    Modal,
    Tooltip,
    Hover,
    ToastLayer,
}

#[derive(Debug, Clone)]
pub struct OverlayRequest {
    pub kind: OverlayKind,
    pub id: GlobalElementId,
    pub root_name: Option<String>,
    pub trigger: Option<GlobalElementId>,
    pub open: Option<Model<bool>>,
    pub presence: OverlayPresence,
    pub initial_focus: Option<GlobalElementId>,
    pub children: Vec<AnyElement>,
}

impl OverlayRequest {
    pub fn dismissible_popover(
        id: GlobalElementId,
        trigger: GlobalElementId,
        open: Model<bool>,
        presence: OverlayPresence,
        children: Vec<AnyElement>,
    ) -> Self {
        Self {
            kind: OverlayKind::NonModalDismissible,
            id,
            root_name: None,
            trigger: Some(trigger),
            open: Some(open),
            presence,
            initial_focus: None,
            children,
        }
    }

    pub fn modal(
        id: GlobalElementId,
        trigger: Option<GlobalElementId>,
        open: Model<bool>,
        presence: OverlayPresence,
        children: Vec<AnyElement>,
    ) -> Self {
        Self {
            kind: OverlayKind::Modal,
            id,
            root_name: None,
            trigger,
            open: Some(open),
            presence,
            initial_focus: None,
            children,
        }
    }

    pub fn tooltip(id: GlobalElementId, presence: OverlayPresence, children: Vec<AnyElement>) -> Self {
        Self {
            kind: OverlayKind::Tooltip,
            id,
            root_name: None,
            trigger: None,
            open: None,
            presence,
            initial_focus: None,
            children,
        }
    }

    pub fn hover(id: GlobalElementId, trigger: GlobalElementId, children: Vec<AnyElement>) -> Self {
        Self {
            kind: OverlayKind::Hover,
            id,
            root_name: None,
            trigger: Some(trigger),
            open: None,
            presence: OverlayPresence {
                present: true,
                interactive: true,
            },
            initial_focus: None,
            children,
        }
    }
}

/// A small, stable facade over `window_overlays` to keep overlay policy wiring out of shadcn code.
pub struct OverlayController;

impl OverlayController {
    pub fn begin_frame<H: UiHost>(app: &mut H, window: AppWindowId) {
        window_overlays::begin_frame(app, window);
    }

    pub fn request<H: UiHost>(cx: &mut ElementCx<'_, H>, request: OverlayRequest) {
        Self::request_for_window(cx.app, cx.window, request);
    }

    pub fn request_for_window<H: UiHost>(app: &mut H, window: AppWindowId, request: OverlayRequest) {
        match request.kind {
            OverlayKind::NonModalDismissible => {
                let open = request.open.expect("NonModalDismissible requires open model");
                let trigger = request.trigger.expect("NonModalDismissible requires trigger");
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::popover_root_name(request.id));
                window_overlays::request_dismissible_popover_for_window(
                    app,
                    window,
                    window_overlays::DismissiblePopoverRequest {
                        id: request.id,
                        root_name,
                        trigger,
                        open,
                        present: request.presence.present,
                        initial_focus: request.initial_focus,
                        children: request.children,
                    },
                );
            }
            OverlayKind::Modal => {
                let open = request.open.expect("Modal requires open model");
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::modal_root_name(request.id));
                window_overlays::request_modal_for_window(
                    app,
                    window,
                    window_overlays::ModalRequest {
                        id: request.id,
                        root_name,
                        trigger: request.trigger,
                        open,
                        present: request.presence.present,
                        initial_focus: request.initial_focus,
                        children: request.children,
                    },
                );
            }
            OverlayKind::Tooltip => {
                if !request.presence.present {
                    return;
                }
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::tooltip_root_name(request.id));
                window_overlays::request_tooltip_for_window(
                    app,
                    window,
                    window_overlays::TooltipRequest {
                        id: request.id,
                        root_name,
                        children: request.children,
                    },
                );
            }
            OverlayKind::Hover => {
                let trigger = request.trigger.expect("Hover requires trigger");
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::hover_overlay_root_name(request.id));
                window_overlays::request_hover_overlay_for_window(
                    app,
                    window,
                    window_overlays::HoverOverlayRequest {
                        id: request.id,
                        root_name,
                        trigger,
                        children: request.children,
                    },
                );
            }
            OverlayKind::ToastLayer => {
                // Toast layers have their own store+timer semantics; keep them explicit for now.
                unimplemented!("ToastLayer requests are not yet routed through OverlayController");
            }
        }
    }

    pub fn render<H: UiHost>(
        ui: &mut UiTree<H>,
        app: &mut H,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
    ) {
        window_overlays::render(ui, app, services, window, bounds);
    }

    pub fn fade_presence<H: UiHost>(
        cx: &mut ElementCx<'_, H>,
        open: bool,
        fade_ticks: u64,
    ) -> PresenceOutput {
        fade_presence(cx, open, fade_ticks)
    }
}

