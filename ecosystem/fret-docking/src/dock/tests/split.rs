use super::*;

#[test]
fn drag_update_fractions_updates_two_panel_split() {
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(300.0), Px(40.0)));
    let fractions = vec![0.5, 0.5];
    let next = resizable::drag_update_fractions(
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

    let layout0 = resizable::compute_layout(
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
    let next = resizable::drag_update_adjacent_fractions(
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

    let layout1 = resizable::compute_layout(
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

    let layout0 = resizable::compute_layout(
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
    let next = resizable::drag_update_adjacent_fractions(
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

    let layout1 = resizable::compute_layout(
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
fn same_axis_nested_split_drag_preserves_inner_sibling_width() {
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

    let inner = graph.insert_node(DockNode::Split {
        axis: fret_core::Axis::Horizontal,
        children: vec![a, b],
        fractions: vec![0.5, 0.5],
    });
    let root = graph.insert_node(DockNode::Split {
        axis: fret_core::Axis::Horizontal,
        children: vec![inner, c],
        fractions: vec![0.5, 0.5],
    });

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(600.0), Px(80.0)));
    let settings = fret_runtime::DockingInteractionSettings::default();
    let layout0 = compute_layout_map(
        &graph,
        root,
        bounds,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
    );

    let a0 = layout0.get(&a).expect("missing a").size.width.0;
    let b0 = layout0.get(&b).expect("missing b").size.width.0;

    let locks = compute_same_axis_locks_for_split_drag(
        &graph,
        &layout0,
        root,
        fret_core::Axis::Horizontal,
        0,
    );
    assert!(
        !locks.is_empty(),
        "expected nested locks for same-axis split"
    );

    let fractions = match graph.node(root).expect("root") {
        DockNode::Split { fractions, .. } => fractions.clone(),
        _ => unreachable!(),
    };

    // Drag the root splitter rightward (increase left subtree width).
    let next = resizable::drag_update_fractions(
        fret_core::Axis::Horizontal,
        bounds,
        2,
        &fractions,
        0,
        Px(0.0),
        Px(6.0),
        &[],
        0.0,
        Point::new(Px(420.0), Px(40.0)),
    )
    .expect("expected root drag to update fractions");
    assert!(graph.update_split_fractions(root, next));

    apply_same_axis_locks(
        &mut graph,
        root,
        bounds,
        fret_core::Axis::Horizontal,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &locks,
    );

    let layout1 = compute_layout_map(
        &graph,
        root,
        bounds,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
    );
    let a1 = layout1.get(&a).expect("missing a").size.width.0;
    let b1 = layout1.get(&b).expect("missing b").size.width.0;

    assert!(
        (a1 - a0).abs() <= 1.0,
        "expected inner sibling width preserved (a), before={a0}, after={a1}"
    );
    assert!(
        b1 > b0 + 10.0,
        "expected touching node to grow (b), before={b0}, after={b1}"
    );

    let inner_f0 = match graph.node(inner).expect("inner") {
        DockNode::Split { fractions, .. } => fractions[0],
        _ => unreachable!(),
    };
    assert!(
        inner_f0 < 0.5,
        "expected inner split fraction to change to keep (a) stable, got {inner_f0}"
    );
}
#[test]
fn drag_update_fractions_handles_nan_bounds() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(f32::NAN), Px(300.0)),
    );
    let fractions = vec![0.5, 0.5];
    let next = resizable::drag_update_fractions(
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
#[test]
fn dock_split_handle_hover_sets_resize_cursor_effect() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
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
        dock.graph.set_window_root(window, split);
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let x = dock_bounds.origin.x.0 + dock_bounds.size.width.0 * 0.5;
    let y = dock_bounds.origin.y.0 + 10.0;

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(x), Px(y)),
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
        "expected a col-resize cursor effect when hovering the split handle gap"
    );
}
