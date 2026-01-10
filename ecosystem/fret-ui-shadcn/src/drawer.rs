//! shadcn/ui `Drawer` facade.
//!
//! Fret currently models drawers as a `Sheet` that defaults to the `Bottom` side.

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, MouseButton, Point, Px, Transform2D};
use fret_runtime::Model;
use fret_ui::action::OnDismissRequest;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, PointerRegionProps, PositionStyle,
    SizeStyle, VisualTransformProps,
};
use fret_ui::element::{MarginEdge, MarginEdges};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::Sheet;
use crate::layout as shadcn_layout;
pub use crate::sheet::{
    SheetContent as DrawerContent, SheetDescription as DrawerDescription,
    SheetFooter as DrawerFooter, SheetHeader as DrawerHeader, SheetSide as DrawerSide,
    SheetTitle as DrawerTitle,
};
use crate::{ChromeRefinement, LayoutRefinement};
use fret_ui_kit::Space;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::primitives::dialog as radix_dialog;

#[derive(Clone)]
pub struct Drawer {
    open: Model<bool>,
    side: DrawerSide,
    inner: Sheet,
    drag_to_dismiss: bool,
}

impl std::fmt::Debug for Drawer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drawer")
            .field("open", &"<model>")
            .field("side", &self.side)
            .field("drag_to_dismiss", &self.drag_to_dismiss)
            .finish()
    }
}

impl Drawer {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open: open.clone(),
            side: DrawerSide::Bottom,
            inner: Sheet::new(open).side(DrawerSide::Bottom),
            drag_to_dismiss: true,
        }
    }

    /// Creates a drawer with a controlled/uncontrolled open model (Radix `open` / `defaultOpen`).
    ///
    /// Note: If `open` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_dialog::DialogRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        Self::new(open)
    }

    pub fn overlay_closable(mut self, overlay_closable: bool) -> Self {
        self.inner = self.inner.overlay_closable(overlay_closable);
        self
    }

    pub fn overlay_color(mut self, overlay_color: fret_core::Color) -> Self {
        self.inner = self.inner.overlay_color(overlay_color);
        self
    }

    /// Sets an optional dismiss request handler (Radix `DismissableLayer`).
    ///
    /// When set, Escape dismissals (overlay root) and overlay-click dismissals (barrier press) are
    /// routed through this handler. To "prevent default", do not close the `open` model inside the
    /// handler.
    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.inner = self.inner.on_dismiss_request(on_dismiss_request);
        self
    }

    /// Enables Vaul-style drag-to-dismiss (shadcn Drawer behavior).
    ///
    /// This is intentionally Drawer-only policy and is not part of the Radix primitives boundary.
    pub fn drag_to_dismiss(mut self, enabled: bool) -> Self {
        self.drag_to_dismiss = enabled;
        self
    }

    /// Sets the drawer size (height by default, since drawers default to `Bottom`).
    pub fn size(mut self, size: fret_core::Px) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    /// Optional escape hatch: allow non-bottom drawers by forwarding to `Sheet`.
    pub fn side(mut self, side: DrawerSide) -> Self {
        self.side = side;
        self.inner = self.inner.side(side);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let open = self.open.clone();
        let side = self.side;
        let drag_to_dismiss = self.drag_to_dismiss;

        self.inner.into_element(cx, trigger, move |cx| {
            let content = content(cx);
            if !drag_to_dismiss || side != DrawerSide::Bottom {
                return content;
            }

            let theme = Theme::global(&*cx.app).clone();
            let muted = theme.color_required("muted");

            let is_open = cx.watch_model(&open).copied().unwrap_or(false);
            let (runtime, offset_model, was_open) = drawer_drag_models(cx);

            if is_open && !was_open {
                let _ = cx.app.models_mut().update(&offset_model, |v| *v = Px(0.0));
            }
            drawer_drag_set_was_open(cx, is_open);

            let offset = cx.watch_model(&offset_model).copied().unwrap_or(Px(0.0));
            let transform = Transform2D::translation(Point::new(Px(0.0), offset));

            let runtime_for_down = runtime.clone();
            let offset_for_down = offset_model.clone();
            let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, _cx, down| {
                if !is_open || down.button != MouseButton::Left {
                    return false;
                }

                let bounds = host.bounds();
                if !drawer_drag_hit_test(bounds, down.position) {
                    return false;
                }

                host.capture_pointer();
                let start_offset = host
                    .models_mut()
                    .read(&offset_for_down, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));
                let _ = host.models_mut().update(&runtime_for_down, |st| {
                    st.dragging = true;
                    st.start = down.position;
                    st.start_offset = start_offset;
                });
                host.request_redraw(_cx.window);
                true
            });

            let runtime_for_move = runtime.clone();
            let offset_for_move = offset_model.clone();
            let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, _cx, mv| {
                let Ok((dragging, start, start_offset)) =
                    host.models_mut().read(&runtime_for_move, |st| {
                        (st.dragging, st.start, st.start_offset)
                    })
                else {
                    return false;
                };
                if !dragging {
                    return false;
                }

                let dy = mv.position.y.0 - start.y.0;
                let bounds = host.bounds();
                let next = Px((start_offset.0 + dy).max(0.0).min(bounds.size.height.0));
                let _ = host.models_mut().update(&offset_for_move, |v| *v = next);
                host.request_redraw(_cx.window);
                true
            });

            let open_for_up = open.clone();
            let runtime_for_up = runtime.clone();
            let offset_for_up = offset_model.clone();
            let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, _cx, _up| {
                let dragging = host
                    .models_mut()
                    .read(&runtime_for_up, |st| st.dragging)
                    .ok()
                    .unwrap_or(false);
                if !dragging {
                    return false;
                }

                host.release_pointer_capture();
                let bounds = host.bounds();
                let threshold = Px((bounds.size.height.0 * 0.25).max(DRAWER_DRAG_DISMISS_MIN_PX));
                let offset = host
                    .models_mut()
                    .read(&offset_for_up, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));

                let should_close = offset.0 >= threshold.0;
                if should_close {
                    let _ = host.models_mut().update(&open_for_up, |v| *v = false);
                } else {
                    let _ = host.models_mut().update(&offset_for_up, |v| *v = Px(0.0));
                }

                let _ = host.models_mut().update(&runtime_for_up, |st| {
                    st.dragging = false;
                });
                host.request_redraw(_cx.window);
                true
            });

            let layout_fill = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };

            let handle_overlay = drawer_drag_handle_overlay(cx, muted);
            let content_root = cx.pointer_region(
                PointerRegionProps {
                    layout: layout_fill,
                    enabled: is_open,
                },
                move |cx| {
                    cx.pointer_region_on_pointer_down(on_down);
                    cx.pointer_region_on_pointer_move(on_move);
                    cx.pointer_region_on_pointer_up(on_up);
                    vec![content, handle_overlay]
                },
            );

            cx.visual_transform_props(
                VisualTransformProps {
                    layout: layout_fill,
                    transform,
                },
                move |_cx| vec![content_root],
            )
        })
    }
}

const DRAWER_DRAG_HANDLE_HIT_HEIGHT: f32 = 32.0;
const DRAWER_DRAG_HANDLE_HIT_HALF_WIDTH: f32 = 80.0;
const DRAWER_DRAG_HANDLE_BAR_WIDTH: Px = Px(100.0);
const DRAWER_DRAG_HANDLE_BAR_HEIGHT: Px = Px(8.0);
const DRAWER_DRAG_HANDLE_MARGIN_TOP: Px = Px(16.0);
const DRAWER_DRAG_DISMISS_MIN_PX: f32 = 30.0;

#[derive(Debug, Clone, Copy, Default)]
struct DrawerDragRuntime {
    dragging: bool,
    start: Point,
    start_offset: Px,
}

#[derive(Default)]
struct DrawerDragState {
    runtime: Option<Model<DrawerDragRuntime>>,
    offset: Option<Model<Px>>,
    was_open: bool,
}

fn drawer_drag_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<DrawerDragRuntime>, Model<Px>, bool) {
    let needs_init = cx.with_state(DrawerDragState::default, |state| {
        state.runtime.is_none() || state.offset.is_none()
    });

    if needs_init {
        let runtime = cx.app.models_mut().insert(DrawerDragRuntime::default());
        let offset = cx.app.models_mut().insert(Px(0.0));
        cx.with_state(DrawerDragState::default, |state| {
            state.runtime = Some(runtime);
            state.offset = Some(offset);
        });
    }

    cx.with_state(DrawerDragState::default, |state| {
        let runtime = state.runtime.clone().expect("drawer runtime model");
        let offset = state.offset.clone().expect("drawer offset model");
        (runtime, offset, state.was_open)
    })
}

fn drawer_drag_set_was_open<H: UiHost>(cx: &mut ElementContext<'_, H>, was_open: bool) {
    cx.with_state(DrawerDragState::default, |state| {
        state.was_open = was_open;
    });
}

fn drawer_drag_hit_test(bounds: fret_core::Rect, position: Point) -> bool {
    let local_y = position.y.0 - bounds.origin.y.0;
    if local_y > DRAWER_DRAG_HANDLE_HIT_HEIGHT {
        return false;
    }

    let center_x = bounds.origin.x.0 + bounds.size.width.0 * 0.5;
    let dx = (position.x.0 - center_x).abs();
    dx <= DRAWER_DRAG_HANDLE_HIT_HALF_WIDTH
}

fn drawer_drag_handle_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    muted: Color,
) -> AnyElement {
    let overlay_layout = LayoutStyle {
        position: PositionStyle::Absolute,
        inset: InsetStyle {
            top: Some(Px(0.0)),
            right: Some(Px(0.0)),
            left: Some(Px(0.0)),
            bottom: None,
        },
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Px(Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    };

    let bar = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(DRAWER_DRAG_HANDLE_BAR_WIDTH),
                    height: Length::Px(DRAWER_DRAG_HANDLE_BAR_HEIGHT),
                    ..Default::default()
                },
                margin: MarginEdges {
                    top: MarginEdge::Px(DRAWER_DRAG_HANDLE_MARGIN_TOP),
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(0.0)),
            background: Some(muted),
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(Px(DRAWER_DRAG_HANDLE_BAR_HEIGHT.0 * 0.5)),
        },
        |_cx| Vec::new(),
    );

    shadcn_layout::container_hstack(
        cx,
        ContainerProps {
            layout: overlay_layout,
            padding: Edges::all(Px(0.0)),
            background: None,
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(Px(0.0)),
        },
        stack::HStackProps::default()
            .gap(Space::N0)
            .layout(LayoutRefinement::default().w_full().h_full())
            .justify_center()
            .items_start(),
        vec![bar],
    )
}

/// shadcn/ui `DrawerTrigger` (v4).
#[derive(Debug, Clone)]
pub struct DrawerTrigger {
    child: AnyElement,
}

impl DrawerTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

/// shadcn/ui `DrawerClose` (v4).
///
/// Upstream `DrawerClose` is a thin wrapper around the underlying primitive's `Close` component.
/// In Fret, drawers are backed by modal overlays, so this delegates to `DialogClose`.
#[derive(Clone)]
pub struct DrawerClose {
    inner: crate::DialogClose,
}

impl std::fmt::Debug for DrawerClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawerClose").finish()
    }
}

impl DrawerClose {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            // DrawerClose should behave like a primitive close affordance (no forced positioning).
            // Delegate visuals to `DialogClose`, but override the default absolute positioning.
            inner: crate::DialogClose::new(open)
                .refine_layout(LayoutRefinement::default().relative()),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.inner = self.inner.refine_style(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.inner = self.inner.refine_layout(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.inner.into_element(cx)
    }
}

pub fn drawer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    Drawer::new(open).into_element(cx, trigger, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle};
    use fret_runtime::FrameId;
    use fret_ui::UiTree;
    use fret_ui::action::DismissReason;
    use fret_ui::element::{ContainerProps, LayoutStyle, Length, PressableProps, SizeStyle};
    use fret_ui_kit::MetricRef;
    use fret_ui_kit::OverlayController;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn drawer_new_controllable_can_build_with_or_without_controlled_open_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controlled = app.models_mut().insert(false);

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let _ = Drawer::new_controllable(cx, None, false);
            let _ = Drawer::new_controllable(cx, Some(controlled.clone()), true);
        });
    }

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

    #[test]
    fn drawer_overlay_click_can_be_intercepted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_activated = app.models_mut().insert(false);

        let dismiss_reason: Rc<Cell<Option<DismissReason>>> = Rc::new(Cell::new(None));
        let dismiss_reason_cell = dismiss_reason.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, reason| {
            dismiss_reason_cell.set(Some(reason));
        });

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "test",
            |cx| {
                let underlay = {
                    let underlay_activated = underlay_activated.clone();
                    cx.pressable(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout.position = fret_ui::element::PositionStyle::Absolute;
                                layout.inset.top = Some(Px(0.0));
                                layout.inset.left = Some(Px(0.0));
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_set_bool(&underlay_activated, true);
                            Vec::new()
                        },
                    )
                };

                let trigger = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.top = Some(Px(200.0));
                            layout.inset.left = Some(Px(200.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st| Vec::new(),
                );

                let drawer = Drawer::new(open.clone())
                    .overlay_closable(true)
                    .on_dismiss_request(Some(handler.clone()))
                    .into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Px(Px(20.0)),
                                            height: Length::Px(Px(20.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            )
                        },
                    );

                vec![underlay, drawer]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, b);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        // Click the underlay area. The modal barrier should catch the click and route it through
        // the dismiss handler without closing.
        let point = Point::new(Px(4.0), Px(4.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: point,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: point,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "underlay should not activate while drawer is open"
        );
        assert_eq!(dismiss_reason.get(), Some(DismissReason::OutsidePress));
    }

    #[test]
    fn drawer_drag_dismiss_closes_open_model_when_past_threshold() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "test",
            |cx| {
                let trigger = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st| Vec::new(),
                );

                let drawer = Drawer::new(open.clone()).into_element(
                    cx,
                    |_cx| trigger,
                    |cx| {
                        DrawerContent::new(vec![
                            cx.container(ContainerProps::default(), |_cx| Vec::new()),
                        ])
                        .into_element(cx)
                    },
                );

                vec![drawer]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, b);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let start = Point::new(Px(100.0), Px(10.0));
        let end = Point::new(Px(100.0), Px(90.0));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: start,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: end,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: end,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn drawer_drag_dismiss_keeps_open_model_when_under_threshold() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "test",
            |cx| {
                let trigger = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st| Vec::new(),
                );

                let drawer = Drawer::new(open.clone()).into_element(
                    cx,
                    |_cx| trigger,
                    |cx| {
                        DrawerContent::new(vec![
                            cx.container(ContainerProps::default(), |_cx| Vec::new()),
                        ])
                        .into_element(cx)
                    },
                );

                vec![drawer]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, b);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let start = Point::new(Px(100.0), Px(10.0));
        let end = Point::new(Px(100.0), Px(30.0));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: start,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: end,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: end,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn drawer_close_closes_open_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let close_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let b = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let mut frame = FrameId(1);
        for _ in 0..2 {
            app.set_frame_id(frame);
            frame = FrameId(frame.0.saturating_add(1));

            let open_for_drawer = open.clone();
            let open_for_content = open.clone();

            OverlayController::begin_frame(&mut app, window);
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                b,
                "test",
                |cx| {
                    let trigger = cx.pressable(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        |_cx, _st| Vec::new(),
                    );

                    let close_id_out = close_id.clone();
                    let drawer = Drawer::new(open_for_drawer.clone()).into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let close = DrawerClose::new(open_for_content.clone())
                                .refine_layout(
                                    LayoutRefinement::default()
                                        .relative()
                                        .w_px(MetricRef::Px(Px(24.0)))
                                        .h_px(MetricRef::Px(Px(24.0))),
                                )
                                .into_element(cx);
                            close_id_out.set(Some(close.id));
                            DrawerContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                                close,
                            ])
                            .into_element(cx)
                        },
                    );

                    vec![drawer]
                },
            );
            ui.set_root(root);
            OverlayController::render(&mut ui, &mut app, &mut services, window, b);
            ui.layout_all(&mut app, &mut services, b, 1.0);
        }

        let close_element = close_id.get().expect("close element id");
        let close_node = fret_ui::elements::node_for_element(&mut app, window, close_element)
            .expect("close node");
        let close_bounds = ui.debug_node_bounds(close_node).expect("close bounds");
        let point = Point::new(
            Px(close_bounds.origin.x.0 + 2.0),
            Px(close_bounds.origin.y.0 + 2.0),
        );
        assert!(
            close_bounds.contains(point),
            "expected click point inside close bounds; point={point:?} bounds={close_bounds:?}"
        );
        assert!(
            b.contains(point),
            "expected click point inside window bounds; point={point:?} window={b:?}"
        );

        let pre_hit = ui.debug_hit_test(point).hit.unwrap_or_else(|| {
            panic!("pre-hit missing; point={point:?} close_bounds={close_bounds:?} window={b:?}")
        });
        let pre_path = ui.debug_node_path(pre_hit);
        assert!(
            pre_path.contains(&close_node),
            "expected click point to hit close subtree; point={point:?} hit={pre_hit:?} close={close_node:?} path={pre_path:?}"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: point,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: point,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
    }
}
