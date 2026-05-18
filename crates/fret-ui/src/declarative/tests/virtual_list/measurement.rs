use super::*;

#[test]
fn virtual_list_skips_redundant_measures_for_clean_measured_rows() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    fn row<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut style = crate::element::LayoutStyle::default();
        style.size.height = crate::element::Length::Px(Px(10.0));
        cx.container(
            crate::element::ContainerProps {
                layout: style,
                ..Default::default()
            },
            |_| Vec::new(),
        )
    }

    fn build_list<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        let options = crate::element::VirtualListOptions::new(Px(10.0), 0);
        cx.virtual_list(100, options, scroll_handle, |cx, items| {
            items
                .iter()
                .copied()
                .map(|item| cx.keyed(item.key, row))
                .collect::<Vec<_>>()
        })
    }

    // Frame 0: establish viewport size so the visible range can be computed.
    crate::virtual_list::debug_take_virtual_list_item_measures();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-skip-measure",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: first layout with a known viewport should measure the initial visible items.
    crate::virtual_list::debug_take_virtual_list_item_measures();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-skip-measure",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let measured = crate::virtual_list::debug_take_virtual_list_item_measures();
    assert!(measured > 0, "expected initial visible rows to be measured");
    app.advance_frame();

    // Frame 2: with a clean tree and stable viewport, measured rows should not be re-measured.
    crate::virtual_list::debug_take_virtual_list_item_measures();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-skip-measure",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let measured = crate::virtual_list::debug_take_virtual_list_item_measures();
    assert_eq!(
        measured, 0,
        "expected no redundant measures once rows are measured and clean"
    );
    app.advance_frame();

    // Frame 3: scrolling by one row should only measure newly revealed items.
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(10.0)));
    crate::virtual_list::debug_take_virtual_list_item_measures();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-skip-measure",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let measured = crate::virtual_list::debug_take_virtual_list_item_measures();
    assert_eq!(
        measured, 1,
        "expected only one newly visible row to be measured"
    );
    app.advance_frame();

    // Note: `items_revision` bumps are handled by the normal invalidation path (layout-dirty rows
    // should re-measure as needed). This test focuses on the steady-state scroll hot path.
}

#[test]
fn virtual_list_measurement_updates_preserve_scroll_anchor_under_overscan() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    fn row_with_height<H: UiHost>(cx: &mut ElementContext<'_, H>, height: Px) -> AnyElement {
        let mut style = crate::element::LayoutStyle::default();
        style.size.height = crate::element::Length::Px(height);
        cx.container(
            crate::element::ContainerProps {
                layout: style,
                ..Default::default()
            },
            |_| Vec::new(),
        )
    }

    // Frame 0: establish viewport height with uniform estimated sizes.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-anchor-measurement",
        |cx| {
            let list = cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 2),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| row_with_height(cx, Px(10.0))))
                        .collect::<Vec<_>>()
                },
            );
            list_element_id = Some(list.id);
            vec![list]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: scroll such that visible start index is 5 (offset=50). Overscan includes
    // indices 3 and 4; measuring those taller rows must not cause the viewport to jump.
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(50.0)));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-anchor-measurement",
        |cx| {
            let list = cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 2),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| {
                            cx.keyed(item.key, |cx| {
                                if item.index == 3 || item.index == 4 {
                                    row_with_height(cx, Px(100.0))
                                } else {
                                    row_with_height(cx, Px(10.0))
                                }
                            })
                        })
                        .collect::<Vec<_>>()
                },
            );
            list_element_id = Some(list.id);
            vec![list]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let state = crate::elements::with_element_state(
        &mut app,
        window,
        list_element_id.expect("list element id"),
        crate::element::VirtualListState::default,
        |s| s.clone(),
    );

    let range = state
        .metrics
        .visible_range(state.offset_y, state.viewport_h, 0)
        .expect("range");
    assert_eq!(
        range.start_index, 5,
        "expected scroll anchor to preserve start index"
    );

    let expected = state.metrics.offset_for_index(5);
    assert!(
        (state.offset_y.0 - expected.0).abs() < 0.01,
        "offset_y={:?} expected={:?}",
        state.offset_y,
        expected
    );
    assert!(
        (scroll_handle.offset().y.0 - expected.0).abs() < 0.01,
        "scroll_handle_offset={:?} expected={:?}",
        scroll_handle.offset().y,
        expected
    );
}

#[test]
fn auto_height_virtual_list_len_growth_reflows_following_siblings() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();
    let mut len = 3;

    fn fixed_row<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut style = crate::element::LayoutStyle::default();
        style.size.width = crate::element::Length::Fill;
        style.size.height = crate::element::Length::Px(Px(10.0));
        cx.container(
            crate::element::ContainerProps {
                layout: style,
                ..Default::default()
            },
            |_| Vec::new(),
        )
    }

    fn following_sibling<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut style = crate::element::LayoutStyle::default();
        style.size.width = crate::element::Length::Fill;
        style.size.height = crate::element::Length::Px(Px(20.0));
        cx.container(
            crate::element::ContainerProps {
                layout: style,
                ..Default::default()
            },
            |_| Vec::new(),
        )
    }

    let mut render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, len: usize| {
        let root = render_root(
            ui,
            app,
            &mut text,
            window,
            bounds,
            "auto-height-vlist-len-growth-reflow",
            |cx| {
                vec![cx.flex(
                    crate::element::FlexProps {
                        direction: fret_core::Axis::Vertical,
                        layout: {
                            let mut layout = crate::element::LayoutStyle::default();
                            layout.size.width = crate::element::Length::Fill;
                            layout.size.height = crate::element::Length::Auto;
                            layout
                        },
                        ..Default::default()
                    },
                    |cx| {
                        let mut list_layout = crate::element::LayoutStyle::default();
                        list_layout.size.width = crate::element::Length::Fill;
                        list_layout.size.height = crate::element::Length::Auto;
                        let mut options = crate::element::VirtualListOptions::fixed(Px(10.0), 0);
                        options.items_revision = len as u64;
                        vec![
                            cx.virtual_list_keyed_with_layout(
                                list_layout,
                                len,
                                options,
                                &scroll_handle,
                                |index| index as crate::ItemKey,
                                |cx, _index| fixed_row(cx),
                            ),
                            following_sibling(cx),
                        ]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(app, &mut text, bounds, 1.0);
        root
    };

    let root = render(&mut ui, &mut app, len);
    let column = ui.children(root)[0];
    let list = ui.children(column)[0];
    let sibling = ui.children(column)[1];
    let list_bounds = ui.debug_node_bounds(list).expect("initial list bounds");
    let sibling_bounds = ui
        .debug_node_bounds(sibling)
        .expect("initial sibling bounds");
    assert_eq!(list_bounds.size.height, Px(30.0));
    assert_eq!(sibling_bounds.origin.y, Px(30.0));

    app.advance_frame();
    len = 6;
    let root = render(&mut ui, &mut app, len);
    let column = ui.children(root)[0];
    let list = ui.children(column)[0];
    let sibling = ui.children(column)[1];
    let list_bounds = ui.debug_node_bounds(list).expect("expanded list bounds");
    let sibling_bounds = ui
        .debug_node_bounds(sibling)
        .expect("expanded sibling bounds");
    assert_eq!(list_bounds.size.height, Px(60.0));
    assert_eq!(sibling_bounds.origin.y, Px(60.0));
}
