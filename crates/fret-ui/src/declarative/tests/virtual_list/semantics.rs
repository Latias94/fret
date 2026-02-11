use super::*;

#[test]
fn virtual_list_scroll_offsets_apply_in_semantics_snapshot() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_list(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> crate::element::AnyElement {
        let mut layout = crate::element::LayoutStyle::default();
        layout.size.width = crate::element::Length::Fill;
        layout.size.height = crate::element::Length::Fill;
        layout.overflow = crate::element::Overflow::Clip;

        cx.virtual_list_with_layout(
            layout,
            10_000,
            crate::element::VirtualListOptions::new(Px(28.0), 10),
            scroll_handle,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| {
                        cx.semantics(
                            crate::element::SemanticsProps {
                                role: fret_core::SemanticsRole::Button,
                                test_id: Some(Arc::<str>::from(format!(
                                    "virtual-list-row-{}",
                                    item.index
                                ))),
                                ..Default::default()
                            },
                            |cx| vec![cx.text("row")],
                        )
                    })
                    .collect::<Vec<_>>()
            },
        )
    }

    // Frame 0: establish viewport.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-semantics-scroll-transform",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Frame 1: request a scroll-to-item and allow final layout to consume it.
    let target = 9000usize;
    scroll_handle.scroll_to_item(target, crate::scroll::ScrollStrategy::Start);
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-semantics-scroll-transform",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Frame 2: rerender the visible range and validate semantics bounds are in viewport space.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-semantics-scroll-transform",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
    let needle = format!("virtual-list-row-{target}");
    let node = snapshot
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(needle.as_str()))
        .expect("expected scrolled-to row to appear in semantics snapshot");
    assert!(
        node.bounds.origin.y.0 >= bounds.origin.y.0
            && node.bounds.origin.y.0 <= bounds.origin.y.0 + bounds.size.height.0,
        "expected semantics bounds to be in viewport space after scroll; got {:?}",
        node.bounds
    );
}
