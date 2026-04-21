use super::*;
use crate::element::{
    ContainerProps, InsetStyle, LayoutStyle, Length, PointerRegionProps, PositionStyle,
    PressableProps, SizeStyle,
};
use std::{cell::Cell, rc::Rc};

#[derive(Clone)]
struct Counts {
    moves: Model<u32>,
    downs: Model<u32>,
    wheels: Model<u32>,
}

struct CounterWidget {
    counts: Counts,
}

impl<H: UiHost> Widget<H> for CounterWidget {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::Pointer(PointerEvent::Move { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.counts.moves, |v: &mut u32| *v += 1);
            }
            Event::Pointer(PointerEvent::Down { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.counts.downs, |v: &mut u32| *v += 1);
            }
            Event::Pointer(PointerEvent::Wheel { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.counts.wheels, |v: &mut u32| *v += 1);
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct HitTestTransparent;

impl<H: UiHost> Widget<H> for HitTestTransparent {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct OutsidePressObserverCounter {
    observer_downs: Model<u32>,
}

impl<H: UiHost> Widget<H> for OutsidePressObserverCounter {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
            return;
        }
        if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
            let _ = cx
                .app
                .models_mut()
                .update(&self.observer_downs, |v: &mut u32| *v += 1);
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct PointerMoveObserverCounter {
    observer_moves: Model<u32>,
}

impl<H: UiHost> Widget<H> for PointerMoveObserverCounter {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
            return;
        }
        if matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
            let _ = cx
                .app
                .models_mut()
                .update(&self.observer_moves, |v: &mut u32| *v += 1);
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct CornerCaptureOverlay {
    moves: Model<u32>,
    downs: Model<u32>,
    observer_downs: Model<u32>,
}

impl<H: UiHost> Widget<H> for CornerCaptureOverlay {
    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        position.x.0 <= 20.0 && position.y.0 <= 20.0
    }

    fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
            return;
        }
        if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
            let _ = cx
                .app
                .models_mut()
                .update(&self.observer_downs, |v: &mut u32| *v += 1);
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Bubble {
            return;
        }

        match event {
            Event::Pointer(PointerEvent::Down { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.downs, |v: &mut u32| *v += 1);
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Move { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.moves, |v: &mut u32| *v += 1);
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

fn render_lower_overlay_occlusion_hover_scene(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    sibling_hovered: Rc<Cell<bool>>,
    sibling_hovered_raw: Rc<Cell<bool>>,
    sibling_hovered_raw_below_barrier: Rc<Cell<bool>>,
) -> GlobalElementId {
    let base_root = crate::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "occlusion-base",
        |_cx| Vec::new(),
    );

    let mut sibling_id: Option<GlobalElementId> = None;
    let parent_root = crate::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "occlusion-parent-overlay",
        |cx| {
            vec![cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    ..Default::default()
                },
                |cx| {
                    vec![
                        cx.pressable_with_id(
                            PressableProps {
                                layout: LayoutStyle {
                                    position: PositionStyle::Absolute,
                                    inset: InsetStyle {
                                        left: Some(Px(0.0)).into(),
                                        top: Some(Px(0.0)).into(),
                                        ..Default::default()
                                    },
                                    size: SizeStyle {
                                        width: Length::Px(Px(80.0)),
                                        height: Length::Px(Px(32.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |_cx, _st, _id| Vec::new(),
                        ),
                        cx.pressable_with_id(
                            PressableProps {
                                layout: LayoutStyle {
                                    position: PositionStyle::Absolute,
                                    inset: InsetStyle {
                                        left: Some(Px(120.0)).into(),
                                        top: Some(Px(0.0)).into(),
                                        ..Default::default()
                                    },
                                    size: SizeStyle {
                                        width: Length::Px(Px(80.0)),
                                        height: Length::Px(Px(32.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |_cx, st, id| {
                                sibling_id = Some(id);
                                sibling_hovered.set(st.hovered);
                                sibling_hovered_raw.set(st.hovered_raw);
                                sibling_hovered_raw_below_barrier.set(st.hovered_raw_below_barrier);
                                Vec::new()
                            },
                        ),
                    ]
                },
            )]
        },
    );

    let child_root = crate::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "occlusion-child-overlay",
        |cx| {
            vec![cx.pointer_region(
                PointerRegionProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            )]
        },
    );

    ui.set_root(base_root);
    let _parent_layer = ui.push_overlay_root(parent_root, false);
    let child_layer = ui.push_overlay_root(child_root, false);
    ui.set_layer_pointer_occlusion(child_layer, PointerOcclusion::BlockMouseExceptScroll);
    ui.layout_all(app, services, bounds, 1.0);

    sibling_id.expect("sibling pressable id")
}

#[test]
fn pointer_occlusion_block_mouse_except_scroll_suppresses_underlay_hit_dispatch_but_allows_wheel() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    // An overlay layer that occludes underlay pointer interactions, but allows scroll.
    let overlay_root = ui.create_node(HitTestTransparent);
    let overlay_layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hit = ui.debug_hit_test(Point::new(Px(10.0), Px(10.0)));
    assert_eq!(hit.hit, Some(base), "expected underlay to be hit-testable");
    assert_eq!(hit.barrier_root, None, "expected no modal barrier");

    // Move and down should not reach the underlay while occlusion is active.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    // Wheel should still route to the underlay scroll target when configured as "except scroll".
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&counts.moves).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.downs).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.wheels).unwrap_or(0), 1);
}

#[test]
fn pointer_occlusion_exposes_raw_hover_for_lower_overlay_pressable_via_secondary_hit_test() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeUiServices;

    let sibling_hovered = Rc::new(Cell::new(false));
    let sibling_hovered_raw = Rc::new(Cell::new(false));
    let sibling_hovered_raw_below_barrier = Rc::new(Cell::new(false));

    let mut ui = UiTree::new();
    ui.set_window(window);
    let sibling = render_lower_overlay_occlusion_hover_scene(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        sibling_hovered.clone(),
        sibling_hovered_raw.clone(),
        sibling_hovered_raw_below_barrier.clone(),
    );

    let sibling_node =
        crate::elements::node_for_element(&mut app, window, sibling).expect("sibling node");
    let sibling_bounds = ui.debug_node_bounds(sibling_node).expect("sibling bounds");
    let pointer = Point::new(
        Px(sibling_bounds.origin.x.0 + sibling_bounds.size.width.0 * 0.5),
        Px(sibling_bounds.origin.y.0 + sibling_bounds.size.height.0 * 0.5),
    );

    let hit = ui.debug_hit_test(pointer);
    assert_ne!(
        hit.hit,
        Some(sibling_node),
        "expected the child occlusion overlay to win top-level hit testing at the sibling trigger position",
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: pointer,
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    app.advance_frame();

    let mut ui = UiTree::new();
    ui.set_window(window);
    let sibling_after = render_lower_overlay_occlusion_hover_scene(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        sibling_hovered.clone(),
        sibling_hovered_raw.clone(),
        sibling_hovered_raw_below_barrier.clone(),
    );
    assert_eq!(
        sibling_after, sibling,
        "expected stable pressable identity across frames for the lower overlay sibling trigger",
    );

    assert!(
        !sibling_hovered.get(),
        "expected the occlusion overlay to suppress normal hovered state for the lower overlay pressable",
    );
    assert!(
        !sibling_hovered_raw.get(),
        "expected the occlusion overlay to suppress raw hovered state for the lower overlay pressable",
    );
    assert!(
        sibling_hovered_raw_below_barrier.get(),
        "expected a secondary under-occlusion hit test to expose hovered_raw_below_barrier for the lower overlay pressable",
    );
}

#[test]
fn pointer_occlusion_block_mouse_suppresses_underlay_hit_dispatch_including_wheel() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(HitTestTransparent);
    let overlay_layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouse);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&counts.moves).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.downs).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.wheels).unwrap_or(0), 0);
}

#[test]
fn modal_barrier_scoping_blocks_underlay_wheel_even_when_barrier_is_hit_test_inert() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    // A modal barrier layer that is hit-test-inert (e.g. during a close transition). The barrier
    // must still scope wheel routing and prevent the underlay from receiving wheel events.
    let barrier_root = ui.create_node(HitTestTransparent);
    let _barrier_layer = ui.push_overlay_root_with_options(
        barrier_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: true,
            hit_testable: false,
        },
    );

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hit = ui.debug_hit_test(Point::new(Px(10.0), Px(10.0)));
    assert_eq!(
        hit.hit, None,
        "expected modal barrier scoping to suppress underlay hit-testing",
    );
    assert_eq!(
        hit.barrier_root,
        Some(barrier_root),
        "expected modal barrier scoping to be active"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&counts.wheels).unwrap_or(0),
        0,
        "expected wheel to be scoped by the modal barrier and not reach the underlay",
    );
}

#[test]
fn pointer_occlusion_block_mouse_except_scroll_is_window_global_across_pointers() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(HitTestTransparent);
    let overlay_layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&counts.moves).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.downs).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.wheels).unwrap_or(0), 2);
}

#[test]
fn pointer_occlusion_does_not_suppress_outside_press_observer_dispatch() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };
    let observer_downs = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    // A hit-test-transparent overlay that requests outside-press observer events and enables
    // pointer occlusion (Radix `disableOutsidePointerEvents` outcome).
    let overlay_root = ui.create_node(OutsidePressObserverCounter {
        observer_downs: observer_downs.clone(),
    });
    let overlay_layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_wants_pointer_down_outside_events(overlay_layer, true);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&observer_downs).unwrap_or(0), 1);
    assert_eq!(
        app.models().get_copied(&counts.downs).unwrap_or(0),
        0,
        "expected pointer occlusion to block underlay pointer-down dispatch"
    );
}

#[test]
fn pointer_occlusion_allows_pointer_move_observer_dispatch_while_suppressing_underlay_hit_dispatch()
{
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };
    let observer_moves = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(PointerMoveObserverCounter {
        observer_moves: observer_moves.clone(),
    });
    let overlay_layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_wants_pointer_move_events(overlay_layer, true);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), None);
    assert_eq!(ui.captured(), None);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&counts.moves).unwrap_or(0),
        0,
        "expected pointer occlusion to suppress underlay pointer-move dispatch"
    );
    assert_eq!(
        app.models().get_copied(&observer_moves).unwrap_or(0),
        1,
        "expected pointer occlusion to still dispatch pointer-move observer events to overlays"
    );
    assert_eq!(ui.focus(), None, "observer pass must not change focus");
    assert_eq!(
        ui.captured(),
        None,
        "observer pass must not capture pointers"
    );
}

#[test]
fn pointer_occlusion_respects_pointer_capture_for_one_pointer_but_occludes_others() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let underlay = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };
    let overlay_moves = app.models_mut().insert(0u32);
    let overlay_downs = app.models_mut().insert(0u32);
    let observer_downs = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: underlay.clone(),
    });
    ui.set_root(base);

    // A small overlay hit region (top-left corner) captures pointer 0 on down, while the layer
    // occludes underlay pointer interaction outside the overlay.
    let overlay_root = ui.create_node(CornerCaptureOverlay {
        moves: overlay_moves.clone(),
        downs: overlay_downs.clone(),
        observer_downs: observer_downs.clone(),
    });
    let overlay_layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_wants_pointer_down_outside_events(overlay_layer, true);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Pointer 0 starts inside the overlay hit region and is captured by the overlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        Some(overlay_root),
        "expected overlay to capture pointer 0"
    );

    // Even after leaving the overlay hit region, pointer 0 should continue to route to the
    // captured overlay node.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(50.0), Px(50.0)),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    // Pointer 1 is not captured. A down outside the overlay should:
    // - trigger outside-press observer dispatch to the overlay,
    // - be suppressed for the underlay due to pointer occlusion.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(50.0), Px(50.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    // Wheel remains allowed for non-captured pointers in BlockMouseExceptScroll mode.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(50.0), Px(50.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&overlay_downs).unwrap_or(0), 1);
    assert_eq!(app.models().get_copied(&overlay_moves).unwrap_or(0), 1);
    assert_eq!(app.models().get_copied(&observer_downs).unwrap_or(0), 1);

    assert_eq!(app.models().get_copied(&underlay.downs).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&underlay.wheels).unwrap_or(0), 1);
}
