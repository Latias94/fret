use super::*;

#[test]
fn public_declarative_dock_space_entry_point_requests_tear_off_after_stable_oob_frame() {
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
    let panel = PanelKey::new("demo.public.declarative.tear-off");

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
            title: "Hierarchy".to_string(),
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
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tear-off-debounce",
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
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: panel.clone(),
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
    }

    let outside = Point::new(Px(-32.0), Px(12.0));

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::InternalDrag(InternalDragEvent {
            position: outside,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|effect| matches!(
            effect,
            Effect::Window(WindowRequest::Create(create))
                if matches!(
                    &create.kind,
                    CreateWindowKind::DockFloating { panel: requested, .. } if requested == &panel
                )
        )),
        "expected no declarative tear-off request on first OOB frame, got: {effects:?}"
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::InternalDrag(InternalDragEvent {
            position: outside,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    let effects = app.take_effects();
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Window(WindowRequest::Create(create))
                if matches!(
                    &create.kind,
                    CreateWindowKind::DockFloating { panel: requested, .. } if requested == &panel
                )
        )),
        "expected declarative tear-off request after stable OOB frame, got: {effects:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_publishes_diagnostics_and_liveness() {
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
    let panel = PanelKey::new("demo.public.declarative.diagnostics");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

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
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Diagnostics".to_string(),
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
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: panel.clone(),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.position = Point::new(Px(48.0), Px(22.0));
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
        "public-declarative-dock-host-diagnostics",
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

    let docking = app
        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
        .and_then(|store| store.docking_for_window(window, app.frame_id()))
        .expect("expected declarative docking diagnostics to be published");
    assert!(
        docking.dock_drag.is_some(),
        "expected declarative dock host to publish active dock-drag diagnostics, got: {docking:?}"
    );
    assert!(
        docking.dock_graph_stats.is_some(),
        "expected declarative dock host to publish graph stats, got: {docking:?}"
    );
    assert!(
        docking.dock_graph_signature.is_some(),
        "expected declarative dock host to publish graph signature, got: {docking:?}"
    );
    assert!(
        ui.debug_prepaint_actions().iter().any(|action| {
            action.kind == fret_ui::tree::UiDebugPrepaintActionKind::RequestAnimationFrame
        }),
        "expected declarative dock host to request animation frames while dock drag is active"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_syncs_viewport_layouts() {
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
    let target = fret_core::RenderTargetId::from(KeyData::from_ffi(42));
    let stale_target = fret_core::RenderTargetId::from(KeyData::from_ffi(43));
    let panel = PanelKey::new("demo.public.declarative.viewport");

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
            title: "Viewport".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: Some(ViewportPanel {
                target,
                target_px_size: (640, 360),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
        let stale_rect = Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(2.0), Px(2.0)));
        let stale_mapping = ViewportMapping {
            content_rect: stale_rect,
            target_px_size: (1, 1),
            fit: ViewportFit::Stretch,
        };
        dock.set_viewport_layout(
            window,
            stale_target,
            DockViewportLayout {
                content_rect: stale_rect,
                mapping: stale_mapping,
                draw_rect: stale_mapping.map().draw_rect,
            },
        );
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
        "public-declarative-dock-host-viewport-layout",
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

    let layout = app
        .global::<DockManager>()
        .and_then(|dock| dock.viewport_layout(window, target))
        .expect("expected declarative dock host to sync viewport layout during layout/prepaint");
    assert_eq!(layout.mapping.target_px_size, (640, 360));
    assert_eq!(layout.mapping.fit, fret_core::ViewportFit::Stretch);
    assert_eq!(layout.content_rect, layout.mapping.content_rect);
    assert_eq!(layout.draw_rect, layout.mapping.map().draw_rect);
    assert!(
        app.global::<DockManager>()
            .and_then(|dock| dock.viewport_layout(window, stale_target))
            .is_none(),
        "expected declarative viewport sync to clear stale layouts for the same window"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_paints_viewport_surface() {
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
    let target = fret_core::RenderTargetId::from(KeyData::from_ffi(44));
    let panel = PanelKey::new("demo.public.declarative.viewport.surface");

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
            title: "Viewport".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: Some(ViewportPanel {
                target,
                target_px_size: (640, 360),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
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
        "public-declarative-dock-host-viewport-surface",
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

    let layout = app
        .global::<DockManager>()
        .and_then(|dock| dock.viewport_layout(window, target))
        .expect("expected declarative dock host to sync viewport layout before paint");
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::ViewportSurface {
                target: painted_target,
                rect,
                ..
            } if *painted_target == target && *rect == layout.draw_rect
        )),
        "expected public declarative dock host to paint the viewport surface, got: {:?}",
        scene.ops()
    );
}

fn render_public_declarative_viewport_host(
    test_id: &'static str,
    target_key: u64,
    panel_key: &'static str,
) -> (
    UiTree<TestHost>,
    TestHost,
    FakeTextService,
    AppWindowId,
    fret_core::RenderTargetId,
    Rect,
    NodeId,
) {
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
    let target = fret_core::RenderTargetId::from(KeyData::from_ffi(target_key));
    let panel = PanelKey::new(panel_key);

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
            title: "Viewport".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: Some(ViewportPanel {
                target,
                target_px_size: (640, 360),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
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
        test_id,
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

    (ui, app, services, window, target, bounds, root)
}

#[test]
fn public_declarative_dock_space_entry_point_captures_viewport_pointer_input() {
    let (mut ui, mut app, mut services, window, target, _bounds, _root) =
        render_public_declarative_viewport_host(
            "public-declarative-dock-host-viewport-capture",
            45,
            "demo.public.declarative.viewport.capture",
        );
    let layout = app
        .global::<DockManager>()
        .and_then(|dock| dock.viewport_layout(window, target))
        .expect("expected declarative dock host to sync viewport layout before pointer input");
    let down_pos = Point::new(
        Px(layout.draw_rect.origin.x.0 + 10.0),
        Px(layout.draw_rect.origin.y.0 + 10.0),
    );
    let outside = Point::new(
        Px(layout.draw_rect.origin.x.0 - 40.0),
        Px(layout.draw_rect.origin.y.0 - 40.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    let Some(Effect::ViewportInput(down)) = effects
        .iter()
        .find(|effect| matches!(effect, Effect::ViewportInput(_)))
    else {
        panic!("expected declarative viewport PointerDown input, got: {effects:?}");
    };
    assert_eq!(down.target, target);
    assert!(
        matches!(
            down.kind,
            fret_core::ViewportInputKind::PointerDown {
                button: fret_core::MouseButton::Left,
                ..
            }
        ),
        "expected PointerDown viewport input, got: {down:?}"
    );
    assert!(
        ui.captured_for(fret_core::PointerId(0)).is_some(),
        "expected declarative viewport input to request pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
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

    let effects = app.take_effects();
    let Some(Effect::ViewportInput(move_input)) = effects
        .iter()
        .find(|effect| matches!(effect, Effect::ViewportInput(_)))
    else {
        panic!("expected declarative viewport PointerMove input during capture, got: {effects:?}");
    };
    assert_eq!(
        move_input.target, target,
        "expected captured moves outside the draw rect to stay on the original viewport"
    );
    assert_eq!(
        move_input.kind,
        fret_core::ViewportInputKind::PointerMove {
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
        }
    );
    assert_eq!(
        move_input.uv,
        (0.0, 0.0),
        "expected captured move uv to clamp outside the draw rect"
    );
    assert_eq!(
        move_input.target_px,
        (0, 0),
        "expected captured move target pixel to clamp outside the draw rect"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: outside,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    let Some(Effect::ViewportInput(up_input)) = effects
        .iter()
        .find(|effect| matches!(effect, Effect::ViewportInput(_)))
    else {
        panic!("expected declarative viewport PointerUp input, got: {effects:?}");
    };
    assert_eq!(up_input.target, target);
    assert!(
        matches!(
            up_input.kind,
            fret_core::ViewportInputKind::PointerUp {
                button: fret_core::MouseButton::Left,
                is_click: false,
                ..
            }
        ),
        "expected PointerUp viewport input, got: {up_input:?}"
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected declarative viewport capture to release on pointer up"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_viewport_capture_ignores_other_pointer_move_and_up() {
    let (mut ui, mut app, mut services, window, target, _bounds, _root) =
        render_public_declarative_viewport_host(
            "public-declarative-dock-host-viewport-capture-other-pointer",
            49,
            "demo.public.declarative.viewport.capture-other-pointer",
        );
    let layout = app
        .global::<DockManager>()
        .and_then(|dock| dock.viewport_layout(window, target))
        .expect("expected declarative dock host to sync viewport layout before pointer input");
    let down_pos = Point::new(
        Px(layout.draw_rect.origin.x.0 + 10.0),
        Px(layout.draw_rect.origin.y.0 + 10.0),
    );
    let outside = Point::new(
        Px(layout.draw_rect.origin.x.0 - 40.0),
        Px(layout.draw_rect.origin.y.0 - 40.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
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
            position: outside,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|effect| matches!(effect, Effect::ViewportInput(_))),
        "other pointer moves must not be forwarded while pointer 0 owns viewport capture, got: {effects:?}"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
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
    let _ = app.take_effects();
    assert!(
        ui.captured_for(fret_core::PointerId(0)).is_some(),
        "other pointer up must not release pointer 0 viewport capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
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

    let effects = app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|effect| matches!(effect, Effect::ViewportInput(_)))
    else {
        panic!("expected original pointer capture to remain active, got: {effects:?}");
    };
    assert_eq!(input.target, target);
    assert_eq!(
        input.kind,
        fret_core::ViewportInputKind::PointerMove {
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
        }
    );
    assert_eq!(input.uv, (0.0, 0.0));
    assert_eq!(input.target_px, (0, 0));
}

#[test]
fn public_declarative_dock_space_entry_point_forwards_viewport_right_click_input() {
    let (mut ui, mut app, mut services, window, target, _bounds, _root) =
        render_public_declarative_viewport_host(
            "public-declarative-dock-host-viewport-right-click",
            47,
            "demo.public.declarative.viewport.right-click",
        );
    let layout = app
        .global::<DockManager>()
        .and_then(|dock| dock.viewport_layout(window, target))
        .expect("expected declarative dock host to sync viewport layout before pointer input");
    let position = Point::new(
        Px(layout.draw_rect.origin.x.0 + 10.0),
        Px(layout.draw_rect.origin.y.0 + 10.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    let Some(Effect::ViewportInput(down)) = effects
        .iter()
        .find(|effect| matches!(effect, Effect::ViewportInput(_)))
    else {
        panic!("expected declarative viewport right PointerDown input, got: {effects:?}");
    };
    assert_eq!(down.target, target);
    assert!(
        matches!(
            down.kind,
            fret_core::ViewportInputKind::PointerDown {
                button: fret_core::MouseButton::Right,
                ..
            }
        ),
        "expected right PointerDown viewport input, got: {down:?}"
    );
    assert!(
        ui.captured_for(fret_core::PointerId(0)).is_some(),
        "expected declarative viewport right input to request pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
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

    let effects = app.take_effects();
    let Some(Effect::ViewportInput(up)) = effects
        .iter()
        .find(|effect| matches!(effect, Effect::ViewportInput(_)))
    else {
        panic!("expected declarative viewport right PointerUp input, got: {effects:?}");
    };
    assert_eq!(up.target, target);
    assert!(
        matches!(
            up.kind,
            fret_core::ViewportInputKind::PointerUp {
                button: fret_core::MouseButton::Right,
                is_click: true,
                ..
            }
        ),
        "expected right-click PointerUp to remain a click without drag, got: {up:?}"
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected declarative viewport right capture to release on pointer up"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_suppresses_viewport_right_drag_click_on_release() {
    let (mut ui, mut app, mut services, window, target, _bounds, _root) =
        render_public_declarative_viewport_host(
            "public-declarative-dock-host-viewport-right-drag",
            48,
            "demo.public.declarative.viewport.right-drag",
        );
    let layout = app
        .global::<DockManager>()
        .and_then(|dock| dock.viewport_layout(window, target))
        .expect("expected declarative dock host to sync viewport layout before pointer input");
    let start = Point::new(
        Px(layout.draw_rect.origin.x.0 + 10.0),
        Px(layout.draw_rect.origin.y.0 + 10.0),
    );
    let end = Point::new(Px(start.x.0 + 20.0), Px(start.y.0 + 20.0));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: fret_core::MouseButton::Right,
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
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
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

    let effects = app.take_effects();
    let Some(Effect::ViewportInput(up)) = effects
        .iter()
        .find(|effect| matches!(effect, Effect::ViewportInput(_)))
    else {
        panic!("expected declarative viewport right-drag PointerUp input, got: {effects:?}");
    };
    assert_eq!(up.target, target);
    assert!(
        matches!(
            up.kind,
            fret_core::ViewportInputKind::PointerUp {
                button: fret_core::MouseButton::Right,
                is_click: false,
                ..
            }
        ),
        "expected right-drag PointerUp to suppress click semantics, got: {up:?}"
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected declarative viewport right capture to release on pointer up"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_cancels_viewport_pointer_capture() {
    let (mut ui, mut app, mut services, window, target, _bounds, _root) =
        render_public_declarative_viewport_host(
            "public-declarative-dock-host-viewport-cancel",
            46,
            "demo.public.declarative.viewport.cancel",
        );
    let layout = app
        .global::<DockManager>()
        .and_then(|dock| dock.viewport_layout(window, target))
        .expect("expected declarative dock host to sync viewport layout before pointer input");
    let down_pos = Point::new(
        Px(layout.draw_rect.origin.x.0 + 10.0),
        Px(layout.draw_rect.origin.y.0 + 10.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = app.take_effects();
    assert!(
        ui.captured_for(fret_core::PointerId(0)).is_some(),
        "expected declarative viewport input to request pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: None,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );

    let effects = app.take_effects();
    let Some(Effect::ViewportInput(cancel_input)) = effects
        .iter()
        .find(|effect| matches!(effect, Effect::ViewportInput(_)))
    else {
        panic!("expected declarative viewport PointerCancel input, got: {effects:?}");
    };
    assert_eq!(cancel_input.target, target);
    assert!(
        matches!(
            cancel_input.kind,
            fret_core::ViewportInputKind::PointerCancel { .. }
        ),
        "expected PointerCancel viewport input, got: {cancel_input:?}"
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected declarative viewport capture to release on pointer cancel"
    );
}
