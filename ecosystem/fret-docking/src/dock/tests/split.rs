use super::*;

#[test]
fn drag_update_fractions_updates_two_panel_split() {
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(300.0), Px(40.0)));
    let fractions = vec![0.5, 0.5];
    let next = split_geometry::drag_update_fractions(
        fret_core::Axis::Horizontal,
        bounds,
        2,
        &fractions,
        0,
        Px(0.0),
        Px(6.0),
        &[],
        0.0,
        Point::new(Px(200.0), Px(20.0)),
    )
    .expect("expected drag to update fractions");
    assert!(next[0] > 0.5, "expected left to grow, got {next:?}");
}

#[test]
fn drag_update_adjacent_fractions_updates_only_adjacent_panels_in_three_panel_split() {
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(600.0), Px(40.0)));
    let fractions = vec![0.33, 0.34, 0.33];
    let min_px = [Px(120.0), Px(120.0), Px(120.0)];

    let layout0 = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        bounds,
        3,
        &fractions,
        Px(0.0),
        Px(10.0),
        &min_px,
    );
    let center0 = layout0.handle_centers[0];

    // Try to drag far enough that the middle panel would hit its min size.
    let next = split_geometry::drag_update_adjacent_fractions(
        fret_core::Axis::Horizontal,
        bounds,
        3,
        &fractions,
        0,
        Px(0.0),
        Px(10.0),
        &min_px,
        0.0,
        Point::new(Px(center0 + 250.0), Px(20.0)),
    )
    .expect("expected drag to update fractions");

    let layout1 = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        bounds,
        3,
        &next,
        Px(0.0),
        Px(10.0),
        &min_px,
    );

    assert!(
        (layout1.sizes[2] - layout0.sizes[2]).abs() < 0.01,
        "expected far-right panel unchanged, before={:?}, after={:?}",
        layout0.sizes,
        layout1.sizes
    );
    assert!(
        (layout1.sizes[1] - 120.0).abs() < 0.01,
        "expected middle panel clamped to min, got {:?}",
        layout1.sizes
    );
    assert!(
        layout1.sizes[0] > layout0.sizes[0] + 1.0,
        "expected left panel to grow, before={:?}, after={:?}",
        layout0.sizes,
        layout1.sizes
    );
}

#[test]
fn drag_update_adjacent_fractions_handle1_keeps_left_panel_stable() {
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(600.0), Px(40.0)));
    let fractions = vec![0.33, 0.34, 0.33];
    let min_px = [Px(120.0), Px(120.0), Px(120.0)];

    let layout0 = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        bounds,
        3,
        &fractions,
        Px(0.0),
        Px(10.0),
        &min_px,
    );
    let center1 = layout0.handle_centers[1];

    // Drag handle 1 left: shrink the middle panel, grow the right panel.
    let next = split_geometry::drag_update_adjacent_fractions(
        fret_core::Axis::Horizontal,
        bounds,
        3,
        &fractions,
        1,
        Px(0.0),
        Px(10.0),
        &min_px,
        0.0,
        Point::new(Px(center1 - 80.0), Px(20.0)),
    )
    .expect("expected drag to update fractions");

    let layout1 = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        bounds,
        3,
        &next,
        Px(0.0),
        Px(10.0),
        &min_px,
    );

    assert!(
        (layout1.sizes[0] - layout0.sizes[0]).abs() < 0.01,
        "expected left panel unchanged, before={:?}, after={:?}",
        layout0.sizes,
        layout1.sizes
    );
    assert!(
        layout1.sizes[2] > layout0.sizes[2] + 1.0,
        "expected right panel to grow, before={:?}, after={:?}",
        layout0.sizes,
        layout1.sizes
    );
}

#[test]
fn nary_split_handle_hit_test_reports_correct_handle_index() {
    let mut graph = DockGraph::new();

    let a = graph.insert_node(DockNode::Tabs {
        tabs: vec![PanelKey::new("test.a")],
        active: 0,
    });
    let b = graph.insert_node(DockNode::Tabs {
        tabs: vec![PanelKey::new("test.b")],
        active: 0,
    });
    let c = graph.insert_node(DockNode::Tabs {
        tabs: vec![PanelKey::new("test.c")],
        active: 0,
    });

    let fractions = vec![0.33, 0.34, 0.33];
    let split = graph.insert_node(DockNode::Split {
        axis: fret_core::Axis::Horizontal,
        children: vec![a, b, c],
        fractions: fractions.clone(),
    });

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(900.0), Px(80.0)));
    let settings = fret_runtime::DockingInteractionSettings::default();
    let layout = compute_layout_map(
        &graph,
        split,
        bounds,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
    );

    let split_bounds = layout.get(&split).copied().expect("expected split bounds");
    let computed = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        split_bounds,
        3,
        &fractions,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &[],
    );
    assert_eq!(computed.handle_hit_rects.len(), 2);

    for expected_handle_ix in 0..2 {
        let rect = computed.handle_hit_rects[expected_handle_ix];
        let pos = Point::new(
            Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
            Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
        );

        let handle = hit_test_split_handle(
            &graph,
            &layout,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
            pos,
            |_split, _axis, _children| Vec::new(),
        )
        .expect("expected split handle hit");

        assert_eq!(handle.split, split);
        assert_eq!(handle.axis, fret_core::Axis::Horizontal);
        assert_eq!(handle.handle_ix, expected_handle_ix);
    }
}
#[test]
fn drag_update_fractions_handles_nan_bounds() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(f32::NAN), Px(300.0)),
    );
    let fractions = vec![0.5, 0.5];
    let next = split_geometry::drag_update_fractions(
        fret_core::Axis::Horizontal,
        bounds,
        2,
        &fractions,
        0,
        Px(0.0),
        Px(6.0),
        &[],
        0.0,
        Point::new(Px(60.0), Px(10.0)),
    );
    assert!(next.is_none());
}

struct EmptyDeclarativeRegistry;

impl DockPanelElementRegistry<TestHost> for EmptyDeclarativeRegistry {
    fn render_panel(
        &self,
        _cx: &mut fret_ui::ElementContext<'_, TestHost>,
        _window: AppWindowId,
        _panel: &PanelKey,
    ) -> Option<fret_ui::element::AnyElement> {
        None
    }
}

#[derive(Clone, Copy)]
struct SplitDragHarnessOptions {
    size: Size,
    fractions: &'static [f32],
    panels: &'static [&'static str],
    viewports: &'static [&'static str],
    policy: Option<fn() -> Arc<dyn DockingPolicy>>,
}

impl Default for SplitDragHarnessOptions {
    fn default() -> Self {
        Self {
            size: Size::new(Px(800.0), Px(600.0)),
            fractions: &[0.5, 0.5],
            panels: &["core.left", "core.right"],
            viewports: &[],
            policy: None,
        }
    }
}

fn render_public_declarative_split_drag_harness(
    options: SplitDragHarnessOptions,
) -> (
    UiTree<TestHost>,
    TestHost,
    FakeTextService,
    AppWindowId,
    Rect,
    NodeId,
    fret_core::DockNodeId,
) {
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyDeclarativeRegistry));
        },
    );
    if let Some(policy) = options.policy {
        app.with_global_mut(DockingPolicyService::default, |svc, _app| {
            svc.set(policy());
        });
    }

    let split = app.with_global_mut(DockManager::default, |dock, _app| {
        let mut children = Vec::new();
        for (index, panel_name) in options.panels.iter().enumerate() {
            let panel = PanelKey::new(*panel_name);
            let is_viewport = options
                .viewports
                .iter()
                .any(|viewport| viewport == panel_name);
            let target_key = index as u64 + 1;
            dock.ensure_panel(&panel, || DockPanel {
                title: format!("Panel {index}"),
                color: Color::TRANSPARENT,
                viewport: is_viewport.then(|| ViewportPanel {
                    target: RenderTargetId::from(KeyData::from_ffi(target_key)),
                    target_px_size: (1, 1),
                    fit: ViewportFit::Stretch,
                    context_menu_enabled: false,
                }),
            });
            children.push(dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel],
                active: 0,
            }));
        }
        let split = dock.workspace.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children,
            fractions: options.fractions.to_vec(),
        });
        dock.workspace.graph.set_window_root(window, split);
        split
    });

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), options.size);
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-split-drag",
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

    (ui, app, services, window, bounds, root, split)
}

fn horizontal_handle_position(
    app: &TestHost,
    bounds: Rect,
    split: fret_core::DockNodeId,
    handle_ix: usize,
) -> (Rect, Point) {
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let settings = fret_runtime::DockingInteractionSettings::default();
    let dock = app.global::<DockManager>().expect("dock manager");
    let DockNode::Split { fractions, .. } = dock.workspace.graph.node(split).expect("split node")
    else {
        panic!("expected split node");
    };
    let computed = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        dock_bounds,
        fractions.len(),
        fractions,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &[],
    );
    let rect = computed.handle_hit_rects[handle_ix];
    (
        dock_bounds,
        Point::new(
            Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
            Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
        ),
    )
}

fn split_fractions_for(app: &TestHost, split: fret_core::DockNodeId) -> Vec<f32> {
    app.global::<DockManager>()
        .and_then(|dock| {
            let DockNode::Split { fractions, .. } = dock.workspace.graph.node(split)? else {
                return None;
            };
            Some(fractions.clone())
        })
        .expect("expected split fractions")
}

#[test]
fn public_declarative_dock_space_split_handle_hover_sets_resize_cursor_effect() {
    let (mut ui, mut app, mut services, window, bounds, _root, split) =
        render_public_declarative_split_drag_harness(SplitDragHarnessOptions::default());
    let (_dock_bounds, handle_pos) = horizontal_handle_position(&app, bounds, split, 0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: handle_pos,
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::CursorSetIcon { window: w, icon }
                if *w == window && *icon == fret_core::CursorIcon::ColResize
        )),
        "expected public declarative dock host to set col-resize when hovering the split handle gap"
    );
}

#[test]
fn public_declarative_dock_space_split_handle_drag_respects_panel_min_size_policy() {
    struct MinSizePolicy;

    impl DockingPolicy for MinSizePolicy {
        fn panel_min_content_size(
            &self,
            panel: &PanelKey,
            _info: Option<&DockPanel>,
        ) -> Option<Size> {
            if panel.kind.0 == "core.right" {
                Some(Size::new(Px(300.0), Px(0.0)))
            } else {
                None
            }
        }
    }

    let (mut ui, mut app, mut services, _window, bounds, _root, split) =
        render_public_declarative_split_drag_harness(SplitDragHarnessOptions {
            policy: Some(|| Arc::new(MinSizePolicy)),
            ..Default::default()
        });
    let (dock_bounds, handle_pos) = horizontal_handle_position(&app, bounds, split, 0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: handle_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured_for(fret_core::PointerId(0)).is_some(),
        "expected declarative divider drag to request pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(handle_pos.x.0 + 500.0), handle_pos.y),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let fractions = split_fractions_for(&app, split);

    let settings = fret_runtime::DockingInteractionSettings::default();
    let computed = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        dock_bounds,
        2,
        &fractions,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &[],
    );

    assert!(
        computed.sizes[1] >= 300.0 - 0.01,
        "expected right panel clamped to min width, got sizes={:?}, fractions={fractions:?}, split={split:?}",
        computed.sizes
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(handle_pos.x.0 + 500.0), handle_pos.y),
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
            Effect::Dock(DockOp::SetSplitFractionsMany { updates })
                if updates.iter().any(|update| update.split == split)
        )),
        "expected declarative divider drag release to commit SetSplitFractionsMany, got: {effects:?}"
    );
    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected declarative divider drag to release pointer capture on pointer up"
    );
}

#[test]
fn public_declarative_dock_space_split_handle_drag_uses_default_viewport_min_size_even_with_policy_installed()
 {
    struct DropTargetOnlyPolicy;

    impl DockingPolicy for DropTargetOnlyPolicy {
        fn allow_dock_drop_target(
            &self,
            _window: AppWindowId,
            _layout_root: fret_core::DockNodeId,
            _tabs: fret_core::DockNodeId,
            _zone: DropZone,
            _outer: bool,
        ) -> bool {
            true
        }
    }

    let (mut ui, mut app, mut services, _window, bounds, _root, split) =
        render_public_declarative_split_drag_harness(SplitDragHarnessOptions {
            viewports: &["core.left", "core.right"],
            policy: Some(|| Arc::new(DropTargetOnlyPolicy)),
            ..Default::default()
        });
    let (dock_bounds, handle_pos) = horizontal_handle_position(&app, bounds, split, 0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: handle_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(handle_pos.x.0 + 2400.0), handle_pos.y),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let fractions = split_fractions_for(&app, split);

    let settings = fret_runtime::DockingInteractionSettings::default();
    let computed = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        dock_bounds,
        2,
        &fractions,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &[],
    );

    assert!(
        computed.sizes[1] >= 240.0 - 0.01,
        "expected viewport panel clamped to default min width, got sizes={:?}, fractions={fractions:?}",
        computed.sizes
    );
}

#[test]
fn public_declarative_dock_space_nary_split_handle_drag_updates_only_adjacent_and_respects_min_size_policy()
 {
    struct MinSizePolicy;

    impl DockingPolicy for MinSizePolicy {
        fn panel_min_content_size(
            &self,
            panel: &PanelKey,
            _info: Option<&DockPanel>,
        ) -> Option<Size> {
            if panel.kind.0 == "core.middle" {
                Some(Size::new(Px(300.0), Px(0.0)))
            } else {
                None
            }
        }
    }

    let (mut ui, mut app, mut services, _window, bounds, _root, split) =
        render_public_declarative_split_drag_harness(SplitDragHarnessOptions {
            size: Size::new(Px(900.0), Px(600.0)),
            fractions: &[0.33, 0.34, 0.33],
            panels: &["core.left", "core.middle", "core.right"],
            policy: Some(|| Arc::new(MinSizePolicy)),
            ..Default::default()
        });
    let (dock_bounds, handle_pos) = horizontal_handle_position(&app, bounds, split, 0);
    let settings = fret_runtime::DockingInteractionSettings::default();

    let fractions0 = split_fractions_for(&app, split);

    let layout0 = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        dock_bounds,
        3,
        &fractions0,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &[],
    );
    assert_eq!(
        layout0.sizes.len(),
        3,
        "expected three-panel split layout, got {:?}",
        layout0.sizes
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: handle_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured_for(fret_core::PointerId(0)).is_some(),
        "expected declarative divider drag to request pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(handle_pos.x.0 + 1200.0), handle_pos.y),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let fractions1 = split_fractions_for(&app, split);

    let layout1 = split_geometry::compute_layout(
        fret_core::Axis::Horizontal,
        dock_bounds,
        3,
        &fractions1,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &[],
    );

    assert!(
        (layout1.sizes[2] - layout0.sizes[2]).abs() < 0.01,
        "expected far-right panel unchanged, before={:?}, after={:?}",
        layout0.sizes,
        layout1.sizes
    );
    assert!(
        layout1.sizes[1] >= 300.0 - 0.01,
        "expected middle panel clamped to min width, got sizes={:?}",
        layout1.sizes
    );
    assert!(
        layout1.sizes[0] > layout0.sizes[0] + 1.0,
        "expected left panel to grow, before={:?}, after={:?}",
        layout0.sizes,
        layout1.sizes
    );
}
