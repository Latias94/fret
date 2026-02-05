#![allow(clippy::arc_with_non_send_sync)]

use super::*;
use fret_runtime::GlobalsHost as _;
use std::sync::Arc;

fn attributed_plain(text: &str) -> fret_core::AttributedText {
    fret_core::AttributedText::new(
        Arc::<str>::from(text),
        [fret_core::TextSpan {
            len: text.len(),
            ..Default::default()
        }],
    )
}

#[test]
fn pressable_state_reports_focused_when_focused() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let focused = Rc::new(Cell::new(false));
    let pressable_element_id: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    fn render_frame(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        focused_out: Rc<Cell<bool>>,
        pressable_id_out: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
    ) -> NodeId {
        render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "pressable-state-reports-focused",
            move |cx| {
                let focused_out = focused_out.clone();
                let pressable_id_out = pressable_id_out.clone();
                vec![cx.pressable_with_id(
                    crate::element::PressableProps::default(),
                    move |cx, st, id| {
                        pressable_id_out.set(Some(id));
                        focused_out.set(st.focused);
                        vec![cx.text("pressable")]
                    },
                )]
            },
        )
    }

    // First frame: render once to establish stable identity + node mapping.
    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        focused.clone(),
        pressable_element_id.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(!focused.get());

    let pressable_element = pressable_element_id.get().expect("pressable element id");
    let pressable_node = crate::elements::node_for_element(&mut app, window, pressable_element)
        .expect("pressable node");
    ui.set_focus(Some(pressable_node));

    // Second frame: the authoring context should observe the focused element.
    app.advance_frame();
    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        focused.clone(),
        pressable_element_id.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(focused.get());
}

#[test]
fn declarative_pointer_region_can_capture_and_receive_move_up() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-capture-move-up",
        |cx| {
            let counter_down = counter.clone();
            let counter_move = counter.clone();
            let counter_up = counter.clone();

            let on_down = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      down: crate::action::PointerDownCx| {
                    if down.button != MouseButton::Left {
                        return false;
                    }
                    host.capture_pointer();
                    let _ = host
                        .models_mut()
                        .update(&counter_down, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let on_move = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      _cx: crate::action::ActionCx,
                      _mv: crate::action::PointerMoveCx| {
                    let _ = host
                        .models_mut()
                        .update(&counter_move, |v: &mut u32| *v = v.saturating_add(10));
                    true
                },
            );

            let on_up = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      up: crate::action::PointerUpCx| {
                    if up.button == MouseButton::Left {
                        host.release_pointer_capture();
                    }
                    let _ = host
                        .models_mut()
                        .update(&counter_up, |v: &mut u32| *v = v.saturating_add(100));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut props = crate::element::PointerRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                cx.pointer_region_on_pointer_move(on_move);
                cx.pointer_region_on_pointer_up(on_up);
                Vec::new()
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region = ui.children(root)[0];
    let region_bounds = ui.debug_node_bounds(region).expect("pointer region bounds");

    let inside = Point::new(
        Px(region_bounds.origin.x.0 + 5.0),
        Px(region_bounds.origin.y.0 + 5.0),
    );
    let outside = Point::new(Px(region_bounds.origin.x.0 + 250.0), inside.y);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: outside,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: outside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&counter), Some(111));
}

#[test]
fn declarative_pointer_region_can_handle_pointer_cancel() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-pointer-cancel",
        |cx| {
            let counter_down = counter.clone();
            let counter_cancel = counter.clone();

            let on_down = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      down: crate::action::PointerDownCx| {
                    if down.button != MouseButton::Left {
                        return false;
                    }
                    host.capture_pointer();
                    let _ = host
                        .models_mut()
                        .update(&counter_down, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let on_cancel = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      cancel: crate::action::PointerCancelCx| {
                    host.release_pointer_capture();
                    let _ = host.models_mut().update(&counter_cancel, |v: &mut u32| {
                        *v = v.saturating_add(match cancel.reason {
                            fret_core::PointerCancelReason::LeftWindow => 100,
                        })
                    });
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut props = crate::element::PointerRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                cx.pointer_region_on_pointer_cancel(on_cancel);
                Vec::new()
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region = ui.children(root)[0];
    let region_bounds = ui.debug_node_bounds(region).expect("pointer region bounds");
    let inside = Point::new(
        Px(region_bounds.origin.x.0 + 5.0),
        Px(region_bounds.origin.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.captured_for(fret_core::PointerId(0)), Some(region));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: None,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );

    assert_eq!(ui.captured_for(fret_core::PointerId(0)), None);
    assert_eq!(app.models().get_copied(&counter), Some(101));
}

#[test]
fn declarative_pointer_region_can_handle_wheel() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-wheel",
        |cx| {
            let counter_wheel = counter.clone();
            let on_wheel = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      _wheel: crate::action::WheelCx| {
                    let _ = host
                        .models_mut()
                        .update(&counter_wheel, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut props = crate::element::PointerRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            vec![cx.pointer_region(props, |cx| {
                cx.pointer_region_on_wheel(on_wheel);
                Vec::new()
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let inside = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: inside,
            delta: Point::new(Px(0.0), Px(10.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let v = app.models_mut().read(&counter, |v| *v).unwrap_or_default();
    assert_eq!(v, 1);
}

#[test]
fn declarative_pointer_region_can_handle_pinch_gesture() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-pinch",
        |cx| {
            let counter_pinch = counter.clone();
            let on_pinch = Arc::new(
                move |host: &mut dyn crate::action::UiPointerActionHost,
                      cx: crate::action::ActionCx,
                      _pinch: crate::action::PinchGestureCx| {
                    let _ = host
                        .models_mut()
                        .update(&counter_pinch, |v: &mut u32| *v = v.saturating_add(1));
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut props = crate::element::PointerRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            vec![cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pinch_gesture(on_pinch);
                Vec::new()
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let inside = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::PinchGesture {
            position: inside,
            delta: 0.5,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let v = app.models_mut().read(&counter, |v| *v).unwrap_or_default();
    assert_eq!(v, 1);
}

#[test]
fn declarative_internal_drag_region_can_handle_internal_drag_events() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut services = FakeTextService::default();

    let counter = app.models_mut().insert(0u32);
    let drag_kind = fret_runtime::DragKindId(0x465245545F494452); // "FRET_IDR"

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "internal-drag-region-basic",
        |cx| {
            let counter = counter.clone();
            let mut props = crate::element::InternalDragRegionProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            vec![cx.internal_drag_region(props, |cx| {
                cx.internal_drag_region_on_internal_drag(Arc::new(
                    move |host: &mut dyn crate::action::UiDragActionHost,
                          acx: crate::action::ActionCx,
                          drag: crate::action::InternalDragCx| {
                        let Some(session) = host.drag(drag.pointer_id) else {
                            return false;
                        };
                        if session.kind != drag_kind {
                            return false;
                        }
                        if drag.kind == fret_core::InternalDragKind::Over {
                            let _ = host
                                .models_mut()
                                .update(&counter, |v: &mut u32| *v = v.saturating_add(1));
                            host.request_redraw(acx.window);
                            return true;
                        }
                        false
                    },
                ));
                vec![cx.text("drop target")]
            })]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        drag_kind,
        window,
        Point::new(Px(4.0), Px(4.0)),
        (),
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::InternalDrag(fret_core::InternalDragEvent {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(10.0)),
            kind: fret_core::InternalDragKind::Over,
            modifiers: Modifiers::default(),
        }),
    );

    let value = app.models_mut().read(&counter, |v| *v).unwrap_or_default();
    assert_eq!(value, 1);
}

#[test]
fn selectable_text_drag_autoscrolls_scroll_container() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-autoscroll",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            out.push(cx.selectable_text(attributed_plain("hello selectable text")));
                            for _ in 0..50 {
                                out.push(cx.text("filler"));
                            }
                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let selectable_node = ui.children(column_node)[0];

    let scroll_bounds = ui.debug_node_bounds(scroll_node).expect("scroll bounds");
    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");

    let inside = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 + 5.0),
    );
    let below = Point::new(
        Px(scroll_bounds.origin.x.0 + 5.0),
        Px(scroll_bounds.origin.y.0 + scroll_bounds.size.height.0 + 10.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: below,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected selectable drag to auto-scroll, got offset={:?}",
        scroll_handle.offset()
    );
}

#[test]
fn selectable_text_drag_autoscrolls_horizontal_scroll_container() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-autoscroll-x",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::X,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    let mut content_layout = crate::element::LayoutStyle::default();
                    content_layout.size.width = Length::Px(Px(600.0));
                    content_layout.size.height = Length::Fill;

                    vec![cx.container(
                        crate::element::ContainerProps {
                            layout: content_layout,
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.selectable_text_props(crate::element::SelectableTextProps {
                                layout: Default::default(),
                                rich: attributed_plain(
                                    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
                                ),
                                style: None,
                                color: None,
                                wrap: fret_core::TextWrap::None,
                                overflow: fret_core::TextOverflow::Clip,
                            })]
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let scroll_node = ui.children(root)[0];
    let selectable_node = ui.children(scroll_node)[0];

    let scroll_bounds = ui.debug_node_bounds(scroll_node).expect("scroll bounds");
    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");

    let inside = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 + 5.0),
    );
    let beyond_right = Point::new(
        Px(scroll_bounds.origin.x.0 + scroll_bounds.size.width.0 + 10.0),
        Px(scroll_bounds.origin.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: beyond_right,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scroll_handle.offset().x.0 > 0.01,
        "expected selectable drag to auto-scroll horizontally, got offset={:?}",
        scroll_handle.offset()
    );
}

#[test]
fn selectable_text_double_and_triple_click_select() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world\nsecond line");

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-double-triple-click",
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let element = record.element;

    let pos = Point::new(Px(5.0), Px(5.0));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (a, b) = crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| (state.selection_anchor, state.caret),
    );
    assert_eq!((a, b), (0, 5), "double click should select first word");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 3,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (a, b) = crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| (state.selection_anchor, state.caret),
    );
    assert_eq!(
        (a, b),
        (0, 12),
        "triple click should select first line (including trailing newline)"
    );
}

#[test]
fn selectable_text_double_click_respects_window_text_boundary_mode_under_render_transform() {
    fn selection_for_mode(mode: fret_runtime::TextBoundaryMode) -> (usize, usize) {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(60.0)));
        let mut services = FakeTextService::default();

        let rich = attributed_plain("can't");

        let transform = Transform2D::translation(Point::new(Px(40.0), Px(10.0)));
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "selectable-text-double-click-boundary-mode-transform",
            |cx| vec![cx.render_transform(transform, |cx| vec![cx.selectable_text(rich.clone())])],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let transform_node = ui.children(root)[0];
        let selectable_node = ui.children(transform_node)[0];
        let record =
            crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
                .expect("selectable record");
        let element = record.element;

        let selectable_bounds = ui
            .debug_node_bounds(selectable_node)
            .expect("selectable bounds");
        let pos = Point::new(
            Px(selectable_bounds.origin.x.0 + 40.0 + 5.0),
            Px(selectable_bounds.origin.y.0 + 10.0 + 5.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 2,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        crate::elements::with_element_state(
            &mut app,
            window,
            element,
            crate::element::SelectableTextState::default,
            |state| (state.selection_anchor, state.caret),
        )
    }

    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::UnicodeWord),
        (0, 5),
        "UnicodeWord should select the whole word"
    );
    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::Identifier),
        (0, 3),
        "Identifier should stop at the apostrophe"
    );
}

#[test]
fn selectable_text_double_click_respects_window_text_boundary_mode_under_scroll_offset() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());
    app.with_global_mut_untracked(
        fret_runtime::WindowTextBoundaryModeService::default,
        |svc, _app| {
            svc.set_base_mode(
                AppWindowId::default(),
                fret_runtime::TextBoundaryMode::Identifier,
            );
        },
    );

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(140.0), Px(50.0)));
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-double-click-boundary-mode-scroll",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            for _ in 0..40 {
                                let mut row_layout = crate::element::LayoutStyle::default();
                                row_layout.size.height = Length::Px(Px(18.0));
                                out.push(cx.container(
                                    crate::element::ContainerProps {
                                        layout: row_layout,
                                        ..Default::default()
                                    },
                                    |cx| vec![cx.text("filler")],
                                ));
                            }
                            out.push(cx.selectable_text(attributed_plain("can't")));
                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Force the selectable text into view via an imperative scroll offset.
    //
    // Note: scroll is applied via a render transform, so `debug_node_bounds` reports the layout
    // bounds in content space. We must subtract the scroll offset to get a screen-space click
    // position.
    scroll_handle.set_offset(Point::new(Px(0.0), Px(100_000.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let selectable_node = *ui
        .children(column_node)
        .last()
        .expect("expected selectable text as last child");

    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");
    let scroll_offset = scroll_handle.offset();
    let pos = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 - scroll_offset.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let (a, b) = crate::elements::with_element_state(
        &mut app,
        window,
        record.element,
        crate::element::SelectableTextState::default,
        |state| (state.selection_anchor, state.caret),
    );

    assert_eq!(
        (a, b),
        (0, 3),
        "Identifier mode should stop at the apostrophe"
    );
}

#[test]
fn selectable_text_pointer_down_requests_focus() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-pointer-down-focus",
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");
    let pos = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 + 5.0),
    );

    assert_eq!(ui.focus(), None, "expected no focus before click");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(selectable_node),
        "expected selectable text to request focus on pointer down"
    );
}

#[test]
fn selectable_text_double_click_sets_primary_selection_when_enabled() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::TextInteractionSettings {
        linux_primary_selection: true,
    });
    let mut caps = fret_runtime::PlatformCapabilities::default();
    caps.clipboard.primary_text = true;
    app.set_global(caps);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root_name = "selectable-text-primary-selection-double-click";
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let selectable_node = ui.children(root)[0];
    let selectable_bounds = ui
        .debug_node_bounds(selectable_node)
        .expect("selectable bounds");
    let pos = Point::new(
        Px(selectable_bounds.origin.x.0 + 5.0),
        Px(selectable_bounds.origin.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        app.take_effects().iter().any(|e| {
            matches!(
                e,
                fret_runtime::Effect::PrimarySelectionSetText { text }
                if text == "hello"
            )
        }),
        "expected selectable text selection to set primary selection when enabled"
    );
}

#[test]
fn selectable_text_arrow_up_down_uses_preferred_x_across_lines() {
    #[derive(Default)]
    struct LineTextService {
        text: String,
    }

    impl LineTextService {
        fn line_range(&self, line: usize) -> Option<(usize, usize)> {
            if line == 0 && self.text.is_empty() {
                return Some((0, 0));
            }

            let mut start = 0usize;
            let mut line_idx = 0usize;
            for (i, ch) in self.text.char_indices() {
                if ch != '\n' {
                    continue;
                }
                if line_idx == line {
                    return Some((start, i));
                }
                start = i + 1;
                line_idx += 1;
            }

            if line_idx == line {
                return Some((start, self.text.len()));
            }
            None
        }

        fn line_count(&self) -> usize {
            if self.text.is_empty() {
                return 1;
            }
            self.text.chars().filter(|c| *c == '\n').count() + 1
        }

        fn index_to_line_col(&self, index: usize) -> (usize, usize) {
            let index = index.min(self.text.len());
            let mut line = 0usize;
            let mut col = 0usize;
            for (i, ch) in self.text.char_indices() {
                if i >= index {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
            }
            (line, col)
        }
    }

    impl fret_core::TextService for LineTextService {
        fn prepare(
            &mut self,
            input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            self.text = input.text().to_string();

            let line_h = Px(10.0);
            let lines = self.line_count().max(1) as f32;
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(200.0), Px(line_h.0 * lines)),
                    baseline: Px(8.0),
                },
            )
        }

        fn caret_rect(
            &mut self,
            _blob: fret_core::TextBlobId,
            index: usize,
            _affinity: fret_core::CaretAffinity,
        ) -> Rect {
            let line_h = Px(10.0);
            let char_w = Px(10.0);

            let (line, col) = self.index_to_line_col(index);
            Rect::new(
                Point::new(Px(char_w.0 * (col as f32)), Px(line_h.0 * (line as f32))),
                Size::new(Px(1.0), line_h),
            )
        }

        fn hit_test_point(
            &mut self,
            _blob: fret_core::TextBlobId,
            point: Point,
        ) -> fret_core::HitTestResult {
            let line_h = 10.0_f32;
            let char_w = 10.0_f32;

            let mut line = (point.y.0 / line_h).floor() as i32;
            line = line.clamp(0, self.line_count().saturating_sub(1) as i32);
            let line = line as usize;

            let (start, end) = self.line_range(line).unwrap_or((0, 0));
            let len = end.saturating_sub(start);

            let mut col = (point.x.0 / char_w).round() as i32;
            col = col.clamp(0, len as i32);
            let col = col as usize;

            fret_core::HitTestResult {
                index: (start + col).min(self.text.len()),
                affinity: fret_core::CaretAffinity::Downstream,
            }
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for LineTextService {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for LineTextService {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = LineTextService::default();

    let text = "0123456789\nabc\n0123456789";

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "selectable-text-preferred-x",
        |cx| {
            vec![
                cx.selectable_text_props(crate::element::SelectableTextProps {
                    layout: Default::default(),
                    rich: attributed_plain(text),
                    style: None,
                    color: None,
                    wrap: fret_core::TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let selectable_node = ui.children(root)[0];
    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let element = record.element;

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(80.0), Px(5.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let (caret, preferred_x) = crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| (state.caret, state.preferred_x),
    );
    assert_eq!(
        caret, 14,
        "expected down to clamp into the short middle line"
    );
    assert_eq!(
        preferred_x,
        Some(Px(80.0)),
        "expected preferred_x to preserve the original column"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let caret = crate::elements::with_element_state(
        &mut app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| state.caret,
    );
    assert_eq!(
        caret, 23,
        "expected preferred_x to restore the original column on the next long line"
    );
}

#[test]
fn selectable_text_sets_active_text_selection() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root_name = "selectable-text-active-text-selection";
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    let record =
        crate::declarative::frame::element_record_for_node(&mut app, window, selectable_node)
            .expect("selectable record");
    let element = record.element;

    let pos = Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let active =
        crate::elements::with_window_state(&mut app, window, |st| st.active_text_selection());
    assert_eq!(
        active,
        Some(crate::elements::ActiveTextSelection {
            root: crate::elements::global_root(window, root_name),
            element,
        }),
        "expected active text selection to be tracked while selection is non-empty"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::SetTextSelection {
            anchor: 0,
            focus: 0,
        },
    );

    let active =
        crate::elements::with_window_state(&mut app, window, |st| st.active_text_selection());
    assert_eq!(
        active, None,
        "expected active text selection to clear when selection is collapsed"
    );
}

#[test]
fn selectable_text_copy_availability_requires_selection() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root_name = "selectable-text-copy-availability";
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    ui.set_focus(Some(selectable_node));

    let copy = CommandId::from("text.copy");
    let select_all = CommandId::from("text.select_all");

    assert!(
        ui.is_command_available(&mut app, &select_all),
        "expected text.select_all to be available for focused selectable text"
    );
    assert!(
        !ui.is_command_available(&mut app, &copy),
        "expected text.copy to be unavailable without a selection"
    );

    assert!(
        ui.dispatch_command(&mut app, &mut services, &select_all),
        "expected text.select_all to be handled by selectable text"
    );

    assert!(
        ui.is_command_available(&mut app, &copy),
        "expected text.copy to be available when a selection exists"
    );
}

#[test]
fn selectable_text_copy_availability_respects_clipboard_capabilities() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities {
        clipboard: fret_runtime::capabilities::ClipboardCapabilities {
            text: false,
            files: false,
            primary_text: false,
        },
        ..fret_runtime::PlatformCapabilities::default()
    });
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let rich = attributed_plain("hello world");
    let root_name = "selectable-text-copy-availability-clipboard-caps";
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        |cx| vec![cx.selectable_text(rich.clone())],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let selectable_node = ui.children(root)[0];
    ui.set_focus(Some(selectable_node));

    let copy = CommandId::from("text.copy");
    let select_all = CommandId::from("text.select_all");

    assert!(
        ui.dispatch_command(&mut app, &mut services, &select_all),
        "expected text.select_all to be handled by selectable text"
    );
    assert!(
        !ui.is_command_available(&mut app, &copy),
        "expected text.copy to be unavailable when clipboard text is unsupported"
    );
    assert!(
        !ui.dispatch_command(&mut app, &mut services, &copy),
        "expected text.copy to not be handled when clipboard text is unsupported"
    );
}

#[test]
fn declarative_command_availability_hooks_participate_in_dispatch_path_queries() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "command-availability-hooks",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let id = cx.root_id();
                    cx.command_on_command_availability_for(
                        id,
                        Arc::new(|_host, acx, command| {
                            if command.as_str() != "edit.copy" {
                                return crate::widget::CommandAvailability::NotHandled;
                            }
                            if !acx.focus_in_subtree {
                                return crate::widget::CommandAvailability::NotHandled;
                            }
                            crate::widget::CommandAvailability::Available
                        }),
                    );
                    vec![cx.text("child")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let child_node = ui.children(container_node)[0];
    ui.set_focus(Some(child_node));

    let copy = CommandId::from("edit.copy");
    assert!(
        ui.is_command_available(&mut app, &copy),
        "expected edit.copy to be available via declarative availability hook"
    );

    ui.set_focus(None);
    assert!(
        !ui.is_command_available(&mut app, &copy),
        "expected edit.copy to be unavailable when no dispatch path exists"
    );
}

#[test]
fn text_input_cut_updates_model_and_availability() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert("hello".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-cut-updates-model",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let copy = CommandId::from("text.copy");
    let cut = CommandId::from("text.cut");
    let select_all = CommandId::from("text.select_all");

    assert!(
        ui.is_command_available(&mut app, &select_all),
        "expected text.select_all to be available for focused text input"
    );
    assert!(
        !ui.is_command_available(&mut app, &copy),
        "expected text.copy to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &cut),
        "expected text.cut to be unavailable without a selection"
    );

    assert!(
        ui.dispatch_command(&mut app, &mut services, &select_all),
        "expected text.select_all to be handled by text input"
    );
    assert!(
        ui.is_command_available(&mut app, &copy),
        "expected text.copy to be available after select_all"
    );
    assert!(
        ui.is_command_available(&mut app, &cut),
        "expected text.cut to be available after select_all"
    );

    assert!(
        ui.dispatch_command(&mut app, &mut services, &cut),
        "expected text.cut to be handled by text input"
    );
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some(""),
        "expected cut to update the bound model"
    );
    assert!(
        app.take_effects()
            .iter()
            .any(|e| matches!(e, fret_runtime::Effect::ClipboardSetText { .. })),
        "expected text.cut to emit ClipboardSetText"
    );
}

#[test]
fn text_input_paste_requests_clipboard_text_when_editable() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert("hello".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-paste-clipboard-get",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let paste = CommandId::from("text.paste");
    assert!(
        ui.is_command_available(&mut app, &paste),
        "expected text.paste to be available for focused editable text input"
    );
    assert!(
        ui.dispatch_command(&mut app, &mut services, &paste),
        "expected text.paste to be handled by text input"
    );

    assert!(
        app.take_effects()
            .iter()
            .any(|e| matches!(e, fret_runtime::Effect::ClipboardGetText { .. })),
        "expected text.paste to request ClipboardGetText"
    );
}

#[test]
fn text_input_key_hooks_can_intercept_navigation_keys() {
    use fret_core::{Event, KeyCode, Modifiers};

    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert("hello".to_string());
    let opened = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let opened_for_hook = opened.clone();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-key-hooks-intercept",
        move |cx| {
            vec![cx.text_input_with_id_props(|cx, id| {
                let opened = opened_for_hook.clone();
                cx.key_add_on_key_down_for(
                    id,
                    Arc::new(move |host, action_cx, down| {
                        if down.key != KeyCode::ArrowDown {
                            return false;
                        }
                        let _ = host.models_mut().update(&opened, |v| *v = true);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );
                crate::element::TextInputProps::new(model.clone())
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert!(
        app.models().get_copied(&opened).unwrap_or(false),
        "expected key hook to run for focused text input"
    );
}

#[test]
fn text_input_middle_click_pastes_primary_selection_when_enabled() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::TextInteractionSettings {
        linux_primary_selection: true,
    });
    let mut caps = fret_runtime::PlatformCapabilities::default();
    caps.clipboard.text = true;
    caps.clipboard.primary_text = true;
    app.set_global(caps);

    let model = app.models_mut().insert(String::new());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-primary-selection-middle-click",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let input_bounds = ui.debug_node_bounds(input_node).expect("input bounds");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 10.0),
        Px(input_bounds.origin.y.0 + 10.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Middle,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    let Some(token) = effects.iter().find_map(|e| match e {
        fret_runtime::Effect::PrimarySelectionGetText { token, .. } => Some(*token),
        _ => None,
    }) else {
        panic!("expected middle click to request PrimarySelectionGetText");
    };

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::PrimarySelectionText {
            token,
            text: "hello".to_string(),
        },
    );

    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some("hello"),
        "expected primary selection paste to insert text into the bound model"
    );
}

#[test]
fn text_input_select_all_is_blocked_when_empty() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert(String::new());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-select-all-empty",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let select_all = CommandId::from("text.select_all");
    let edit_select_all = CommandId::from("edit.select_all");
    let clear = CommandId::from("text.clear");
    let edit_copy = CommandId::from("edit.copy");
    let edit_cut = CommandId::from("edit.cut");
    let unknown = CommandId::from("text.unknown");

    assert!(
        !ui.is_command_available(&mut app, &select_all),
        "expected text.select_all to be unavailable for empty text input"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_select_all),
        "expected edit.select_all to be unavailable for empty text input"
    );
    assert!(
        !ui.is_command_available(&mut app, &clear),
        "expected text.clear to be unavailable for empty text input"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_copy),
        "expected edit.copy to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_cut),
        "expected edit.cut to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &unknown),
        "expected unknown text.* commands to be NotHandled for availability"
    );
}

#[test]
fn text_area_select_all_is_blocked_when_empty() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert(String::new());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-select-all-empty",
        |cx| vec![cx.text_area(crate::element::TextAreaProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let area_node = ui.children(root)[0];
    ui.set_focus(Some(area_node));

    let select_all = CommandId::from("text.select_all");
    let edit_select_all = CommandId::from("edit.select_all");
    let clear = CommandId::from("text.clear");
    let edit_copy = CommandId::from("edit.copy");
    let edit_cut = CommandId::from("edit.cut");
    let unknown = CommandId::from("text.unknown");

    assert!(
        !ui.is_command_available(&mut app, &select_all),
        "expected text.select_all to be unavailable for empty text area"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_select_all),
        "expected edit.select_all to be unavailable for empty text area"
    );
    assert!(
        !ui.is_command_available(&mut app, &clear),
        "expected text.clear to be unavailable for empty text area"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_copy),
        "expected edit.copy to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_cut),
        "expected edit.cut to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &unknown),
        "expected unknown text.* commands to be NotHandled for availability"
    );
}

#[test]
fn text_input_supports_edit_select_all_and_copy() {
    let mut app = TestHost::new();
    let mut caps = fret_runtime::PlatformCapabilities::default();
    caps.clipboard.text = true;
    app.set_global(caps);

    let model = app.models_mut().insert("hello".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-edit-select-all-copy",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let select_all = CommandId::from("edit.select_all");
    assert!(
        ui.dispatch_command(&mut app, &mut services, &select_all),
        "expected edit.select_all to be handled by text input"
    );

    let copy = CommandId::from("edit.copy");
    assert!(
        ui.is_command_available(&mut app, &copy),
        "expected edit.copy to be available after select_all"
    );
    assert!(
        ui.dispatch_command(&mut app, &mut services, &copy),
        "expected edit.copy to be handled by text input"
    );
    assert!(
        app.take_effects().iter().any(
            |e| matches!(e, fret_runtime::Effect::ClipboardSetText { text } if text == "hello")
        ),
        "expected edit.copy to emit ClipboardSetText for the selected text"
    );
}

#[test]
fn text_input_double_click_respects_window_text_boundary_mode_under_render_transform() {
    fn selection_for_mode(mode: fret_runtime::TextBoundaryMode) -> Option<(u32, u32)> {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let model = app.models_mut().insert("can't".to_string());

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(60.0)));
        let mut services = FakeTextService::default();

        let transform = Transform2D::translation(Point::new(Px(40.0), Px(10.0)));
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "text-input-double-click-boundary-mode-transform",
            |cx| {
                vec![cx.render_transform(transform, |cx| {
                    let mut props = crate::element::TextInputProps::new(model.clone());
                    props.layout.size.width = Length::Px(Px(120.0));
                    props.layout.size.height = Length::Px(Px(32.0));
                    vec![cx.text_input(props)]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let transform_node = ui.children(root)[0];
        let input_node = ui.children(transform_node)[0];
        let input_bounds = ui
            .debug_node_visual_bounds(input_node)
            .expect("input bounds");
        let pos = Point::new(
            Px(input_bounds.origin.x.0 + 5.0),
            Px(input_bounds.origin.y.0 + 5.0),
        );
        assert_eq!(
            ui.debug_hit_test(pos).hit,
            Some(input_node),
            "expected the translated hit-test position to target the text input"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_eq!(ui.focus(), Some(input_node));
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 2,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_eq!(ui.focus(), Some(input_node));

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .cloned()
            .expect("expected a window text input snapshot");
        assert!(snapshot.focus_is_text_input);
        snapshot.selection_utf16
    }

    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::UnicodeWord),
        Some((0, 5)),
        "UnicodeWord should select the whole word"
    );
    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::Identifier),
        Some((0, 3)),
        "Identifier should stop at the apostrophe"
    );
}

#[test]
fn text_input_double_click_respects_window_text_boundary_mode_under_scroll_offset() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());
    app.with_global_mut_untracked(
        fret_runtime::WindowTextBoundaryModeService::default,
        |svc, _app| {
            svc.set_base_mode(
                AppWindowId::default(),
                fret_runtime::TextBoundaryMode::Identifier,
            );
        },
    );

    let model = app.models_mut().insert("can't".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(60.0)));
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-double-click-boundary-mode-scroll",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            for _ in 0..40 {
                                let mut row_layout = crate::element::LayoutStyle::default();
                                row_layout.size.height = Length::Px(Px(18.0));
                                out.push(cx.container(
                                    crate::element::ContainerProps {
                                        layout: row_layout,
                                        ..Default::default()
                                    },
                                    |cx| vec![cx.text("filler")],
                                ));
                            }

                            let mut props = crate::element::TextInputProps::new(model.clone());
                            props.layout.size.width = Length::Px(Px(120.0));
                            props.layout.size.height = Length::Px(Px(32.0));
                            out.push(cx.text_input(props));

                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let input_node = *ui
        .children(column_node)
        .last()
        .expect("expected input as last child");
    let input_bounds = ui.debug_node_bounds(input_node).expect("input bounds");

    scroll_handle.set_offset(Point::new(Px(0.0), input_bounds.origin.y));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let input_bounds = ui
        .debug_node_visual_bounds(input_node)
        .expect("input bounds after scroll");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 5.0),
        Px(input_bounds.origin.y.0 + 5.0),
    );
    assert_eq!(
        ui.debug_hit_test(pos).hit,
        Some(input_node),
        "expected the scrolled hit-test position to target the text input"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    let selection_utf16 = snapshot.selection_utf16;

    assert_eq!(
        selection_utf16,
        Some((0, 3)),
        "Identifier mode should stop at the apostrophe"
    );
}

#[test]
fn text_area_double_click_respects_window_text_boundary_mode_under_render_transform() {
    fn selection_for_mode(mode: fret_runtime::TextBoundaryMode) -> Option<(u32, u32)> {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let model = app.models_mut().insert("can't".to_string());

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(220.0), Px(120.0)),
        );
        let mut services = FakeTextService::default();

        let transform = Transform2D::translation(Point::new(Px(30.0), Px(10.0)));
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "text-area-double-click-boundary-mode-transform",
            |cx| {
                vec![cx.render_transform(transform, |cx| {
                    let mut props = crate::element::TextAreaProps::new(model.clone());
                    props.layout.size.width = Length::Px(Px(160.0));
                    props.layout.size.height = Length::Px(Px(80.0));
                    vec![cx.text_area(props)]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let transform_node = ui.children(root)[0];
        let area_node = ui.children(transform_node)[0];
        let area_bounds = ui.debug_node_visual_bounds(area_node).expect("area bounds");
        let pos = Point::new(
            Px(area_bounds.origin.x.0 + 5.0),
            Px(area_bounds.origin.y.0 + 5.0),
        );
        assert_eq!(
            ui.debug_hit_test(pos).hit,
            Some(area_node),
            "expected the translated hit-test position to target the text area"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_eq!(ui.focus(), Some(area_node));
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 2,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_eq!(ui.focus(), Some(area_node));

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .cloned()
            .expect("expected a window text input snapshot");
        assert!(snapshot.focus_is_text_input);
        snapshot.selection_utf16
    }

    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::UnicodeWord),
        Some((0, 5)),
        "UnicodeWord should select the whole word"
    );
    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::Identifier),
        Some((0, 3)),
        "Identifier should stop at the apostrophe"
    );
}

#[test]
fn text_area_double_click_respects_window_text_boundary_mode_under_scroll_offset() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());
    app.with_global_mut_untracked(
        fret_runtime::WindowTextBoundaryModeService::default,
        |svc, _app| {
            svc.set_base_mode(
                AppWindowId::default(),
                fret_runtime::TextBoundaryMode::Identifier,
            );
        },
    );

    let model = app.models_mut().insert("can't".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-area-double-click-boundary-mode-scroll",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            for _ in 0..40 {
                                let mut row_layout = crate::element::LayoutStyle::default();
                                row_layout.size.height = Length::Px(Px(18.0));
                                out.push(cx.container(
                                    crate::element::ContainerProps {
                                        layout: row_layout,
                                        ..Default::default()
                                    },
                                    |cx| vec![cx.text("filler")],
                                ));
                            }

                            let mut props = crate::element::TextAreaProps::new(model.clone());
                            props.layout.size.width = Length::Px(Px(160.0));
                            props.layout.size.height = Length::Px(Px(80.0));
                            out.push(cx.text_area(props));

                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let area_node = *ui
        .children(column_node)
        .last()
        .expect("expected area as last child");
    let area_bounds = ui.debug_node_bounds(area_node).expect("area bounds");

    scroll_handle.set_offset(Point::new(Px(0.0), area_bounds.origin.y));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let area_bounds = ui
        .debug_node_visual_bounds(area_node)
        .expect("area bounds after scroll");
    let pos = Point::new(
        Px(area_bounds.origin.x.0 + 5.0),
        Px(area_bounds.origin.y.0 + 5.0),
    );
    assert_eq!(
        ui.debug_hit_test(pos).hit,
        Some(area_node),
        "expected the scrolled hit-test position to target the text area"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(area_node));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    let selection_utf16 = snapshot.selection_utf16;

    assert_eq!(
        selection_utf16,
        Some((0, 3)),
        "Identifier mode should stop at the apostrophe"
    );
}

#[test]
fn declarative_pointer_region_hook_can_request_focus_for_other_element() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pointer-region-can-request-focus-other-element",
        |cx| {
            vec![cx.semantics(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Slider,
                    label: Some(Arc::from("focus-target")),
                    ..Default::default()
                },
                |cx| {
                    let target = cx.root_id();

                    vec![cx.pointer_region(
                        crate::element::PointerRegionProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: crate::element::Length::Fill,
                                    height: crate::element::Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            enabled: true,
                        },
                        |cx| {
                            cx.pointer_region_on_pointer_down(Arc::new(move |host, _cx, down| {
                                if down.button != MouseButton::Left {
                                    return false;
                                }
                                host.request_focus(target);
                                true
                            }));
                            vec![]
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let semantics_node = ui.children(root)[0];
    let pointer_node = ui.children(semantics_node)[0];
    let pointer_bounds = ui.debug_node_bounds(pointer_node).expect("pointer bounds");
    let position = Point::new(
        Px(pointer_bounds.origin.x.0 + 2.0),
        Px(pointer_bounds.origin.y.0 + 2.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.focus(), Some(semantics_node));
}

#[test]
fn dismissible_layer_pointer_move_observer_does_not_break_click_through() {
    struct CountPointerDown {
        clicks: fret_runtime::Model<u32>,
    }

    impl<H: UiHost> Widget<H> for CountPointerDown {
        fn hit_test(&self, bounds: Rect, position: Point) -> bool {
            bounds.contains(position)
        }

        fn event(&mut self, cx: &mut crate::widget::EventCx<'_, H>, event: &fret_core::Event) {
            if matches!(
                event,
                fret_core::Event::Pointer(fret_core::PointerEvent::Down { .. })
            ) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.clicks, |v: &mut u32| *v = v.saturating_add(1));
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let clicks = app.models_mut().insert(0u32);
    let base = ui.create_node(CountPointerDown {
        clicks: clicks.clone(),
    });
    ui.set_root(base);

    let moves = app.models_mut().insert(0u32);
    let moves_for_hook = moves.clone();
    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-pointer-move-observer",
        move |cx| {
            cx.dismissible_on_pointer_move(Arc::new(move |host, _acx, _mv| {
                let _ = host
                    .models_mut()
                    .update(&moves_for_hook, |v: &mut u32| *v = v.saturating_add(1));
                false
            }));
            Vec::new()
        },
    );
    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_visible(layer, true);
    ui.set_layer_hit_testable(layer, true);
    ui.set_layer_wants_pointer_move_events(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let p = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: p,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&moves), Some(1));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: p,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        app.models().get_copied(&clicks),
        Some(1),
        "expected click-through dispatch to reach the underlay"
    );
}

#[test]
fn declarative_resizable_panel_group_updates_model_on_drag() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(40.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(vec![0.33, 0.34, 0.33]);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "resizable-panel-group-drag",
        |cx| {
            let mut props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            props.min_px = vec![Px(10.0)];
            props.chrome = crate::ResizablePanelGroupStyle {
                hit_thickness: Px(10.0),
                ..Default::default()
            };
            vec![cx.resizable_panel_group(props, |cx| {
                vec![
                    cx.spacer(crate::element::SpacerProps::default()),
                    cx.spacer(crate::element::SpacerProps::default()),
                    cx.spacer(crate::element::SpacerProps::default()),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let fractions_now = app.models().get_cloned(&model).unwrap_or_default();
    let layout = crate::resizable_panel_group::compute_resizable_panel_group_layout(
        fret_core::Axis::Horizontal,
        bounds,
        3,
        fractions_now,
        Px(0.0),
        Px(10.0),
        &[Px(10.0)],
    );
    let down_x = layout.handle_centers.first().copied().unwrap_or(0.0);
    let down = Point::new(Px(down_x), Px(20.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(128.0), Px(20.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(128.0), Px(20.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let v = app.models().get_cloned(&model).unwrap_or_default();
    assert!(
        v.first().copied().unwrap_or(0.0) > 0.33,
        "expected left panel to grow, got {v:?}"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn resizable_panel_group_registers_viewport_roots_for_panels() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(vec![0.5, 0.5]);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "resizable-panel-group-flow-islands",
        |cx| {
            let props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            vec![cx.resizable_panel_group(props, |cx| {
                vec![
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.hover_region(
                                crate::element::HoverRegionProps::default(),
                                |cx, _hovered| vec![cx.text("left")],
                            )]
                        },
                    ),
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("right")],
                    ),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let group = ui.children(root)[0];
    let panel_a = ui.children(group)[0];
    let panel_b = ui.children(group)[1];
    let hover = ui.children(panel_a)[0];
    let hover_text = ui.children(hover)[0];
    let panel_b_text = ui.children(panel_b)[0];

    let viewport_root_nodes: Vec<_> = ui
        .viewport_roots()
        .iter()
        .map(|(node, _bounds)| *node)
        .collect();
    assert_eq!(viewport_root_nodes.len(), 2);
    assert!(viewport_root_nodes.contains(&panel_a));
    assert!(viewport_root_nodes.contains(&panel_b));

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(panel_a).is_some());
    assert!(engine.layout_id_for_node(hover).is_some());
    assert!(engine.layout_id_for_node(hover_text).is_some());
    assert!(engine.layout_id_for_node(panel_b).is_some());
    assert!(engine.layout_id_for_node(panel_b_text).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn pressable_on_activate_hook_runs_on_pointer_activation() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let activated = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-activate-hook-pointer",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let activated = activated.clone();
                    cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                        assert_eq!(reason, ActivateReason::Pointer);
                        let _ = host
                            .models_mut()
                            .update(&activated, |v: &mut bool| *v = true);
                    }));
                    vec![cx.text("activate")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&activated), Some(false));

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let position = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&activated), Some(true));
}

#[test]
fn pressable_clears_pressed_and_releases_capture_on_move_without_buttons() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let pressable_element_id_cell: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let pressable_element_id_cell_for_render = pressable_element_id_cell.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-move-without-buttons",
        move |cx| {
            let pressable_element_id_cell = pressable_element_id_cell_for_render.clone();
            vec![cx.pressable_with_id(
                crate::element::PressableProps::default(),
                move |cx, _state, id| {
                    pressable_element_id_cell.set(Some(id));
                    vec![cx.text("pressable")]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_element_id = pressable_element_id_cell
        .get()
        .expect("missing pressable element id");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        Some(pressable_node)
    );
    assert!(
        crate::elements::is_pressed_pressable(&mut app, window, pressable_element_id),
        "expected pressable to set pressed state on pointer down"
    );

    // Simulate a runner/platform edge case: we never receive `PointerEvent::Up`, but we do observe
    // that no buttons are pressed anymore.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(inside.x.0 + 10.0), inside.y),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.captured_for(fret_core::PointerId(0)), None);
    assert!(!crate::elements::is_pressed_pressable(
        &mut app,
        window,
        pressable_element_id
    ));
}

#[test]
fn pressable_clears_pressed_state_when_element_is_removed() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let pressable_element_id_cell: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let pressable_element_id_cell_for_render = pressable_element_id_cell.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-unmount",
        move |cx| {
            let pressable_element_id_cell = pressable_element_id_cell_for_render.clone();
            vec![cx.pressable_with_id(
                crate::element::PressableProps::default(),
                move |cx, _state, id| {
                    pressable_element_id_cell.set(Some(id));
                    vec![cx.text("pressable")]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_element_id = pressable_element_id_cell
        .get()
        .expect("missing pressable element id");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        crate::elements::is_pressed_pressable(&mut app, window, pressable_element_id),
        "expected pressable to be pressed after pointer down"
    );

    // Drop the pressable element without sending pointer up/cancel events (e.g. overlay closes).
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-unmount",
        |_cx| Vec::new(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        !crate::elements::is_pressed_pressable(&mut app, window, pressable_element_id),
        "expected pressed state to clear when the element is removed"
    );
}

#[test]
fn pressable_on_hover_change_hook_runs_on_pointer_move() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-hover-change-hook",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let hovered = hovered.clone();
                    cx.pressable_on_hover_change(Arc::new(move |host, _cx, is_hovered| {
                        let _ = host
                            .models_mut()
                            .update(&hovered, |v: &mut bool| *v = is_hovered);
                    }));
                    vec![cx.text("hover me")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&hovered), Some(false));

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: inside,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&hovered), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&hovered), Some(false));
}

#[test]
fn pressable_on_hover_change_hook_runs_after_wheel_scroll_without_pointer_move() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(None::<u32>);
    let handle = crate::scroll::ScrollHandle::default();
    let item_h = Px(20.0);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-hover-after-wheel-scroll",
        |cx| {
            let scroll = cx.scroll(
                crate::element::ScrollProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = crate::element::Length::Fill;
                        layout.size.height = crate::element::Length::Fill;
                        layout.overflow = crate::element::Overflow::Clip;
                        layout
                    },
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                        (0..20)
                            .map(|idx| {
                                let hovered = hovered.clone();
                                cx.keyed(idx, move |cx| {
                                    cx.pressable(
                                        crate::element::PressableProps {
                                            layout: {
                                                let mut layout =
                                                    crate::element::LayoutStyle::default();
                                                layout.size.width = crate::element::Length::Fill;
                                                layout.size.height =
                                                    crate::element::Length::Px(item_h);
                                                layout
                                            },
                                            ..Default::default()
                                        },
                                        move |cx, _state| {
                                            cx.pressable_on_hover_change(Arc::new(
                                                move |host, _cx, is_hovered| {
                                                    if !is_hovered {
                                                        return;
                                                    }
                                                    let _ = host
                                                        .models_mut()
                                                        .update(&hovered, |v: &mut Option<u32>| {
                                                            *v = Some(idx as u32)
                                                        });
                                                },
                                            ));
                                            vec![cx.text(format!("Item {idx}"))]
                                        },
                                    )
                                })
                            })
                            .collect::<Vec<_>>()
                    })]
                },
            );
            vec![scroll]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&hovered), Some(None));

    let position = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(Some(0)));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            pointer_id: fret_core::PointerId(0),
            position,
            delta: Point::new(Px(0.0), Px(-20.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(Some(1)));
}

#[test]
fn pressable_hover_state_ignores_touch_pointer_moves() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-hover-ignores-touch",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let hovered = hovered.clone();
                    cx.pressable_on_hover_change(Arc::new(move |host, _cx, is_hovered| {
                        let _ = host
                            .models_mut()
                            .update(&hovered, |v: &mut bool| *v = is_hovered);
                    }));
                    vec![cx.text("hover me")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: inside,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(false));
}

#[test]
fn pressable_on_activate_hook_runs_on_keyboard_activation() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let activated = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-activate-hook-keyboard",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let activated = activated.clone();
                    cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                        assert_eq!(reason, ActivateReason::Keyboard);
                        let _ = host
                            .models_mut()
                            .update(&activated, |v: &mut bool| *v = true);
                    }));
                    vec![cx.text("activate")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&activated), Some(false));

    let pressable_node = ui.children(root)[0];
    ui.set_focus(Some(pressable_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyUp {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
        },
    );

    assert_eq!(app.models().get_copied(&activated), Some(true));
}

#[test]
fn dismissible_on_dismiss_request_hook_runs_on_escape() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let dismissed = app.models_mut().insert(false);

    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = super::super::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-escape",
        |cx| {
            let dismissed = dismissed.clone();
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, req| {
                assert_eq!(req.reason, DismissReason::Escape);
                let _ = host
                    .models_mut()
                    .update(&dismissed, |v: &mut bool| *v = true);
            }));

            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _| {
                    vec![cx.text("child")]
                }),
            ]
        },
    );

    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Focus a descendant in the overlay so Escape bubbles up to the dismissible layer.
    let focused = ui.children(overlay_root)[0];
    ui.set_focus(Some(focused));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get_copied(&dismissed), Some(true));
}

#[test]
fn dismissible_on_dismiss_request_hook_runs_on_outside_press_observer() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let dismissed = app.models_mut().insert(false);

    // Base root provides a hit-test target so the pointer down is "outside" the overlay.
    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = super::super::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-outside-press",
        |cx| {
            let dismissed = dismissed.clone();
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, req| {
                match req.reason {
                    DismissReason::OutsidePress { pointer: Some(cx) } => {
                        assert_eq!(cx.pointer_id, fret_core::PointerId(0));
                        assert_eq!(cx.pointer_type, fret_core::PointerType::Mouse);
                        assert_eq!(cx.button, MouseButton::Left);
                        assert_eq!(cx.modifiers, Modifiers::default());
                        assert_eq!(cx.click_count, 1);
                    }
                    other => panic!("expected outside-press dismissal, got {other:?}"),
                }
                let _ = host
                    .models_mut()
                    .update(&dismissed, |v: &mut bool| *v = true);
            }));
            Vec::new()
        },
    );

    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Pointer down hits the base root (overlay has no children and is hit-test transparent),
    // so outside-press observer dispatch runs for the overlay root.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(2.0), Px(2.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&dismissed), Some(true));
}

#[test]
fn roving_flex_arrow_keys_move_focus_and_update_selection() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let model = app
        .models_mut()
        .insert(Option::<Arc<str>>::Some(Arc::from("a")));
    let values: Arc<[Arc<str>]> = Arc::from([Arc::from("a"), Arc::from("b"), Arc::from("c")]);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "roving-flex",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, true, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                let values = values.clone();
                let model = model.clone();
                cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                    use crate::action::RovingNavigateResult;
                    use fret_core::KeyCode;

                    let Some(current) = it.current else {
                        return RovingNavigateResult::NotHandled;
                    };

                    let forward = match it.key {
                        KeyCode::ArrowDown => true,
                        KeyCode::ArrowUp => false,
                        _ => return RovingNavigateResult::NotHandled,
                    };

                    let len = it.len;
                    let is_disabled =
                        |idx: usize| -> bool { it.disabled.get(idx).copied().unwrap_or(false) };

                    let mut target: Option<usize> = None;
                    if it.wrap {
                        for step in 1..=len {
                            let idx = if forward {
                                (current + step) % len
                            } else {
                                (current + len - (step % len)) % len
                            };
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    } else if forward {
                        target = ((current + 1)..len).find(|&i| !is_disabled(i));
                    } else if current > 0 {
                        target = (0..current).rev().find(|&i| !is_disabled(i));
                    }

                    RovingNavigateResult::Handled { target }
                }));
                cx.roving_on_active_change(Arc::new(move |host, _cx, idx| {
                    let Some(value) = values.get(idx).cloned() else {
                        return;
                    };
                    let next = Some(value);
                    let _ = host
                        .models_mut()
                        .update(&model, |v: &mut Option<Arc<str>>| *v = next);
                }));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        |child_cx, _st| vec![child_cx.text(label)],
                    )
                };
                vec![make("a"), make("b"), make("c")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let c = ui.children(roving)[2];
    ui.set_focus(Some(a));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        ui.focus(),
        Some(c),
        "expected ArrowDown to skip disabled child"
    );
    assert_eq!(
        app.models().get_cloned(&model).flatten().as_deref(),
        Some("c"),
    );
}

#[test]
fn roving_flex_treats_descendant_focus_as_active_item() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "roving-flex-focus-within",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                    use crate::action::RovingNavigateResult;
                    use fret_core::KeyCode;

                    let Some(current) = it.current else {
                        return RovingNavigateResult::NotHandled;
                    };

                    let forward = match it.key {
                        KeyCode::ArrowDown => true,
                        KeyCode::ArrowUp => false,
                        _ => return RovingNavigateResult::NotHandled,
                    };

                    let len = it.len;
                    let target = if forward {
                        (current + 1) % len
                    } else {
                        (current + len - 1) % len
                    };

                    RovingNavigateResult::Handled {
                        target: Some(target),
                    }
                }));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        move |child_cx, _st| {
                            vec![child_cx.pressable(
                                crate::element::PressableProps::default(),
                                |inner_cx, _st| vec![inner_cx.text(label)],
                            )]
                        },
                    )
                };

                vec![make("a"), make("b")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let a_inner = ui.children(a)[0];
    let b = ui.children(roving)[1];

    ui.set_focus(Some(a_inner));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        ui.focus(),
        Some(b),
        "expected roving to treat descendant focus as within the active item",
    );
}

#[test]
fn roving_flex_typeahead_hook_can_choose_target_index() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "roving-flex-typeahead-hook",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, false, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                cx.roving_on_typeahead(Arc::new(
                    |_host, _cx, it| {
                        if it.input == 'c' { Some(2) } else { None }
                    },
                ));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        |child_cx, _st| vec![child_cx.text(label)],
                    )
                };
                vec![make("a"), make("b"), make("c")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let c = ui.children(roving)[2];
    ui.set_focus(Some(a));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::KeyC,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(ui.focus(), Some(c));
}

#[test]
fn pressable_semantics_checked_is_exposed() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-pressable-checked",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: true,
                    a11y: crate::element::PressableA11y {
                        role: Some(fret_core::SemanticsRole::Checkbox),
                        label: Some(Arc::from("checked")),
                        checked: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |cx, _state| vec![cx.text("x")],
            )]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == fret_core::SemanticsRole::Checkbox && n.label.as_deref() == Some("checked")
        })
        .expect("expected checkbox semantics node");

    assert_eq!(node.flags.checked, Some(true));
    assert!(node.actions.invoke, "expected checkbox to be invokable");
}

#[test]
fn text_input_semantics_controls_element_is_exposed() {
    use std::cell::Cell;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert("hello".to_string());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text-input-controls",
        |cx| {
            let listbox_id_out: Cell<Option<crate::elements::GlobalElementId>> = Cell::new(None);
            let listbox = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::ListBox,
                    test_id: Some(Arc::from("listbox")),
                    ..Default::default()
                },
                |_cx, id| {
                    listbox_id_out.set(Some(id));
                    Vec::new()
                },
            );

            let mut props = crate::element::TextInputProps::new(model.clone());
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            props.test_id = Some(Arc::from("combo"));
            props.a11y_role = Some(fret_core::SemanticsRole::ComboBox);
            props.controls_element = listbox_id_out.get().map(|id| id.0);

            vec![cx.text_input(props), listbox]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let combo = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("combo"))
        .expect("expected combobox semantics node");
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("listbox"))
        .expect("expected listbox semantics node");

    assert!(
        combo.controls.contains(&listbox.id),
        "expected combobox to control the listbox"
    );
}
