use super::*;
use std::{collections::HashMap, sync::Mutex};

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

fn fill_semantics_props() -> SemanticsProps {
    let mut props = SemanticsProps::default();
    props.layout.size.width = fret_ui::element::Length::Fill;
    props.layout.size.height = fret_ui::element::Length::Fill;
    props
}

fn absolute_anchor_props(left: Px, top: Px) -> SemanticsProps {
    SemanticsProps {
        layout: LayoutStyle {
            position: PositionStyle::Absolute,
            inset: fret_ui::element::InsetStyle {
                left: InsetEdge::Px(left),
                top: InsetEdge::Px(top),
                ..Default::default()
            },
            size: SizeStyle {
                width: fret_ui::element::Length::Px(Px(100.0)),
                height: fret_ui::element::Length::Px(Px(30.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn setup_single_panel_window(
    app: &mut TestHost,
    window: AppWindowId,
    panel: &PanelKey,
    target: Option<fret_core::RenderTargetId>,
) {
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(panel, || crate::DockPanel {
            title: panel.kind.0.clone(),
            color: fret_core::Color::TRANSPARENT,
            viewport: target.map(|target| ViewportPanel {
                target,
                target_px_size: (320, 240),
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
}

#[test]
fn declarative_managed_surface_consumes_dock_space_layout_snapshot_for_panel_roots() {
    let window = AppWindowId::default();
    let panel_left = PanelKey::new("demo.declarative.left");
    let panel_right = PanelKey::new("demo.declarative.right");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;

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
        let left_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_left.clone()],
            active: 0,
        });
        let right_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_right.clone()],
            active: 0,
        });
        let root = dock.workspace.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left_tabs, right_tabs],
            fractions: vec![0.35, 0.65],
        });
        dock.workspace.graph.set_window_root(window, root);
    });

    let expected = {
        let dock = app.global::<DockManager>().expect("dock manager");
        let root = dock.workspace.graph.window_root(window).expect("dock root");
        let (_chrome, dock_bounds) = dock_space_regions(bounds);
        let settings = fret_runtime::DockingInteractionSettings::default();
        let layout = compute_layout_map(
            &dock.workspace.graph,
            root,
            dock_bounds,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
        );
        active_panel_content_bounds(&dock.workspace.graph, &layout)
    };

    let left_for_bind = panel_left.clone();
    let right_for_bind = panel_right.clone();
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "declarative-dock-host-proof",
        move |cx| {
            let left_for_layout = left_for_bind.clone();
            let right_for_layout = right_for_bind.clone();
            vec![cx.managed_surface(
                fret_ui::element::ManagedSurfaceProps::default(),
                move |cx| {
                    let children = cx.children().to_vec();
                    cx.app()
                        .with_global_mut(DockPanelContentService::default, |content, _app| {
                            content.replace_window(
                                window,
                                [
                                    (left_for_layout.clone(), children[0]),
                                    (right_for_layout.clone(), children[1]),
                                ],
                            );
                        });

                    let settings = fret_runtime::DockingInteractionSettings::default();
                    let bounds = cx.bounds();
                    let (_chrome, dock_bounds) = dock_space_regions(bounds);
                    let Some(snapshot) = cx.app().global::<DockManager>().and_then(|dock| {
                        DockSpaceLayoutSnapshot::build(
                            dock,
                            window,
                            dock_bounds,
                            settings.split_handle_gap,
                            settings.split_handle_hit_thickness,
                            &HashMap::new(),
                        )
                    }) else {
                        return;
                    };

                    let panel_nodes: HashMap<PanelKey, NodeId> = cx
                        .app()
                        .global::<DockPanelContentService>()
                        .map(|content| content.panel_nodes(window).into_iter().collect())
                        .unwrap_or_default();
                    let mut panel_last_sizes = HashMap::new();
                    for (_panel, node, rect) in panel_root_placements_for_snapshot(
                        &snapshot,
                        &panel_nodes,
                        &mut panel_last_sizes,
                    ) {
                        let _ = cx.layout_child_root(node, rect);
                    }
                    cx.set_output(snapshot.paint_panel_bounds.clone());
                },
                move |cx| {
                    let Some(paint_panels) = cx.output::<Vec<(PanelKey, Rect)>>().cloned() else {
                        return;
                    };
                    let panel_nodes: HashMap<PanelKey, NodeId> = cx
                        .app()
                        .global::<DockPanelContentService>()
                        .map(|content| content.panel_nodes(window).into_iter().collect())
                        .unwrap_or_default();
                    for (panel, rect) in paint_panels {
                        if let Some(node) = panel_nodes.get(&panel).copied() {
                            cx.paint_child(node, rect);
                        }
                    }
                },
                |cx| {
                    vec![
                        cx.canvas(fret_ui::element::CanvasProps::default(), |p| {
                            let rect = p.bounds();
                            p.scene().push(SceneOp::Quad {
                                order: fret_core::DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(Color {
                                    r: 1.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                })
                                .into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        }),
                        cx.canvas(fret_ui::element::CanvasProps::default(), |p| {
                            let rect = p.bounds();
                            p.scene().push(SceneOp::Quad {
                                order: fret_core::DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(Color {
                                    r: 0.0,
                                    g: 1.0,
                                    b: 0.0,
                                    a: 1.0,
                                })
                                .into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        }),
                    ]
                },
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let left_node = app
        .global::<DockPanelContentService>()
        .and_then(|svc| svc.get(window, &panel_left))
        .expect("expected left panel node to be bound");
    let right_node = app
        .global::<DockPanelContentService>()
        .and_then(|svc| svc.get(window, &panel_right))
        .expect("expected right panel node to be bound");
    assert_eq!(
        ui.debug_node_bounds(left_node),
        expected.get(&panel_left).copied()
    );
    assert_eq!(
        ui.debug_node_bounds(right_node),
        expected.get(&panel_right).copied()
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let painted: Vec<(Rect, Color)> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::Quad {
                rect, background, ..
            } => match background.paint {
                fret_core::Paint::Solid(color) => Some((*rect, color)),
                _ => None,
            },
            _ => None,
        })
        .collect();

    assert_eq!(painted.len(), 2);
    assert_eq!(painted[0].0, expected[&panel_left]);
    assert_eq!(painted[0].1.r, 1.0);
    assert_eq!(painted[1].0, expected[&panel_right]);
    assert_eq!(painted[1].1.g, 1.0);
}

#[test]
fn public_declarative_dock_space_entry_point_hosts_registry_panel_roots() {
    struct DeclarativePanelRegistry;

    impl DockPanelElementRegistry<TestHost> for DeclarativePanelRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            let color = if panel.kind.0.ends_with(".left") {
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                }
            };
            Some(
                cx.canvas(fret_ui::element::CanvasProps::default(), move |p| {
                    let rect = p.bounds();
                    p.scene().push(SceneOp::Quad {
                        order: fret_core::DrawOrder(0),
                        rect,
                        background: fret_core::Paint::Solid(color).into(),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }),
            )
        }
    }

    let window = AppWindowId::default();
    let panel_left = PanelKey::new("demo.public.declarative.left");
    let panel_right = PanelKey::new("demo.public.declarative.right");

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

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;

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
        let left_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_left.clone()],
            active: 0,
        });
        let right_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_right.clone()],
            active: 0,
        });
        let root = dock.workspace.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left_tabs, right_tabs],
            fractions: vec![0.35, 0.65],
        });
        dock.workspace.graph.set_window_root(window, root);
    });

    let expected = {
        let dock = app.global::<DockManager>().expect("dock manager");
        let root = dock.workspace.graph.window_root(window).expect("dock root");
        let (_chrome, dock_bounds) = dock_space_regions(bounds);
        let settings = fret_runtime::DockingInteractionSettings::default();
        let layout = compute_layout_map(
            &dock.workspace.graph,
            root,
            dock_bounds,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
        );
        active_panel_content_bounds(&dock.workspace.graph, &layout)
    };

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host",
        move |cx| {
            vec![dock_space_element_from_registry(
                cx,
                window,
                DockSpaceElementOptions {
                    test_id: Some("public-declarative-dock-space"),
                    ..Default::default()
                },
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let left_node = app
        .global::<DockPanelContentService>()
        .and_then(|svc| svc.get(window, &panel_left))
        .expect("expected left panel node to be bound");
    let right_node = app
        .global::<DockPanelContentService>()
        .and_then(|svc| svc.get(window, &panel_right))
        .expect("expected right panel node to be bound");
    assert_eq!(
        ui.debug_node_bounds(left_node),
        expected.get(&panel_left).copied()
    );
    assert_eq!(
        ui.debug_node_bounds(right_node),
        expected.get(&panel_right).copied()
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let painted: Vec<(Rect, Color)> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::Quad {
                rect, background, ..
            } => match background.paint {
                fret_core::Paint::Solid(color) => Some((*rect, color)),
                _ => None,
            },
            _ => None,
        })
        .collect();

    assert!(
        painted
            .iter()
            .any(|(rect, color)| *rect == expected[&panel_left] && color.r == 1.0),
        "expected public declarative dock host to paint the left panel root, got: {painted:?}"
    );
    assert!(
        painted
            .iter()
            .any(|(rect, color)| *rect == expected[&panel_right] && color.g == 1.0),
        "expected public declarative dock host to paint the right panel root, got: {painted:?}"
    );
    assert!(
        painted.iter().any(|(rect, color)| {
            rect.size.width.0 <= 1.1 && rect.size.height.0 > 100.0 && color.a > 0.0
        }),
        "expected public declarative dock host to paint the split handle from shared paint inputs, got: {painted:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_records_panel_root_bounds_for_element_queries() {
    let window = AppWindowId::default();
    let panel_left = PanelKey::new("demo.public.declarative.bounds.left");
    let panel_right = PanelKey::new("demo.public.declarative.bounds.right");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );
    let mut services = FakeTextService;

    let split = app.with_global_mut(DockManager::default, |dock, _app| {
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
        let left_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_left.clone()],
            active: 0,
        });
        let right_tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_right.clone()],
            active: 0,
        });
        let split = dock.workspace.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left_tabs, right_tabs],
            fractions: vec![0.35, 0.65],
        });
        dock.workspace.graph.set_window_root(window, split);
        split
    });

    let expected = |app: &TestHost| {
        let dock = app.global::<DockManager>().expect("dock manager");
        let root = dock.workspace.graph.window_root(window).expect("dock root");
        let (_chrome, dock_bounds) = dock_space_regions(bounds);
        let settings = fret_runtime::DockingInteractionSettings::default();
        let layout = compute_layout_map(
            &dock.workspace.graph,
            root,
            dock_bounds,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
        );
        active_panel_content_bounds(&dock.workspace.graph, &layout)
    };

    let render_host =
        |ui: &mut UiTree<TestHost>,
         app: &mut TestHost,
         services: &mut FakeTextService,
         left_id: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>,
         right_id: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>| {
            let panel_left_for_render = panel_left.clone();
            let panel_right_for_render = panel_right.clone();
            declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "public-declarative-dock-host-bounds",
                move |cx| {
                    let mut left_props = fret_ui::element::SemanticsProps::default();
                    left_props.layout.size.width = fret_ui::element::Length::Fill;
                    left_props.layout.size.height = fret_ui::element::Length::Fill;
                    let left = cx.semantics_with_id(left_props, {
                        let left_id = left_id.clone();
                        move |cx, id| {
                            *left_id.lock().expect("left id mutex") = Some(id);
                            vec![cx.text("left")]
                        }
                    });

                    let mut right_props = fret_ui::element::SemanticsProps::default();
                    right_props.layout.size.width = fret_ui::element::Length::Fill;
                    right_props.layout.size.height = fret_ui::element::Length::Fill;
                    let right = cx.semantics_with_id(right_props, {
                        let right_id = right_id.clone();
                        move |cx, id| {
                            *right_id.lock().expect("right id mutex") = Some(id);
                            vec![cx.text("right")]
                        }
                    });

                    vec![dock_space_element(
                        cx,
                        window,
                        DockSpaceElementOptions {
                            test_id: Some("public-declarative-dock-space-bounds"),
                            ..Default::default()
                        },
                        [
                            dock_panel_element(panel_left_for_render.clone(), left),
                            dock_panel_element(panel_right_for_render.clone(), right),
                        ],
                    )]
                },
            )
        };

    let left_id = Arc::new(Mutex::new(None));
    let right_id = Arc::new(Mutex::new(None));

    let root = render_host(
        &mut ui,
        &mut app,
        &mut services,
        left_id.clone(),
        right_id.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let expected_0 = expected(&app);
    let left_element = left_id
        .lock()
        .expect("left id mutex")
        .expect("expected left element id");
    let right_element = right_id
        .lock()
        .expect("right id mutex")
        .expect("expected right element id");
    let left_node = fret_ui::elements::node_for_element(&mut app, window, left_element)
        .expect("expected left node");
    let right_node = fret_ui::elements::node_for_element(&mut app, window, right_element)
        .expect("expected right node");

    assert_eq!(
        ui.debug_node_bounds(left_node),
        Some(expected_0[&panel_left])
    );
    assert_eq!(
        ui.debug_node_bounds(right_node),
        Some(expected_0[&panel_right])
    );
    assert_eq!(
        fret_ui::elements::current_bounds_for_element(&mut app, window, left_element),
        Some(expected_0[&panel_left])
    );
    assert_eq!(
        fret_ui::elements::current_bounds_for_element(&mut app, window, right_element),
        Some(expected_0[&panel_right])
    );

    app.with_global_mut(DockManager::default, |dock, _app| {
        assert!(
            dock.workspace
                .graph
                .update_split_fractions(split, vec![0.5, 0.5]),
            "expected split fraction update to succeed"
        );
    });
    let expected_1 = expected(&app);
    assert_ne!(expected_0[&panel_left], expected_1[&panel_left]);
    assert_ne!(expected_0[&panel_right], expected_1[&panel_right]);

    app.advance_frame();
    let root = render_host(&mut ui, &mut app, &mut services, left_id, right_id);
    ui.set_root(root);
    if let Some(host) = ui.children(root).first().copied() {
        ui.invalidate(host, fret_ui::Invalidation::Layout);
    }
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window, left_element),
        Some(expected_0[&panel_left]),
        "cross-frame element bounds should report the previously committed dock panel rect"
    );
    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window, right_element),
        Some(expected_0[&panel_right]),
        "cross-frame element bounds should report the previously committed dock panel rect"
    );
    assert_eq!(
        fret_ui::elements::current_bounds_for_element(&mut app, window, left_element),
        Some(expected_1[&panel_left]),
        "current element bounds should reflect the latest declarative dock layout"
    );
    assert_eq!(
        fret_ui::elements::current_bounds_for_element(&mut app, window, right_element),
        Some(expected_1[&panel_right]),
        "current element bounds should reflect the latest declarative dock layout"
    );
    assert_eq!(
        ui.debug_node_bounds(left_node),
        Some(expected_1[&panel_left])
    );
    assert_eq!(
        ui.debug_node_bounds(right_node),
        Some(expected_1[&panel_right])
    );
}

#[test]
fn public_declarative_dock_space_entry_point_keeps_bounds_window_scoped_across_windows() {
    let window_a = AppWindowId::default();
    let window_b = AppWindowId::from(KeyData::from_ffi(42));
    let panel_a = PanelKey::new("demo.public.declarative.multi-window.a");
    let panel_b = PanelKey::new("demo.public.declarative.multi-window.b");
    let bounds_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(240.0)),
    );
    let bounds_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(480.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    setup_single_panel_window(&mut app, window_a, &panel_a, None);
    setup_single_panel_window(&mut app, window_b, &panel_b, None);

    let mut services = FakeTextService;
    let mut ui_a: UiTree<TestHost> = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b: UiTree<TestHost> = UiTree::new();
    ui_b.set_window(window_b);

    let element_a = Arc::new(Mutex::new(None));
    let element_b = Arc::new(Mutex::new(None));

    let render_window_a =
        |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut FakeTextService| {
            let panel = panel_a.clone();
            let element = element_a.clone();
            declarative::render_root(
                ui,
                app,
                services,
                window_a,
                bounds_a,
                "public-declarative-dock-host-multi-window-a",
                move |cx| {
                    let child = cx.semantics_with_id(fill_semantics_props(), {
                        let element = element.clone();
                        move |cx, id| {
                            *element.lock().expect("window a element mutex") = Some(id);
                            vec![cx.text("a")]
                        }
                    });
                    vec![dock_space_element(
                        cx,
                        window_a,
                        DockSpaceElementOptions::default(),
                        [dock_panel_element(panel.clone(), child)],
                    )]
                },
            )
        };
    let render_window_b =
        |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut FakeTextService| {
            let panel = panel_b.clone();
            let element = element_b.clone();
            declarative::render_root(
                ui,
                app,
                services,
                window_b,
                bounds_b,
                "public-declarative-dock-host-multi-window-b",
                move |cx| {
                    let child = cx.semantics_with_id(fill_semantics_props(), {
                        let element = element.clone();
                        move |cx, id| {
                            *element.lock().expect("window b element mutex") = Some(id);
                            vec![cx.text("b")]
                        }
                    });
                    vec![dock_space_element(
                        cx,
                        window_b,
                        DockSpaceElementOptions::default(),
                        [dock_panel_element(panel.clone(), child)],
                    )]
                },
            )
        };

    let root_a = render_window_a(&mut ui_a, &mut app, &mut services);
    ui_a.set_root(root_a);
    let root_b = render_window_b(&mut ui_b, &mut app, &mut services);
    ui_b.set_root(root_b);

    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    let element_a = element_a
        .lock()
        .expect("window a element mutex")
        .expect("expected window a element id");
    let element_b = element_b
        .lock()
        .expect("window b element mutex")
        .expect("expected window b element id");
    let expected_a_0 = fret_ui::elements::current_bounds_for_element(&mut app, window_a, element_a)
        .expect("expected window a current bounds");
    let expected_b_0 = fret_ui::elements::current_bounds_for_element(&mut app, window_b, element_b)
        .expect("expected window b current bounds");
    assert_ne!(
        expected_a_0, expected_b_0,
        "test setup should make window-local panel rects distinguishable"
    );

    app.advance_frame();
    let root_a = render_window_a(&mut ui_a, &mut app, &mut services);
    ui_a.set_root(root_a);
    let root_b = render_window_b(&mut ui_b, &mut app, &mut services);
    ui_b.set_root(root_b);
    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window_a, element_a),
        Some(expected_a_0),
        "window A committed element bounds must stay scoped to window A"
    );
    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window_b, element_b),
        Some(expected_b_0),
        "window B committed element bounds must stay scoped to window B"
    );
    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window_b, element_a),
        None,
        "window B must not resolve window A element bounds"
    );
    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window_a, element_b),
        None,
        "window A must not resolve window B element bounds"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_uses_window_local_anchor_for_overlay_placement() {
    let window_a = AppWindowId::default();
    let window_b = AppWindowId::from(KeyData::from_ffi(43));
    let panel_a = PanelKey::new("demo.public.declarative.overlay-anchor.a");
    let panel_b = PanelKey::new("demo.public.declarative.overlay-anchor.b");
    let bounds_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(240.0)),
    );
    let bounds_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(480.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    setup_single_panel_window(&mut app, window_a, &panel_a, None);
    setup_single_panel_window(&mut app, window_b, &panel_b, None);

    let mut services = FakeTextService;
    let mut ui_a: UiTree<TestHost> = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b: UiTree<TestHost> = UiTree::new();
    ui_b.set_window(window_b);

    let anchor_a = Arc::new(Mutex::new(None));
    let anchor_b = Arc::new(Mutex::new(None));

    let render_anchor_a =
        |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut FakeTextService| {
            let panel = panel_a.clone();
            let anchor = anchor_a.clone();
            declarative::render_root(
                ui,
                app,
                services,
                window_a,
                bounds_a,
                "public-declarative-dock-host-overlay-anchor-a",
                move |cx| {
                    let child = cx.semantics_with_id(absolute_anchor_props(Px(240.0), Px(20.0)), {
                        let anchor = anchor.clone();
                        move |cx, id| {
                            *anchor.lock().expect("window a anchor mutex") = Some(id);
                            vec![cx.text("a")]
                        }
                    });
                    vec![dock_space_element(
                        cx,
                        window_a,
                        DockSpaceElementOptions::default(),
                        [dock_panel_element(panel.clone(), child)],
                    )]
                },
            )
        };
    let render_anchor_b =
        |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut FakeTextService| {
            let panel = panel_b.clone();
            let anchor = anchor_b.clone();
            declarative::render_root(
                ui,
                app,
                services,
                window_b,
                bounds_b,
                "public-declarative-dock-host-overlay-anchor-b",
                move |cx| {
                    let child = cx.semantics_with_id(absolute_anchor_props(Px(40.0), Px(20.0)), {
                        let anchor = anchor.clone();
                        move |cx, id| {
                            *anchor.lock().expect("window b anchor mutex") = Some(id);
                            vec![cx.text("b")]
                        }
                    });
                    vec![dock_space_element(
                        cx,
                        window_b,
                        DockSpaceElementOptions::default(),
                        [dock_panel_element(panel.clone(), child)],
                    )]
                },
            )
        };

    let root_a = render_anchor_a(&mut ui_a, &mut app, &mut services);
    ui_a.set_root(root_a);
    let root_b = render_anchor_b(&mut ui_b, &mut app, &mut services);
    ui_b.set_root(root_b);
    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    let anchor_a = anchor_a
        .lock()
        .expect("window a anchor mutex")
        .expect("expected window a anchor id");
    let anchor_b = anchor_b
        .lock()
        .expect("window b anchor mutex")
        .expect("expected window b anchor id");
    let expected_a_anchor =
        fret_ui::elements::current_bounds_for_element(&mut app, window_a, anchor_a)
            .expect("expected window a anchor bounds");
    let expected_b_anchor =
        fret_ui::elements::current_bounds_for_element(&mut app, window_b, anchor_b)
            .expect("expected window b anchor bounds");
    assert_ne!(
        expected_a_anchor, expected_b_anchor,
        "test setup should make same-kind anchors distinguishable across windows"
    );

    app.advance_frame();
    let root_a = render_anchor_a(&mut ui_a, &mut app, &mut services);
    ui_a.set_root(root_a);
    let root_b = render_anchor_b(&mut ui_b, &mut app, &mut services);
    ui_b.set_root(root_b);
    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    let desired = Size::new(Px(80.0), Px(24.0));
    let layout_out = app.models_mut().insert(AnchoredPanelLayout {
        rect: Rect::default(),
        side: Side::Bottom,
        align: Align::End,
        arrow: None,
    });
    let overlay_root = declarative::render_root(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds_b,
        "public-declarative-dock-host-overlay-anchor-layer-b",
        |cx| {
            assert_eq!(
                cx.last_bounds_for_element(anchor_b),
                Some(expected_b_anchor),
                "overlay policy should read window B's committed anchor bounds"
            );
            assert_eq!(
                cx.last_bounds_for_element(anchor_a),
                None,
                "overlay policy in window B must not read window A's anchor bounds"
            );
            vec![cx.anchored_props(
                AnchoredProps {
                    anchor: Rect::default(),
                    anchor_element: Some(anchor_b.0),
                    side: Side::Bottom,
                    align: Align::End,
                    side_offset: Px(6.0),
                    options: AnchoredPanelOptions::default(),
                    layout_out: Some(layout_out.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: fret_ui::element::Length::Px(desired.width),
                                    height: fret_ui::element::Length::Px(desired.height),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    )]
                },
            )]
        },
    );
    let _overlay_layer = ui_b.push_overlay_root(overlay_root, false);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    let expected = fret_ui::overlay_placement::anchored_panel_layout_sized(
        bounds_b,
        expected_b_anchor,
        desired,
        Px(6.0),
        Side::Bottom,
        Align::End,
        AnchoredPanelOptions::default(),
    );
    assert_eq!(
        app.models().get_copied(&layout_out),
        Some(expected),
        "declarative overlay placement should use the window-local anchor bounds"
    );
}

#[test]
fn public_declarative_registry_binds_viewport_panel_element_when_registry_returns_one() {
    struct ViewportOverlayRegistry;

    impl DockPanelElementRegistry<TestHost> for ViewportOverlayRegistry {
        fn render_panel(
            &self,
            cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            let props = fret_ui::element::PointerRegionProps {
                layout: fill_semantics_props().layout,
                enabled: true,
                capture_phase_pointer_moves: false,
            };
            Some(cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(
                    |host: &mut dyn fret_ui::action::UiPointerActionHost, cx, _down| {
                        host.request_focus(cx.target);
                        true
                    },
                ));
                vec![cx.text("viewport overlay")]
            }))
        }
    }

    let window = AppWindowId::default();
    let panel = PanelKey::new("demo.public.declarative.viewport.with-element");
    let target = fret_core::RenderTargetId::from(KeyData::from_ffi(41));

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    setup_single_panel_window(&mut app, window, &panel, Some(target));
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(ViewportOverlayRegistry));
        },
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(180.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-viewport-with-element",
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

    let node = app
        .global::<DockPanelContentService>()
        .and_then(|content| content.get(window, &panel))
        .expect("expected viewport panel element returned by registry to be bound");
    let rect = ui
        .debug_node_bounds(node)
        .expect("expected viewport panel element bounds");
    assert!(rect.size.width.0 > 0.0 && rect.size.height.0 > 0.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(rect.origin.x.0 + 1.0), Px(rect.origin.y.0 + 1.0)),
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
        "registry-provided viewport panel element should remain focusable and event-reachable"
    );
}

#[test]
fn public_declarative_registry_falls_back_to_placeholder_for_missing_non_viewport_panel_ui() {
    struct AlwaysMissingRegistry;

    impl DockPanelElementRegistry<TestHost> for AlwaysMissingRegistry {
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
    let panel = PanelKey::new("demo.public.declarative.missing.non-viewport");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    setup_single_panel_window(&mut app, window, &panel, None);
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(AlwaysMissingRegistry));
        },
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-missing-non-viewport",
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

    let node = app
        .global::<DockPanelContentService>()
        .and_then(|content| content.get(window, &panel))
        .expect("expected placeholder element for missing non-viewport panel UI");
    let rect = ui
        .debug_node_bounds(node)
        .expect("expected placeholder element bounds");
    assert!(
        rect.size.width.0 > 0.0 && rect.size.height.0 > 0.0,
        "placeholder should be laid out as the active dock panel content"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Text { order, .. } if *order == fret_core::DrawOrder(0)
        )),
        "missing non-viewport panel placeholder should paint explanatory text, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_paints_tab_chrome() {
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
    let panel = PanelKey::new("demo.public.declarative.tab.chrome");

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
            title: "Chrome".to_string(),
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
        "public-declarative-dock-host-tab-chrome",
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad {
                order,
                rect,
                ..
            } if *order == fret_core::DrawOrder(1) && *rect == tab_bar
        )),
        "expected public declarative dock host to paint the tab bar chrome, got: {:?}",
        scene.ops()
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad {
                order,
                rect,
                ..
            } if *order == fret_core::DrawOrder(3)
                && rect.origin.y.0 >= tab_bar.origin.y.0
                && rect.origin.y.0 + rect.size.height.0 <= tab_bar.origin.y.0 + tab_bar.size.height.0
        )),
        "expected public declarative dock host to paint the active tab underline, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_paints_tab_details() {
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
        let panels: Vec<PanelKey> = (0..8)
            .map(|index| PanelKey::new(format!("demo.public.declarative.tab-detail.{index}")))
            .collect();
        for (index, panel) in panels.iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("Tab Detail {index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: panels,
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(120.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tab-details",
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
            SceneOp::Text { order, .. } if *order == fret_core::DrawOrder(4)
        )),
        "expected public declarative dock host to paint tab title text, got: {:?}",
        scene.ops()
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Text { order, .. } if *order == fret_core::DrawOrder(6)
        )),
        "expected public declarative dock host to paint active tab close text, got: {:?}",
        scene.ops()
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Text { order, .. } if *order == fret_core::DrawOrder(11)
        )),
        "expected public declarative dock host to paint overflow button text, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_closes_tab_from_close_affordance() {
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
    let panel = PanelKey::new("demo.public.declarative.close.active");
    let sibling = PanelKey::new("demo.public.declarative.close.sibling");

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
            title: "Close Active".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&sibling, || crate::DockPanel {
            title: "Close Sibling".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone(), sibling.clone()],
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
        "public-declarative-dock-host-tab-close",
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
    let theme = fret_ui::Theme::global(&app).snapshot();
    let tab_rect =
        super::super::tab_bar_geometry::TabBarGeometry::fixed(tab_bar, 2).tab_rect(0, Px(0.0));
    let close_rect = super::super::hit_test::tab_close_rect(theme, tab_rect);
    let close_pos = Point::new(
        Px(close_rect.origin.x.0 + close_rect.size.width.0 * 0.5),
        Px(close_rect.origin.y.0 + close_rect.size.height.0 * 0.5),
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
            Effect::Dock(DockOp::ClosePanel {
                window: effect_window,
                panel: effect_panel,
            }) if *effect_window == window && *effect_panel == panel
        )),
        "expected declarative dock host to emit ClosePanel for the active tab close affordance, got: {effects:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_handles_overflow_menu_close() {
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
        let panels: Vec<PanelKey> = (0..12)
            .map(|index| PanelKey::new(format!("demo.public.declarative.overflow.{index}")))
            .collect();
        for (index, panel) in panels.iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("Overflow {index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: panels,
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(120.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-overflow-menu",
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
    let theme = fret_ui::Theme::global(&app).snapshot();
    let items = super::super::tab_overflow::compute_tab_overflow_menu_items(
        theme.clone(),
        tab_bar,
        12,
        None,
        Px(0.0),
        0,
    );
    assert!(
        items.len() >= 2,
        "expected overflow menu items to include at least one non-active tab, got: {items:?}"
    );
    let tab_ix_to_close = *items.get(1).expect("items has at least 2 rows");
    let item_count = items.len();

    let button_rect = super::super::tab_overflow::tab_overflow_button_rect(theme.clone(), tab_bar);
    let open_pos = Point::new(
        Px(button_rect.origin.x.0 + button_rect.size.width.0 * 0.5),
        Px(button_rect.origin.y.0 + button_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: open_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let menu_rect =
        super::super::tab_overflow::tab_overflow_menu_rect(theme.clone(), tab_bar, item_count);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, rect, .. }
                if *order == fret_core::DrawOrder(100) && *rect == menu_rect
        )),
        "expected declarative overflow menu to be painted after opening it, got: {:?}",
        scene.ops()
    );

    let row_rect =
        super::super::tab_overflow::overflow_menu_row_rect(menu_rect, tab_bar, Px(0.0), 1);
    let close_rect = super::super::tab_overflow::overflow_menu_close_rect(theme.clone(), row_rect);
    let close_pos = Point::new(
        Px(close_rect.origin.x.0 + close_rect.size.width.0 * 0.5),
        Px(close_rect.origin.y.0 + close_rect.size.height.0 * 0.5),
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

    let effects = app.take_effects();
    let close_panel = effects.iter().find_map(|effect| match effect {
        Effect::Dock(DockOp::ClosePanel { panel, .. }) => Some(panel.kind.0.clone()),
        _ => None,
    });
    let expected = format!("demo.public.declarative.overflow.{tab_ix_to_close}");
    assert_eq!(
        close_panel.as_deref(),
        Some(expected.as_str()),
        "expected ClosePanel for declarative overflow menu row 1, got: {effects:?}"
    );
    assert!(
        !effects
            .iter()
            .any(|effect| matches!(effect, Effect::Dock(DockOp::SetActiveTab { .. }))),
        "expected declarative overflow menu close to not activate tabs, got: {effects:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_activates_overflow_menu_row() {
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
    let tabs_node = app.with_global_mut(DockManager::default, |dock, _app| {
        let panels: Vec<PanelKey> = (0..12)
            .map(|index| {
                PanelKey::new(format!("demo.public.declarative.overflow.activate.{index}"))
            })
            .collect();
        for (index, panel) in panels.iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("Overflow Activate {index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: panels,
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
        tabs
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(120.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-overflow-menu-activate",
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
    let theme = fret_ui::Theme::global(&app).snapshot();
    let items = super::super::tab_overflow::compute_tab_overflow_menu_items(
        theme.clone(),
        tab_bar,
        12,
        None,
        Px(0.0),
        0,
    );
    assert!(
        items.len() >= 2,
        "expected overflow menu items to include at least one non-active tab, got: {items:?}"
    );
    let tab_ix_to_activate = *items.get(1).expect("items has at least 2 rows");
    let item_count = items.len();

    let button_rect = super::super::tab_overflow::tab_overflow_button_rect(theme.clone(), tab_bar);
    let open_pos = Point::new(
        Px(button_rect.origin.x.0 + button_rect.size.width.0 * 0.5),
        Px(button_rect.origin.y.0 + button_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: open_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let menu_rect =
        super::super::tab_overflow::tab_overflow_menu_rect(theme.clone(), tab_bar, item_count);
    let row_rect =
        super::super::tab_overflow::overflow_menu_row_rect(menu_rect, tab_bar, Px(0.0), 1);
    let activate_pos = Point::new(
        Px(row_rect.origin.x.0 + 8.0),
        Px(row_rect.origin.y.0 + row_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: activate_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Dock(DockOp::SetActiveTab { tabs, active })
                if *tabs == tabs_node && *active == tab_ix_to_activate
        )),
        "expected SetActiveTab for declarative overflow menu row activation, got: {effects:?}"
    );
    assert!(
        !effects
            .iter()
            .any(|effect| matches!(effect, Effect::Dock(DockOp::ClosePanel { .. }))),
        "expected declarative overflow menu row activation to not close tabs, got: {effects:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_scrolls_overflow_menu_with_wheel() {
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
    let tabs_node = app.with_global_mut(DockManager::default, |dock, _app| {
        let panels: Vec<PanelKey> = (0..24)
            .map(|index| PanelKey::new(format!("demo.public.declarative.overflow.wheel.{index}")))
            .collect();
        for (index, panel) in panels.iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("Overflow Wheel {index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: panels,
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
        tabs
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(160.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-overflow-menu-wheel",
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
    let theme = fret_ui::Theme::global(&app).snapshot();
    let items = super::super::tab_overflow::compute_tab_overflow_menu_items(
        theme.clone(),
        tab_bar,
        24,
        None,
        Px(0.0),
        0,
    );
    let item_count = items.len();
    let row_h = super::super::tab_overflow::overflow_menu_row_height(tab_bar);
    let scroll = Px(row_h.0 * 2.0);
    assert!(
        super::super::tab_overflow::overflow_menu_max_scroll(tab_bar, item_count).0 >= scroll.0,
        "expected overflow menu to have at least two rows of scroll, got {item_count} items"
    );

    let button_rect = super::super::tab_overflow::tab_overflow_button_rect(theme.clone(), tab_bar);
    let open_pos = Point::new(
        Px(button_rect.origin.x.0 + button_rect.size.width.0 * 0.5),
        Px(button_rect.origin.y.0 + button_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: open_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let menu_rect =
        super::super::tab_overflow::tab_overflow_menu_rect(theme.clone(), tab_bar, item_count);
    let wheel_pos = Point::new(
        Px(menu_rect.origin.x.0 + 8.0),
        Px(menu_rect.origin.y.0 + row_h.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: Point::new(Px(0.0), Px(-scroll.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let row_rect =
        super::super::tab_overflow::overflow_menu_row_rect(menu_rect, tab_bar, scroll, 2);
    let activate_pos = Point::new(
        Px(row_rect.origin.x.0 + 8.0),
        Px(row_rect.origin.y.0 + row_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: activate_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let expected_tab = *items.get(2).expect("expected scrolled row to exist");
    let effects = app.take_effects();
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Dock(DockOp::SetActiveTab { tabs, active })
                if *tabs == tabs_node && *active == expected_tab
        )),
        "expected SetActiveTab for row exposed by declarative overflow menu wheel scrolling, got: {effects:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_scrolls_tab_strip_with_wheel() {
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
    let panel_0 = PanelKey::new("demo.public.declarative.tab-strip-wheel.0");
    let panel_1 = PanelKey::new("demo.public.declarative.tab-strip-wheel.1");

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
        let panels: Vec<PanelKey> = (0..12)
            .map(|index| PanelKey::new(format!("demo.public.declarative.tab-strip-wheel.{index}")))
            .collect();
        for (index, panel) in panels.iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("T{index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: panels,
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(160.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tab-strip-wheel",
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
    let theme = fret_ui::Theme::global(&app).snapshot();
    let strip =
        super::super::tab_overflow::tab_strip_rect_with_overflow_button(theme.clone(), tab_bar);
    let scroll = Px(120.0);

    let wheel_pos = Point::new(Px(strip.origin.x.0 + 8.0), Px(strip.origin.y.0 + 8.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: Point::new(Px(0.0), Px(-scroll.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );
    let _ = app.take_effects();

    let scrolled_tab_rect =
        super::super::tab_bar_geometry::TabBarGeometry::fixed(strip, 12).tab_rect(1, scroll);
    let close_rect = super::super::hit_test::tab_close_rect(theme, scrolled_tab_rect);
    let close_pos = Point::new(
        Px(close_rect.origin.x.0 + close_rect.size.width.0 * 0.5),
        Px(close_rect.origin.y.0 + close_rect.size.height.0 * 0.5),
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
            Effect::Dock(DockOp::ClosePanel { panel, .. }) if *panel == panel_1
        )),
        "expected declarative tab-strip wheel scroll to make panel 1 close hit-testable, got: {effects:?}"
    );
    assert!(
        !effects.iter().any(|effect| matches!(
            effect,
            Effect::Dock(DockOp::ClosePanel { panel, .. }) if *panel == panel_0
        )),
        "expected scrolled close hit-test to not close panel 0, got: {effects:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_hovers_tab() {
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
        let panel_0 = PanelKey::new("demo.public.declarative.tab-hover.0");
        let panel_1 = PanelKey::new("demo.public.declarative.tab-hover.1");
        for (index, panel) in [panel_0.clone(), panel_1.clone()].iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("Hover {index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_0, panel_1],
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
        "public-declarative-dock-host-tab-hover",
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
        super::super::tab_bar_geometry::TabBarGeometry::fixed(tab_bar, 2).tab_rect(1, Px(0.0));
    let hover_pos = Point::new(
        Px(tab_rect.origin.x.0 + tab_rect.size.width.0 * 0.5),
        Px(tab_rect.origin.y.0 + tab_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: hover_pos,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let theme = fret_ui::Theme::global(&app).snapshot();
    let expected_hover = theme.color_token("accent");
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
                && *rect == tab_rect
                && matches!(background.paint, fret_core::Paint::Solid(color) if color == expected_hover)
        )),
        "expected declarative dock host to paint hovered tab background, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_hovers_tab_overflow_button() {
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
        let panels: Vec<PanelKey> = (0..12)
            .map(|index| PanelKey::new(format!("demo.public.declarative.tab-hover.{index}")))
            .collect();
        for (index, panel) in panels.iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("Hover {index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: panels,
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(160.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tab-overflow-hover",
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
    let theme = fret_ui::Theme::global(&app).snapshot();
    let button_rect = super::super::tab_overflow::tab_overflow_button_rect(theme, tab_bar);
    let hover_pos = Point::new(
        Px(button_rect.origin.x.0 + button_rect.size.width.0 * 0.5),
        Px(button_rect.origin.y.0 + button_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: hover_pos,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, rect, .. }
                if *order == fret_core::DrawOrder(10) && *rect == button_rect
        )),
        "expected declarative dock host to paint overflow button hover background, got: {:?}",
        scene.ops()
    );
}

#[test]
fn public_declarative_dock_space_entry_point_hovers_overflow_menu_row() {
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
        let panels: Vec<PanelKey> = (0..12)
            .map(|index| PanelKey::new(format!("demo.public.declarative.menu-hover.{index}")))
            .collect();
        for (index, panel) in panels.iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("Menu Hover {index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: panels,
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(160.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-overflow-menu-row-hover",
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
    let theme = fret_ui::Theme::global(&app).snapshot();
    let items = super::super::tab_overflow::compute_tab_overflow_menu_items(
        theme.clone(),
        tab_bar,
        12,
        None,
        Px(0.0),
        0,
    );
    let item_count = items.len();
    assert!(item_count >= 2, "expected overflow menu items");

    let button_rect = super::super::tab_overflow::tab_overflow_button_rect(theme.clone(), tab_bar);
    let open_pos = Point::new(
        Px(button_rect.origin.x.0 + button_rect.size.width.0 * 0.5),
        Px(button_rect.origin.y.0 + button_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: open_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let menu_rect =
        super::super::tab_overflow::tab_overflow_menu_rect(theme.clone(), tab_bar, item_count);
    let row_rect =
        super::super::tab_overflow::overflow_menu_row_rect(menu_rect, tab_bar, Px(0.0), 1);
    let hover_pos = Point::new(
        Px(row_rect.origin.x.0 + 8.0),
        Px(row_rect.origin.y.0 + row_rect.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: hover_pos,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Unknown,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            SceneOp::Quad { order, rect, .. }
                if *order == fret_core::DrawOrder(101) && *rect == row_rect
        )),
        "expected declarative dock host to paint overflow menu row hover background, got: {:?}",
        scene.ops()
    );
}

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
        super::super::tab_bar_geometry::TabBarGeometry::fixed(tab_bar, 2).tab_rect(0, Px(0.0));
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
        super::super::tab_bar_geometry::TabBarGeometry::fixed(tab_bar, 1).tab_rect(0, Px(0.0));
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
        super::super::tab_bar_geometry::TabBarGeometry::fixed(tab_bar, 1).tab_rect(0, Px(0.0));
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
    let close_size = super::super::consts::DOCK_FLOATING_CLOSE_SIZE.0;
    let close_pad = super::super::consts::DOCK_FLOATING_BORDER.0.max(4.0);
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

    let close_size = super::super::consts::DOCK_FLOATING_CLOSE_SIZE.0;
    let close_pos = Point::new(
        Px(floating_rect.origin.x.0 + floating_rect.size.width.0 - 8.0 - close_size * 0.5),
        Px(floating_rect.origin.y.0
            + 1.0
            + (super::super::consts::DOCK_FLOATING_TITLE_H.0 - close_size) * 0.5
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
        super::super::manager::ActivatePanelOptions { focus: true },
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

struct DeclarativeTabDropHarness {
    ui: UiTree<TestHost>,
    app: TestHost,
    services: FakeTextService,
    window: AppWindowId,
    tabs_node: fret_core::DockNodeId,
    dragged_panel: PanelKey,
    bounds: Rect,
}

impl DeclarativeTabDropHarness {
    fn single_tabs(tab_count: usize, bounds: Rect) -> Self {
        Self::single_tabs_with_dragged_index(tab_count, bounds, 0)
    }

    fn single_tabs_with_dragged_index(
        tab_count: usize,
        bounds: Rect,
        dragged_index: usize,
    ) -> Self {
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

        let (tabs_node, dragged_panel) = app.with_global_mut(DockManager::default, |dock, _app| {
            let panels: Vec<PanelKey> = (0..tab_count)
                .map(|index| PanelKey::new(format!("demo.public.declarative.tab-drop.{index}")))
                .collect();
            let dragged_panel = panels
                .get(dragged_index)
                .cloned()
                .expect("dragged index must reference an existing tab");
            for (index, panel) in panels.iter().enumerate() {
                dock.ensure_panel(panel, || crate::DockPanel {
                    title: format!("Panel {index}"),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                });
            }
            let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: panels,
                active: 0,
            });
            dock.workspace.graph.set_window_root(window, tabs);
            (tabs, dragged_panel)
        });

        let mut services = FakeTextService;
        let root = declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "public-declarative-dock-host-tab-drop",
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

        Self {
            ui,
            app,
            services,
            window,
            tabs_node,
            dragged_panel,
            bounds,
        }
    }

    fn begin_drag(&mut self) {
        self.app.begin_cross_window_drag_with_kind(
            fret_core::PointerId(0),
            DRAG_KIND_DOCK_PANEL,
            self.window,
            Point::new(Px(24.0), Px(12.0)),
            DockPanelDragPayload {
                panel: self.dragged_panel.clone(),
                grab_offset: Point::new(Px(0.0), Px(0.0)),
                tear_off_requested: false,
                tear_off_requested_at_tick: None,
                tear_off_oob_start_frame: None,
                dock_previews_enabled: true,
            },
        );
        if let Some(drag) = self.app.drag_mut(fret_core::PointerId(0)) {
            drag.dragging = true;
            drag.current_window = self.window;
        }
        let _ = self.app.take_effects();
    }

    fn drop_at(&mut self, position: Point) -> DockOp {
        for kind in [InternalDragKind::Over, InternalDragKind::Drop] {
            self.ui.dispatch_event(
                &mut self.app,
                &mut self.services,
                &Event::InternalDrag(InternalDragEvent {
                    position,
                    kind,
                    modifiers: Modifiers::default(),
                    pointer_id: fret_core::PointerId(0),
                }),
            );
        }
        self.app
            .take_effects()
            .into_iter()
            .find_map(|effect| match effect {
                Effect::Dock(op) => Some(op),
                _ => None,
            })
            .expect("expected declarative tab drop to emit a Dock op")
    }

    fn tab_drop_position(&self, tab_index: usize, fraction: f32) -> Point {
        let tab_count = self
            .app
            .global::<DockManager>()
            .and_then(|dock| match dock.workspace.graph.node(self.tabs_node) {
                Some(DockNode::Tabs { tabs, .. }) => Some(tabs.len()),
                _ => None,
            })
            .expect("expected target tabs node");
        let theme = fret_ui::Theme::global(&self.app).snapshot();
        let (_chrome, dock_bounds) = dock_space_regions(self.bounds);
        let (tab_bar, _content) = split_tab_bar(dock_bounds);
        let tab_width = super::super::tab_bar_geometry::dock_tab_width_for_title(
            theme.clone(),
            Px(240.0),
            true,
        );
        let tab_widths: Arc<[Px]> = vec![tab_width; tab_count].into();
        let candidate = super::super::tab_bar_kernel::compute_tab_bar_overflow_candidate_geometry(
            theme,
            tab_bar,
            tab_count,
            Some(&tab_widths),
        );
        let geom = if candidate.overflows {
            candidate.geom
        } else {
            super::super::tab_bar_geometry::TabBarGeometry::variable(tab_bar, tab_widths)
        };
        let tab_rect = geom.tab_rect(tab_index, Px(0.0));
        Point::new(
            Px(tab_rect.origin.x.0 + tab_rect.size.width.0 * fraction),
            Px(tab_rect.origin.y.0 + tab_rect.size.height.0 * 0.5),
        )
    }

    fn tab_bar_reserved_header_position(&self, tab_count: usize) -> Point {
        let theme = fret_ui::Theme::global(&self.app).snapshot();
        let (_chrome, dock_bounds) = dock_space_regions(self.bounds);
        let (tab_bar, _content) = split_tab_bar(dock_bounds);
        let candidate = super::super::tab_bar_kernel::compute_tab_bar_overflow_candidate_geometry(
            theme, tab_bar, tab_count, None,
        );
        assert!(candidate.overflows, "expected tab strip to overflow");
        let strip_end = candidate.strip_rect.origin.x.0 + candidate.strip_rect.size.width.0;
        let button_start = candidate.overflow_button_rect.origin.x.0;
        assert!(
            button_start > strip_end,
            "expected reserved header space before overflow button"
        );
        Point::new(
            Px((strip_end + button_start) * 0.5),
            Px(tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5),
        )
    }
}

#[test]
fn public_declarative_dock_space_entry_point_tab_drop_uses_over_tab_halves_for_insert_index() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut left = DeclarativeTabDropHarness::single_tabs(3, bounds);
    left.begin_drag();
    let op = left.drop_at(left.tab_drop_position(1, 0.25));
    assert!(
        left.app.drag(fret_core::PointerId(0)).is_none(),
        "expected declarative tab drop to end the drag session"
    );
    assert!(
        matches!(
            op,
            DockOp::MovePanel {
                target_tabs,
                zone: DropZone::Center,
                insert_index: Some(1),
                ..
            } if target_tabs == left.tabs_node
        ),
        "expected left half of tab 1 to insert before tab 1, got: {op:?}"
    );

    let mut right = DeclarativeTabDropHarness::single_tabs(3, bounds);
    right.begin_drag();
    let op = right.drop_at(right.tab_drop_position(1, 0.75));
    assert!(
        matches!(
            op,
            DockOp::MovePanel {
                target_tabs,
                zone: DropZone::Center,
                insert_index: Some(2),
                ..
            } if target_tabs == right.tabs_node
        ),
        "expected right half of tab 1 to insert after tab 1, got: {op:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_tab_drop_reorders_tabs_when_move_op_is_applied() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut harness = DeclarativeTabDropHarness::single_tabs_with_dragged_index(4, bounds, 3);
    let source_panel = harness.dragged_panel.clone();
    harness.begin_drag();
    let op = harness.drop_at(harness.tab_drop_position(1, 0.75));

    harness
        .app
        .with_global_mut(DockManager::default, |dock, _app| {
            assert!(
                dock.workspace
                    .graph
                    .apply_op_checked(&op)
                    .expect("apply must succeed"),
                "expected emitted tab-drop op to mutate the graph"
            );
            let Some(DockNode::Tabs { tabs, .. }) = dock.workspace.graph.node(harness.tabs_node)
            else {
                panic!("expected target tabs node to remain tabs");
            };
            assert_eq!(
                tabs,
                &vec![
                    PanelKey::new("demo.public.declarative.tab-drop.0"),
                    PanelKey::new("demo.public.declarative.tab-drop.1"),
                    source_panel,
                    PanelKey::new("demo.public.declarative.tab-drop.2"),
                ],
                "expected applied declarative tab drop to insert dragged panel after tab 1"
            );
        });
}

#[test]
fn public_declarative_dock_space_entry_point_tab_drop_reserved_overflow_header_inserts_at_end() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(260.0), Px(180.0)),
    );
    let tab_count = 10;
    let mut harness = DeclarativeTabDropHarness::single_tabs(tab_count, bounds);
    let position = harness.tab_bar_reserved_header_position(tab_count);
    harness.begin_drag();
    let op = harness.drop_at(position);

    assert!(
        matches!(
            op,
            DockOp::MovePanel {
                target_tabs,
                zone: DropZone::Center,
                insert_index: Some(ix),
                ..
            } if target_tabs == harness.tabs_node && ix == tab_count
        ),
        "expected reserved overflow header space to insert at tab_count={tab_count}, got: {op:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_auto_scrolls_tab_bar_during_dock_drag() {
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
    let tabs_node = app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs: Vec<PanelKey> = (0..30)
            .map(|index| PanelKey::new(format!("demo.public.declarative.auto-scroll.{index}")))
            .collect();
        for (index, panel) in tabs.iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("T{index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs_node = dock
            .workspace
            .graph
            .insert_node(DockNode::Tabs { tabs, active: 0 });
        dock.workspace.graph.set_window_root(window, tabs_node);
        dock.ensure_panel(
            &PanelKey::new("demo.public.declarative.auto-scroll.dragged"),
            || crate::DockPanel {
                title: "Dragged".to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            },
        );
        tabs_node
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
        "public-declarative-dock-host-tab-bar-drag-auto-scroll",
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

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("demo.public.declarative.auto-scroll.dragged"),
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

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let pos_right = Point::new(
        Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - 2.0),
        Px(tab_bar.origin.y.0 + 6.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::InternalDrag(InternalDragEvent {
            position: pos_right,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let first_ix = match app
        .global::<DockManager>()
        .and_then(|dock| dock.presentation.hover.clone())
    {
        Some(DockDropTarget::Dock(target)) => {
            assert_eq!(target.tabs, tabs_node);
            target.insert_index.expect("expected a tab insert index")
        }
        other => panic!("expected declarative tab-bar dock hover target, got: {other:?}"),
    };

    let mut ix_after_scroll = first_ix;
    for _ in 0..6 {
        app.advance_frame();
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::InternalDrag(InternalDragEvent {
                position: pos_right,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
        if let Some(DockDropTarget::Dock(target)) = app
            .global::<DockManager>()
            .and_then(|dock| dock.presentation.hover.clone())
        {
            ix_after_scroll = target.insert_index.expect("expected insert index");
        }
    }

    assert!(
        ix_after_scroll > first_ix,
        "expected declarative auto-scroll at the right edge to increase the insert index, before={first_ix}, after={ix_after_scroll}",
    );
}

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
            Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel: requested, .. })
                if *requested == panel
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
            Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel: requested, .. })
                if *requested == panel
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
        dock.set_viewport_content_rect(
            window,
            stale_target,
            Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(2.0), Px(2.0))),
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
