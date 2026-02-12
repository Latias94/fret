use super::*;

#[test]
fn dock_drop_hint_rects_can_select_zone() {
    let window = AppWindowId::default();

    let mut dock = DockManager::default();
    let tabs = dock.graph.insert_node(DockNode::Tabs {
        tabs: vec![PanelKey::new("core.hierarchy")],
        active: 0,
    });
    dock.graph.set_window_root(window, tabs);

    let rect = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut layout = std::collections::HashMap::new();
    layout.insert(tabs, rect);
    let tab_scroll = std::collections::HashMap::new();

    for (expected, hint_rect) in dock_hint_rects_with_font(rect, Px(13.0), false) {
        if expected == DropZone::Center {
            continue;
        }
        let position = Point::new(
            Px(hint_rect.origin.x.0 + hint_rect.size.width.0 * 0.5),
            Px(hint_rect.origin.y.0 + hint_rect.size.height.0 * 0.5),
        );
        let hit = hit_test_drop_target(&dock.graph, &layout, &tab_scroll, position)
            .expect("hit should resolve to a dock target");
        assert_eq!(hit.zone, expected);
        assert!(hit.insert_index.is_none());
    }
}
#[test]
fn dock_outer_drop_rects_target_window_root_even_when_root_is_split() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut root_split: Option<DockNodeId> = None;
    app.with_global_mut(DockManager::default, |dock, _app| {
        let left = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.left")],
            active: 0,
        });
        let right = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.right")],
            active: 0,
        });
        let split = dock.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left, right],
            fractions: vec![0.5, 0.5],
        });
        root_split = Some(split);
        dock.graph.set_window_root(window, split);
        for (key, title) in [
            (PanelKey::new("core.left"), "Left"),
            (PanelKey::new("core.right"), "Right"),
        ] {
            dock.panels.insert(
                key,
                DockPanel {
                    title: title.to_string(),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );
        }
    });
    let root_split = root_split.expect("expected window root split");

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.left"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    let root_rect = bounds;
    let outer_left = dock_hint_rects_with_font(root_rect, Px(13.0), true)
        .into_iter()
        .find_map(|(zone, rect)| (zone == DropZone::Left).then_some(rect))
        .expect("expected outer left rect");
    let position = Point::new(
        Px(outer_left.origin.x.0 + outer_left.size.width.0 * 0.5),
        Px(outer_left.origin.y.0 + outer_left.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
    assert!(
        matches!(hover, Some(DockDropTarget::Dock(t)) if t.tabs == root_split && t.zone == DropZone::Left && t.outer),
        "expected outer docking to target window root split, got: {hover:?}",
    );
}
#[test]
fn dock_drop_left_emits_move_panel_and_splits_tabs_node() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut tabs_node: Option<DockNodeId> = None;
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.left"), PanelKey::new("core.right")],
            active: 0,
        });
        tabs_node = Some(tabs);
        dock.graph.set_window_root(window, tabs);
        for (key, title) in [
            (PanelKey::new("core.left"), "Left"),
            (PanelKey::new("core.right"), "Right"),
        ] {
            dock.panels.insert(
                key,
                DockPanel {
                    title: title.to_string(),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );
        }
    });
    let tabs_node = tabs_node.expect("expected tabs node");

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.left"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
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
            &mut text,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
    }

    let effects = app.take_effects();
    let op = effects.iter().find_map(|e| match e {
        Effect::Dock(op) => Some(op.clone()),
        _ => None,
    });
    let Some(op) = op else {
        panic!("expected a Dock op, got: {effects:?}");
    };
    let DockOp::MovePanel {
        target_tabs,
        zone,
        panel,
        ..
    } = &op
    else {
        panic!("expected MovePanel, got: {op:?}");
    };
    assert_eq!(*target_tabs, tabs_node);
    assert_eq!(*zone, DropZone::Left);
    assert_eq!(*panel, PanelKey::new("core.left"));

    app.with_global_mut(DockManager::default, |dock, _app| {
        let applied = dock
            .graph
            .apply_op_checked(&op)
            .expect("apply must succeed");
        assert!(applied);

        let root = dock.graph.window_root(window).expect("window root exists");
        let Some(DockNode::Split { axis, children, .. }) = dock.graph.node(root) else {
            panic!(
                "expected root to become a split after left docking, got: {:?}",
                dock.graph.node(root)
            );
        };
        assert_eq!(*axis, fret_core::Axis::Horizontal);
        assert_eq!(children.len(), 2);

        let left = children[0];
        let right = children[1];
        let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(left) else {
            panic!("expected left child tabs");
        };
        let Some(DockNode::Tabs {
            tabs: right_tabs, ..
        }) = dock.graph.node(right)
        else {
            panic!("expected right child tabs");
        };
        assert!(tabs.contains(&PanelKey::new("core.left")));
        assert!(right_tabs.contains(&PanelKey::new("core.right")));
    });
}
#[test]
fn dock_drop_outer_left_emits_move_panel_and_wraps_window_root() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut root_split: Option<DockNodeId> = None;
    app.with_global_mut(DockManager::default, |dock, _app| {
        let left = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.left")],
            active: 0,
        });
        let right = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.right"), PanelKey::new("core.right2")],
            active: 0,
        });
        let split = dock.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left, right],
            fractions: vec![0.5, 0.5],
        });
        root_split = Some(split);
        dock.graph.set_window_root(window, split);
        for (key, title) in [
            (PanelKey::new("core.left"), "Left"),
            (PanelKey::new("core.right"), "Right"),
            (PanelKey::new("core.right2"), "Right2"),
        ] {
            dock.panels.insert(
                key,
                DockPanel {
                    title: title.to_string(),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );
        }
    });
    let root_split = root_split.expect("expected window root split");

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.right"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
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
        .expect("expected outer left rect");
    let position = Point::new(
        Px(outer_left.origin.x.0 + outer_left.size.width.0 * 0.5),
        Px(outer_left.origin.y.0 + outer_left.size.height.0 * 0.5),
    );

    for kind in [InternalDragKind::Over, InternalDragKind::Drop] {
        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
    }

    let effects = app.take_effects();
    let op = effects.iter().find_map(|e| match e {
        Effect::Dock(op) => Some(op.clone()),
        _ => None,
    });
    let Some(op) = op else {
        panic!("expected a Dock op, got: {effects:?}");
    };
    let DockOp::MovePanel {
        target_tabs,
        zone,
        panel,
        ..
    } = &op
    else {
        panic!("expected MovePanel, got: {op:?}");
    };
    assert_eq!(*target_tabs, root_split);
    assert_eq!(*zone, DropZone::Left);
    assert_eq!(*panel, PanelKey::new("core.right"));

    app.with_global_mut(DockManager::default, |dock, _app| {
        let applied = dock
            .graph
            .apply_op_checked(&op)
            .expect("apply must succeed");
        assert!(applied);

        let root = dock.graph.window_root(window).expect("window root exists");
        let Some(DockNode::Split { axis, children, .. }) = dock.graph.node(root) else {
            panic!("expected window root split");
        };
        assert_eq!(*axis, fret_core::Axis::Horizontal);
        assert_eq!(children.len(), 2);

        let left = children[0];
        let right = children[1];
        let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(left) else {
            panic!("expected left child tabs");
        };
        assert!(tabs.contains(&PanelKey::new("core.right")));

        // The previous root split should become the right child.
        assert_eq!(right, root_split);
        fn collect_panels(graph: &DockGraph, node: DockNodeId, out: &mut Vec<PanelKey>) {
            let Some(node) = graph.node(node) else {
                return;
            };
            match node {
                DockNode::Tabs { tabs, .. } => out.extend(tabs.iter().cloned()),
                DockNode::Split { children, .. } => {
                    for &child in children {
                        collect_panels(graph, child, out);
                    }
                }
                DockNode::Floating { child } => collect_panels(graph, *child, out),
            }
        }

        let mut subtree_panels = Vec::new();
        collect_panels(&dock.graph, right, &mut subtree_panels);
        assert!(subtree_panels.contains(&PanelKey::new("core.left")));
        assert!(subtree_panels.contains(&PanelKey::new("core.right2")));
    });
}
#[test]
fn dock_center_drop_overlay_excludes_tab_bar() {
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
        dock.hover = Some(DockDropTarget::Dock(HoverTarget {
            tabs,
            root: tabs,
            leaf_tabs: tabs,
            zone: DropZone::Center,
            insert_index: None,
            outer: false,
            explicit: true,
        }));
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, content) = split_tab_bar(dock_bounds);

    let has_tab_quad = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::Quad { rect, .. } if *rect == tab_bar
        )
    });
    assert!(has_tab_quad, "expected a tab-bar overlay quad");

    let has_content_quad = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::Quad { rect, .. } if *rect == content
        )
    });
    assert!(
        has_content_quad,
        "expected center drop overlay quad to cover content rect (excluding tab bar)"
    );
}
#[test]
fn dock_center_drop_overlay_draws_tab_preview_for_drag_payload() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let target_tabs = app.with_global_mut(DockManager::default, |dock, _app| {
        let target_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("target.panel")],
            active: 0,
        });
        dock.graph.set_window_root(window, target_tabs);
        dock.panels.insert(
            PanelKey::new("target.panel"),
            DockPanel {
                title: "Target".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
        dock.panels.insert(
            PanelKey::new("drag.panel"),
            DockPanel {
                title: "Dragged".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
        dock.hover = Some(DockDropTarget::Dock(HoverTarget {
            tabs: target_tabs,
            root: target_tabs,
            leaf_tabs: target_tabs,
            zone: DropZone::Center,
            insert_index: None,
            outer: false,
            explicit: true,
        }));
        target_tabs
    });

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("drag.panel"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.current_window = window;
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, .. } if *order == fret_core::DrawOrder(9_995)
        )),
        "expected a tab preview quad when hovering center while dragging",
    );

    let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
    assert!(
        matches!(hover, Some(DockDropTarget::Dock(t)) if t.tabs == target_tabs),
        "expected hover to remain a dock target, got: {hover:?}",
    );
}
