use super::*;

#[test]
fn docking_bounds_for_element_reports_last_frame_panel_rects() {
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

    let split_root = app.with_global_mut(DockManager::default, |dock, _app| {
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
        root
    });

    let left_root_name = "dock-geom-left";
    let right_root_name = "dock-geom-right";

    let left_node = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        left_root_name,
        |cx| vec![cx.text("left")],
    );
    let right_node = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        right_root_name,
        |cx| vec![cx.text("right")],
    );

    let dock_space = ui.create_node_retained(
        DockSpace::new(window)
            .with_panel_content(panel_left.clone(), left_node)
            .with_panel_content(panel_right.clone(), right_node),
    );
    ui.set_children(dock_space, vec![left_node, right_node]);
    ui.set_root(dock_space);

    let (expected_left_0, expected_right_0) = {
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
        (
            active.get(&panel_left).copied().expect("left bounds"),
            active.get(&panel_right).copied().expect("right bounds"),
        )
    };

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let left_element = fret_ui::elements::global_root(window, left_root_name);
    let right_element = fret_ui::elements::global_root(window, right_root_name);
    let left_root_node =
        fret_ui::elements::node_for_element(&mut app, window, left_element).expect("left node");
    let right_root_node =
        fret_ui::elements::node_for_element(&mut app, window, right_element).expect("right node");

    assert_eq!(left_root_node, left_node);
    assert_eq!(right_root_node, right_node);
    assert_eq!(ui.debug_node_bounds(left_root_node), Some(expected_left_0));
    assert_eq!(
        ui.debug_node_bounds(right_root_node),
        Some(expected_right_0)
    );

    app.with_global_mut(DockManager::default, |dock, _app| {
        assert!(
            dock.graph
                .update_split_fractions(split_root, vec![0.5, 0.5]),
            "expected split fraction update to succeed"
        );
    });
    ui.invalidate(dock_space, Invalidation::Layout);

    let (expected_left_1, expected_right_1) = {
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
        (
            active.get(&panel_left).copied().expect("left bounds"),
            active.get(&panel_right).copied().expect("right bounds"),
        )
    };
    assert_ne!(expected_left_0, expected_left_1);
    assert_ne!(expected_right_0, expected_right_1);

    app.advance_frame();

    let _ = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        left_root_name,
        |cx| vec![cx.text("left")],
    );
    let _ = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        right_root_name,
        |cx| vec![cx.text("right")],
    );

    let ok = Arc::new(AtomicBool::new(false));
    let overlay = ui.create_node_retained(OverlayAssertsLastFrameElementBounds {
        window,
        left_element,
        right_element,
        left_node: left_root_node,
        right_node: right_root_node,
        expected_left_last: expected_left_0,
        expected_right_last: expected_right_0,
        expected_left_now: expected_left_1,
        expected_right_now: expected_right_1,
        outer: bounds,
        desired: Size::new(Px(80.0), Px(24.0)),
        side_offset: Px(6.0),
        side: OverlaySide::Bottom,
        align: OverlayAlign::End,
        ok: ok.clone(),
    });
    ui.push_overlay_root(overlay, false);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        ok.load(Ordering::Relaxed),
        "expected overlay layout to read last-frame element bounds while observing current viewport layout"
    );
    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window, left_element),
        Some(expected_left_0)
    );
    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window, right_element),
        Some(expected_right_0)
    );
    assert_eq!(ui.debug_node_bounds(left_root_node), Some(expected_left_1));
    assert_eq!(
        ui.debug_node_bounds(right_root_node),
        Some(expected_right_1)
    );
}
#[test]
fn bounds_for_element_is_window_scoped_across_multi_window_docking() {
    let window_a = AppWindowId::default();
    let window_b = AppWindowId::from(KeyData::from_ffi(42));

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(240.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(480.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService::default();

    let panel_a = PanelKey::new("mw.viewport.a");
    let panel_b = PanelKey::new("mw.viewport.b");
    let target_a = fret_core::RenderTargetId::from(KeyData::from_ffi(10));
    let target_b = fret_core::RenderTargetId::from(KeyData::from_ffi(11));

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_a, || DockPanel {
            title: "A".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_a,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        dock.ensure_panel(&panel_b, || DockPanel {
            title: "B".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_b,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });

        let tabs_a = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_a, tabs_a);

        let tabs_b = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_b.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_b, tabs_b);
    });

    let root_a_name = "mw-panel-a";
    let root_b_name = "mw-panel-b";

    let mut ui_a: UiTree<TestHost> = UiTree::new();
    ui_a.set_window(window_a);
    let node_a = declarative::render_root(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds_a,
        root_a_name,
        |cx| vec![cx.text("a")],
    );
    let dock_space_a = ui_a
        .create_node_retained(DockSpace::new(window_a).with_panel_content(panel_a.clone(), node_a));
    ui_a.set_children(dock_space_a, vec![node_a]);
    ui_a.set_root(dock_space_a);

    let mut ui_b: UiTree<TestHost> = UiTree::new();
    ui_b.set_window(window_b);
    let node_b = declarative::render_root(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds_b,
        root_b_name,
        |cx| vec![cx.text("b")],
    );
    let dock_space_b = ui_b
        .create_node_retained(DockSpace::new(window_b).with_panel_content(panel_b.clone(), node_b));
    ui_b.set_children(dock_space_b, vec![node_b]);
    ui_b.set_root(dock_space_b);

    // Frame 0: write current bounds (stored as "cur_bounds" in the element runtime).
    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    let expected_a_0 = ui_a
        .debug_node_bounds(node_a)
        .expect("expected window a bounds");
    let expected_b_0 = ui_b
        .debug_node_bounds(node_b)
        .expect("expected window b bounds");

    let element_a = fret_ui::elements::global_root(window_a, root_a_name);
    let element_b = fret_ui::elements::global_root(window_b, root_b_name);

    // Frame 1: swap prev/cur, so `bounds_for_element` returns frame 0 bounds.
    app.advance_frame();
    let _ = declarative::render_root(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds_a,
        root_a_name,
        |cx| vec![cx.text("a")],
    );
    let _ = declarative::render_root(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds_b,
        root_b_name,
        |cx| vec![cx.text("b")],
    );
    ui_a.invalidate(dock_space_a, Invalidation::Layout);
    ui_b.invalidate(dock_space_b, Invalidation::Layout);

    let ok = Arc::new(AtomicBool::new(false));
    let overlay = ui_b.create_node_retained(OverlayAssertsWindowScopedBoundsForElement {
        window_a,
        window_b,
        element_a,
        element_b,
        expected_a_last: expected_a_0,
        expected_b_last: expected_b_0,
        ok: ok.clone(),
    });
    ui_b.push_overlay_root(overlay, false);

    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    assert!(
        ok.load(Ordering::Relaxed),
        "expected bounds_for_element to be window-scoped across multi-window docking"
    );
}
#[test]
fn overlay_placement_must_use_window_local_anchor_bounds() {
    let window_a = AppWindowId::default();
    let window_b = AppWindowId::from(KeyData::from_ffi(42));

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(240.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(480.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService::default();

    let panel_a = PanelKey::new("mw.viewport.a");
    let panel_b = PanelKey::new("mw.viewport.b");
    let target_a = fret_core::RenderTargetId::from(KeyData::from_ffi(10));
    let target_b = fret_core::RenderTargetId::from(KeyData::from_ffi(11));

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_a, || DockPanel {
            title: "A".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_a,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        dock.ensure_panel(&panel_b, || DockPanel {
            title: "B".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_b,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });

        let tabs_a = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_a, tabs_a);

        let tabs_b = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_b.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_b, tabs_b);
    });

    let root_a_name = "mw-panel-a";
    let root_b_name = "mw-panel-b";

    let anchor_a_id: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
        Arc::new(Mutex::new(None));
    let anchor_b_id: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
        Arc::new(Mutex::new(None));

    let mut ui_a: UiTree<TestHost> = UiTree::new();
    ui_a.set_window(window_a);
    let anchor_a_id_setter = anchor_a_id.clone();
    let node_a = declarative::render_root(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds_a,
        root_a_name,
        move |cx| {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.position = fret_ui::element::PositionStyle::Absolute;
            layout.inset.top = Some(Px(20.0));
            layout.inset.left = Some(Px(240.0));
            layout.size.width = fret_ui::element::Length::Px(Px(100.0));
            layout.size.height = fret_ui::element::Length::Px(Px(30.0));

            let props = fret_ui::element::SemanticsProps {
                layout,
                ..Default::default()
            };
            vec![cx.semantics_with_id(props, move |cx, id| {
                *anchor_a_id_setter.lock().expect("anchor mutex") = Some(id);
                vec![cx.text("a")]
            })]
        },
    );
    let dock_space_a = ui_a
        .create_node_retained(DockSpace::new(window_a).with_panel_content(panel_a.clone(), node_a));
    ui_a.set_children(dock_space_a, vec![node_a]);
    ui_a.set_root(dock_space_a);

    let mut ui_b: UiTree<TestHost> = UiTree::new();
    ui_b.set_window(window_b);
    let anchor_b_id_setter = anchor_b_id.clone();
    let node_b = declarative::render_root(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds_b,
        root_b_name,
        move |cx| {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.position = fret_ui::element::PositionStyle::Absolute;
            layout.inset.top = Some(Px(20.0));
            layout.inset.left = Some(Px(40.0));
            layout.size.width = fret_ui::element::Length::Px(Px(100.0));
            layout.size.height = fret_ui::element::Length::Px(Px(30.0));

            let props = fret_ui::element::SemanticsProps {
                layout,
                ..Default::default()
            };
            vec![cx.semantics_with_id(props, move |cx, id| {
                *anchor_b_id_setter.lock().expect("anchor mutex") = Some(id);
                vec![cx.text("b")]
            })]
        },
    );
    let dock_space_b = ui_b
        .create_node_retained(DockSpace::new(window_b).with_panel_content(panel_b.clone(), node_b));
    ui_b.set_children(dock_space_b, vec![node_b]);
    ui_b.set_root(dock_space_b);

    let element_a = anchor_a_id.lock().expect("anchor mutex").unwrap();
    let element_b = anchor_b_id.lock().expect("anchor mutex").unwrap();

    // Frame 0: record current bounds into the element runtime's "cur" storage.
    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    let node_a_anchor = fret_ui::elements::node_for_element(&mut app, window_a, element_a)
        .expect("expected window a anchor node");
    let node_b_anchor = fret_ui::elements::node_for_element(&mut app, window_b, element_b)
        .expect("expected window b anchor node");

    let expected_a_0 = ui_a
        .debug_node_bounds(node_a_anchor)
        .expect("expected window a anchor bounds");
    let expected_b_0 = ui_b
        .debug_node_bounds(node_b_anchor)
        .expect("expected window b anchor bounds");

    app.advance_frame();
    let anchor_a_id_setter = anchor_a_id.clone();
    let _ = declarative::render_root(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds_a,
        root_a_name,
        move |cx| {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.position = fret_ui::element::PositionStyle::Absolute;
            layout.inset.top = Some(Px(20.0));
            layout.inset.left = Some(Px(240.0));
            layout.size.width = fret_ui::element::Length::Px(Px(100.0));
            layout.size.height = fret_ui::element::Length::Px(Px(30.0));

            let props = fret_ui::element::SemanticsProps {
                layout,
                ..Default::default()
            };
            vec![cx.semantics_with_id(props, move |cx, id| {
                *anchor_a_id_setter.lock().expect("anchor mutex") = Some(id);
                vec![cx.text("a")]
            })]
        },
    );
    let anchor_b_id_setter = anchor_b_id.clone();
    let _ = declarative::render_root(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds_b,
        root_b_name,
        move |cx| {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.position = fret_ui::element::PositionStyle::Absolute;
            layout.inset.top = Some(Px(20.0));
            layout.inset.left = Some(Px(40.0));
            layout.size.width = fret_ui::element::Length::Px(Px(100.0));
            layout.size.height = fret_ui::element::Length::Px(Px(30.0));

            let props = fret_ui::element::SemanticsProps {
                layout,
                ..Default::default()
            };
            vec![cx.semantics_with_id(props, move |cx, id| {
                *anchor_b_id_setter.lock().expect("anchor mutex") = Some(id);
                vec![cx.text("b")]
            })]
        },
    );
    ui_a.invalidate(dock_space_a, Invalidation::Layout);
    ui_b.invalidate(dock_space_b, Invalidation::Layout);

    let ok = Arc::new(AtomicBool::new(false));
    let overlay = ui_b.create_node_retained(OverlayAssertsWindowLocalOverlayPlacement {
        window_a,
        window_b,
        element_a,
        element_b,
        expected_a_last: expected_a_0,
        expected_b_last: expected_b_0,
        outer: bounds_b,
        desired: Size::new(Px(80.0), Px(24.0)),
        side_offset: Px(6.0),
        side: OverlaySide::Bottom,
        align: OverlayAlign::End,
        ok: ok.clone(),
    });
    ui_b.push_overlay_root(overlay, false);

    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    assert!(
        ok.load(Ordering::Relaxed),
        "expected overlay placement to only use window-local anchor bounds"
    );
}
