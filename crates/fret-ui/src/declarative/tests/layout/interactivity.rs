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
            pointer_id: fret_core::PointerId(0),
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
fn attach_semantics_is_layout_transparent_for_flex_items() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_row(cx: &mut ElementContext<'_, TestHost>, decorate: bool) -> Vec<AnyElement> {
        let mut row = crate::element::FlexProps::default();
        row.layout.size.width = crate::element::Length::Fill;
        row.layout.size.height = crate::element::Length::Fill;
        row.direction = fret_core::Axis::Horizontal;

        vec![cx.flex(row, |cx| {
            let left = cx.text("x");

            let mut right_props = crate::element::TextProps::new("a much longer title");
            right_props.wrap = fret_core::TextWrap::None;
            right_props.overflow = fret_core::TextOverflow::Ellipsis;
            right_props.layout.flex.grow = 1.0;
            right_props.layout.flex.shrink = 1.0;
            right_props.layout.flex.basis = crate::element::Length::Px(Px(0.0));
            right_props.layout.size.min_width = Some(Px(0.0));

            let mut right = cx.text_props(right_props);
            if decorate {
                right = right.attach_semantics(
                    crate::element::SemanticsDecoration::default().test_id("row-title"),
                );
            }

            vec![left, right]
        })]
    }

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "attach-semantics-layout-a",
        |cx| build_row(cx, false),
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "attach-semantics-layout-b",
        |cx| build_row(cx, true),
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(220.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(260.0), Px(0.0)),
        Size::new(Px(220.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_a = ui.children(root_a)[0];
    let right_a = ui.children(flex_a)[1];
    let bounds_a = ui
        .debug_node_bounds(right_a)
        .expect("decorator-free right text bounds");
    assert!(
        bounds_a.size.width.0 > 50.0,
        "expected flex child to expand, got {:?}",
        bounds_a
    );

    let flex_b = ui.children(root_b)[0];
    let right_b = ui.children(flex_b)[1];
    let bounds_b = ui
        .debug_node_bounds(right_b)
        .expect("decorated right text bounds");
    assert!(
        (bounds_a.size.width.0 - bounds_b.size.width.0).abs() < 0.01,
        "expected attach_semantics to be layout-transparent"
    );
}


#[test]
fn pressable_does_not_stretch_spacer_child_in_engine_tree() {
    struct RegistersViewportRoot {
        viewport: Rect,
    }

    impl<H: UiHost> Widget<H> for RegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let child = cx.children[0];
            let _ = cx.layout_viewport_root(child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(140.0)),
    );
    let viewport = Rect::new(
        fret_core::Point::new(Px(7.0), Px(11.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    let mut text = FakeTextService::default();

    let child_root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "pressable-engine-no-stretch",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.pressable(props, |cx, _state| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );

    let base = ui.create_node(RegistersViewportRoot { viewport });
    ui.set_children(base, vec![child_root]);
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let pressable = ui.children(child_root)[0];
    let spacer = ui.children(pressable)[0];

    let pressable_bounds = ui.debug_node_bounds(pressable).expect("pressable bounds");
    let spacer_bounds = ui.debug_node_bounds(spacer).expect("spacer bounds");

    assert_eq!(pressable_bounds, viewport);
    assert_eq!(spacer_bounds.origin, viewport.origin);
    assert!(spacer_bounds.size.width.0.abs() < 0.01);
    assert!(spacer_bounds.size.height.0.abs() < 0.01);
}


#[test]
fn hover_region_precomputes_flow_islands_for_multiple_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![cx.hover_region(
            crate::element::HoverRegionProps::default(),
            |cx, _hovered| {
                vec![
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("a")],
                    ),
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("b")],
                    ),
                ]
            },
        )]
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "hover-region-multi-child-flow-islands",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let hover = ui.children(root)[0];
    let flex_a = ui.children(hover)[0];
    let flex_b = ui.children(hover)[1];
    let text_a = ui.children(flex_a)[0];
    let text_b = ui.children(flex_b)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(flex_a).is_some());
    assert!(engine.layout_id_for_node(text_a).is_some());
    assert!(engine.layout_id_for_node(flex_b).is_some());
    assert!(engine.layout_id_for_node(text_b).is_some());
    ui.put_layout_engine(engine);
}


#[test]
fn pressable_wraps_multiple_children_in_engine_tree() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![cx.flex(
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
                vec![
                    cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                        vec![cx.text("a"), cx.text("b")]
                    }),
                ]
            },
        )]
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "pressable-engine-children",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex = ui.children(root)[0];
    let pressable = ui.children(flex)[0];
    let a = ui.children(pressable)[0];
    let b = ui.children(pressable)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(pressable).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
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
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        None,
        "expected pressable not to be focused after pointer down"
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: fret_core::Point::new(Px(4.0), Px(4.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(pressable_node),
        "expected pressable to be focused after pointer up"
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
fn model_observation_persists_after_frame_advance_without_render_root() {
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
    // has no per-frame observation data to re-register, but UiTree still retains the last known
    // observation sets (needed for cache-hit frames where a subtree is reused without being rebuilt).
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // A second model change still invalidates based on the previous observation set, even though
    // `render_root(...)` was not called on this frame.
    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(
        ui.propagate_model_changes(&mut app, &changed),
        "expected invalidation based on the last recorded observation set"
    );
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
            pointer_id: fret_core::PointerId(0),
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
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
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
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
}

