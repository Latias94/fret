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
pub struct ToastLayerSpec {
    pub store: Model<window_overlays::ToastStore>,
    pub position: window_overlays::ToastPosition,
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
    pub toast_layer: Option<ToastLayerSpec>,
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
            toast_layer: None,
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
            toast_layer: None,
        }
    }

    pub fn tooltip(
        id: GlobalElementId,
        presence: OverlayPresence,
        children: Vec<AnyElement>,
    ) -> Self {
        Self {
            kind: OverlayKind::Tooltip,
            id,
            root_name: None,
            trigger: None,
            open: None,
            presence,
            initial_focus: None,
            children,
            toast_layer: None,
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
            toast_layer: None,
        }
    }

    pub fn toast_layer(id: GlobalElementId, store: Model<window_overlays::ToastStore>) -> Self {
        Self {
            kind: OverlayKind::ToastLayer,
            id,
            root_name: None,
            trigger: None,
            open: None,
            presence: OverlayPresence::hidden(),
            initial_focus: None,
            children: Vec::new(),
            toast_layer: Some(ToastLayerSpec {
                store,
                position: window_overlays::ToastPosition::default(),
            }),
        }
    }

    pub fn toast_position(mut self, position: window_overlays::ToastPosition) -> Self {
        let spec = self
            .toast_layer
            .as_mut()
            .expect("toast_position requires a ToastLayer request");
        spec.position = position;
        self
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

    pub fn request_for_window<H: UiHost>(
        app: &mut H,
        window: AppWindowId,
        request: OverlayRequest,
    ) {
        match request.kind {
            OverlayKind::NonModalDismissible => {
                let open = request
                    .open
                    .expect("NonModalDismissible requires open model");
                let trigger = request
                    .trigger
                    .expect("NonModalDismissible requires trigger");
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
                let spec = request
                    .toast_layer
                    .expect("ToastLayer requires toast_layer spec");
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::toast_layer_root_name(request.id));
                window_overlays::request_toast_layer_for_window(
                    app,
                    window,
                    window_overlays::ToastLayerRequest {
                        id: request.id,
                        root_name,
                        store: spec.store,
                        position: spec.position,
                    },
                );
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

    pub fn toast_store<H: UiHost>(app: &mut H) -> Model<window_overlays::ToastStore> {
        window_overlays::toast_store(app)
    }

    pub fn toast_action(
        host: &mut dyn fret_ui::action::UiActionHost,
        store: Model<window_overlays::ToastStore>,
        window: AppWindowId,
        request: window_overlays::ToastRequest,
    ) -> window_overlays::ToastId {
        window_overlays::toast_action(host, store, window, request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{
        Point, Px, Rect, TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle,
    };

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    #[test]
    fn toast_layer_request_enables_timer_events_when_toasts_exist() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        // Base root.
        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |_| Vec::new(),
        );
        ui.set_root(base);

        // Add a toast entry.
        let store = OverlayController::toast_store(&mut app);
        let _ = OverlayController::toast_action(
            &mut fret_ui::action::UiActionHostAdapter { app: &mut app },
            store.clone(),
            window,
            window_overlays::ToastRequest::new("Hello"),
        );

        // Request toast layer through the controller and render.
        OverlayController::begin_frame(&mut app, window);
        OverlayController::request_for_window(
            &mut app,
            window,
            OverlayRequest::toast_layer(GlobalElementId(0xbeef), store)
                .toast_position(window_overlays::ToastPosition::BottomRight),
        );
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let layers = ui.debug_layers_in_paint_order();
        assert!(
            layers.iter().any(|l| l.wants_timer_events),
            "expected at least one layer to request timer events when toasts exist"
        );
    }
}
