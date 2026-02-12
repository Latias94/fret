use super::*;

#[test]
fn dock_tab_drop_outside_window_requests_float() {
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

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
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
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(-32.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected a float request effect when dropping outside the window"
    );
}
#[test]
fn dock_tab_drop_outside_window_does_not_request_tear_off_twice() {
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

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
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
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    let outside = Point::new(Px(-32.0), Px(12.0));

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: outside,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: outside,
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let effects = app.take_effects();
    let count = effects
        .iter()
        .filter(|e| {
            matches!(
                e,
                Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel, .. })
                    if *panel == PanelKey::new("core.hierarchy")
            )
        })
        .count();
    assert_eq!(
        count, 1,
        "expected at most one tear-off request for a single drag session"
    );
}
#[test]
fn dock_tab_drop_on_float_zone_floats_in_window_even_when_tear_off_enabled() {
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

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
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
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let zone = float_zone(dock_bounds);
    let pos = Point::new(
        Px(zone.origin.x.0 + zone.size.width.0 * 0.5),
        Px(zone.origin.y.0 + zone.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: pos,
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::FloatPanelInWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected in-window floating when dropping on float_zone(...)"
    );
    assert!(
        !effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "float_zone(...) must not request a new OS window"
    );
}
#[test]
fn floating_window_can_be_dragged_from_tab() {
    // This verifies imgui/egui-style affordance: dragging a tab in an in-window floating container
    // moves the container itself (unless Alt is held).
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let (floating, floating_tabs, start_rect) =
        app.with_global_mut(DockManager::default, |dock, _app| {
            let main_tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("core.hierarchy")],
                active: 0,
            });
            dock.graph.set_window_root(window, main_tabs);

            let floating_tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("core.inspector")],
                active: 0,
            });
            let floating = dock.graph.insert_node(DockNode::Floating {
                child: floating_tabs,
            });
            let start_rect = Rect::new(
                Point::new(Px(180.0), Px(140.0)),
                Size::new(Px(320.0), Px(240.0)),
            );
            dock.graph
                .floating_windows_mut(window)
                .push(fret_core::DockFloatingWindow {
                    floating,
                    rect: start_rect,
                });

            dock.panels.insert(
                PanelKey::new("core.hierarchy"),
                DockPanel {
                    title: "Hierarchy".to_string(),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );
            dock.panels.insert(
                PanelKey::new("core.inspector"),
                DockPanel {
                    title: "Inspector".to_string(),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );

            (floating, floating_tabs, start_rect)
        });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    ui.layout(&mut app, &mut text, root, size, 1.0);

    // Derive the floating tab-bar position from the same layout math DockSpace uses.
    let border = Px(1.0);
    let title_h = Px(22.0);
    let inner = Rect::new(
        Point::new(
            Px(start_rect.origin.x.0 + border.0),
            Px(start_rect.origin.y.0 + border.0 + title_h.0),
        ),
        Size::new(
            Px((start_rect.size.width.0 - border.0 * 2.0).max(0.0)),
            Px((start_rect.size.height.0 - border.0 * 2.0 - title_h.0).max(0.0)),
        ),
    );
    let settings = fret_runtime::DockingInteractionSettings::default();
    let layout = app.with_global_mut(DockManager::default, |dock, _app| {
        compute_layout_map(
            &dock.graph,
            floating,
            inner,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
        )
    });
    let tabs_rect = *layout.get(&floating_tabs).expect("floating tabs rect");
    let (tab_bar, _content) = split_tab_bar(tabs_rect);

    let down = Point::new(Px(tab_bar.origin.x.0 + 40.0), Px(tab_bar.origin.y.0 + 10.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: down,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let mut buttons = fret_core::MouseButtons::default();
    buttons.left = true;
    let moved = Point::new(Px(down.x.0 + 50.0), Px(down.y.0 + 20.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: moved,
            buttons,
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    let set_rect = effects.iter().find_map(|e| match e {
        Effect::Dock(DockOp::SetFloatingRect {
            window: w,
            floating: f,
            rect,
        }) if *w == window && *f == floating => Some(*rect),
        _ => None,
    });
    let rect = set_rect.expect("expected SetFloatingRect during floating drag");
    assert_eq!(rect.origin.x, Px(start_rect.origin.x.0 + 50.0));
    assert_eq!(rect.origin.y, Px(start_rect.origin.y.0 + 20.0));
}
#[test]
fn dock_tab_drop_outside_window_floats_in_window_when_tear_off_disabled() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    let mut caps = PlatformCapabilities::default();
    caps.ui.window_tear_off = false;
    app.set_global(caps);
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

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
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
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(-32.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::FloatPanelInWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected an in-window float effect when dropping outside with tear-off disabled"
    );
}
#[test]
fn dock_tab_drop_outside_window_floats_in_window_when_multi_window_is_disabled() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    let mut caps = PlatformCapabilities::default();
    caps.ui.multi_window = false;
    caps.ui.window_tear_off = true;
    app.set_global(caps);
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

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
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
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(-32.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::FloatPanelInWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected an in-window float effect when multi-window is disabled"
    );
}
#[test]
fn dock_tab_drop_outside_routes_to_dock_space() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(TestStack);
    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.add_child(root, dock_space);
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
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
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
    }

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(-32.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected DockSpace to receive the drop even when hit-testing fails"
    );
}
