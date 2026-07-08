use super::*;

#[test]
fn public_declarative_dock_space_entry_point_paints_floating_chrome() {
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
    let main_panel = PanelKey::new("demo.public.declarative.floating.main");
    let floating_panel = PanelKey::new("demo.public.declarative.floating.inspector");
    let floating_rect = Rect::new(
        Point::new(Px(180.0), Px(80.0)),
        Size::new(Px(220.0), Px(150.0)),
    );

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
        dock.ensure_panel(&main_panel, || crate::DockPanel {
            title: "Main".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&floating_panel, || crate::DockPanel {
            title: "Inspector".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let main_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![main_panel.clone()],
            active: 0,
        });
        let floating_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![floating_panel.clone()],
            active: 0,
        });
        let floating = dock.workspace.graph.insert_node(DockNode::Floating {
            child: floating_tabs,
        });
        dock.workspace.graph.set_window_root(window, main_tabs);
        dock.workspace
            .graph
            .floating_windows_mut(window)
            .push(fret_core::DockFloatingWindow {
                floating,
                rect: floating_rect,
            });
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
        "public-declarative-dock-host-floating-chrome",
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

    let title_bar = Rect::new(
        Point::new(
            Px(floating_rect.origin.x.0 + 1.0),
            Px(floating_rect.origin.y.0 + 1.0),
        ),
        Size::new(Px(floating_rect.size.width.0 - 2.0), Px(22.0)),
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad {
                order,
                rect,
                ..
            } if *order == fret_core::DrawOrder(0) && *rect == floating_rect
        )),
        "expected public declarative dock host to paint the floating outer chrome, got: {:?}",
        scene.ops()
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad {
                order,
                rect,
                ..
            } if *order == fret_core::DrawOrder(1) && *rect == title_bar
        )),
        "expected public declarative dock host to paint the floating title-bar chrome, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_hovers_floating_chrome() {
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
    let main_panel = PanelKey::new("demo.public.declarative.floating.hover.main");
    let floating_panel = PanelKey::new("demo.public.declarative.floating.hover.inspector");
    let floating_rect = Rect::new(
        Point::new(Px(180.0), Px(80.0)),
        Size::new(Px(220.0), Px(150.0)),
    );

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
        dock.ensure_panel(&main_panel, || crate::DockPanel {
            title: "Main".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&floating_panel, || crate::DockPanel {
            title: "Inspector".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let main_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![main_panel.clone()],
            active: 0,
        });
        let floating_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![floating_panel.clone()],
            active: 0,
        });
        let floating = dock.workspace.graph.insert_node(DockNode::Floating {
            child: floating_tabs,
        });
        dock.workspace.graph.set_window_root(window, main_tabs);
        dock.workspace
            .graph
            .floating_windows_mut(window)
            .push(fret_core::DockFloatingWindow {
                floating,
                rect: floating_rect,
            });
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
        "public-declarative-dock-host-floating-hover",
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

    let title_bar = Rect::new(
        Point::new(
            Px(floating_rect.origin.x.0 + 1.0),
            Px(floating_rect.origin.y.0 + 1.0),
        ),
        Size::new(Px(floating_rect.size.width.0 - 2.0), Px(22.0)),
    );
    let close_size = crate::dock::consts::DOCK_FLOATING_CLOSE_SIZE.0;
    let close_pad = crate::dock::consts::DOCK_FLOATING_BORDER.0.max(4.0);
    let close_button = Rect::new(
        Point::new(
            Px(title_bar.origin.x.0 + title_bar.size.width.0 - close_pad - close_size),
            Px(title_bar.origin.y.0 + (title_bar.size.height.0 - close_size) * 0.5),
        ),
        Size::new(Px(close_size), Px(close_size)),
    );
    let title_hover_pos = Point::new(
        Px(title_bar.origin.x.0 + 24.0),
        Px(title_bar.origin.y.0 + title_bar.size.height.0 * 0.5),
    );
    let close_hover_pos = Point::new(
        Px(close_button.origin.x.0 + close_button.size.width.0 * 0.5),
        Px(close_button.origin.y.0 + close_button.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: title_hover_pos,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let theme = fret_ui::Theme::global(&app).snapshot();
    let expected_title_hover = Color {
        a: 0.22,
        ..theme.color_token("accent")
    };
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad {
                order,
                rect,
                background,
                ..
            } if *order == fret_core::DrawOrder(1)
                && *rect == title_bar
                && matches!(background.paint, fret_core::Paint::Solid(color) if color == expected_title_hover)
        )),
        "expected declarative dock host to paint hovered floating title-bar background, got: {:?}",
        scene.ops()
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: close_hover_pos,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad {
                order,
                rect,
                background,
                ..
            } if *order == fret_core::DrawOrder(2)
                && *rect == close_button
                && matches!(background.paint, fret_core::Paint::Solid(color) if color == theme.color_token("accent"))
        )),
        "expected declarative dock host to paint hovered floating close affordance, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_closes_floating_chrome() {
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
    let main_panel = PanelKey::new("demo.public.declarative.floating.close.main");
    let floating_panel = PanelKey::new("demo.public.declarative.floating.close.inspector");
    let floating_rect = Rect::new(
        Point::new(Px(180.0), Px(80.0)),
        Size::new(Px(220.0), Px(150.0)),
    );

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
    let (main_tabs, floating) = app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&main_panel, || crate::DockPanel {
            title: "Main".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&floating_panel, || crate::DockPanel {
            title: "Inspector".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let main_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![main_panel.clone()],
            active: 0,
        });
        let floating_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![floating_panel.clone()],
            active: 0,
        });
        let floating = dock.workspace.graph.insert_node(DockNode::Floating {
            child: floating_tabs,
        });
        dock.workspace.graph.set_window_root(window, main_tabs);
        dock.workspace
            .graph
            .floating_windows_mut(window)
            .push(fret_core::DockFloatingWindow {
                floating,
                rect: floating_rect,
            });
        (main_tabs, floating)
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
        "public-declarative-dock-host-floating-close",
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

    let close_size = crate::dock::consts::DOCK_FLOATING_CLOSE_SIZE.0;
    let close_pos = Point::new(
        Px(floating_rect.origin.x.0 + floating_rect.size.width.0 - 8.0 - close_size * 0.5),
        Px(floating_rect.origin.y.0
            + 1.0
            + (crate::dock::consts::DOCK_FLOATING_TITLE_H.0 - close_size) * 0.5
            + close_size * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: close_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: close_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Dock(DockOp::RaiseFloating {
                window: effect_window,
                floating: effect_floating,
            }) if *effect_window == window && *effect_floating == floating
        )),
        "expected declarative dock host to raise the floating container on close press, got: {effects:?}"
    );
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Dock(DockOp::MergeFloatingInto {
                window: effect_window,
                floating: effect_floating,
                target_tabs,
            }) if *effect_window == window
                && *effect_floating == floating
                && *target_tabs == main_tabs
        )),
        "expected declarative dock host to merge the floating container back into the main tabs on close release, got: {effects:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_drags_floating_title_bar() {
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
    let main_panel = PanelKey::new("demo.public.declarative.floating.drag.main");
    let floating_panel = PanelKey::new("demo.public.declarative.floating.drag.inspector");
    let floating_rect = Rect::new(
        Point::new(Px(180.0), Px(80.0)),
        Size::new(Px(220.0), Px(150.0)),
    );

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
    let floating = app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&main_panel, || crate::DockPanel {
            title: "Main".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&floating_panel, || crate::DockPanel {
            title: "Inspector".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let main_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![main_panel.clone()],
            active: 0,
        });
        let floating_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![floating_panel.clone()],
            active: 0,
        });
        let floating = dock.workspace.graph.insert_node(DockNode::Floating {
            child: floating_tabs,
        });
        dock.workspace.graph.set_window_root(window, main_tabs);
        dock.workspace
            .graph
            .floating_windows_mut(window)
            .push(fret_core::DockFloatingWindow {
                floating,
                rect: floating_rect,
            });
        floating
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
        "public-declarative-dock-host-floating-title-drag",
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

    let down_pos = Point::new(
        Px(floating_rect.origin.x.0 + 32.0),
        Px(floating_rect.origin.y.0 + 12.0),
    );
    let move_pos = Point::new(Px(down_pos.x.0 + 12.0), Px(down_pos.y.0 + 8.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );
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
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Dock(DockOp::RaiseFloating {
                window: effect_window,
                floating: effect_floating,
            }) if *effect_window == window && *effect_floating == floating
        )),
        "expected declarative dock host to raise the floating container on title-bar press, got: {effects:?}"
    );
    let expected_rect = Rect::new(
        Point::new(
            Px(floating_rect.origin.x.0 + 12.0),
            Px(floating_rect.origin.y.0 + 8.0),
        ),
        floating_rect.size,
    );
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Dock(DockOp::SetFloatingRect {
                window: effect_window,
                floating: effect_floating,
                rect,
            }) if *effect_window == window
                && *effect_floating == floating
                && *rect == expected_rect
        )),
        "expected declarative dock host to emit SetFloatingRect while dragging the floating title bar, got: {effects:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_merges_floating_title_bar_drag_on_center_drop() {
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
    let main_panel = PanelKey::new("demo.public.declarative.floating.merge.main");
    let floating_panel = PanelKey::new("demo.public.declarative.floating.merge.inspector");
    let floating_rect = Rect::new(
        Point::new(Px(260.0), Px(80.0)),
        Size::new(Px(140.0), Px(120.0)),
    );

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
    let (main_tabs, floating) = app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&main_panel, || crate::DockPanel {
            title: "Main".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&floating_panel, || crate::DockPanel {
            title: "Inspector".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let main_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![main_panel.clone()],
            active: 0,
        });
        let floating_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![floating_panel.clone()],
            active: 0,
        });
        let floating = dock.workspace.graph.insert_node(DockNode::Floating {
            child: floating_tabs,
        });
        dock.workspace.graph.set_window_root(window, main_tabs);
        dock.workspace
            .graph
            .floating_windows_mut(window)
            .push(fret_core::DockFloatingWindow {
                floating,
                rect: floating_rect,
            });
        (main_tabs, floating)
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
        "public-declarative-dock-host-floating-title-merge",
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

    let down_pos = Point::new(
        Px(floating_rect.origin.x.0 + 32.0),
        Px(floating_rect.origin.y.0 + 12.0),
    );
    let center_pos = Point::new(Px(210.0), Px(120.0));
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
            position: center_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
                zone: DropZone::Center,
                ..
            })) if tabs == main_tabs
        ),
        "expected declarative floating title-bar drag to resolve a center dock hover, got: {hover:?}"
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: center_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Dock(DockOp::MergeFloatingInto {
                window: effect_window,
                floating: effect_floating,
                target_tabs,
            }) if *effect_window == window
                && *effect_floating == floating
                && *target_tabs == main_tabs
        )),
        "expected declarative floating title-bar drop to merge into the main tabs, got: {effects:?}"
    );
    let hover_after = app
        .global::<DockManager>()
        .and_then(|dock| dock.presentation.hover.clone());
    assert!(
        hover_after.is_none(),
        "expected declarative floating title-bar drop to clear hover, got: {hover_after:?}"
    );
}
