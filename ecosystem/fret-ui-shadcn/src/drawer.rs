//! shadcn/ui `Drawer` facade.
//!
//! Fret currently models drawers as a `Sheet` that defaults to the `Bottom` side.

use fret_runtime::Model;
use fret_ui::action::OnDismissRequest;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::Sheet;
pub use crate::sheet::{
    SheetContent as DrawerContent, SheetDescription as DrawerDescription,
    SheetFooter as DrawerFooter, SheetHeader as DrawerHeader, SheetSide as DrawerSide,
    SheetTitle as DrawerTitle,
};
use crate::{ChromeRefinement, LayoutRefinement};

#[derive(Clone)]
pub struct Drawer {
    inner: Sheet,
}

impl std::fmt::Debug for Drawer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drawer").finish()
    }
}

impl Drawer {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            inner: Sheet::new(open).side(DrawerSide::Bottom),
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
        Self {
            inner: Sheet::new_controllable(cx, open, default_open).side(DrawerSide::Bottom),
        }
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

    /// Sets the drawer size (height by default, since drawers default to `Bottom`).
    pub fn size(mut self, size: fret_core::Px) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    /// Optional escape hatch: allow non-bottom drawers by forwarding to `Sheet`.
    pub fn side(mut self, side: DrawerSide) -> Self {
        self.inner = self.inner.side(side);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.inner.into_element(cx, trigger, content)
    }
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
