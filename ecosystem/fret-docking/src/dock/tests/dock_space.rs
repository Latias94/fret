use super::*;
use crate::DockViewportLayout;
use std::{collections::HashMap, sync::Mutex};

use fret_runtime::CreateWindowKind;

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

mod drag_drop;
mod floating;
mod preview;
mod tab_drop;
mod viewport;
