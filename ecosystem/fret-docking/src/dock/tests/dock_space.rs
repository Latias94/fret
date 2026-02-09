use super::*;

#[test]
fn render_and_bind_dock_panels_keeps_non_viewport_panel_alive() {
    let window = AppWindowId::default();
    let panel = fret_core::PanelKey::new("demo.controls");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(dock_space);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(CachedRetainedPanelRegistry::new()));
        },
    );

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Controls".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
    });

    let mut services = FakeTextService;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    );

    render_and_bind_dock_panels(&mut ui, &mut app, &mut services, window, bounds, dock_space);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let node = app
        .global::<DockPanelContentService>()
        .and_then(|svc| svc.get(window, &panel))
        .expect("expected panel node to be bound");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(60.0)),
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        ui.focus(),
        Some(node),
        "expected bound panel node to be focusable and receive pointer events"
    );
}
#[test]
fn dock_space_layout_assigns_active_panel_content_bounds_via_panel_nodes() {
    let window = AppWindowId::default();
    let panel = fret_core::PanelKey::new("demo.controls");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(dock_space);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(CachedRetainedPanelRegistry::new()));
        },
    );

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Controls".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
    });

    let mut services = FakeTextService;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );

    render_and_bind_dock_panels(&mut ui, &mut app, &mut services, window, bounds, dock_space);

    let root = app
        .global::<DockManager>()
        .and_then(|dock| dock.graph.window_root(window))
        .expect("expected dock window root");
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let settings = fret_runtime::DockingInteractionSettings::default();
    let dock_layout = compute_layout_map(
        &app.global::<DockManager>().unwrap().graph,
        root,
        dock_bounds,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
    );
    let active =
        active_panel_content_bounds(&app.global::<DockManager>().unwrap().graph, &dock_layout);
    let expected = active
        .get(&panel)
        .copied()
        .expect("expected active panel bounds");

    let _ = ui.layout(&mut app, &mut services, dock_space, bounds.size, 1.0);

    let node = app
        .global::<DockPanelContentService>()
        .and_then(|svc| svc.get(window, &panel))
        .expect("expected panel node to be bound");

    let laid_out = ui
        .debug_node_bounds(node)
        .expect("expected panel node bounds");
    assert!((laid_out.origin.x.0 - expected.origin.x.0).abs() < 0.01);
    assert!((laid_out.origin.y.0 - expected.origin.y.0).abs() < 0.01);
    assert!((laid_out.size.width.0 - expected.size.width.0).abs() < 0.01);
    assert!((laid_out.size.height.0 - expected.size.height.0).abs() < 0.01);
}
#[test]
fn dock_space_installs_internal_drag_route_anchor() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(dock_space);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let route = fret_ui::internal_drag::route(&app, window, DRAG_KIND_DOCK_PANEL);

    assert_eq!(
        route,
        Some(dock_space),
        "expected DockSpace to install an internal drag route anchor during paint"
    );
}
#[test]
fn dock_space_paints_empty_state_when_no_window_root() {
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node_retained(DockSpace::new(AppWindowId::default()));
    ui.set_root(root);

    let mut app = TestHost::new();
    let mut text = FakeTextService;

    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);

    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::Quad { .. }))
    );
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::Text { .. }))
    );
}
#[test]
fn dock_space_clears_hover_on_drop_without_drag_session() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
        dock.hover = Some(DockDropTarget::Float { window });
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(12.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
    assert!(hover.is_none(), "dock hover should be cleared on drop");
}
#[test]
fn dock_space_kicks_paint_cache_on_drag_transition_for_cache_root() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.set_view_cache_enabled(true);
    ui.set_paint_cache_enabled(true);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_node_view_cache_flags(root, true, false, false);
    ui.set_root(root);

    let mut app = TestHost::new();
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);

    // Frame 0: establish a paint cache entry while no drag is active.
    app.advance_frame();
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);
    let effects = app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(_))),
        "expected no animation-frame requests when no drag is active"
    );

    // Start a cross-window dock drag between frames, without dispatching any events to the dock.
    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.position = Point::new(Px(48.0), Px(22.0));
    }

    // Frame 1: prepaint should kick the paint cache so `DockSpace::paint()` runs and can
    // establish the animation-frame loop.
    app.advance_frame();
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected DockSpace to request animation frames during a dock drag"
    );
    assert_eq!(
        ui.debug_stats().paint_cache_hits,
        0,
        "expected DockSpace paint to run (not replay) on drag transition"
    );
}
#[test]
fn dock_space_paints_float_zone_affordance() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);

    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let rect = float_zone(dock_bounds);

    let has_background = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::Quad { order, rect: r, .. }
                if *order == fret_core::DrawOrder(20) && *r == rect
        )
    });
    assert!(
        has_background,
        "expected a float-zone background quad at {rect:?}"
    );

    let has_glyph = scene.ops().iter().any(|op| match op {
        SceneOp::Text { order, origin, .. } if *order == fret_core::DrawOrder(21) => {
            // `FakeTextService` returns a very wide `TextMetrics.size.width`, so the centered
            // origin can end up outside the float-zone rect. The Y coordinate (baseline origin)
            // still tracks the float-zone placement.
            origin.y.0 >= rect.origin.y.0 && origin.y.0 <= rect.origin.y.0 + rect.size.height.0
        }
        _ => false,
    });
    assert!(
        has_glyph,
        "expected a float-zone glyph draw call at {rect:?}"
    );
}
#[test]
fn render_and_bind_panels_falls_back_to_placeholder_for_missing_ui() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(dock_space);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let panel = PanelKey::new("core.missing");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            panel.clone(),
            DockPanel {
                title: "Missing".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    struct AlwaysMissingRegistry;
    impl DockPanelRegistry<TestHost> for AlwaysMissingRegistry {
        fn render_panel(
            &self,
            _ui: &mut UiTree<TestHost>,
            _app: &mut TestHost,
            _services: &mut dyn UiServices,
            _window: AppWindowId,
            _bounds: Rect,
            _panel: &PanelKey,
        ) -> Option<NodeId> {
            None
        }
    }

    app.with_global_mut(
        DockPanelRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(AlwaysMissingRegistry));
        },
    );

    let mut text = FakeTextService;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(480.0)),
    );
    render_and_bind_dock_panels(&mut ui, &mut app, &mut text, window, bounds, dock_space);

    let service = app
        .global::<DockPanelContentService>()
        .expect("DockPanelContentService should exist after render_and_bind_dock_panels");
    assert!(
        service.get(window, &panel).is_some(),
        "expected a placeholder node for a non-viewport panel with missing UI"
    );
}
