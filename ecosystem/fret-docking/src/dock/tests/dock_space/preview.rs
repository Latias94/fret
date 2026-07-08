use super::*;

#[test]
fn public_declarative_dock_space_entry_point_paints_drag_payload_ghost() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let panel = PanelKey::new("demo.public.declarative.drag.ghost");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Drag Ghost".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: panel.clone(),
            grab_offset: Point::new(Px(8.0), Px(4.0)),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.current_window = window;
        drag.position = Point::new(Px(240.0), Px(120.0));
    }

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-drag-ghost",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, .. } if *order == fret_core::DrawOrder(10_020)
        )),
        "expected public declarative dock host to paint the drag payload ghost, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_paints_center_drop_overlay() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let target_panel = PanelKey::new("demo.public.declarative.drop.target");
    let drag_panel = PanelKey::new("demo.public.declarative.drop.drag");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&target_panel, || crate::DockPanel {
            title: "Target".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&drag_panel, || crate::DockPanel {
            title: "Dragged".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let target_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![target_panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, target_tabs);
        dock.presentation.hover = Some(DockDropTarget::Dock(HoverTarget {
            tabs: target_tabs,
            root: target_tabs,
            leaf_tabs: target_tabs,
            zone: DropZone::Center,
            insert_index: None,
            outer: false,
            explicit: true,
        }));
    });

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: drag_panel,
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.current_window = window;
        drag.position = Point::new(Px(240.0), Px(120.0));
    }

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-center-drop-overlay",
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
    let (tab_bar, content) = split_tab_bar(dock_bounds);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { rect, .. } if *rect == content
        )),
        "expected public declarative dock host to paint the center content drop overlay, got: {:?}",
        scene.ops()
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { rect, .. } if *rect == tab_bar
        )),
        "expected public declarative dock host to paint the center tab-bar drop overlay, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_paints_drop_hint_pads() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let target_panel = PanelKey::new("demo.public.declarative.drop.hints.target");
    let drag_panel = PanelKey::new("demo.public.declarative.drop.hints.drag");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&target_panel, || crate::DockPanel {
            title: "Target".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&drag_panel, || crate::DockPanel {
            title: "Dragged".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let target_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![target_panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, target_tabs);
        dock.presentation.hover = Some(DockDropTarget::Dock(HoverTarget {
            tabs: target_tabs,
            root: target_tabs,
            leaf_tabs: target_tabs,
            zone: DropZone::Left,
            insert_index: None,
            outer: false,
            explicit: true,
        }));
    });

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: drag_panel,
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.current_window = window;
        drag.position = Point::new(Px(240.0), Px(120.0));
    }

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-drop-hints",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, .. } if *order == fret_core::DrawOrder(10_098)
        )),
        "expected public declarative dock host to paint the drop-hint plate, got: {:?}",
        scene.ops()
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, .. } if *order == fret_core::DrawOrder(10_100)
        )),
        "expected public declarative dock host to paint drop-hint pads, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_paints_edge_drop_preview_slot() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );

    let (root_split, target_tabs) = app.with_global_mut(DockManager::default, |dock, _app| {
        let left_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("demo.public.declarative.edge.left")],
            active: 0,
        });
        let right_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("demo.public.declarative.edge.right")],
            active: 0,
        });
        let split = dock.workspace.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left_tabs, right_tabs],
            fractions: vec![0.4, 0.6],
        });
        dock.workspace.graph.set_window_root(window, split);

        for (key, title) in [
            (PanelKey::new("demo.public.declarative.edge.left"), "Left"),
            (PanelKey::new("demo.public.declarative.edge.right"), "Right"),
        ] {
            dock.ensure_panel(&key, || crate::DockPanel {
                title: title.to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }

        dock.presentation.hover = Some(DockDropTarget::Dock(HoverTarget {
            tabs: right_tabs,
            root: split,
            leaf_tabs: right_tabs,
            zone: DropZone::Left,
            insert_index: None,
            outer: false,
            explicit: true,
        }));

        (split, right_tabs)
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let settings = fret_runtime::DockingInteractionSettings::default();
    let expected = {
        let dock = app.global::<DockManager>().expect("expected dock manager");
        let (_chrome, dock_bounds) = dock_space_regions(bounds);
        let layout = compute_layout_map(
            &dock.workspace.graph,
            root_split,
            dock_bounds,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
        );
        let split_bounds = layout
            .get(&root_split)
            .copied()
            .expect("expected split bounds");
        let Some(fret_core::EdgeDockDecision::InsertIntoSplit {
            anchor_index,
            insert_index,
            ..
        }) = dock
            .workspace
            .graph
            .edge_dock_decision(window, target_tabs, DropZone::Left)
        else {
            panic!("expected insert-into-split decision");
        };
        let (axis, children_len, mut next_fractions) = match dock.workspace.graph.node(root_split) {
            Some(DockNode::Split {
                axis,
                children,
                fractions,
            }) => (*axis, children.len(), fractions.clone()),
            _ => panic!("expected split root"),
        };
        let old = *next_fractions
            .get(anchor_index)
            .expect("expected anchor fraction");
        next_fractions[anchor_index] = old * 0.5;
        next_fractions.insert(insert_index.min(next_fractions.len()), old * 0.5);
        let computed = split_geometry::compute_layout(
            axis,
            split_bounds,
            children_len.saturating_add(1),
            &next_fractions,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
            &[],
        );
        computed
            .panel_rects
            .get(insert_index)
            .copied()
            .expect("expected inserted slot rect")
    };

    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-edge-preview",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad {
                order,
                rect,
                ..
            } if *order == fret_core::DrawOrder(10_000) && *rect == expected
        )),
        "expected public declarative dock host to paint edge drop preview slot {expected:?}, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_paints_tab_insert_marker() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![
                PanelKey::new("demo.public.declarative.marker.left"),
                PanelKey::new("demo.public.declarative.marker.right"),
            ],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
        for (key, title) in [
            (PanelKey::new("demo.public.declarative.marker.left"), "Left"),
            (
                PanelKey::new("demo.public.declarative.marker.right"),
                "Right",
            ),
        ] {
            dock.ensure_panel(&key, || crate::DockPanel {
                title: title.to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        dock.presentation.hover = Some(DockDropTarget::Dock(HoverTarget {
            tabs,
            root: tabs,
            leaf_tabs: tabs,
            zone: DropZone::Center,
            insert_index: Some(1),
            outer: false,
            explicit: false,
        }));
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tab-insert-marker",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, .. } if *order == fret_core::DrawOrder(10_000)
        )),
        "expected public declarative dock host to paint a tab insert marker, got: {:?}",
        scene.ops()
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, .. } if *order == fret_core::DrawOrder(10_001)
        )),
        "expected public declarative dock host to paint tab insert marker caps, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_paints_tab_insert_preview_title() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let target_panel = PanelKey::new("demo.public.declarative.preview.target");
    let drag_panel = PanelKey::new("demo.public.declarative.preview.drag");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&target_panel, || crate::DockPanel {
            title: "Target".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&drag_panel, || crate::DockPanel {
            title: "Dragged".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let target_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![target_panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, target_tabs);
        dock.presentation.hover = Some(DockDropTarget::Dock(HoverTarget {
            tabs: target_tabs,
            root: target_tabs,
            leaf_tabs: target_tabs,
            zone: DropZone::Center,
            insert_index: None,
            outer: false,
            explicit: true,
        }));
    });

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: drag_panel,
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.current_window = window;
        drag.position = Point::new(Px(240.0), Px(120.0));
    }

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tab-insert-preview-title",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, .. } if *order == fret_core::DrawOrder(9_995)
        )),
        "expected public declarative dock host to paint the tab insert preview title plate, got: {:?}",
        scene.ops()
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Text { order, .. } if *order == fret_core::DrawOrder(9_996)
        )),
        "expected public declarative dock host to paint the tab insert preview title text, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_handles_focus_requested_panel_command() {
    struct DeclarativePanelRegistry;

    impl DockPanelElementRegistry<TestHost> for DeclarativePanelRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let panel_left = PanelKey::new("demo.public.declarative.focus.left");
    let panel_right = PanelKey::new("demo.public.declarative.focus.right");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(DeclarativePanelRegistry));
        },
    );

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_left, || crate::DockPanel {
            title: "Left".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&panel_right, || crate::DockPanel {
            title: "Right".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_left.clone(), panel_right.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-focus-request",
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

    let right_node = app
        .global::<DockPanelContentService>()
        .and_then(|svc| svc.get(window, &panel_right))
        .expect("expected right panel node to be bound");
    assert_ne!(ui.focus(), Some(right_node));

    assert!(DockManager::request_activate_panel(
        &mut app,
        window,
        [window],
        panel_right.clone(),
        crate::dock::manager::ActivatePanelOptions { focus: true },
    ));

    for effect in app.take_effects() {
        match effect {
            fret_runtime::Effect::Dock(op) => {
                assert!(crate::runtime::handle_dock_op(&mut app, op));
            }
            fret_runtime::Effect::Command {
                window: Some(effect_window),
                command,
            } => {
                assert_eq!(effect_window, window);
                assert!(ui.dispatch_command(&mut app, &mut services, &command));
            }
            other => panic!("unexpected activation effect: {other:?}"),
        }
    }

    assert_eq!(
        ui.focus(),
        Some(right_node),
        "declarative dock host should focus the requested panel root"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_installs_internal_drag_route_anchor() {
    struct DeclarativePanelRegistry;

    impl DockPanelElementRegistry<TestHost> for DeclarativePanelRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let panel = PanelKey::new("demo.public.declarative.route");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(DeclarativePanelRegistry));
        },
    );
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Route".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-route-anchor",
        move |cx| {
            vec![dock_space_element_from_registry(
                cx,
                window,
                DockSpaceElementOptions::default(),
            )]
        },
    );
    ui.set_root(root);
    let dock_host = ui.children(root)[0];

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        app.global::<DockManager>()
            .and_then(|dock| dock.dock_space_node(window)),
        Some(dock_host),
        "expected declarative dock host to register as the window dock-space node"
    );
    assert_eq!(
        fret_ui::internal_drag::route(&app, window, DRAG_KIND_DOCK_PANEL),
        Some(dock_host),
        "expected declarative dock host to install the dock-panel route during layout"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        fret_ui::internal_drag::route(&app, window, fret_runtime::DRAG_KIND_DOCK_TABS),
        Some(dock_host),
        "expected declarative dock host to refresh the dock-tabs route during paint/prepaint"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_clears_hover_on_internal_drag_drop_without_drag_session()
 {
    struct DeclarativePanelRegistry;

    impl DockPanelElementRegistry<TestHost> for DeclarativePanelRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let panel = PanelKey::new("demo.public.declarative.internal-drag-drop");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(DeclarativePanelRegistry));
        },
    );
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Drop".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
        dock.presentation.hover = Some(DockDropTarget::Float { window });
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-internal-drag-drop",
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

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(12.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let hover = app
        .global::<DockManager>()
        .and_then(|dock| dock.presentation.hover.clone());
    assert!(
        hover.is_none(),
        "declarative dock host should clear stale hover on internal drag drop, got: {hover:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_resolves_internal_drag_over_outer_hint_rect() {
    struct DeclarativePanelRegistry;

    impl DockPanelElementRegistry<TestHost> for DeclarativePanelRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let left_panel = PanelKey::new("demo.public.declarative.internal-drag.left");
    let right_panel = PanelKey::new("demo.public.declarative.internal-drag.right");
    let drag_panel = left_panel.clone();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.set_global(fret_runtime::WindowInteractionDiagnosticsStore::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(DeclarativePanelRegistry));
        },
    );
    let root_split = app.with_global_mut(DockManager::default, |dock, _app| {
        for (panel, title) in [(&left_panel, "Left"), (&right_panel, "Right")] {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: title.to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let left = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![left_panel.clone()],
            active: 0,
        });
        let right = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![right_panel.clone()],
            active: 0,
        });
        let split = dock.workspace.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left, right],
            fractions: vec![0.5, 0.5],
        });
        dock.workspace.graph.set_window_root(window, split);
        split
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-internal-drag-over",
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

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: drag_panel,
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    let outer_left = dock_hint_rects_with_font(bounds, Px(13.0), true)
        .into_iter()
        .find_map(|(zone, rect)| (zone == DropZone::Left).then_some(rect))
        .expect("expected outer left hint rect");
    let position = Point::new(
        Px(outer_left.origin.x.0 + outer_left.size.width.0 * 0.5),
        Px(outer_left.origin.y.0 + outer_left.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::InternalDrag(InternalDragEvent {
            position,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let hover = app
        .global::<DockManager>()
        .and_then(|dock| dock.presentation.hover.clone());
    assert!(
        matches!(
            hover,
            Some(DockDropTarget::Dock(HoverTarget {
                tabs,
                zone: DropZone::Left,
                outer: true,
                ..
            })) if tabs == root_split
        ),
        "expected declarative internal-drag over to target the root split outer-left hint, got: {hover:?}"
    );

    let docking = app
        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
        .and_then(|store| store.docking_latest_for_window(window))
        .expect("expected resolved drop transaction diagnostics");
    let drop_resolve = docking
        .dock_drop_resolve
        .as_ref()
        .expect("expected dock drop resolve diagnostics");
    assert_eq!(
        drop_resolve.source,
        fret_runtime::DockDropResolveSource::OuterHintRect
    );
    assert_eq!(
        drop_resolve.command,
        fret_runtime::DockDropCommandKindDiagnostics::MovePanel
    );
    assert_eq!(
        drop_resolve.policy,
        fret_runtime::DockDropPolicyDecisionDiagnostics::Allowed
    );
    assert!(drop_resolve.commit_capable);
    assert!(drop_resolve.clears_hover);
    assert!(drop_resolve.invalidates_layout);
    assert_eq!(
        drop_resolve.resolved.as_ref().map(|target| target.zone),
        Some(DropZone::Left)
    );
    assert!(drop_resolve.denied.is_none());
}

#[test]
fn public_declarative_dock_space_entry_point_drops_panel_on_inner_left_hint_rect() {
    struct DeclarativePanelRegistry;

    impl DockPanelElementRegistry<TestHost> for DeclarativePanelRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            Some(
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let left_panel = PanelKey::new("demo.public.declarative.drop.left");
    let right_panel = PanelKey::new("demo.public.declarative.drop.right");
    let drag_panel = left_panel.clone();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(DeclarativePanelRegistry));
        },
    );
    let tabs_node = app.with_global_mut(DockManager::default, |dock, _app| {
        for (panel, title) in [(&left_panel, "Left"), (&right_panel, "Right")] {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: title.to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![left_panel.clone(), right_panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
        tabs
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-drop-panel-left",
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
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let _ = app.take_effects();

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: drag_panel.clone(),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let left_rect = dock_hint_rects_with_font(dock_bounds, Px(13.0), false)
        .into_iter()
        .find_map(|(zone, rect)| (zone == DropZone::Left).then_some(rect))
        .expect("expected inner left rect");
    let position = Point::new(
        Px(left_rect.origin.x.0 + left_rect.size.width.0 * 0.5),
        Px(left_rect.origin.y.0 + left_rect.size.height.0 * 0.5),
    );

    for kind in [InternalDragKind::Over, InternalDragKind::Drop] {
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
    }

    assert!(
        app.drag(fret_core::PointerId(0)).is_none(),
        "expected declarative dock host to end the active dock drag session on drop"
    );

    let effects = app.take_effects();
    let op = effects.iter().find_map(|effect| match effect {
        Effect::Dock(op) => Some(op.clone()),
        _ => None,
    });
    let Some(op) = op else {
        panic!("expected declarative dock host to emit a Dock op, got: {effects:?}");
    };
    let DockOp::MovePanel {
        target_tabs,
        zone,
        panel,
        ..
    } = &op
    else {
        panic!("expected declarative dock host to emit MovePanel, got: {op:?}");
    };
    assert_eq!(*target_tabs, tabs_node);
    assert_eq!(*zone, DropZone::Left);
    assert_eq!(*panel, drag_panel);

    app.with_global_mut(DockManager::default, |dock, _app| {
        let applied = dock
            .workspace
            .graph
            .apply_op_checked(&op)
            .expect("apply must succeed");
        assert!(applied);

        let root = dock
            .workspace
            .graph
            .window_root(window)
            .expect("window root exists");
        let Some(DockNode::Split { axis, children, .. }) = dock.workspace.graph.node(root) else {
            panic!(
                "expected root to become a split after declarative left docking, got: {:?}",
                dock.workspace.graph.node(root)
            );
        };
        assert_eq!(*axis, fret_core::Axis::Horizontal);
        assert_eq!(children.len(), 2);

        let left = children[0];
        let right = children[1];
        let Some(DockNode::Tabs { tabs, .. }) = dock.workspace.graph.node(left) else {
            panic!("expected left child tabs");
        };
        let Some(DockNode::Tabs {
            tabs: right_tabs, ..
        }) = dock.workspace.graph.node(right)
        else {
            panic!("expected right child tabs");
        };
        assert!(tabs.contains(&left_panel));
        assert!(right_tabs.contains(&right_panel));
    });
}
