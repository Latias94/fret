use super::*;

#[test]
fn virtual_list_shared_scroll_handle_invalidates_other_bound_lists() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(60.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    fn build_pair(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        list_a_id: &mut Option<crate::elements::GlobalElementId>,
        list_b_id: &mut Option<crate::elements::GlobalElementId>,
    ) -> Vec<crate::element::AnyElement> {
        let mut options = crate::element::VirtualListOptions::new(Px(10.0), 0);
        options.axis = fret_core::Axis::Horizontal;

        let list_a = cx.virtual_list(100, options.clone(), scroll_handle, |cx, items| {
            items
                .iter()
                .copied()
                .map(|item| cx.keyed(item.key, |cx| cx.text("a")))
                .collect::<Vec<_>>()
        });
        *list_a_id = Some(list_a.id);

        let list_b = cx.virtual_list(100, options, scroll_handle, |cx, items| {
            items
                .iter()
                .copied()
                .map(|item| cx.keyed(item.key, |cx| cx.text("b")))
                .collect::<Vec<_>>()
        });
        *list_b_id = Some(list_b.id);

        vec![list_a, list_b]
    }

    // Frame 1: mount and measure.
    let mut list_a_id: Option<crate::elements::GlobalElementId> = None;
    let mut list_b_id: Option<crate::elements::GlobalElementId> = None;
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp60-vlist-x-sync",
        |cx| build_pair(cx, &scroll_handle, &mut list_a_id, &mut list_b_id),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Frame 2: build visible rows now that viewport is known.
    app.advance_frame();
    let prev_list_a_id = list_a_id;
    let prev_list_b_id = list_b_id;
    let mut list_a_id: Option<crate::elements::GlobalElementId> = None;
    let mut list_b_id: Option<crate::elements::GlobalElementId> = None;
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp60-vlist-x-sync",
        |cx| build_pair(cx, &scroll_handle, &mut list_a_id, &mut list_b_id),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(list_a_id, prev_list_a_id);
    assert_eq!(list_b_id, prev_list_b_id);

    // Before scroll, there is no scroll transform applied (offset=0).
    let mut scene_before = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene_before, 1.0);
    let before_push_transforms = scene_before
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PushTransform { .. }))
        .count();
    assert_eq!(
        before_push_transforms, 0,
        "expected no children scroll transforms before wheel scroll"
    );

    // Scroll the first list; the second list should also be invalidated and updated because it
    // shares the same `VirtualListScrollHandle`.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: fret_core::Point::new(Px(5.0), Px(5.0)),
            delta: fret_core::Point::new(Px(-20.0), Px(0.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene_after = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene_after, 1.0);
    let after_push_transforms: Vec<String> = scene_after
        .ops()
        .iter()
        .filter(|op| matches!(op, SceneOp::PushTransform { .. }))
        .map(|op| format!("{op:?}"))
        .collect();

    // Both lists apply the same scroll handle offset via a children-only render transform, so we
    // expect two identical translation transforms (one per list).
    assert_eq!(
        after_push_transforms.len(),
        2,
        "expected both bound virtual lists to apply a scroll transform"
    );
    assert_eq!(
        after_push_transforms[0], after_push_transforms[1],
        "expected both bound virtual lists to apply the same scroll transform"
    );
    assert!(
        after_push_transforms[0].contains("-20"),
        "expected scroll transform to include the wheel delta (-20px): {:?}",
        after_push_transforms[0]
    );
}

#[test]
fn virtual_list_triggers_visible_range_rerender_on_wheel_scroll_when_cached() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(50.0)),
    );
    let mut text = FakeTextService::default();

    let render_calls: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    fn build_tree(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        render_calls: Arc<AtomicUsize>,
    ) -> AnyElement {
        let mut cache = crate::element::ViewCacheProps::default();
        cache.layout.size.width = crate::element::Length::Fill;
        cache.layout.size.height = crate::element::Length::Fill;
        cache.cache_key = 1;
        cx.view_cache(cache, move |cx| {
            render_calls.fetch_add(1, Ordering::SeqCst);
            vec![cx.virtual_list(
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
            )]
        })
    }

    // Frame 0: establish viewport size so the visible range can be computed.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: mount initial visible rows (0..=4).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let cache_node = ui.children(root)[0];
    let list_node = ui.children(cache_node)[0];
    assert_eq!(ui.children(list_node).len(), 5);
    app.advance_frame();

    // Frame 2: allow any one-time layout-driven invalidations (viewport/content size) to settle.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 3: prefer a cache hit; render closure should be skipped.
    //
    // Some one-time layout/prepaint bookkeeping (viewport/content size, window-range tracking) can
    // legitimately force a second rerender before the steady-state cache hit frame. Allow one
    // extra settle frame if needed so this test remains robust as the caching pipeline evolves.
    let mut calls_before_wheel = render_calls.load(Ordering::SeqCst);
    let mut root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    if render_calls.load(Ordering::SeqCst) != calls_before_wheel {
        app.advance_frame();
        calls_before_wheel = render_calls.load(Ordering::SeqCst);
        root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-refresh-on-wheel",
            |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
    }
    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_wheel,
        "expected view-cache root to be a cache hit before wheel scroll"
    );
    let cache_node = ui.children(root)[0];
    let _list_node = ui.children(cache_node)[0];

    // Wheel-scroll far enough that the desired visible range no longer overlaps the mounted rows.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: fret_core::Point::new(Px(5.0), Px(5.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-80.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        scroll_handle.offset().y,
        Px(80.0),
        "expected wheel scroll to update the bound scroll handle"
    );

    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_wheel,
        "expected wheel scroll to schedule rerender without breaking the current cache-hit frame"
    );
    assert!(
        !ui.should_reuse_view_cache_node(cache_node),
        "expected wheel scroll to notify the view-cache root for a one-shot rerender"
    );
    app.advance_frame();

    // Frame 4: rerender should run once to rebuild the visible range.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_wheel.saturating_add(1),
        "expected exactly one rerender after visible-range refresh"
    );

    let cache_node = ui.children(root)[0];
    let list_node = ui.children(cache_node)[0];
    let props = app.with_global_mut(
        crate::declarative::frame::ElementFrame::default,
        |frame, _app| {
            frame
                .windows
                .get(&window)
                .and_then(|w| w.instances.get(list_node))
                .cloned()
        },
    );
    let crate::declarative::ElementInstance::VirtualList(props) =
        props.expect("list instance exists").instance
    else {
        panic!("expected VirtualList instance");
    };
    assert_eq!(
        props
            .visible_items
            .iter()
            .map(|item| item.index)
            .collect::<Vec<_>>(),
        vec![8, 9, 10, 11, 12]
    );
    assert_eq!(ui.children(list_node).len(), 5);

    app.advance_frame();

    // Frame 5: allow any one-time post-refresh invalidations to settle.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 6: cache hit again (steady-state).
    let calls_after_settle = render_calls.load(Ordering::SeqCst);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_after_settle,
        "expected view-cache root to return to cache-hit steady state"
    );
}

#[test]
fn virtual_list_triggers_visible_range_rerender_on_scrollbar_wheel_when_cached() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(50.0)),
    );
    let mut text = FakeTextService::default();

    let render_calls: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

    fn build_tree(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        render_calls: Arc<AtomicUsize>,
    ) -> AnyElement {
        let mut cache = crate::element::ViewCacheProps::default();
        cache.layout.size.width = crate::element::Length::Fill;
        cache.layout.size.height = crate::element::Length::Fill;
        cache.cache_key = 1;

        cx.view_cache(cache, move |cx| {
            render_calls.fetch_add(1, Ordering::SeqCst);

            let mut list_layout = crate::element::LayoutStyle::default();
            list_layout.size.width = crate::element::Length::Fill;
            list_layout.size.height = crate::element::Length::Fill;

            let mut scrollbar_layout = crate::element::LayoutStyle::default();
            scrollbar_layout.size.width = crate::element::Length::Px(Px(10.0));
            scrollbar_layout.size.height = crate::element::Length::Fill;

            let mut list_element_id: Option<crate::elements::GlobalElementId> = None;
            let handle = scroll_handle.clone();
            vec![cx.row(crate::element::RowProps::default(), |cx| {
                let list = cx.virtual_list_with_layout(
                    list_layout,
                    100,
                    crate::element::VirtualListOptions::new(Px(10.0), 0),
                    &handle,
                    |cx, items| {
                        items
                            .iter()
                            .copied()
                            .map(|item| cx.keyed(item.key, |cx| cx.text("row")))
                            .collect::<Vec<_>>()
                    },
                );
                list_element_id = Some(list.id);

                let scrollbar = crate::element::ScrollbarProps {
                    layout: scrollbar_layout,
                    axis: crate::element::ScrollbarAxis::Vertical,
                    scroll_handle: handle.base_handle().clone(),
                    scroll_target: list_element_id,
                    ..Default::default()
                };

                vec![list, cx.scrollbar(scrollbar)]
            })]
        })
    }

    // Frame 0: establish viewport size so the visible range can be computed.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-scrollbar-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: mount initial visible rows (0..=4).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-scrollbar-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let cache_node = ui.children(root)[0];
    let row_node = ui.children(cache_node)[0];
    let list_node = ui.children(row_node)[0];
    assert_eq!(ui.children(list_node).len(), 5);
    app.advance_frame();

    // Frame 2: allow any one-time layout-driven invalidations (viewport/content size) to settle.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-scrollbar-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 3: cache hit; render closure should be skipped.
    let calls_before_wheel = render_calls.load(Ordering::SeqCst);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-scrollbar-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_wheel,
        "expected view-cache root to be a cache hit before scrollbar wheel scroll"
    );
    let cache_node = ui.children(root)[0];

    // Wheel-scroll over the scrollbar (x=195 falls within the 10px rail), far enough that the
    // desired visible range no longer overlaps the mounted rows.
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: fret_core::Point::new(Px(195.0), Px(5.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-80.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        scroll_handle.offset().y,
        Px(80.0),
        "expected scrollbar wheel scroll to update the bound scroll handle"
    );

    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_wheel,
        "expected scrollbar wheel scroll to schedule rerender without breaking the current cache-hit frame"
    );
    assert!(
        !ui.should_reuse_view_cache_node(cache_node),
        "expected scrollbar wheel scroll to notify the view-cache root for a one-shot rerender"
    );

    app.advance_frame();

    // Frame 4: rerender should run once to rebuild the visible range.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-refresh-on-scrollbar-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_wheel.saturating_add(1),
        "expected exactly one rerender after visible-range refresh"
    );

    let cache_node = ui.children(root)[0];
    let row_node = ui.children(cache_node)[0];
    let list_node = ui.children(row_node)[0];
    let props = app.with_global_mut(
        crate::declarative::frame::ElementFrame::default,
        |frame, _app| {
            frame
                .windows
                .get(&window)
                .and_then(|w| w.instances.get(list_node))
                .cloned()
        },
    );
    let crate::declarative::ElementInstance::VirtualList(props) =
        props.expect("list instance exists").instance
    else {
        panic!("expected VirtualList instance");
    };
    assert_eq!(
        props
            .visible_items
            .iter()
            .map(|item| item.index)
            .collect::<Vec<_>>(),
        vec![8, 9, 10, 11, 12]
    );
    assert_eq!(ui.children(list_node).len(), 5);
}

#[test]
fn virtual_list_row_view_cache_reuses_rows_across_small_scroll_deltas() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    let render_counts: Arc<Mutex<HashMap<u64, usize>>> = Arc::new(Mutex::new(HashMap::new()));

    fn build_list(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        render_counts: Arc<Mutex<HashMap<u64, usize>>>,
    ) -> AnyElement {
        let options = crate::element::VirtualListOptions::new(Px(10.0), 0);
        cx.virtual_list_keyed_with_layout(
            crate::element::LayoutStyle::default(),
            20,
            options,
            scroll_handle,
            |i| i as crate::ItemKey,
            move |cx, index| {
                let key = index as u64;
                let render_counts = Arc::clone(&render_counts);

                let view_cache = crate::element::ViewCacheProps {
                    cache_key: key,
                    ..Default::default()
                };
                cx.view_cache(view_cache, move |cx| {
                    render_counts
                        .lock()
                        .expect("render_counts lock")
                        .entry(key)
                        .and_modify(|v| *v += 1)
                        .or_insert(1);

                    let mut style = crate::element::LayoutStyle::default();
                    style.size.height = crate::element::Length::Px(Px(10.0));
                    vec![cx.container(
                        crate::element::ContainerProps {
                            layout: style,
                            ..Default::default()
                        },
                        |_| Vec::new(),
                    )]
                })
            },
        )
    }

    // Frame 0: establish viewport size so the visible range can be computed.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-row-cache-reuse",
        |cx| vec![build_list(cx, &scroll_handle, Arc::clone(&render_counts))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: mount initial visible rows (0..=2).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-row-cache-reuse",
        |cx| vec![build_list(cx, &scroll_handle, Arc::clone(&render_counts))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 2: scroll by one row; should only render the newly revealed row (3).
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(10.0)));
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-row-cache-reuse",
        |cx| vec![build_list(cx, &scroll_handle, Arc::clone(&render_counts))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 3: scroll by one more row; should only render the newly revealed row (4).
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(20.0)));
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-row-cache-reuse",
        |cx| vec![build_list(cx, &scroll_handle, Arc::clone(&render_counts))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let counts = render_counts.lock().expect("render_counts lock");
    assert_eq!(
        counts.len(),
        5,
        "expected only the 5 unique rows observed across scroll steps to render"
    );
    for (key, count) in counts.iter() {
        assert_eq!(*count, 1, "row {key} rendered unexpectedly multiple times");
    }
}
