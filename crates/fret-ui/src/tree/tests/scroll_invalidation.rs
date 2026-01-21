use super::*;
use fret_core::Modifiers;

#[test]
fn scroll_wheel_invalidation_is_hit_test_only() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeUiServices;

    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "scroll",
        |cx| {
            let handle = scroll_handle.clone();
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(handle),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                        (0..100)
                            .map(|i| cx.text(format!("row-{i}")))
                            .collect::<Vec<_>>()
                    })]
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let scroll_node = ui.children(root)[0];
    let scroll_bounds = ui.debug_node_bounds(scroll_node).expect("scroll bounds");
    let p = Point::new(
        Px(scroll_bounds.origin.x.0 + 5.0),
        Px(scroll_bounds.origin.y.0 + 5.0),
    );

    for (_, node) in ui.nodes.iter_mut() {
        node.invalidation.clear();
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: PointerId(0),
            position: p,
            delta: Point::new(Px(0.0), Px(-60.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected wheel to update scroll offset"
    );

    let scroll_flags = ui.nodes[scroll_node].invalidation;
    assert!(
        !scroll_flags.layout,
        "expected scroll wheel to avoid layout invalidation"
    );
    assert!(
        scroll_flags.hit_test && scroll_flags.paint,
        "expected scroll wheel to invalidate hit-test + paint"
    );
}

#[test]
fn scroll_offset_changes_do_not_replay_paint_cache() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.set_paint_cache_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeUiServices;

    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "scroll",
        |cx| {
            let handle = scroll_handle.clone();
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = crate::element::Length::Fill;
            scroll_layout.size.height = crate::element::Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(handle),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                        (0..100)
                            .map(|i| cx.text(format!("row-{i}")))
                            .collect::<Vec<_>>()
                    })]
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let bindings = crate::declarative::frame::bound_elements_for_scroll_handle(
        &mut app,
        window,
        scroll_handle.binding_key(),
    );
    assert!(
        !bindings.is_empty(),
        "expected scroll handle bindings to be registered for the scroll node"
    );
    assert!(
        bindings.iter().any(
            |&element| crate::declarative::node_for_element_in_window_frame(
                &mut app, window, element
            ) == Some(scroll_node)
        ),
        "expected scroll handle binding to resolve to the scroll node"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let transforms1: Vec<Transform2D> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::PushTransform { transform } => Some(*transform),
            _ => None,
        })
        .collect();
    let fp1 = scene.fingerprint();
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    for (_, node) in ui.nodes.iter_mut() {
        node.invalidation.clear();
    }

    let prev_offset = scroll_handle.offset();
    let prev_children_transform = ui
        .node_children_render_transform(scroll_node)
        .unwrap_or(Transform2D::IDENTITY);

    let prev_revision = scroll_handle.revision();
    scroll_handle.set_offset(Point::new(Px(0.0), Px(100.0)));
    assert!(
        scroll_handle.revision() > prev_revision,
        "expected programmatic scroll to bump scroll handle revision"
    );
    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected programmatic scroll to update scroll offset"
    );
    assert!(
        (scroll_handle.offset().y.0 - prev_offset.y.0).abs() > 0.01,
        "expected programmatic scroll to change scroll offset"
    );
    let children_transform = ui
        .node_children_render_transform(scroll_node)
        .unwrap_or(Transform2D::IDENTITY);
    assert_ne!(
        children_transform,
        Transform2D::IDENTITY,
        "expected a non-identity children render transform after scrolling"
    );
    assert_ne!(
        children_transform, prev_children_transform,
        "expected scroll offset changes to update children render transform"
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let transforms2: Vec<Transform2D> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::PushTransform { transform } => Some(*transform),
            _ => None,
        })
        .collect();
    let fp2 = scene.fingerprint();

    assert!(
        !ui.debug_paint_cache_replays.contains_key(&root)
            && !ui.debug_paint_cache_replays.contains_key(&scroll_node),
        "expected scroll offset changes to prevent paint-cache replay for scroll ancestors, got: {:?} (root={:?}, scroll_node={:?})",
        ui.debug_paint_cache_replays,
        root,
        scroll_node,
    );
    assert_ne!(
        transforms1, transforms2,
        "expected scroll offset changes to update emitted transform ops"
    );
    assert_ne!(
        fp1, fp2,
        "expected scroll offset changes to update scene output"
    );
}
