use std::cmp::Reverse;
use std::sync::Arc;

use fret_canvas::view::PanZoom2D;
use fret_core::{Modifiers, Point, Rect};
use fret_runtime::Model;
use fret_ui::action::{
    OnPinchGesture, OnPointerDown, OnPointerMove, OnPointerUp, OnWheel, PinchGestureCx,
    PointerDownCx, PointerMoveCx, PointerUpCx, WheelCx,
};
use fret_ui::canvas::CanvasPainter;
use fret_ui::{ElementContext, UiHost};

use super::canvas_pan_zoom::{
    PanZoomCanvasPaintCx, PanZoomCanvasSurfacePanelProps, pan_zoom_canvas_surface_panel,
};
use crate::declarative::controllable_state::use_controllable_model;
use crate::declarative::model_watch::ModelWatchExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasToolId(pub u64);

impl CanvasToolId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CanvasToolEventCx {
    pub bounds: Rect,
    pub view: PanZoom2D,
    pub screen: Point,
    pub canvas: Point,
    pub pixels_per_point: f32,
    pub modifiers: Modifiers,
    /// Suggested scale factor for hosted raster caches (text shaping, tessellation).
    pub raster_scale_factor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasToolDownResult {
    pub handled: bool,
    pub activate: bool,
    pub capture: bool,
}

impl CanvasToolDownResult {
    pub const fn unhandled() -> Self {
        Self {
            handled: false,
            activate: false,
            capture: false,
        }
    }

    pub const fn handled() -> Self {
        Self {
            handled: true,
            activate: false,
            capture: false,
        }
    }

    pub const fn activate_and_capture() -> Self {
        Self {
            handled: true,
            activate: true,
            capture: true,
        }
    }
}

pub type OnCanvasToolPointerDown = Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiPointerActionHost,
            fret_ui::action::ActionCx,
            CanvasToolEventCx,
            PointerDownCx,
        ) -> CanvasToolDownResult
        + 'static,
>;

pub type OnCanvasToolPointerMove = Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiPointerActionHost,
            fret_ui::action::ActionCx,
            CanvasToolEventCx,
            PointerMoveCx,
        ) -> bool
        + 'static,
>;

pub type OnCanvasToolPointerUp = Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiPointerActionHost,
            fret_ui::action::ActionCx,
            CanvasToolEventCx,
            PointerUpCx,
        ) -> bool
        + 'static,
>;

pub type OnCanvasToolWheel = Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiPointerActionHost,
            fret_ui::action::ActionCx,
            CanvasToolEventCx,
            WheelCx,
        ) -> bool
        + 'static,
>;

pub type OnCanvasToolPinch = Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiPointerActionHost,
            fret_ui::action::ActionCx,
            CanvasToolEventCx,
            PinchGestureCx,
        ) -> bool
        + 'static,
>;

pub type OnCanvasToolPaint =
    Arc<dyn for<'p> Fn(&mut CanvasPainter<'p>, PanZoomCanvasPaintCx) + 'static>;

#[derive(Clone, Default)]
pub struct CanvasToolHandlers {
    pub on_pointer_down: Option<OnCanvasToolPointerDown>,
    pub on_pointer_move: Option<OnCanvasToolPointerMove>,
    pub on_pointer_up: Option<OnCanvasToolPointerUp>,
    pub on_wheel: Option<OnCanvasToolWheel>,
    pub on_pinch: Option<OnCanvasToolPinch>,
    pub on_paint: Option<OnCanvasToolPaint>,
}

#[derive(Clone)]
pub struct CanvasToolEntry {
    pub id: CanvasToolId,
    pub priority: i32,
    pub handlers: CanvasToolHandlers,
}

#[derive(Clone)]
pub struct CanvasToolRouterProps {
    /// Pan/zoom substrate props used for the canvas surface.
    pub pan_zoom: PanZoomCanvasSurfacePanelProps,

    /// Optional externally-controlled active tool id.
    pub active_tool: Option<Model<Option<CanvasToolId>>>,
}

impl Default for CanvasToolRouterProps {
    fn default() -> Self {
        Self {
            pan_zoom: PanZoomCanvasSurfacePanelProps::default(),
            active_tool: None,
        }
    }
}

fn compute_tool_cx(
    view: PanZoom2D,
    bounds: Rect,
    screen: Point,
    pixels_per_point: f32,
    modifiers: Modifiers,
) -> CanvasToolEventCx {
    let canvas = view.screen_to_canvas(bounds, screen);
    let raster_scale_factor = pixels_per_point * PanZoom2D::sanitize_zoom(view.zoom, 1.0);
    CanvasToolEventCx {
        bounds,
        view,
        screen,
        canvas,
        pixels_per_point,
        modifiers,
        raster_scale_factor,
    }
}

fn tool_by_id<'a>(tools: &'a [CanvasToolEntry], id: CanvasToolId) -> Option<&'a CanvasToolEntry> {
    tools.iter().find(|t| t.id == id)
}

#[track_caller]
pub fn canvas_tool_router_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut props: CanvasToolRouterProps,
    mut tools: Vec<CanvasToolEntry>,
    paint: impl for<'p> Fn(&mut CanvasPainter<'p>, PanZoomCanvasPaintCx) + 'static,
) -> fret_ui::element::AnyElement {
    tools.sort_by_key(|t| Reverse(t.priority));
    let tools: Arc<[CanvasToolEntry]> = tools.into();

    let view_model = use_controllable_model(cx, props.pan_zoom.view.clone(), || {
        props.pan_zoom.default_view
    })
    .model();
    props.pan_zoom.view = Some(view_model.clone());

    let active_tool_model: Model<Option<CanvasToolId>> =
        use_controllable_model(cx, props.active_tool.clone(), || None).model();

    let view_value = cx
        .watch_model(&view_model)
        .paint()
        .copied()
        .unwrap_or(props.pan_zoom.default_view);
    let active_tool_value = cx
        .watch_model(&active_tool_model)
        .paint()
        .copied()
        .unwrap_or(None);

    let tools_down = tools.clone();
    let view_model_c = view_model.clone();
    let active_tool_model_c = active_tool_model.clone();
    let on_tool_pointer_down: OnPointerDown = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost, action_cx, down| {
            let view = host
                .models_mut()
                .read(&view_model_c, |v| *v)
                .ok()
                .unwrap_or(PanZoom2D::default());
            let bounds = host.bounds();
            let cx2 = compute_tool_cx(
                view,
                bounds,
                down.position,
                down.pixels_per_point,
                down.modifiers,
            );

            for tool in tools_down.iter() {
                let Some(handler) = tool.handlers.on_pointer_down.as_ref() else {
                    continue;
                };
                let res = handler(host, action_cx, cx2, down);
                if !res.handled {
                    continue;
                }

                if res.activate {
                    let _ = host
                        .models_mut()
                        .update(&active_tool_model_c, |t| *t = Some(tool.id));
                }
                if res.capture {
                    host.capture_pointer();
                }
                host.request_redraw(action_cx.window);
                return true;
            }
            false
        },
    );

    let tools_move = tools.clone();
    let view_model_c = view_model.clone();
    let active_tool_model_c = active_tool_model.clone();
    let on_tool_pointer_move: OnPointerMove = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost, action_cx, mv| {
            let view = host
                .models_mut()
                .read(&view_model_c, |v| *v)
                .ok()
                .unwrap_or(PanZoom2D::default());
            let bounds = host.bounds();
            let cx2 = compute_tool_cx(view, bounds, mv.position, mv.pixels_per_point, mv.modifiers);

            let active = host
                .models_mut()
                .read(&active_tool_model_c, |t| *t)
                .ok()
                .flatten();
            if let Some(active) = active
                && let Some(tool) = tool_by_id(&tools_move, active)
                && let Some(handler) = tool.handlers.on_pointer_move.as_ref()
            {
                let handled = handler(host, action_cx, cx2, mv);
                if handled {
                    host.request_redraw(action_cx.window);
                }
                return handled;
            }

            for tool in tools_move.iter() {
                let Some(handler) = tool.handlers.on_pointer_move.as_ref() else {
                    continue;
                };
                let handled = handler(host, action_cx, cx2, mv);
                if handled {
                    host.request_redraw(action_cx.window);
                    return true;
                }
            }
            false
        },
    );

    let tools_up = tools.clone();
    let view_model_c = view_model.clone();
    let active_tool_model_c = active_tool_model.clone();
    let on_tool_pointer_up: OnPointerUp = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost, action_cx, up| {
            let view = host
                .models_mut()
                .read(&view_model_c, |v| *v)
                .ok()
                .unwrap_or(PanZoom2D::default());
            let bounds = host.bounds();
            let cx2 = compute_tool_cx(view, bounds, up.position, up.pixels_per_point, up.modifiers);

            let active = host
                .models_mut()
                .read(&active_tool_model_c, |t| *t)
                .ok()
                .flatten();

            let handled = if let Some(active) = active
                && let Some(tool) = tool_by_id(&tools_up, active)
                && let Some(handler) = tool.handlers.on_pointer_up.as_ref()
            {
                handler(host, action_cx, cx2, up)
            } else {
                false
            };

            let _ = host
                .models_mut()
                .update(&active_tool_model_c, |t| *t = None);
            if handled {
                host.request_redraw(action_cx.window);
            }
            handled
        },
    );

    let tools_wheel = tools.clone();
    let view_model_c = view_model.clone();
    let active_tool_model_c = active_tool_model.clone();
    let on_tool_wheel: OnWheel = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost, action_cx, wheel| {
            let view = host
                .models_mut()
                .read(&view_model_c, |v| *v)
                .ok()
                .unwrap_or(PanZoom2D::default());
            let bounds = host.bounds();
            let cx2 = compute_tool_cx(
                view,
                bounds,
                wheel.position,
                wheel.pixels_per_point,
                wheel.modifiers,
            );

            let active = host
                .models_mut()
                .read(&active_tool_model_c, |t| *t)
                .ok()
                .flatten();
            if let Some(active) = active
                && let Some(tool) = tool_by_id(&tools_wheel, active)
                && let Some(handler) = tool.handlers.on_wheel.as_ref()
            {
                let handled = handler(host, action_cx, cx2, wheel);
                if handled {
                    host.request_redraw(action_cx.window);
                }
                return handled;
            }

            for tool in tools_wheel.iter() {
                let Some(handler) = tool.handlers.on_wheel.as_ref() else {
                    continue;
                };
                let handled = handler(host, action_cx, cx2, wheel);
                if handled {
                    host.request_redraw(action_cx.window);
                    return true;
                }
            }
            false
        },
    );

    let tools_pinch = tools.clone();
    let view_model_c = view_model.clone();
    let active_tool_model_c = active_tool_model.clone();
    let on_tool_pinch: OnPinchGesture = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost, action_cx, pinch| {
            let view = host
                .models_mut()
                .read(&view_model_c, |v| *v)
                .ok()
                .unwrap_or(PanZoom2D::default());
            let bounds = host.bounds();
            let cx2 = compute_tool_cx(
                view,
                bounds,
                pinch.position,
                pinch.pixels_per_point,
                pinch.modifiers,
            );

            let active = host
                .models_mut()
                .read(&active_tool_model_c, |t| *t)
                .ok()
                .flatten();
            if let Some(active) = active
                && let Some(tool) = tool_by_id(&tools_pinch, active)
                && let Some(handler) = tool.handlers.on_pinch.as_ref()
            {
                let handled = handler(host, action_cx, cx2, pinch);
                if handled {
                    host.request_redraw(action_cx.window);
                }
                return handled;
            }

            for tool in tools_pinch.iter() {
                let Some(handler) = tool.handlers.on_pinch.as_ref() else {
                    continue;
                };
                let handled = handler(host, action_cx, cx2, pinch);
                if handled {
                    host.request_redraw(action_cx.window);
                    return true;
                }
            }
            false
        },
    );

    props.pan_zoom.on_pointer_down = Some(on_tool_pointer_down);
    props.pan_zoom.on_pointer_move = Some(on_tool_pointer_move);
    props.pan_zoom.on_pointer_up = Some(on_tool_pointer_up);
    props.pan_zoom.on_wheel = Some(on_tool_wheel);
    props.pan_zoom.on_pinch_gesture = Some(on_tool_pinch);

    let tools_paint = tools.clone();
    let paint_view = view_value;
    let paint_active_tool = active_tool_value;
    let paint = move |painter: &mut CanvasPainter<'_>, paint_cx: PanZoomCanvasPaintCx| {
        paint(painter, paint_cx);

        let active = paint_active_tool;
        for tool in tools_paint.iter() {
            let Some(on_paint) = tool.handlers.on_paint.as_ref() else {
                continue;
            };

            // For now, paint all tools. Tool-specific filtering can be implemented by
            // tool-owned models (preferred), or by supplying different tool sets per mode.
            let _ = paint_view;
            let _ = active;
            on_paint(painter, paint_cx);
        }
    };

    pan_zoom_canvas_surface_panel(cx, props.pan_zoom, paint)
}

#[cfg(all(test, feature = "recipes"))]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
        PathStyle, Point, PointerType, Px, Rect, Scene, Size, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, TextStyle,
    };
    use fret_runtime::{FrameId, TickId};
    use fret_ui::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
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

    fn bounds() -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(80.0)))
    }

    fn bump_frame(app: &mut App) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn tool_router_captures_and_routes_drag() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let down_count = app.models_mut().insert(0u32);
        let move_count = app.models_mut().insert(0u32);
        let up_count = app.models_mut().insert(0u32);

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "tool-router",
            |cx| {
                let down_m = down_count.clone();
                let move_m = move_count.clone();
                let up_m = up_count.clone();

                let tool = CanvasToolEntry {
                    id: CanvasToolId::new(1),
                    priority: 0,
                    handlers: CanvasToolHandlers {
                        on_pointer_down: Some(Arc::new(
                            move |host, _cx, _tool_cx, down: PointerDownCx| {
                                if down.button != fret_core::MouseButton::Left {
                                    return CanvasToolDownResult::unhandled();
                                }
                                let _ = host
                                    .models_mut()
                                    .update(&down_m, |v: &mut u32| *v = v.saturating_add(1));
                                CanvasToolDownResult::activate_and_capture()
                            },
                        )),
                        on_pointer_move: Some(Arc::new(move |host, _cx, _tool_cx, _mv| {
                            let _ = host
                                .models_mut()
                                .update(&move_m, |v: &mut u32| *v = v.saturating_add(1));
                            true
                        })),
                        on_pointer_up: Some(Arc::new(move |host, _cx, _tool_cx, _up| {
                            let _ = host
                                .models_mut()
                                .update(&up_m, |v: &mut u32| *v = v.saturating_add(1));
                            true
                        })),
                        ..Default::default()
                    },
                };

                let props = CanvasToolRouterProps::default();
                vec![canvas_tool_router_panel(
                    cx,
                    props,
                    vec![tool],
                    |_p, _cx| {},
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);
        ui.paint_all(
            &mut app,
            &mut services,
            bounds(),
            &mut Scene::default(),
            1.0,
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(1000.0), Px(1000.0)),
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(1000.0), Px(1000.0)),
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        assert_eq!(app.models().get_copied(&down_count), Some(1));
        assert_eq!(app.models().get_copied(&move_count), Some(1));
        assert_eq!(app.models().get_copied(&up_count), Some(1));
    }
}
