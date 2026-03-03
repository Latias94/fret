use super::*;

#[test]
fn dock_tab_bar_insert_index_respects_before_after_halves() {
    let window = AppWindowId::default();

    let mut dock = DockManager::default();
    let tabs = dock.graph.insert_node(DockNode::Tabs {
        tabs: vec![
            PanelKey::new("core.a"),
            PanelKey::new("core.b"),
            PanelKey::new("core.c"),
        ],
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

    let (tab_bar, _content) = split_tab_bar(rect);
    let scroll = Px(0.0);

    let tab_b = TabBarGeometry::fixed(tab_bar, 3).tab_rect(1, scroll);
    let y = Px(tab_b.origin.y.0 + tab_b.size.height.0 * 0.5);

    let left_half = Point::new(Px(tab_b.origin.x.0 + tab_b.size.width.0 * 0.25), y);
    let hit_left = hit_test_drop_target(&dock.graph, &layout, &tab_scroll, left_half)
        .expect("hit should resolve to a dock target");
    assert_eq!(hit_left.tabs, tabs);
    assert_eq!(hit_left.zone, DropZone::Center);
    assert_eq!(hit_left.insert_index, Some(1));

    let right_half = Point::new(Px(tab_b.origin.x.0 + tab_b.size.width.0 * 0.75), y);
    let hit_right = hit_test_drop_target(&dock.graph, &layout, &tab_scroll, right_half)
        .expect("hit should resolve to a dock target");
    assert_eq!(hit_right.tabs, tabs);
    assert_eq!(hit_right.zone, DropZone::Center);
    assert_eq!(hit_right.insert_index, Some(2));

    let far_right = Point::new(Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - 1.0), y);
    let hit_end = hit_test_drop_target(&dock.graph, &layout, &tab_scroll, far_right)
        .expect("hit should resolve to a dock target");
    assert_eq!(hit_end.tabs, tabs);
    assert_eq!(hit_end.zone, DropZone::Center);
    assert_eq!(hit_end.insert_index, Some(3));
}
#[test]
fn dock_tab_drop_emits_insert_index_based_on_over_tab_halves() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let panel_a = PanelKey::new("core.a");
    let panel_b = PanelKey::new("core.b");
    let panel_c = PanelKey::new("core.c");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let tabs = app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone(), panel_c.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        for panel in [&panel_a, &panel_b, &panel_c] {
            dock.panels.insert(
                panel.clone(),
                DockPanel {
                    title: "Panel".to_string(),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );
        }
        tabs
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let scroll = Px(0.0);

    let over_rect = TabBarGeometry::fixed(tab_bar, 3).tab_rect(1, scroll);
    let y = Px(over_rect.origin.y.0 + over_rect.size.height.0 * 0.5);

    let check_drop = |app: &mut TestHost,
                      ui: &mut UiTree<TestHost>,
                      position: Point,
                      expect: usize| {
        app.begin_cross_window_drag_with_kind(
            fret_core::PointerId(0),
            DRAG_KIND_DOCK_PANEL,
            window,
            Point::new(Px(24.0), Px(12.0)),
            DockPanelDragPayload {
                panel: panel_a.clone(),
                grab_offset: Point::new(Px(0.0), Px(0.0)),
                start_tick: fret_runtime::TickId(0),
                tear_off_requested: false,
                tear_off_requested_at_tick: None,
                tear_off_oob_start_frame: None,
                dock_previews_enabled: true,
            },
        );
        if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
            drag.dragging = true;
        }

        let mut services = FakeTextService;
        ui.dispatch_event(
            app,
            &mut services,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
        ui.dispatch_event(
            app,
            &mut services,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind: InternalDragKind::Drop,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );

        let effects = app.take_effects();
        let moves: Vec<_> = effects
            .iter()
            .filter_map(|e| match e {
                Effect::Dock(DockOp::MovePanel {
                    panel,
                    target_tabs,
                    zone,
                    insert_index,
                    ..
                }) if panel == &panel_a && *target_tabs == tabs && *zone == DropZone::Center => {
                    Some(*insert_index)
                }
                _ => None,
            })
            .collect();
        assert_eq!(moves, vec![Some(expect)]);
    };

    let left_half = Point::new(Px(over_rect.origin.x.0 + over_rect.size.width.0 * 0.25), y);
    check_drop(&mut app, &mut ui, left_half, 1);

    let right_half = Point::new(Px(over_rect.origin.x.0 + over_rect.size.width.0 * 0.75), y);
    check_drop(&mut app, &mut ui, right_half, 2);
}
#[test]
fn dock_tab_drop_reorders_tabs_when_applying_move_panel() {
    fn run(position_in_tab_bar: Point) -> Vec<PanelKey> {
        let window = AppWindowId::default();

        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node_retained(DockSpace::new(window));
        ui.set_root(root);

        let panel_a = PanelKey::new("core.a");
        let panel_b = PanelKey::new("core.b");
        let panel_c = PanelKey::new("core.c");
        let panel_d = PanelKey::new("core.d");

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let tabs = app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![
                    panel_a.clone(),
                    panel_b.clone(),
                    panel_c.clone(),
                    panel_d.clone(),
                ],
                active: 0,
            });
            dock.graph.set_window_root(window, tabs);
            for panel in [&panel_a, &panel_b, &panel_c, &panel_d] {
                dock.panels.insert(
                    panel.clone(),
                    DockPanel {
                        title: "Panel".to_string(),
                        color: Color::TRANSPARENT,
                        viewport: None,
                    },
                );
            }
            tabs
        });

        let mut text = FakeTextService;
        let size = Size::new(Px(800.0), Px(600.0));
        let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

        app.begin_cross_window_drag_with_kind(
            fret_core::PointerId(0),
            DRAG_KIND_DOCK_PANEL,
            window,
            Point::new(Px(24.0), Px(12.0)),
            DockPanelDragPayload {
                panel: panel_d.clone(),
                grab_offset: Point::new(Px(0.0), Px(0.0)),
                start_tick: fret_runtime::TickId(0),
                tear_off_requested: false,
                tear_off_requested_at_tick: None,
                tear_off_oob_start_frame: None,
                dock_previews_enabled: true,
            },
        );
        if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
            drag.dragging = true;
        }

        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::InternalDrag(InternalDragEvent {
                position: position_in_tab_bar,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::InternalDrag(InternalDragEvent {
                position: position_in_tab_bar,
                kind: InternalDragKind::Drop,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );

        let effects = app.take_effects();
        let op = effects
            .iter()
            .find_map(|e| match e {
                Effect::Dock(DockOp::MovePanel {
                    panel,
                    target_tabs,
                    zone,
                    insert_index,
                    ..
                }) if panel == &panel_d && *target_tabs == tabs && *zone == DropZone::Center => {
                    Some(DockOp::MovePanel {
                        source_window: window,
                        panel: panel.clone(),
                        target_window: window,
                        target_tabs: *target_tabs,
                        zone: *zone,
                        insert_index: *insert_index,
                    })
                }
                _ => None,
            })
            .expect("expected a MovePanel op for the drop");

        app.with_global_mut(DockManager::default, |dock, _app| {
            assert!(dock.graph.apply_op(&op));
            match dock.graph.node(tabs) {
                Some(DockNode::Tabs { tabs, .. }) => tabs.clone(),
                other => panic!("expected tabs node, got {other:?}"),
            }
        })
    }

    let rect = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let (_chrome, dock_bounds) = dock_space_regions(rect);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let scroll = Px(0.0);

    let over_rect = TabBarGeometry::fixed(tab_bar, 3).tab_rect(1, scroll);
    let y = Px(over_rect.origin.y.0 + over_rect.size.height.0 * 0.5);

    let left_half = Point::new(Px(over_rect.origin.x.0 + over_rect.size.width.0 * 0.25), y);
    assert_eq!(
        run(left_half),
        vec![
            PanelKey::new("core.a"),
            PanelKey::new("core.d"),
            PanelKey::new("core.b"),
            PanelKey::new("core.c"),
        ]
    );

    let right_half = Point::new(Px(over_rect.origin.x.0 + over_rect.size.width.0 * 0.75), y);
    assert_eq!(
        run(right_half),
        vec![
            PanelKey::new("core.a"),
            PanelKey::new("core.b"),
            PanelKey::new("core.d"),
            PanelKey::new("core.c"),
        ]
    );
}

#[test]
fn dock_tab_drop_across_panes_end_inserts_at_target_end() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let panel_a = PanelKey::new("core.a");
    let panel_b = PanelKey::new("core.b");
    let panel_c = PanelKey::new("core.c");
    let panel_d = PanelKey::new("core.d");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(fret_runtime::DockingInteractionSettings::default());

    let (source_tabs, target_tabs) = app.with_global_mut(DockManager::default, |dock, _app| {
        let source_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone()],
            active: 0,
        });
        let target_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_c.clone(), panel_d.clone()],
            active: 0,
        });
        let split = dock.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![source_tabs, target_tabs],
            fractions: vec![0.5, 0.5],
        });
        dock.graph.set_window_root(window, split);

        for panel in [&panel_a, &panel_b, &panel_c, &panel_d] {
            dock.panels.insert(
                panel.clone(),
                DockPanel {
                    title: "Panel".to_string(),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );
        }

        (source_tabs, target_tabs)
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    let settings = fret_runtime::DockingInteractionSettings::default();
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (split_root, layout) = app.with_global_mut(DockManager::default, |dock, _app| {
        let split_root = dock
            .graph
            .window_root(window)
            .expect("expected window root");
        let layout = compute_layout_map(
            &dock.graph,
            split_root,
            dock_bounds,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
        );
        (split_root, layout)
    });
    assert_ne!(
        split_root, source_tabs,
        "expected split root for cross-pane test"
    );

    let target_bounds = layout
        .get(&target_tabs)
        .copied()
        .expect("expected target tabs bounds");
    let (target_tab_bar, _content) = split_tab_bar(target_bounds);
    let drop_pos = Point::new(
        Px(target_tab_bar.origin.x.0 + target_tab_bar.size.width.0 - 1.0),
        Px(target_tab_bar.origin.y.0 + target_tab_bar.size.height.0 * 0.5),
    );

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: panel_b.clone(),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: drop_pos,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: drop_pos,
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let effects = app.take_effects();
    let op = effects
        .iter()
        .find_map(|e| match e {
            Effect::Dock(DockOp::MovePanel {
                source_window,
                panel,
                target_window,
                target_tabs: effect_target_tabs,
                zone,
                insert_index,
                ..
            }) if *source_window == window
                && panel == &panel_b
                && *target_window == window
                && *effect_target_tabs == target_tabs
                && *zone == DropZone::Center =>
            {
                Some(DockOp::MovePanel {
                    source_window: *source_window,
                    panel: panel.clone(),
                    target_window: *target_window,
                    target_tabs: *effect_target_tabs,
                    zone: *zone,
                    insert_index: *insert_index,
                })
            }
            _ => None,
        })
        .expect("expected a MovePanel op for the drop");

    match &op {
        DockOp::MovePanel { insert_index, .. } => assert_eq!(*insert_index, Some(2)),
        other => panic!("expected MovePanel op, got {other:?}"),
    }

    app.with_global_mut(DockManager::default, |dock, _app| {
        assert!(dock.graph.apply_op(&op));

        match dock.graph.node(source_tabs) {
            Some(DockNode::Tabs { tabs, .. }) => {
                assert_eq!(
                    tabs,
                    &vec![PanelKey::new("core.a")],
                    "expected panel_b to move out of source tabs"
                );
            }
            other => panic!("expected source tabs node, got {other:?}"),
        }

        match dock.graph.node(target_tabs) {
            Some(DockNode::Tabs { tabs, .. }) => {
                assert_eq!(
                    tabs,
                    &vec![
                        PanelKey::new("core.c"),
                        PanelKey::new("core.d"),
                        PanelKey::new("core.b"),
                    ],
                    "expected panel_b to insert at end of target tabs"
                );
            }
            other => panic!("expected target tabs node, got {other:?}"),
        }
    });
}

#[test]
fn dock_tab_drop_overflow_end_emits_insert_index_tab_count() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let tab_count = 10usize;
    let panels: Vec<PanelKey> = (0..tab_count)
        .map(|ix| PanelKey::new(format!("core.{ix}")))
        .collect();

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let tabs = app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: panels.clone(),
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);

        for (ix, panel) in panels.iter().enumerate() {
            dock.panels.insert(
                panel.clone(),
                DockPanel {
                    title: format!("Panel {ix}"),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );
        }

        tabs
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(260.0), Px(180.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    let theme = fret_ui::Theme::global(&app).snapshot();
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);

    let candidate = super::super::tab_bar_kernel::compute_tab_bar_overflow_candidate_geometry(
        theme.clone(),
        tab_bar,
        tab_count,
        None,
    );
    assert!(candidate.overflows, "expected tab strip to overflow");

    let strip_end = candidate.strip_rect.origin.x.0 + candidate.strip_rect.size.width.0;
    let button_start = candidate.overflow_button_rect.origin.x.0;
    assert!(
        button_start > strip_end + 2.0,
        "expected reserved header space between strip and overflow button"
    );

    // Pick a point inside the reserved header space but outside the overflow button rect.
    let x0 = strip_end + 1.0;
    let x1 = button_start - 1.0;
    let x = Px((x0 + x1) * 0.5);
    let y = Px(tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5);
    let drop_pos = Point::new(x, y);

    let dragged_panel = panels
        .first()
        .cloned()
        .expect("expected at least one panel");

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: dragged_panel.clone(),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: drop_pos,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: drop_pos,
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let effects = app.take_effects();
    let insert_indices: Vec<_> = effects
        .iter()
        .filter_map(|e| match e {
            Effect::Dock(DockOp::MovePanel {
                panel,
                target_tabs,
                zone,
                insert_index,
                ..
            }) if panel == &dragged_panel && *target_tabs == tabs && *zone == DropZone::Center => {
                Some(*insert_index)
            }
            _ => None,
        })
        .collect();

    assert_eq!(insert_indices, vec![Some(tab_count)]);
}
