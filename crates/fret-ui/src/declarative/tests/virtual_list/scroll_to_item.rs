use super::*;

#[test]
fn virtual_list_scroll_to_item_keeps_target_visible() {
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

    // Frame 0: establish viewport height.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-to",
        |cx| {
            let list = cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
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

    // Frame 1: request scroll-to on a row below the viewport.
    let target = 6usize; // row_top=60, viewport=30 => needs offset ~= 40..60
    scroll_handle.scroll_to_item(target, crate::scroll::ScrollStrategy::Nearest);
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((target, crate::scroll::ScrollStrategy::Nearest))
    );
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-to",
        |cx| {
            let list = cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                        .collect::<Vec<_>>()
                },
            );
            list_element_id = Some(list.id);
            vec![list]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(scroll_handle.deferred_scroll_to_item().is_none());

    let state = crate::elements::with_element_state(
        &mut app,
        window,
        list_element_id.expect("list element id"),
        crate::element::VirtualListState::default,
        |s| s.clone(),
    );
    assert_eq!(state.viewport_h, Px(30.0));
    assert!((state.metrics.offset_for_index(target).0 - 60.0).abs() < 0.01);
    assert!(
        (state.offset_y.0 - 40.0).abs() < 0.01,
        "state_offset_y={:?}",
        state.offset_y
    );

    assert!(
        (scroll_handle.offset().y.0 - 40.0).abs() < 0.01,
        "offset_y={:?}",
        scroll_handle.offset().y
    );
}

#[test]
fn virtual_list_fixed_scroll_to_item_does_not_force_layout_invalidation() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    // Frame 0: establish viewport height and register scroll-handle bindings.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-fixed-scroll-to-hit-test-only",
        |cx| {
            vec![cx.virtual_list(
                100,
                crate::element::VirtualListOptions::fixed(Px(10.0), 0),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                        .collect::<Vec<_>>()
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: request scroll-to-item; fixed lists should not require a scroll-handle Layout
    // invalidation in order to consume the request.
    let target = 6usize;
    scroll_handle.scroll_to_item(target, crate::scroll::ScrollStrategy::Nearest);
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((target, crate::scroll::ScrollStrategy::Nearest))
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        scroll_handle.deferred_scroll_to_item().is_none(),
        "expected fixed virtual list to consume deferred scroll-to-item"
    );

    let scroll_walks: Vec<_> = ui
        .debug_invalidation_walks()
        .iter()
        .filter(|w| {
            matches!(
                w.detail,
                crate::tree::UiDebugInvalidationDetail::ScrollHandleHitTestOnly
                    | crate::tree::UiDebugInvalidationDetail::ScrollHandleLayout
                    | crate::tree::UiDebugInvalidationDetail::ScrollHandleWindowUpdate
            )
        })
        .collect();
    assert!(
        !scroll_walks.is_empty(),
        "expected scroll-handle invalidation walk"
    );
    assert!(
        scroll_walks.iter().all(|w| w.inv != Invalidation::Layout),
        "expected fixed scroll-to-item to avoid a scroll-handle Layout invalidation; got {:?}",
        scroll_walks
    );
}

#[test]
fn virtual_list_measured_scroll_to_item_does_not_force_layout_invalidation_in_common_case() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    fn build_list(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> crate::element::AnyElement {
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

    // Frame 0: establish viewport height and register scroll-handle bindings.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-measured-scroll-to-hit-test-only",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: mount visible items so measured-mode has stable cached metrics to consume against.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-measured-scroll-to-hit-test-only",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 2: request scroll-to-item; measured lists should be able to consume without a
    // scroll-handle Layout invalidation in the common case.
    let target = 6usize;
    scroll_handle.scroll_to_item(target, crate::scroll::ScrollStrategy::Nearest);
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((target, crate::scroll::ScrollStrategy::Nearest))
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        scroll_handle.deferred_scroll_to_item().is_none(),
        "expected measured virtual list to consume deferred scroll-to-item"
    );

    let scroll_walks: Vec<_> = ui
        .debug_invalidation_walks()
        .iter()
        .filter(|w| {
            matches!(
                w.detail,
                crate::tree::UiDebugInvalidationDetail::ScrollHandleHitTestOnly
                    | crate::tree::UiDebugInvalidationDetail::ScrollHandleLayout
                    | crate::tree::UiDebugInvalidationDetail::ScrollHandleWindowUpdate
            )
        })
        .collect();
    assert!(
        !scroll_walks.is_empty(),
        "expected scroll-handle invalidation walk"
    );
    assert!(
        scroll_walks.iter().all(|w| w.inv != Invalidation::Layout),
        "expected measured scroll-to-item to avoid a scroll-handle Layout invalidation in the common case; got {:?}",
        scroll_walks
    );
}

#[test]
fn virtual_list_probe_layout_does_not_consume_deferred_scroll_request() {
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

    // Frame 0: establish viewport height.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-probe-scroll-to",
        |cx| {
            vec![cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                        .collect::<Vec<_>>()
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: request scroll-to on a row below the viewport.
    let target = 6usize;
    scroll_handle.scroll_to_item(target, crate::scroll::ScrollStrategy::Nearest);
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((target, crate::scroll::ScrollStrategy::Nearest))
    );

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-probe-scroll-to",
        |cx| {
            vec![cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                        .collect::<Vec<_>>()
                },
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all_with_pass_kind(
        &mut app,
        &mut text,
        bounds,
        1.0,
        crate::layout_pass::LayoutPassKind::Probe,
    );
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((target, crate::scroll::ScrollStrategy::Nearest)),
        "probe layout must not consume deferred scroll requests"
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        scroll_handle.deferred_scroll_to_item().is_none(),
        "final layout must consume deferred scroll requests"
    );
}

#[test]
fn virtual_list_scroll_to_item_triggers_layout_even_without_other_invalidations() {
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
    let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

    fn build_list(
        cx: &mut ElementContext<'_, TestHost>,
        list_element_id: &mut Option<crate::elements::GlobalElementId>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> crate::element::AnyElement {
        let list = cx.virtual_list(
            500,
            crate::element::VirtualListOptions::new(Px(10.0), 0),
            scroll_handle,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                    .collect::<Vec<_>>()
            },
        );
        *list_element_id = Some(list.id);
        list
    }

    // Frame 0: establish viewport height.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-invalidation",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Frame 1: mount list with known viewport height.
    app.advance_frame();
    let prev_list_element_id = list_element_id;
    list_element_id = None;
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-invalidation",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        prev_list_element_id, list_element_id,
        "virtual list element id should be stable across frames"
    );
    let list_element_id = list_element_id.expect("list element id");

    // Without touching the UI tree, request a deferred scroll-to-item.
    let target = 80usize;
    scroll_handle.scroll_to_item(target, crate::scroll::ScrollStrategy::Start);
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((target, crate::scroll::ScrollStrategy::Start))
    );

    // Frame 2: layout should still run for the list (consuming the deferred scroll request).
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(scroll_handle.deferred_scroll_to_item().is_none());
    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected scroll offset to change"
    );
    let offset_y = crate::elements::with_element_state(
        &mut app,
        window,
        list_element_id,
        crate::element::VirtualListState::default,
        |s| s.offset_y,
    );
    assert!(offset_y.0 > 0.01, "expected state offset to change");
}

#[test]
fn virtual_list_scroll_to_item_uses_measured_row_heights() {
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

    fn build_list<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        list_element_id: &mut Option<crate::elements::GlobalElementId>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        let list = cx.virtual_list(
            100,
            crate::element::VirtualListOptions::new(Px(10.0), 0),
            scroll_handle,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| {
                        cx.keyed(item.key, |cx| {
                            if item.index == 0 {
                                row_with_height(cx, Px(100.0))
                            } else {
                                row_with_height(cx, Px(10.0))
                            }
                        })
                    })
                    .collect::<Vec<_>>()
            },
        );
        *list_element_id = Some(list.id);
        list
    }

    // Frame 0: establish viewport height.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-measure",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: ensure row 0 gets mounted and measured.
    let prev_list_element_id = list_element_id;
    list_element_id = None;
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-measure",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        prev_list_element_id, list_element_id,
        "virtual list element id should be stable across frames"
    );
    app.advance_frame();

    // Frame 2: scroll to item 1; should account for the measured height of item 0.
    scroll_handle.scroll_to_item(1, crate::scroll::ScrollStrategy::Start);
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((1, crate::scroll::ScrollStrategy::Start))
    );
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-measure",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        (scroll_handle.offset().y.0 - 100.0).abs() < 0.01,
        "offset_y={:?}",
        scroll_handle.offset().y
    );
}
