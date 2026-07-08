use super::*;

#[test]
fn public_declarative_dock_space_entry_point_starts_tab_drag_after_threshold() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            _cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            None
        }
    }

    let window = AppWindowId::default();
    let panel_0 = PanelKey::new("demo.public.declarative.tab-drag.0");
    let panel_1 = PanelKey::new("demo.public.declarative.tab-drag.1");
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(fret_runtime::DockingInteractionSettings {
        tab_drag_threshold: Px(4.0),
        ..Default::default()
    });
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    app.with_global_mut(DockManager::default, |dock, _app| {
        for (index, panel) in [panel_0.clone(), panel_1.clone()].iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("Drag {index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_0.clone(), panel_1],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(160.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tab-drag",
        move |cx| {
            vec![dock_space_element_from_registry(
                cx,
                window,
                DockSpaceElementOptions::default(),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let tab_rect =
        crate::dock::tab_bar_geometry::TabBarGeometry::fixed(tab_bar, 2).tab_rect(0, Px(0.0));
    let start = Point::new(
        Px(tab_rect.origin.x.0 + 20.0),
        Px(tab_rect.origin.y.0 + tab_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = app.take_effects();

    let move_pos = Point::new(Px(start.x.0 + 12.0), start.y);
    ui.dispatch_event(
        &mut app,
        &mut services,
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

    let drag = app
        .drag(fret_core::PointerId(0))
        .and_then(|drag| {
            drag.payload::<DockPanelDragPayload>()
                .map(|payload| (drag, payload))
        })
        .expect("expected declarative dock host to activate a dock panel drag");
    assert!(drag.0.dragging, "expected drag session to be active");
    assert_eq!(drag.0.kind, DRAG_KIND_DOCK_PANEL);
    assert_eq!(drag.0.source_window, window);
    assert_eq!(drag.0.position, move_pos);
    assert_eq!(
        drag.0.cursor_grab_offset,
        Some(Point::new(Px(20.0), Px(14.0)))
    );
    assert_eq!(drag.1.panel, panel_0);
    assert_eq!(drag.1.grab_offset, Point::new(Px(20.0), Px(14.0)));
    assert!(
        drag.1.dock_previews_enabled,
        "expected default drag inversion policy to enable dock previews"
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "declarative host should release pointer capture once the runtime drag session starts"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_respects_tab_drag_threshold() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            _cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            None
        }
    }

    let window = AppWindowId::default();
    let panel = PanelKey::new("demo.public.declarative.tab-drag-threshold");
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(fret_runtime::DockingInteractionSettings {
        tab_drag_threshold: Px(100.0),
        ..Default::default()
    });
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Threshold".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(160.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tab-drag-threshold",
        move |cx| {
            vec![dock_space_element_from_registry(
                cx,
                window,
                DockSpaceElementOptions::default(),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let tab_rect =
        crate::dock::tab_bar_geometry::TabBarGeometry::fixed(tab_bar, 1).tab_rect(0, Px(0.0));
    let start = Point::new(
        Px(tab_rect.origin.x.0 + 20.0),
        Px(tab_rect.origin.y.0 + tab_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(start.x.0 + 12.0), start.y),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        app.drag(fret_core::PointerId(0)).is_none(),
        "expected declarative tab drag to remain pending below the configured threshold"
    );
    assert!(
        ui.captured_for(fret_core::PointerId(0)).is_some(),
        "expected pending declarative tab drag to retain pointer capture"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_respects_panel_drag_policy() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            _cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            None
        }
    }

    struct DisallowDrag;

    impl DockingPolicy for DisallowDrag {
        fn allow_panel_drag(
            &self,
            _window: AppWindowId,
            _panel: &PanelKey,
            _info: Option<&DockPanel>,
        ) -> bool {
            false
        }
    }

    let window = AppWindowId::default();
    let panel = PanelKey::new("demo.public.declarative.tab-drag-policy");
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(fret_runtime::DockingInteractionSettings {
        tab_drag_threshold: Px(0.0),
        ..Default::default()
    });
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    app.with_global_mut(DockingPolicyService::default, |svc, _app| {
        svc.set(Arc::new(DisallowDrag));
    });
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Policy".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(160.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tab-drag-policy",
        move |cx| {
            vec![dock_space_element_from_registry(
                cx,
                window,
                DockSpaceElementOptions::default(),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let tab_rect =
        crate::dock::tab_bar_geometry::TabBarGeometry::fixed(tab_bar, 1).tab_rect(0, Px(0.0));
    let start = Point::new(
        Px(tab_rect.origin.x.0 + 20.0),
        Px(tab_rect.origin.y.0 + tab_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(start.x.0 + 12.0), start.y),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        app.drag(fret_core::PointerId(0)).is_none(),
        "expected declarative tab drag to respect DockingPolicy::allow_panel_drag"
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "policy-disabled declarative tab drag should not capture the pointer"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_starts_tabs_group_drag_after_threshold() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            _cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            None
        }
    }

    let window = AppWindowId::default();
    let panel_0 = PanelKey::new("demo.public.declarative.tabs-drag.0");
    let panel_1 = PanelKey::new("demo.public.declarative.tabs-drag.1");
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(fret_runtime::DockingInteractionSettings {
        tab_drag_threshold: Px(4.0),
        ..Default::default()
    });
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    let tabs_node = app.with_global_mut(DockManager::default, |dock, _app| {
        for (index, panel) in [panel_0.clone(), panel_1.clone()].iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("G{index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_0.clone(), panel_1.clone()],
            active: 1,
        });
        dock.workspace.graph.set_window_root(window, tabs);
        tabs
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(160.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tabs-group-drag",
        move |cx| {
            vec![dock_space_element_from_registry(
                cx,
                window,
                DockSpaceElementOptions::default(),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let start = Point::new(
        Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - 32.0),
        Px(tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = app.take_effects();

    let move_pos = Point::new(Px(start.x.0 + 12.0), start.y);
    ui.dispatch_event(
        &mut app,
        &mut services,
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

    let drag = app
        .drag(fret_core::PointerId(0))
        .and_then(|drag| {
            drag.payload::<DockTabsDragPayload>()
                .map(|payload| (drag, payload))
        })
        .expect("expected declarative dock host to activate a dock tabs drag");
    assert!(drag.0.dragging, "expected drag session to be active");
    assert_eq!(drag.0.kind, DRAG_KIND_DOCK_TABS);
    assert_eq!(drag.0.source_window, window);
    assert_eq!(drag.0.position, move_pos);
    assert_eq!(
        drag.0.cursor_grab_offset,
        Some(Point::new(Px(388.0), Px(14.0)))
    );
    assert_eq!(drag.1.source_tabs, tabs_node);
    assert_eq!(drag.1.tabs, vec![panel_0, panel_1]);
    assert_eq!(drag.1.active, 1);
    assert_eq!(drag.1.grab_offset, Point::new(Px(388.0), Px(14.0)));
    assert!(
        drag.1.dock_previews_enabled,
        "expected default drag inversion policy to enable dock previews"
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "declarative host should release pointer capture once the runtime tabs drag starts"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_respects_tabs_group_drag_policy() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            _cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            None
        }
    }

    struct DisallowTabsGroupDrag;

    impl DockingPolicy for DisallowTabsGroupDrag {
        fn allow_tabs_group_drag(&self, _window: AppWindowId, _tabs: DockNodeId) -> bool {
            false
        }
    }

    let window = AppWindowId::default();
    let panel = PanelKey::new("demo.public.declarative.tabs-drag-policy");
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(fret_runtime::DockingInteractionSettings {
        tab_drag_threshold: Px(0.0),
        ..Default::default()
    });
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    app.with_global_mut(DockingPolicyService::default, |svc, _app| {
        svc.set(Arc::new(DisallowTabsGroupDrag));
    });
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Policy".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(160.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tabs-group-drag-policy",
        move |cx| {
            vec![dock_space_element_from_registry(
                cx,
                window,
                DockSpaceElementOptions::default(),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let start = Point::new(
        Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - 32.0),
        Px(tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(start.x.0 + 12.0), start.y),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        app.drag(fret_core::PointerId(0)).is_none(),
        "expected declarative tabs drag to respect DockingPolicy::allow_tabs_group_drag"
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "policy-disabled declarative tabs drag should not capture the pointer"
    );
}
