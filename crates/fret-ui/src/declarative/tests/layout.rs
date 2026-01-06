use super::*;

#[test]
fn hover_region_reports_hovered_even_when_child_is_pressable() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![cx.hover_region(
            crate::element::HoverRegionProps::default(),
            |cx, hovered| {
                let trigger = cx
                    .pressable(crate::element::PressableProps::default(), |cx, _state| {
                        vec![cx.text("trigger")]
                    });

                let mut children = vec![trigger];
                if hovered {
                    children.push(cx.text("hovered"));
                }
                children
            },
        )]
    }

    // Frame 0: not hovered yet, so only the trigger is present.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "hover-region",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let hover_region_node = ui.children(root)[0];
    assert_eq!(ui.children(hover_region_node).len(), 1);
    let trigger_node = ui.children(hover_region_node)[0];
    let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");

    let pos = fret_core::Point::new(
        Px(trigger_bounds.origin.x.0 + 2.0),
        Px(trigger_bounds.origin.y.0 + 2.0),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: pos,
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    // Frame 1: hover_region should now observe hovered=true even though the hit node is a Pressable.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "hover-region",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let hover_region_node = ui.children(root)[0];
    assert_eq!(ui.children(hover_region_node).len(), 2);
}

#[test]
fn row_justify_center_and_align_end_positions_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(20.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "row-align",
        |cx| {
            let mut props = crate::element::RowProps {
                gap: Px(5.0),
                justify: MainAlign::Center,
                align: CrossAlign::End,
                ..Default::default()
            };
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            vec![cx.row(props, |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let row_node = ui.children(root)[0];
    let children = ui.children(row_node);
    assert_eq!(children.len(), 3);

    let b0 = ui.debug_node_bounds(children[0]).expect("child0 bounds");
    let b1 = ui.debug_node_bounds(children[1]).expect("child1 bounds");
    let b2 = ui.debug_node_bounds(children[2]).expect("child2 bounds");

    // Each text measures to 10x10. With gap=5 and width=100:
    // content_w = 3*10 + 2*5 = 40; remaining=60; center => start_offset=30.
    assert!((b0.origin.x.0 - 30.0).abs() < 0.01, "x0={:?}", b0.origin.x);
    assert!((b1.origin.x.0 - 45.0).abs() < 0.01, "x1={:?}", b1.origin.x);
    assert!((b2.origin.x.0 - 60.0).abs() < 0.01, "x2={:?}", b2.origin.x);

    // align-end with row height 20 => y = 0 + (20-10)=10.
    assert!((b0.origin.y.0 - 10.0).abs() < 0.01, "y0={:?}", b0.origin.y);
    assert!((b1.origin.y.0 - 10.0).abs() < 0.01, "y1={:?}", b1.origin.y);
    assert!((b2.origin.y.0 - 10.0).abs() < 0.01, "y2={:?}", b2.origin.y);
}

#[test]
fn flex_wrap_positions_children_on_multiple_rows() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(25.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "flex-wrap",
        |cx| {
            let mut props = crate::element::FlexProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.wrap = true;
            vec![cx.flex(props, |cx| {
                vec![
                    cx.text("a"),
                    cx.text("b"),
                    cx.text("c"),
                    cx.text("d"),
                    cx.text("e"),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 5);

    let b0 = ui.debug_node_bounds(children[0]).expect("child0 bounds");
    let b1 = ui.debug_node_bounds(children[1]).expect("child1 bounds");
    let b2 = ui.debug_node_bounds(children[2]).expect("child2 bounds");
    let b3 = ui.debug_node_bounds(children[3]).expect("child3 bounds");
    let b4 = ui.debug_node_bounds(children[4]).expect("child4 bounds");

    assert!((b0.origin.x.0 - 0.0).abs() < 0.01, "x0={:?}", b0.origin.x);
    assert!((b1.origin.x.0 - 10.0).abs() < 0.01, "x1={:?}", b1.origin.x);
    assert!((b2.origin.x.0 - 0.0).abs() < 0.01, "x2={:?}", b2.origin.x);
    assert!((b3.origin.x.0 - 10.0).abs() < 0.01, "x3={:?}", b3.origin.x);
    assert!((b4.origin.x.0 - 0.0).abs() < 0.01, "x4={:?}", b4.origin.x);

    assert!((b0.origin.y.0 - 0.0).abs() < 0.01, "y0={:?}", b0.origin.y);
    assert!((b1.origin.y.0 - 0.0).abs() < 0.01, "y1={:?}", b1.origin.y);
    assert!((b2.origin.y.0 - 10.0).abs() < 0.01, "y2={:?}", b2.origin.y);
    assert!((b3.origin.y.0 - 10.0).abs() < 0.01, "y3={:?}", b3.origin.y);
    assert!((b4.origin.y.0 - 20.0).abs() < 0.01, "y4={:?}", b4.origin.y);
}

#[test]
fn pressable_keyboard_activation_dispatches_click_command() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let cmd = CommandId::new("test.pressable.click");

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "pressable-keyboard",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let cmd = cmd.clone();
                    cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                        host.dispatch_command(Some(acx.window), cmd.clone());
                    }));
                    vec![cx.text("ok")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    ui.set_focus(Some(pressable_node));
    assert_eq!(ui.focus(), Some(pressable_node));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyUp {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
        },
    );
    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Command { command, .. } if *command == cmd)),
        "expected click command effect"
    );
}

#[test]
fn pressable_disabled_is_not_focusable() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "pressable-disabled-focus",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: false,
                    ..Default::default()
                },
                |cx, _state| vec![cx.text("disabled")],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert_eq!(ui.first_focusable_descendant(root), None);
}

#[test]
fn image_paints_image_op() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    let img = fret_core::ImageId::default();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-image",
        |cx| {
            let mut p = crate::element::ImageProps::new(img);
            p.layout.size.width = crate::element::Length::Px(Px(160.0));
            p.layout.size.height = crate::element::Length::Px(Px(80.0));
            vec![cx.image_props(p)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::Image { image, .. } if *image == img)),
        "expected an Image op for the declarative image element"
    );
}

#[test]
fn overflow_clip_pushes_clip_rect_for_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-overflow-clip",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.overflow = crate::element::Overflow::Clip;
            vec![cx.container(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let pushes = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PushClipRect { .. }))
        .count();
    let pops = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PopClip))
        .count();

    assert!(
        pushes >= 1,
        "expected container overflow clip to push a clip rect"
    );
    assert!(
        pops >= 1,
        "expected container overflow clip to pop a clip rect"
    );
}

#[test]
fn overflow_clip_with_corner_radii_pushes_rounded_clip_rect() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-overflow-clip-rounded",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.overflow = crate::element::Overflow::Clip;
            props.corner_radii = fret_core::Corners::all(Px(8.0));
            vec![cx.container(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::PushClipRRect { .. })),
        "expected container overflow clip + corner radii to push a rounded clip rect"
    );
}

#[test]
fn overflow_visible_does_not_push_clip_rect() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-overflow-visible",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("child")])],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert!(
        !scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::PushClipRect { .. } | SceneOp::PushClipRRect { .. }
        )),
        "expected no clip ops by default"
    );
}

#[test]
fn scroll_wheel_updates_offset_and_shifts_child_bounds() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-wheel",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Px(Px(20.0));
            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                )]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui.debug_node_bounds(column_node).expect("column bounds");

    let wheel_pos = fret_core::Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_bounds(column_node)
        .expect("column bounds after scroll");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after wheel scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_thumb_drag_updates_offset() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            stack_layout.size.height = crate::element::Length::Px(Px(20.0));

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.column(
                                crate::element::ColumnProps {
                                    gap: Px(0.0),
                                    ..Default::default()
                                },
                                |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: None,
                        },
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Vertical,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scroll_node = ui.children(stack_node)[0];
    let scrollbar_node = ui.children(stack_node)[1];
    let column_node = ui.children(scroll_node)[0];
    let before = ui.debug_node_bounds(column_node).expect("column bounds");

    // Click/drag the scrollbar thumb down (thumb starts at the top at offset=0).
    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let down_pos = fret_core::Point::new(
        Px(scrollbar_bounds.origin.x.0 + 1.0),
        Px(scrollbar_bounds.origin.y.0 + 2.0),
    );
    let move_pos = fret_core::Point::new(down_pos.x, Px(down_pos.y.0 + 8.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
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
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected thumb drag to update scroll handle offset, got {:?}",
        scroll_handle.offset().y
    );

    ui.invalidate(scroll_node, Invalidation::Layout);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_bounds(column_node)
        .expect("column bounds after drag");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after thumb drag: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_thumb_drag_updates_offset_horizontal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag-x",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            stack_layout.size.height = crate::element::Length::Px(Px(20.0));

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            axis: crate::element::ScrollAxis::X,
                            scroll_handle: Some(scroll_handle.clone()),
                        },
                        |cx| {
                            let mut content_layout = crate::element::LayoutStyle::default();
                            content_layout.size.width = crate::element::Length::Px(Px(300.0));
                            content_layout.size.height = crate::element::Length::Fill;

                            vec![cx.container(
                                crate::element::ContainerProps {
                                    layout: content_layout,
                                    ..Default::default()
                                },
                                |cx| vec![cx.text("abc")],
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: None,
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: Some(Px(0.0)),
                        },
                        size: crate::element::SizeStyle {
                            height: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Horizontal,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scroll_node = ui.children(stack_node)[0];
    let scrollbar_node = ui.children(stack_node)[1];
    let content_node = ui.children(scroll_node)[0];
    let before = ui.debug_node_bounds(content_node).expect("content bounds");

    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let down_pos = fret_core::Point::new(
        Px(scrollbar_bounds.origin.x.0 + 2.0),
        Px(scrollbar_bounds.origin.y.0 + 1.0),
    );
    let move_pos = fret_core::Point::new(Px(down_pos.x.0 + 12.0), down_pos.y);
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
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
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().x.0 > 0.01,
        "expected thumb drag to update scroll handle offset, got {:?}",
        scroll_handle.offset().x
    );

    ui.invalidate(scroll_node, Invalidation::Layout);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_bounds(content_node)
        .expect("content bounds after drag");

    assert!(
        after.origin.x.0 < before.origin.x.0,
        "expected content to move left after thumb drag: before={:?} after={:?}",
        before.origin.x,
        after.origin.x
    );
}

#[test]
fn fill_respects_max_width_constraint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(100.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-max-width",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Fill,
                            max_width: Some(Px(100.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx| vec![],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let rect = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    assert_eq!(rect.size.width, Px(100.0));
}

#[test]
fn flex_child_margin_affects_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-flex-margin",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Px(Px(10.0));
                    b.layout.size.height = crate::element::Length::Px(Px(10.0));
                    b.layout.margin.left = crate::element::MarginEdge::Px(Px(5.0));

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 2);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin.x, Px(0.0));
    assert_eq!(b_bounds.origin.x, Px(15.0));
}

#[test]
fn flex_child_auto_margins_center_child() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-flex-mx-auto",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));
                    a.layout.margin.left = crate::element::MarginEdge::Auto;
                    a.layout.margin.right = crate::element::MarginEdge::Auto;
                    vec![cx.container(a, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");
    assert_eq!(flex_bounds.size.width, Px(100.0));
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 1);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");

    assert_eq!(a_bounds.origin.x, Px(45.0));
}

#[test]
fn flex_child_negative_margin_shifts_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-flex-negative-margin",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Px(Px(10.0));
                    a.layout.size.height = crate::element::Length::Px(Px(10.0));

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Px(Px(10.0));
                    b.layout.size.height = crate::element::Length::Px(Px(10.0));
                    b.layout.margin.left = crate::element::MarginEdge::Px(Px(-5.0));

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 2);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin.x, Px(0.0));
    assert_eq!(b_bounds.origin.x, Px(5.0));
}

#[test]
fn container_absolute_inset_positions_child() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-stack-absolute",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let mut base = crate::element::ContainerProps::default();
                    base.layout.size.width = crate::element::Length::Px(Px(100.0));
                    base.layout.size.height = crate::element::Length::Px(Px(80.0));

                    let mut badge = crate::element::ContainerProps::default();
                    badge.layout.size.width = crate::element::Length::Px(Px(10.0));
                    badge.layout.size.height = crate::element::Length::Px(Px(10.0));
                    badge.layout.position = crate::element::PositionStyle::Absolute;
                    badge.layout.inset.top = Some(Px(0.0));
                    badge.layout.inset.right = Some(Px(0.0));

                    vec![
                        cx.container(base, |_cx| vec![]),
                        cx.container(badge, |_cx| vec![]),
                    ]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    assert_eq!(container_bounds.size.width, Px(100.0));
    assert_eq!(container_bounds.size.height, Px(80.0));

    let children = ui.children(container_node);
    assert_eq!(children.len(), 2);
    let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
    assert_eq!(badge_bounds.origin.x, Px(90.0));
    assert_eq!(badge_bounds.origin.y, Px(0.0));
}

#[test]
fn container_absolute_negative_inset_offsets_outside_parent() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-stack-absolute-negative-inset",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let mut base = crate::element::ContainerProps::default();
                    base.layout.size.width = crate::element::Length::Px(Px(100.0));
                    base.layout.size.height = crate::element::Length::Px(Px(80.0));

                    let mut badge = crate::element::ContainerProps::default();
                    badge.layout.size.width = crate::element::Length::Px(Px(10.0));
                    badge.layout.size.height = crate::element::Length::Px(Px(10.0));
                    badge.layout.position = crate::element::PositionStyle::Absolute;
                    badge.layout.inset.top = Some(Px(-5.0));
                    badge.layout.inset.left = Some(Px(-6.0));

                    vec![
                        cx.container(base, |_cx| vec![]),
                        cx.container(badge, |_cx| vec![]),
                    ]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let children = ui.children(container_node);
    assert_eq!(children.len(), 2);
    let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
    assert_eq!(badge_bounds.origin.x, Px(-6.0));
    assert_eq!(badge_bounds.origin.y, Px(-5.0));
}

#[test]
fn grid_places_children_in_columns() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-grid",
        |cx| {
            vec![cx.grid(
                crate::element::GridProps {
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = crate::element::Length::Fill;
                        l.size.height = crate::element::Length::Fill;
                        l
                    },
                    cols: 2,
                    ..Default::default()
                },
                |cx| {
                    let mut a = crate::element::ContainerProps::default();
                    a.layout.size.width = crate::element::Length::Fill;
                    a.layout.size.height = crate::element::Length::Fill;

                    let mut b = crate::element::ContainerProps::default();
                    b.layout.size.width = crate::element::Length::Fill;
                    b.layout.size.height = crate::element::Length::Fill;

                    vec![cx.container(a, |_cx| vec![]), cx.container(b, |_cx| vec![])]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let grid_node = ui.children(root)[0];
    let children = ui.children(grid_node);
    assert_eq!(children.len(), 2);
    let a_bounds = ui.debug_node_bounds(children[0]).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(children[1]).expect("b bounds");

    assert_eq!(a_bounds.origin.x, Px(0.0));
    assert_eq!(b_bounds.origin.x, Px(100.0));
    assert_eq!(a_bounds.size.width, Px(100.0));
    assert_eq!(b_bounds.size.width, Px(100.0));
}

#[test]
fn focus_ring_is_focus_visible_only() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(64.0), Px(32.0)),
    );
    let mut text = FakeTextService::default();

    let ring = crate::element::RingStyle {
        placement: crate::element::RingPlacement::Outset,
        width: Px(2.0),
        offset: Px(2.0),
        color: Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        offset_color: None,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-focus-visible",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Fill,
                            height: crate::element::Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    focus_ring: Some(ring),
                    ..Default::default()
                },
                |_cx, _st| vec![],
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let pressable_node = ui.children(root)[0];

    // Focus the pressable via pointer: should *not* show focus-visible ring.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: fret_core::Point::new(Px(4.0), Px(4.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(pressable_node),
        "expected pressable to be focused after pointer down"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    assert_eq!(
        scene.ops().len(),
        0,
        "expected no ring ops for mouse-focused control"
    );

    // Enable focus-visible via keyboard navigation: ring should appear for focused control.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(
        ui.focus(),
        Some(pressable_node),
        "expected focus to remain on pressable after keydown"
    );
    assert!(
        crate::focus_visible::is_focus_visible(&mut app, Some(window)),
        "expected focus-visible to be enabled after Tab keydown"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    assert!(
        !scene.ops().is_empty(),
        "expected ring ops for keyboard navigation focus-visible"
    );
}

#[test]
fn declarative_elements_can_observe_models_for_invalidation() {
    let mut app = TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root_name = "mvp50-observe-model";

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        root_name,
        |cx| {
            vec![cx.container(Default::default(), |cx| {
                cx.observe_model(&model, Invalidation::Layout);
                let v = cx.app.models().get_copied(&model).unwrap_or_default();
                vec![cx.text(format!("Value {v}"))]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let stats0 = ui.debug_stats();
    assert!(
        stats0.layout_nodes_visited > 0,
        "expected layout traversal: visited={} performed={}",
        stats0.layout_nodes_visited,
        stats0.layout_nodes_performed
    );
    let performed0 = stats0.layout_nodes_performed;
    assert!(performed0 > 0, "expected initial layout work");

    // A second layout pass with no changes and no re-render should perform no node layouts.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let performed1 = ui.debug_stats().layout_nodes_performed;
    assert_eq!(performed1, 0, "expected no layout work when clean");

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    ui.propagate_model_changes(&mut app, &changed);

    // The observed model change should invalidate the declarative host, enabling layout work.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let performed2 = ui.debug_stats().layout_nodes_performed;
    assert!(performed2 > 0, "expected model change to trigger relayout");
}

#[test]
fn model_observation_requires_rerender_after_frame_advance() {
    let mut app = TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-observe-contract-frame-advance",
        |cx| {
            vec![cx.container(Default::default(), |cx| {
                cx.observe_model(&model, Invalidation::Layout);
                let v = cx.app.models().get_copied(&model).unwrap_or_default();
                vec![cx.text(format!("Value {v}"))]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Advance the frame but intentionally skip the render pass.
    app.advance_frame();

    // The first model change still invalidates because UiTree retains the previous observation
    // index until the next layout/paint pass records observations again.
    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(
        ui.propagate_model_changes(&mut app, &changed),
        "expected invalidation from the last recorded observation index"
    );

    // Layout now runs on the advanced frame. Without a new render pass, the declarative layer
    // has no per-frame observation data to re-register, so the observation index is cleared.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // A second model change no longer invalidates: this encodes the ADR 0028 execution contract
    // that `render_root(...)` must be called each frame before layout/paint.
    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(
        !ui.propagate_model_changes(&mut app, &changed),
        "expected no invalidation without re-rendering after a frame advance"
    );
}

#[test]
fn container_applies_padding_and_paints_background() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-container",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)),
                    background: Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    ..Default::default()
                },
                |cx| vec![cx.text("hi")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let container_node = ui.children(root)[0];
    let text_node = ui.children(container_node)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");
    assert_eq!(text_bounds.origin.x, Px(4.0));
    assert_eq!(text_bounds.origin.y, Px(6.0));
    assert_eq!(text_bounds.size.width, Px(10.0));
    assert_eq!(text_bounds.size.height, Px(10.0));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 2);
    match scene.ops()[0] {
        SceneOp::Quad {
            rect, background, ..
        } => {
            assert_eq!(rect, container_bounds);
            assert_eq!(background.a, 1.0);
        }
        _ => panic!("expected quad op"),
    }
}

#[test]
fn container_paints_shadow_before_background() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp60-shadow",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    background: Some(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    shadow: Some(crate::element::ShadowStyle {
                        color: Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.5,
                        },
                        offset_x: Px(2.0),
                        offset_y: Px(3.0),
                        spread: Px(1.0),
                        softness: 0,
                        corner_radii: fret_core::Corners::all(Px(4.0)),
                    }),
                    corner_radii: fret_core::Corners::all(Px(4.0)),
                    ..Default::default()
                },
                |cx| vec![cx.text("hi")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let container_node = ui.children(root)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);

    let shadow_bounds = match scene.ops()[0] {
        SceneOp::Quad { rect, .. } => rect,
        _ => panic!("expected shadow quad first"),
    };
    match scene.ops()[1] {
        SceneOp::Quad {
            rect, background, ..
        } => {
            assert_eq!(rect, container_bounds);
            assert_eq!(background.a, 1.0);
        }
        _ => panic!("expected background quad second"),
    }

    assert!(shadow_bounds.origin.x.0 > container_bounds.origin.x.0);
    assert!(shadow_bounds.origin.y.0 > container_bounds.origin.y.0);
    assert!(shadow_bounds.size.width.0 > container_bounds.size.width.0);
    assert!(shadow_bounds.size.height.0 > container_bounds.size.height.0);
}

#[test]
fn pressable_dispatches_click_command_when_released_over_self() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let command = CommandId::from("test.click");

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-pressable",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: true,
                    ..Default::default()
                },
                |cx, _state| {
                    let command = command.clone();
                    cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                        host.dispatch_command(Some(acx.window), command.clone());
                    }));
                    vec![cx.container(
                        crate::element::ContainerProps {
                            padding: fret_core::Edges::all(Px(4.0)),
                            ..Default::default()
                        },
                        |cx| vec![cx.text("hi")],
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let position = Point::new(
        Px(pressable_bounds.origin.x.0 + 10.0),
        Px(pressable_bounds.origin.y.0 + 10.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Command { command: c, .. } if c.as_str() == "test.click")),
        "expected Effect::Command(test.click), got {effects:?}"
    );

    // Sanity: move outside should clear hover state for future interactions.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(200.0), Px(200.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
}

#[test]
fn flex_defaults_to_fit_content_under_constraints() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "flex-fit",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(5.0),
                    padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)),
                    ..Default::default()
                },
                |cx| vec![cx.text("a"), cx.text("b")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");

    // FakeTextService measures each text to 10x10. With gap=5 and padding (4,6):
    // inner_w = 10 + 5 + 10 = 25, outer_w = 25 + 8 = 33
    // inner_h = 10, outer_h = 10 + 12 = 22
    assert!(
        (flex_bounds.size.width.0 - 33.0).abs() < 0.01,
        "w={:?}",
        flex_bounds.size.width
    );
    assert!(
        (flex_bounds.size.height.0 - 22.0).abs() < 0.01,
        "h={:?}",
        flex_bounds.size.height
    );
}
