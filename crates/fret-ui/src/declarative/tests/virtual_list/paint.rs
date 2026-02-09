use super::*;

#[test]
fn virtual_list_paint_clips_each_visible_row() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(50.0)),
    );
    let mut text = FakeTextService::default();

    fn build_list<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        cx.virtual_list(
            100,
            crate::element::VirtualListOptions::new(Px(10.0), 0),
            scroll_handle,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                    .collect::<Vec<_>>()
            },
        )
    }

    // Frame 0: record viewport height (no visible children yet).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-clip",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: mount visible children based on the recorded viewport height.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-clip",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    // One clip for the list viewport + one clip per visible row child.
    let pushes = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PushClipRect { .. }))
        .count();
    assert_eq!(pushes, 1 + 5);
}

#[test]
fn virtual_list_scroll_transform_does_not_double_transform_per_row_clip_rects() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(50.0)),
    );
    let mut text = FakeTextService::default();

    fn build_list(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        cx.virtual_list(
            10_000,
            crate::element::VirtualListOptions::new(Px(10.0), 0),
            scroll_handle,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                    .collect::<Vec<_>>()
            },
        )
    }

    // Frame 0: record viewport size (no visible children yet).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-clip-transform",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: mount visible children.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-clip-transform",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Request a deep scroll so the first visible row is far away in content space.
    scroll_handle.scroll_to_item(9000, crate::scroll::ScrollStrategy::Start);

    // Frame 2: consume deferred scroll during layout.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-clip-transform",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Frame 3: paint with the applied scroll transform.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-clip-transform",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    // When using a children-only scroll transform, per-row clip rects must remain in the
    // pre-transform/content coordinate space (not pre-transformed into screen space). A deep scroll
    // should therefore produce at least one clip rect with a large y origin (e.g. ~9000 * 10px).
    let has_large_clip = scene.ops().iter().any(|op| match *op {
        SceneOp::PushClipRect { rect } => rect.origin.y.0 > 1_000.0,
        _ => false,
    });
    assert!(
        has_large_clip,
        "expected at least one per-row clip rect to remain in content space under scroll transforms"
    );
}
