//! Window-scoped overlay manager (policy layer).
//!
//! This is a small component-layer orchestration helper that installs `UiTree` overlay roots
//! (ADR 0067) and coordinates dismissal + focus restore rules (ADR 0069).

use fret_core::{AppWindowId, Rect};
use fret_runtime::Model;
use fret_ui::declarative;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::tree::UiLayerId;
use fret_ui::{ElementCx, UiHost, UiTree};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct DismissiblePopoverRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    pub trigger: GlobalElementId,
    pub open: Model<bool>,
    pub initial_focus: Option<GlobalElementId>,
    pub children: Vec<AnyElement>,
}

impl std::fmt::Debug for DismissiblePopoverRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DismissiblePopoverRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("trigger", &self.trigger)
            .field("open", &"<model>")
            .field("initial_focus", &self.initial_focus)
            .field("children_len", &self.children.len())
            .finish()
    }
}

#[derive(Default)]
struct WindowOverlayFrame {
    frame_id: fret_core::FrameId,
    popovers: Vec<DismissiblePopoverRequest>,
}

struct ActivePopover {
    layer: UiLayerId,
    root_name: String,
    trigger: GlobalElementId,
    initial_focus: Option<GlobalElementId>,
}

#[derive(Default)]
struct WindowOverlays {
    windows: HashMap<AppWindowId, WindowOverlayFrame>,
    popovers: HashMap<(AppWindowId, GlobalElementId), ActivePopover>,
}

pub fn begin_frame<H: UiHost>(app: &mut H, window: AppWindowId) {
    let frame_id = app.frame_id();
    app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        if w.frame_id != frame_id {
            w.frame_id = frame_id;
            w.popovers.clear();
        }
    });
}

pub fn request_dismissible_popover<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    request: DismissiblePopoverRequest,
) {
    request_dismissible_popover_for_window(cx.app, cx.window, request);
}

pub fn request_dismissible_popover_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    request: DismissiblePopoverRequest,
) {
    app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        let w = overlays.windows.entry(window).or_default();
        w.popovers.push(request);
    });
}

pub fn render<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) {
    let requests = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        overlays
            .windows
            .get_mut(&window)
            .map(|w| std::mem::take(&mut w.popovers))
            .unwrap_or_default()
    });

    let mut seen: HashSet<GlobalElementId> = HashSet::new();

    for req in requests {
        seen.insert(req.id);

        let root = declarative::render_dismissible_root(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            req.open,
            |_cx| req.children,
        );

        let key = (window, req.id);

        let mut should_focus_initial = false;
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let entry = overlays
                .popovers
                .entry(key)
                .or_insert_with(|| ActivePopover {
                    layer: ui.push_overlay_root_ex(root, false, true),
                    root_name: req.root_name.clone(),
                    trigger: req.trigger,
                    initial_focus: req.initial_focus,
                });
            entry.root_name = req.root_name.clone();
            entry.trigger = req.trigger;
            entry.initial_focus = req.initial_focus;
            ui.set_layer_wants_pointer_down_outside_events(entry.layer, true);

            let was_visible = ui.is_layer_visible(entry.layer);
            ui.set_layer_visible(entry.layer, true);
            should_focus_initial = !was_visible;
        });

        if should_focus_initial {
            let focus = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
                overlays.popovers.get(&key).and_then(|p| p.initial_focus)
            });

            if let Some(focus) = focus
                && let Some(node) = fret_ui::elements::node_for_element(app, window, focus)
            {
                ui.set_focus(Some(node));
            } else if let Some(node) = ui.first_focusable_descendant(root) {
                ui.set_focus(Some(node));
            }
        }
    }

    let to_hide: Vec<(UiLayerId, GlobalElementId)> =
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays
                .popovers
                .iter()
                .filter_map(|((w, id), active)| {
                    if *w != window || seen.contains(id) {
                        return None;
                    }
                    Some((active.layer, active.trigger))
                })
                .collect()
        });

    for (layer, trigger) in to_hide {
        let focus = ui.focus();
        if focus.is_some_and(|n| ui.node_layer(n) == Some(layer))
            && let Some(trigger_node) = fret_ui::elements::node_for_element(app, window, trigger)
        {
            ui.set_layer_visible(layer, false);
            ui.set_focus(Some(trigger_node));
        } else {
            ui.set_layer_visible(layer, false);
        }
    }
}

pub fn popover_root_name(id: GlobalElementId) -> String {
    format!("window-overlays.popover.{:x}", id.0)
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
    use fret_ui::element::{ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps};

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
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

    fn render_base_with_trigger(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
    ) -> GlobalElementId {
        begin_frame(app, window);

        let mut trigger_id: Option<GlobalElementId> = None;
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        toggle_model: Some(open),
                        a11y: PressableA11y::default(),
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        trigger_id = Some(id);
                        vec![cx.container(ContainerProps::default(), |_| Vec::new())]
                    },
                )]
            });
        ui.set_root(root);
        trigger_id.expect("trigger id")
    }

    #[test]
    fn dismissible_popover_closes_on_outside_press() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        // First frame: render base to establish stable bounds for the trigger element.
        let trigger =
            render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, open);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get(open).copied(), Some(true));

        // Second frame: request and render a dismissible popover.
        begin_frame(&mut app, window);
        let _ = render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, open);

        request_dismissible_popover_for_window(
            &mut app,
            window,
            DismissiblePopoverRequest {
                id: trigger,
                root_name: popover_root_name(trigger),
                trigger,
                open,
                initial_focus: None,
                children: vec![],
            },
        );

        render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Pointer down outside should close (observer pass).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(250.0), Px(180.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get(open).copied(), Some(false));
    }
}
