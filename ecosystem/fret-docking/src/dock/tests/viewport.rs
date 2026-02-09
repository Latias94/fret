use super::*;

#[test]
fn docking_viewport_panels_are_laid_out_before_overlay_layout_and_do_not_couple_fill() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService::default();

    let panel_left = PanelKey::new("test.viewport.left");
    let panel_right = PanelKey::new("test.viewport.right");

    let target_left = fret_core::RenderTargetId::from(KeyData::from_ffi(1));
    let target_right = fret_core::RenderTargetId::from(KeyData::from_ffi(2));

    let left_root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dock-test-left",
        |cx| {
            let flex = fret_ui::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(fret_ui::element::SpacerProps::default())]
            })]
        },
    );
    let right_root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dock-test-right",
        |cx| {
            let flex = fret_ui::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(fret_ui::element::SpacerProps::default())]
            })]
        },
    );

    let left_flex = ui.children(left_root)[0];
    let left_spacer = ui.children(left_flex)[0];

    let right_flex = ui.children(right_root)[0];
    let right_spacer = ui.children(right_flex)[0];

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_left, || DockPanel {
            title: "Left".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_left,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        dock.ensure_panel(&panel_right, || DockPanel {
            title: "Right".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_right,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });

        let left_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_left.clone()],
            active: 0,
        });
        let right_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_right.clone()],
            active: 0,
        });
        let root = dock.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left_tabs, right_tabs],
            fractions: vec![0.35, 0.65],
        });
        dock.graph.set_window_root(window, root);
    });

    let (expected_left, expected_right) = {
        let dock = app.global::<DockManager>().expect("dock manager");
        let root = dock.graph.window_root(window).expect("dock root");
        let (_chrome, dock_bounds) = dock_space_regions(bounds);
        let settings = fret_runtime::DockingInteractionSettings::default();
        let layout = compute_layout_map(
            &dock.graph,
            root,
            dock_bounds,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
        );
        let active = active_panel_content_bounds(&dock.graph, &layout);
        let left = active.get(&panel_left).copied().expect("left bounds");
        let right = active.get(&panel_right).copied().expect("right bounds");
        (left, right)
    };

    let dock_space = ui.create_node_retained(
        DockSpace::new(window)
            .with_panel_content(panel_left.clone(), left_root)
            .with_panel_content(panel_right.clone(), right_root),
    );
    ui.set_children(dock_space, vec![left_root, right_root]);
    ui.set_root(dock_space);

    let ok = Arc::new(AtomicBool::new(false));
    let overlay = ui.create_node_retained(OverlayAssertsViewportBounds {
        left_spacer,
        right_spacer,
        expected_left,
        expected_right,
        ok: ok.clone(),
    });
    ui.push_overlay_root(overlay, false);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        ok.load(Ordering::Relaxed),
        "expected overlay layout to observe viewport-laid-out spacer bounds"
    );
    assert_eq!(ui.debug_node_bounds(left_spacer), Some(expected_left));
    assert_eq!(ui.debug_node_bounds(right_spacer), Some(expected_right));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let dock = app.global::<DockManager>().expect("dock manager");
    assert_eq!(
        dock.viewport_content_rect(window, target_left),
        Some(expected_left)
    );
    assert_eq!(
        dock.viewport_content_rect(window, target_right),
        Some(expected_right)
    );
}
#[test]
fn docking_viewport_panels_keep_scroll_and_virtual_list_extents_constraint_correct() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(240.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService::default();

    let panel_left = PanelKey::new("test.viewport.scroll");
    let panel_right = PanelKey::new("test.viewport.vlist");

    let target_left = fret_core::RenderTargetId::from(KeyData::from_ffi(3));
    let target_right = fret_core::RenderTargetId::from(KeyData::from_ffi(4));

    let scroll_handle = ScrollHandle::default();
    let vlist_handle = VirtualListScrollHandle::new();

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_left, || DockPanel {
            title: "Scroll".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_left,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        dock.ensure_panel(&panel_right, || DockPanel {
            title: "List".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_right,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });

        let left_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_left.clone()],
            active: 0,
        });
        let right_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_right.clone()],
            active: 0,
        });
        let root = dock.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left_tabs, right_tabs],
            fractions: vec![0.5, 0.5],
        });
        dock.graph.set_window_root(window, root);
    });

    fn build_scroll_panel(
        cx: &mut fret_ui::ElementContext<'_, TestHost>,
        handle: ScrollHandle,
    ) -> Vec<fret_ui::element::AnyElement> {
        let mut props = fret_ui::element::ScrollProps::default();
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.layout.size.height = fret_ui::element::Length::Fill;
        props.axis = fret_ui::element::ScrollAxis::Y;
        props.scroll_handle = Some(handle);
        props.probe_unbounded = true;

        vec![cx.scroll(props, |cx| {
            let flex = fret_ui::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![cx.flex(flex, |cx| {
                vec![
                    cx.text("hello"),
                    cx.spacer(fret_ui::element::SpacerProps::default()),
                ]
            })]
        })]
    }

    fn build_vlist_panel(
        cx: &mut fret_ui::ElementContext<'_, TestHost>,
        handle: &VirtualListScrollHandle,
    ) -> Vec<fret_ui::element::AnyElement> {
        let options = fret_ui::element::VirtualListOptions::new(Px(10.0), 0);
        vec![cx.virtual_list(50, options, handle, |cx, items| {
            items
                .iter()
                .copied()
                .map(|item| {
                    cx.keyed(item.key, |cx| {
                        let flex = fret_ui::element::FlexProps {
                            direction: fret_core::Axis::Vertical,
                            layout: fret_ui::element::LayoutStyle {
                                size: fret_ui::element::SizeStyle {
                                    width: fret_ui::element::Length::Fill,
                                    height: fret_ui::element::Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        cx.flex(flex, |cx| {
                            vec![
                                cx.text("row"),
                                cx.spacer(fret_ui::element::SpacerProps::default()),
                            ]
                        })
                    })
                })
                .collect::<Vec<_>>()
        })]
    }

    let left_root_name = "dock-scroll-panel";
    let right_root_name = "dock-vlist-panel";

    let left_node = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        left_root_name,
        |cx| build_scroll_panel(cx, scroll_handle.clone()),
    );
    let right_node = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        right_root_name,
        |cx| build_vlist_panel(cx, &vlist_handle),
    );

    let dock_space = ui.create_node_retained(
        DockSpace::new(window)
            .with_panel_content(panel_left.clone(), left_node)
            .with_panel_content(panel_right.clone(), right_node),
    );
    ui.set_children(dock_space, vec![left_node, right_node]);
    ui.set_root(dock_space);

    // Frame 0: virtual list has not recorded viewport size yet, so it will mount no rows. Scroll
    // extents should still be constraint-correct and should not explode due to unbounded probes.
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let scroll_extent_0 = scroll_handle.content_size().height;
    assert!(
        scroll_extent_0.0 > 10.0 && scroll_extent_0.0 < 300.0,
        "expected scroll content height to stay bounded, got {scroll_extent_0:?}"
    );

    // Frame 1: virtual list now mounts rows and measures them. Its content extent must still stay
    // bounded (no 1e9-style probe expansion).
    app.advance_frame();
    let _ = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        left_root_name,
        |cx| build_scroll_panel(cx, scroll_handle.clone()),
    );
    let _ = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        right_root_name,
        |cx| build_vlist_panel(cx, &vlist_handle),
    );
    ui.invalidate(dock_space, Invalidation::Layout);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_extent_1 = scroll_handle.content_size().height;
    assert!(
        scroll_extent_1.0 > 10.0 && scroll_extent_1.0 < 300.0,
        "expected scroll content height to stay bounded after second frame, got {scroll_extent_1:?}"
    );

    let list_extent = vlist_handle.content_size().height;
    assert!(
        list_extent.0 > 100.0 && list_extent.0 < 100_000.0,
        "expected virtual list extent to be finite and measured, got {list_extent:?}"
    );
}
#[test]
fn split_viewports_forward_input_to_captured_viewport() {
    let mut harness = DockSplitViewportHarness::new();
    harness.layout();

    let down_pos = harness.viewport_point(harness.target_left);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!("expected a ViewportInput effect on down, got: {effects:?}");
    };
    assert_eq!(
        input.target, harness.target_left,
        "expected pointer down to forward to the left viewport"
    );

    let move_pos = harness.viewport_point(harness.target_right);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!("expected a ViewportInput effect on move, got: {effects:?}");
    };
    assert_eq!(
        input.target, harness.target_left,
        "expected viewport capture to keep forwarding to the captured viewport"
    );
    assert!(
        (0.0..=1.0).contains(&input.uv.0) && (0.0..=1.0).contains(&input.uv.1),
        "expected clamped uv during capture, got: {:?}",
        input.uv
    );
}
#[test]
fn viewport_capture_emits_clamped_pointer_moves_outside_draw_rect() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let down_pos = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    let outside = Point::new(Px(-50.0), Px(-50.0));
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
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

    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!("expected a ViewportInput effect during viewport capture, got: {effects:?}");
    };

    assert_eq!(
        input.kind,
        ViewportInputKind::PointerMove {
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
        }
    );
    assert_eq!(input.uv, (0.0, 0.0), "expected clamped uv at top-left");
    assert_eq!(
        input.target_px,
        (0, 0),
        "expected clamped target_px at top-left"
    );
}
#[test]
fn viewport_overlay_hooks_can_implement_layout_api_only() {
    use fret_core::DrawOrder;

    #[derive(Debug)]
    struct LayoutOnlyHooks;

    impl DockViewportOverlayHooks for LayoutOnlyHooks {
        fn paint_with_layout(
            &self,
            _theme: fret_ui::ThemeSnapshot,
            _window: AppWindowId,
            _panel: &fret_core::PanelKey,
            _viewport: super::ViewportPanel,
            layout: super::DockViewportLayout,
            scene: &mut Scene,
        ) {
            scene.push(SceneOp::Quad {
                order: DrawOrder(9999),
                rect: layout.draw_rect,
                background: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.35,
                },
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
    }

    let mut harness = DockViewportHarness::new();
    harness.app.with_global_mut(
        super::DockViewportOverlayHooksService::default,
        |svc, _app| {
            svc.set(Arc::new(LayoutOnlyHooks));
        },
    );

    let scene = harness.paint_scene();
    let layout = harness
        .app
        .global::<DockManager>()
        .and_then(|dock| dock.viewport_layout(harness.window, harness.target))
        .expect("expected viewport layout to be recorded during paint");

    assert!(
        scene.ops().iter().any(|op| match op {
            SceneOp::Quad { order, rect, .. } => {
                *order == DrawOrder(9999) && *rect == layout.draw_rect
            }
            _ => false,
        }),
        "expected overlay hook quad to be painted using layout.draw_rect"
    );
}
#[test]
fn viewport_capture_requests_animation_frames_while_active() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let down_pos = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.layout();

    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == harness.window)),
        "expected viewport capture to request animation frames, got: {effects:?}",
    );
}
#[test]
fn viewport_capture_emits_pointer_cancel_and_releases_capture() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let down_pos = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        Some(harness.root),
        "expected viewport capture to request pointer capture on down"
    );

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: None,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );

    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected pointer capture to be released on cancel",
    );

    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(evt)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!("expected viewport cancel input effect, got: {effects:?}");
    };
    assert!(
        matches!(evt.kind, fret_core::ViewportInputKind::PointerCancel { .. }),
        "expected ViewportInputKind::PointerCancel, got: {evt:?}",
    );
}
#[test]
fn viewport_capture_suppresses_viewport_moves_for_other_pointers() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "viewport capture must suppress viewport moves for other pointers, got: {effects:?}",
    );
}
#[test]
fn viewport_capture_does_not_clear_on_other_pointer_up() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let down_pos = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    let outside = Point::new(Px(-50.0), Px(-50.0));
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
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

    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!("expected a ViewportInput effect during viewport capture, got: {effects:?}");
    };
    assert_eq!(
        input.kind,
        ViewportInputKind::PointerMove {
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
        }
    );
    assert_eq!(input.uv, (0.0, 0.0), "expected clamped uv at top-left");
}
#[test]
fn viewport_capture_suppresses_secondary_right_click_bubbling() {
    let mut harness = DockViewportPropagationHarness::new();
    harness.layout();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.reset_spy();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (downs, ups) = harness.spy_counts();
    assert_eq!(
        (downs, ups),
        (0, 0),
        "secondary right click must not bubble while viewport capture is active, got downs={downs} ups={ups}",
    );
}
#[test]
fn viewport_right_click_bubbles_when_not_dragging() {
    let mut harness = DockViewportPropagationHarness::new();
    harness.layout();
    harness.reset_spy();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (downs, ups) = harness.spy_counts();
    assert_eq!(
        (downs, ups),
        (0, 1),
        "right click without drag should bubble on release so context menus can trigger, got downs={downs} ups={ups}",
    );
}
#[test]
fn viewport_right_drag_suppresses_context_menu_bubbling_on_release() {
    let mut harness = DockViewportPropagationHarness::new();
    harness.layout();
    harness.reset_spy();

    let start = harness.viewport_point();
    let end = Point::new(Px(start.x.0 + 20.0), Px(start.y.0 + 20.0));

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: end,
            buttons: fret_core::MouseButtons {
                right: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    harness.reset_spy();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: end,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (downs, ups) = harness.spy_counts();
    assert_eq!(
        (downs, ups),
        (0, 0),
        "right-drag release must not bubble to avoid triggering context menus, got downs={downs} ups={ups}",
    );
}
