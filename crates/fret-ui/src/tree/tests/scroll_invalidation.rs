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
                        (0..100).map(|i| cx.text(format!("row-{i}"))).collect()
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
