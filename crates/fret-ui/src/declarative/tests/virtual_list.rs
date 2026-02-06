use super::*;
use fret_runtime::GlobalsHost;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn virtual_list_computes_visible_range_after_first_layout() {
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
        );
        *list_element_id = Some(list.id);
        list
    }

    // Frame 0: no viewport height is known yet (it is written during layout), so the list
    // renders with an empty visible range.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let list_node = ui.children(root)[0];
    assert_eq!(ui.children(list_node).len(), 0);
    let viewport_h = crate::elements::with_element_state(
        &mut app,
        window,
        list_element_id.unwrap(),
        crate::element::VirtualListState::default,
        |s| s.viewport_h,
    );
    assert_eq!(viewport_h, Px(50.0));

    // Frame 1: the list has recorded its viewport height during layout, so the authoring layer
    // can compute a visible range and mount only the visible children.
    app.advance_frame();
    let prev_list_element_id = list_element_id;
    let mut list_element_id: Option<crate::elements::GlobalElementId> = None;
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        prev_list_element_id, list_element_id,
        "virtual list element id should be stable across frames"
    );

    let list_node = ui.children(root)[0];
    let props = app.with_global_mut(super::super::frame::ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&list_node))
            .cloned()
    });
    let super::super::ElementInstance::VirtualList(props) =
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
        vec![0, 1, 2, 3, 4]
    );
    assert_eq!(ui.children(list_node).len(), 5);
}

#[test]
fn virtual_list_can_scroll_to_deep_index_then_to_end() {
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
        );
        *list_element_id = Some(list.id);
        list
    }

    // Frame 0: establish viewport size.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-end",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Frame 1: compute visible range for offset=0.
    app.advance_frame();
    let prev_list_element_id = list_element_id;
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-end",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(prev_list_element_id, list_element_id);

    let list_node = ui.children(root)[0];
    let props = app.with_global_mut(super::super::frame::ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&list_node))
            .cloned()
    });
    let super::super::ElementInstance::VirtualList(props) =
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
        vec![0, 1, 2, 3, 4]
    );

    scroll_handle.scroll_to_item(9000, crate::scroll::ScrollStrategy::Start);

    // Frame 2: consume the deferred scroll request during layout.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-end",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Frame 3: render the updated visible range.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-end",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let list_node = ui.children(root)[0];
    let props = app.with_global_mut(super::super::frame::ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&list_node))
            .cloned()
    });
    let super::super::ElementInstance::VirtualList(props) =
        props.expect("list instance exists").instance
    else {
        panic!("expected VirtualList instance");
    };
    assert!(props.visible_items.iter().any(|item| item.index == 9000));

    scroll_handle.scroll_to_item(9999, crate::scroll::ScrollStrategy::Start);
    assert_eq!(
        scroll_handle.deferred_scroll_to_item(),
        Some((9999, crate::scroll::ScrollStrategy::Start)),
        "scroll_to_item should record a deferred request until the next layout pass consumes it"
    );

    // Frame 4: consume the deferred scroll request during layout.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-end",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        scroll_handle.deferred_scroll_to_item().is_none(),
        "layout pass should consume the deferred scroll request"
    );

    // Frame 5: render the updated visible range.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-scroll-end",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let list_node = ui.children(root)[0];
    let props = app.with_global_mut(super::super::frame::ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&list_node))
            .cloned()
    });
    let super::super::ElementInstance::VirtualList(props) =
        props.expect("list instance exists").instance
    else {
        panic!("expected VirtualList instance");
    };
    if !props.visible_items.iter().any(|item| item.index == 9999) {
        let indices: Vec<usize> = props.visible_items.iter().map(|item| item.index).collect();
        let list_element_id = list_element_id.expect("list element id");
        let (state_items_len, state_offset_y, state_total_height) =
            crate::elements::with_element_state(
                &mut app,
                window,
                list_element_id,
                crate::element::VirtualListState::default,
                |s| (s.items_len, s.offset_y, s.metrics.total_height()),
            );
        panic!(
            "expected to be able to scroll to the final item after scrolling to a deep index; visible={indices:?} state.items_len={state_items_len} state.offset_y={state_offset_y:?} state.total_height={state_total_height:?}"
        );
    }
}

#[test]
fn virtual_list_computes_visible_range_after_first_layout_horizontal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(50.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();
    let mut list_element_id: Option<crate::elements::GlobalElementId> = None;

    fn build_list(
        cx: &mut ElementContext<'_, TestHost>,
        list_element_id: &mut Option<crate::elements::GlobalElementId>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> crate::element::AnyElement {
        let mut options = crate::element::VirtualListOptions::new(Px(10.0), 0);
        options.axis = fret_core::Axis::Horizontal;

        let list = cx.virtual_list(100, options, scroll_handle, |cx, items| {
            items
                .iter()
                .copied()
                .map(|item| {
                    cx.keyed(item.key, |cx| {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = crate::element::Length::Px(Px(10.0));
                        layout.size.height = crate::element::Length::Fill;
                        cx.container(
                            crate::element::ContainerProps {
                                layout,
                                ..Default::default()
                            },
                            |cx| vec![cx.text("col")],
                        )
                    })
                })
                .collect::<Vec<_>>()
        });
        *list_element_id = Some(list.id);
        list
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-x",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let list_node = ui.children(root)[0];
    assert_eq!(ui.children(list_node).len(), 0);
    let viewport_w = crate::elements::with_element_state(
        &mut app,
        window,
        list_element_id.unwrap(),
        crate::element::VirtualListState::default,
        |s| s.viewport_w,
    );
    assert_eq!(viewport_w, Px(50.0));

    app.advance_frame();
    let prev_list_element_id = list_element_id;
    let mut list_element_id: Option<crate::elements::GlobalElementId> = None;
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-x",
        |cx| vec![build_list(cx, &mut list_element_id, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        prev_list_element_id, list_element_id,
        "virtual list element id should be stable across frames"
    );

    let list_node = ui.children(root)[0];
    let props = app.with_global_mut(super::super::frame::ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&list_node))
            .cloned()
    });
    let super::super::ElementInstance::VirtualList(props) =
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
        vec![0, 1, 2, 3, 4]
    );
    assert_eq!(ui.children(list_node).len(), 5);
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn virtual_list_wraps_visible_items_in_engine_tree() {
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
    ) -> crate::element::AnyElement {
        cx.virtual_list(
            100,
            crate::element::VirtualListOptions::new(Px(10.0), 0),
            scroll_handle,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| {
                        cx.keyed(item.key, |cx| {
                            let mut props = crate::element::HoverRegionProps::default();
                            props.layout.size.width = crate::element::Length::Fill;
                            props.layout.size.height = crate::element::Length::Auto;

                            cx.hover_region(props, |cx, _hovered| vec![cx.text("row")])
                        })
                    })
                    .collect::<Vec<_>>()
            },
        )
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-engine-tree",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let list_node = ui.children(root)[0];
    assert_eq!(ui.children(list_node).len(), 0);

    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-engine-tree",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let list_node = ui.children(root)[0];
    let item_root = ui.children(list_node)[0];
    let item_text = ui.children(item_root)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(item_root).is_some());
    assert!(engine.layout_id_for_node(item_text).is_some());
    ui.put_layout_engine(engine);
}

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
    let props = app.with_global_mut(super::super::frame::ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&list_node))
            .cloned()
    });
    let super::super::ElementInstance::VirtualList(props) =
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
fn retained_virtual_list_updates_visible_range_on_wheel_scroll_without_notifying_view_cache() {
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

            let list_layout = crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: crate::element::Length::Fill,
                    height: crate::element::Length::Fill,
                    ..Default::default()
                },
                overflow: crate::element::Overflow::Clip,
                ..Default::default()
            };

            let key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn =
                Arc::new(|i| i as crate::ItemKey);
            let row: crate::windowed_surface_host::RetainedVirtualListRowFn<TestHost> =
                Arc::new(|cx, _| cx.text("row"));

            vec![cx.virtual_list_keyed_retained_with_layout(
                list_layout,
                100,
                crate::element::VirtualListOptions::new(Px(10.0), 0),
                scroll_handle,
                key_at,
                row,
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
        "mvp50-vlist-retained-refresh-on-wheel",
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
        "mvp50-vlist-retained-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let cache_node = ui.children(root)[0];
    let list_node = ui.children(cache_node)[0];
    assert_eq!(ui.children(list_node).len(), 5);
    app.advance_frame();

    // Frame 2: allow layout-driven bookkeeping (viewport/content size) to settle.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-retained-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 3: prefer a cache hit; render closure should be skipped.
    let calls_before_wheel = render_calls.load(Ordering::SeqCst);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-retained-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
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
        "expected retained host to schedule a window update without rerendering the view-cache root"
    );
    assert!(
        ui.should_reuse_view_cache_node(cache_node),
        "expected retained host wheel scroll to avoid `notify()` on the view-cache root"
    );
    app.advance_frame();

    // Frame 4: cache hit again, but the retained host reconcile should attach/detach rows to match
    // the new window without re-running the parent render closure.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-retained-refresh-on-wheel",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_wheel,
        "expected retained host window refresh to avoid rerendering the view-cache root"
    );

    let cache_node = ui.children(root)[0];
    let list_node = ui.children(cache_node)[0];
    let props = app.with_global_mut(super::super::frame::ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&list_node))
            .cloned()
    });
    let super::super::ElementInstance::VirtualList(props) =
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
        vec![8, 9, 10, 11, 12],
        "expected retained host reconcile to update the visible-item set after wheel scroll"
    );
    assert_eq!(ui.children(list_node).len(), 5);
}

#[test]
fn retained_virtual_list_prefetches_window_before_escape_without_rerendering_cache_root() {
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

            let list_layout = crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: crate::element::Length::Fill,
                    height: crate::element::Length::Fill,
                    ..Default::default()
                },
                overflow: crate::element::Overflow::Clip,
                ..Default::default()
            };

            let key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn =
                Arc::new(|i| i as crate::ItemKey);
            let row: crate::windowed_surface_host::RetainedVirtualListRowFn<TestHost> =
                Arc::new(|cx, i| cx.text(format!("row {i}")));

            // Overscan > 0 is required to test staged prefetch (shift before escape).
            let options = crate::element::VirtualListOptions::new(Px(10.0), 4);
            vec![cx.virtual_list_keyed_retained_with_layout(
                list_layout,
                100,
                options,
                scroll_handle,
                key_at,
                row,
            )]
        })
    }

    // Establish viewport and mount initial children.
    for _frame in 0..3 {
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-retained-prefetch",
            |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        app.advance_frame();
    }

    // Prefer a cache hit before changing the scroll offset.
    let calls_before_scroll = render_calls.load(Ordering::SeqCst);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-retained-prefetch",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_scroll,
        "expected a cache hit before prefetching"
    );
    let list_element = ui
        .debug_virtual_list_windows()
        .last()
        .expect("expected at least one virtual list window debug record")
        .element;

    // Move near the overscan boundary but stay within it so this is a prefetch (not an escape).
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(30.0)));
    app.advance_frame();

    // Frame: cache hit; prepaint should request a prefetch reconcile.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-retained-prefetch",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    // Ensure prepaint still observes a lagging "render window" (the previous window) while the
    // scroll offset and ideal window have advanced. This matches the steady-state cache-hit case
    // we care about: prepaint should request a staged prefetch reconcile without forcing the
    // cache-root subtree to rerender.
    crate::elements::with_element_state(
        &mut app,
        window,
        list_element,
        crate::element::VirtualListState::default,
        |state| {
            state.render_window_range = Some(crate::virtual_list::VirtualRange {
                start_index: 0,
                end_index: 4,
                overscan: 4,
                count: 100,
            });
        },
    );
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_scroll,
        "expected staged prefetch to avoid rerendering the view-cache root"
    );
    let last_window = ui
        .debug_virtual_list_windows()
        .iter()
        .rev()
        .find(|w| {
            matches!(
                w.source,
                crate::tree::UiDebugVirtualListWindowSource::Prepaint
            )
        })
        .expect("expected a prepaint virtual list window debug record");
    assert!(
        !last_window.window_mismatch,
        "expected prefetch to occur while still within the rendered prefetch window"
    );
    assert_eq!(
        last_window.window_shift_kind,
        crate::tree::UiDebugVirtualListWindowShiftKind::Prefetch,
        "expected prepaint to stage a prefetch window shift (record={last_window:?})"
    );

    // Next frame: reconcile executes (still cache-hit) and is attributed as a prefetch.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-retained-prefetch",
        |cx| vec![build_tree(cx, &scroll_handle, Arc::clone(&render_calls))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        render_calls.load(Ordering::SeqCst),
        calls_before_scroll,
        "expected staged prefetch reconcile to avoid rerendering the view-cache root"
    );
    assert!(
        ui.debug_retained_virtual_list_reconciles()
            .iter()
            .any(|r| r.reconcile_kind
                == crate::tree::UiDebugRetainedVirtualListReconcileKind::Prefetch),
        "expected at least one prefetch-attributed retained virtual-list reconcile"
    );
}

#[test]
fn retained_virtual_list_keep_alive_reuses_detached_items_when_scrolling_back() {
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

    fn build_tree(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        let mut cache = crate::element::ViewCacheProps::default();
        cache.layout.size.width = crate::element::Length::Fill;
        cache.layout.size.height = crate::element::Length::Fill;
        cache.cache_key = 1;

        cx.view_cache(cache, move |cx| {
            let list_layout = crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: crate::element::Length::Fill,
                    height: crate::element::Length::Fill,
                    ..Default::default()
                },
                overflow: crate::element::Overflow::Clip,
                ..Default::default()
            };

            let key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn =
                Arc::new(|i| i as crate::ItemKey);
            let row: crate::windowed_surface_host::RetainedVirtualListRowFn<TestHost> =
                Arc::new(|cx, i| cx.text(format!("row {i}")));

            let options = crate::element::VirtualListOptions::new(Px(10.0), 0).keep_alive(32);
            vec![cx.virtual_list_keyed_retained_with_layout(
                list_layout,
                100,
                options,
                scroll_handle,
                key_at,
                row,
            )]
        })
    }

    // Establish viewport and mount initial children.
    for _frame in 0..3 {
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "retained-virt-keep-alive",
            |cx| vec![build_tree(cx, &scroll_handle)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        app.advance_frame();
    }

    // Scroll down across the window boundary (forces reconcile + detach).
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(60.0)));
    let mut kept_alive_any = false;
    for _frame in 0..2 {
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "retained-virt-keep-alive",
            |cx| vec![build_tree(cx, &scroll_handle)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        kept_alive_any |= ui
            .debug_retained_virtual_list_reconciles()
            .iter()
            .any(|r| r.kept_alive_items > 0);
        app.advance_frame();
    }
    assert!(
        kept_alive_any,
        "expected reconcile to retain some keep-alive items"
    );

    // Scroll back to the top; the host should reuse some kept-alive items instead of mounting them
    // again from scratch.
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(0.0)));
    let mut reused_any = false;
    for _frame in 0..2 {
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "retained-virt-keep-alive",
            |cx| vec![build_tree(cx, &scroll_handle)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        reused_any |= ui
            .debug_retained_virtual_list_reconciles()
            .iter()
            .any(|r| r.reused_from_keep_alive_items > 0);
        app.advance_frame();
    }
    assert!(
        reused_any,
        "expected reconcile to reuse items from the keep-alive bucket when scrolling back"
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
    let props = app.with_global_mut(super::super::frame::ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&list_node))
            .cloned()
    });
    let super::super::ElementInstance::VirtualList(props) =
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

#[cfg(feature = "layout-engine-v2")]
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

#[test]
fn virtual_list_click_focus_does_not_trigger_scroll_jump_under_children_transform() {
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
                        cx.keyed(item.key, |cx| {
                            let mut props = crate::element::PressableProps::default();
                            props.layout.size.width = crate::element::Length::Fill;
                            props.layout.size.height = crate::element::Length::Px(Px(28.0));
                            props.focusable = true;
                            props.a11y.role = Some(fret_core::SemanticsRole::Button);
                            props.a11y.test_id =
                                Some(Arc::<str>::from(format!("virtual-list-row-{}", item.index)));
                            cx.pressable(props, |cx, _state| vec![cx.text("row")])
                        })
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
        "mvp50-vlist-focus-click-no-scroll-jump",
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
        "mvp50-vlist-focus-click-no-scroll-jump",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    // Frame 2: rerender the visible range and click a focusable row; it must not trigger an
    // erroneous scroll-into-view jump.
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-vlist-focus-click-no-scroll-jump",
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

    let pos = Point::new(
        Px(node.bounds.origin.x.0 + 2.0),
        Px(node.bounds.origin.y.0 + 2.0),
    );

    let before = scroll_handle.offset().y;
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let after = scroll_handle.offset().y;
    assert!(
        (after.0 - before.0).abs() <= 0.01,
        "clicking a visible focusable row must not cause a scroll jump: before={:?} after={:?}",
        before,
        after
    );
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
fn retained_virtual_list_host_updates_window_without_rerendering_view_cache_root() {
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

    fn build_list(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        let mut options = crate::element::VirtualListOptions::new(Px(10.0), 0);
        options.measure_mode = crate::element::VirtualListMeasureMode::Fixed;
        let mut layout = crate::element::LayoutStyle::default();
        layout.size.height = crate::element::Length::Fill;

        let key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn =
            Arc::new(|i| i as crate::ItemKey);

        let row: crate::windowed_surface_host::RetainedVirtualListRowFn<TestHost> =
            Arc::new(|cx, index| {
                let mut style = crate::element::LayoutStyle::default();
                style.size.height = crate::element::Length::Px(Px(10.0));
                cx.semantics(
                    crate::element::SemanticsProps {
                        role: fret_core::SemanticsRole::ListItem,
                        label: Some(Arc::<str>::from(format!("Row {index}"))),
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.container(
                            crate::element::ContainerProps {
                                layout: style,
                                ..Default::default()
                            },
                            |cx| vec![cx.text(format!("Row {index}"))],
                        )]
                    },
                )
            });

        cx.virtual_list_keyed_retained_with_layout(
            layout,
            200,
            options,
            scroll_handle,
            Arc::clone(&key_at),
            Arc::clone(&row),
        )
    }

    // Virtual lists need a "viewport-known" render to mount their first visible window. Do an
    // initial two-frame warmup with view caching disabled so `VirtualListState.viewport_*` gets
    // populated during layout, then allow the next render to build the initial children.
    for _frame in 0..2 {
        let scroll_handle = scroll_handle.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "retained-virt-003",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        vec![build_list(cx, &scroll_handle)]
                    }),
                ]
            },
        );

        ui.set_root(root_node);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
        app.advance_frame();
    }

    ui.set_view_cache_enabled(true);
    let renders = Arc::new(AtomicUsize::new(0));

    // Establish cache-hit steady state: the view-cache closure should only run once.
    for _frame in 0..6 {
        let renders = Arc::clone(&renders);
        let scroll_handle = scroll_handle.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "retained-virt-003",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        vec![build_list(cx, &scroll_handle)]
                    }),
                ]
            },
        );

        ui.set_root(root_node);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
        app.advance_frame();
    }

    let baseline_renders = renders.load(Ordering::SeqCst);
    assert!(
        baseline_renders <= 2,
        "expected view-cache warmup to stabilize quickly (renders={baseline_renders})"
    );

    let (vlist_element, vlist_instances) =
        crate::declarative::with_window_frame(&mut app, window, |window_frame| {
            let window_frame = window_frame?;
            let vlists = window_frame
                .instances
                .iter()
                .filter_map(|(_, record)| {
                    matches!(
                        record.instance,
                        crate::declarative::frame::ElementInstance::VirtualList(_)
                    )
                    .then_some(record.element)
                })
                .collect::<Vec<_>>();
            Some(vlists)
        })
        .map(|vlists| {
            let first = *vlists.first().expect("virtual list instance");
            (first, vlists.len())
        })
        .expect("window frame");
    assert_eq!(
        vlist_instances, 1,
        "expected a single virtual list instance"
    );

    assert!(
        crate::elements::with_window_state(&mut app, window, |window_state| {
            window_state.has_state::<crate::windowed_surface_host::RetainedVirtualListHostMarker>(
                vlist_element,
            )
        }),
        "expected retained virtual list host marker state to exist"
    );

    let bound = crate::declarative::frame::bound_elements_for_scroll_handle(
        &mut app,
        window,
        scroll_handle.base_handle().binding_key(),
    );
    assert!(
        bound.contains(&vlist_element),
        "expected the virtual list element to be bound to the scroll handle (bound={bound:?})"
    );

    let (registry_frame_id, registry_handles, registry_by_handle) = app.with_global_mut_untracked(
        crate::declarative::frame::ScrollHandleRegistry::default,
        |registry, _| {
            let window_registry = registry
                .windows
                .get(&window)
                .expect("scroll handle registry window");
            (
                window_registry.frame_id,
                window_registry.handles.len(),
                window_registry.by_handle.len(),
            )
        },
    );
    assert!(
        registry_handles > 0,
        "expected scroll handle registry to have handles after warmup (frame_id={registry_frame_id:?}, handles={registry_handles}, by_handle={registry_by_handle})"
    );

    // Jump outside the previously rendered overscan window (e.g. 0..=2 -> 3..=5).
    //
    // Use a relative delta so we guarantee an actual offset change even if a previous frame
    // adjusted the handle internally (e.g. clamping).
    let prev_offset = scroll_handle.offset();
    let prev_revision = scroll_handle.revision();
    let next_offset = Px(prev_offset.y.0 + 30.0);
    scroll_handle.set_offset(fret_core::Point::new(prev_offset.x, next_offset));
    assert_ne!(
        scroll_handle.revision(),
        prev_revision,
        "expected set_offset to bump the scroll handle revision (prev_offset={prev_offset:?}, next_offset={:?})",
        scroll_handle.offset(),
    );

    let expected_start = (next_offset.0 / 10.0) as usize;

    let mut renders_after_scroll_frames: Vec<usize> = Vec::new();
    for _frame in 0..2 {
        let renders_for_closure = Arc::clone(&renders);
        let scroll_handle = scroll_handle.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "retained-virt-003",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), |cx| {
                        renders_for_closure.fetch_add(1, Ordering::SeqCst);
                        vec![build_list(cx, &scroll_handle)]
                    }),
                ]
            },
        );
        ui.set_root(root_node);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
        renders_after_scroll_frames.push(renders.load(Ordering::SeqCst));
        app.advance_frame();
    }

    assert_eq!(
        renders_after_scroll_frames.len(),
        2,
        "expected two scroll frames"
    );
    assert_eq!(
        renders_after_scroll_frames[0], renders_after_scroll_frames[1],
        "expected scroll frames to stabilize after the first frame (renders_after_scroll_frames={renders_after_scroll_frames:?})"
    );
    assert!(
        renders_after_scroll_frames[1] <= baseline_renders.saturating_add(1),
        "expected retained host reconciliation to avoid repeated rerendering the cache root (baseline_renders={baseline_renders}, renders_after_scroll_frames={renders_after_scroll_frames:?})"
    );

    let vlist_node =
        crate::declarative::node_for_element_in_window_frame(&mut app, window, vlist_element)
            .expect("virtual list node exists");
    let (visible_indices, frame_children_len, ui_children_len): (Vec<usize>, usize, usize) =
        crate::declarative::with_window_frame(&mut app, window, |window_frame| {
            let window_frame = window_frame?;
            let record = window_frame.instances.get(&vlist_node)?;
            let crate::declarative::frame::ElementInstance::VirtualList(props) = &record.instance
            else {
                return None;
            };
            let visible_indices = props
                .visible_items
                .iter()
                .map(|item| item.index)
                .collect::<Vec<_>>();
            let frame_children_len = window_frame
                .children
                .get(&vlist_node)
                .map(|c| c.len())
                .unwrap_or(0);
            Some((
                visible_indices,
                frame_children_len,
                ui.children(vlist_node).len(),
            ))
        })
        .expect("window frame");

    assert!(
        !visible_indices.is_empty(),
        "expected virtual list to have visible items after scroll (expected_start={expected_start}, ui_children_len={ui_children_len}, frame_children_len={frame_children_len})"
    );
    assert!(
        visible_indices.contains(&expected_start),
        "expected the reconciled window to include index {expected_start} (visible_indices={visible_indices:?})"
    );
    assert!(
        visible_indices.contains(&(expected_start + 1)),
        "expected the reconciled window to include index {} (visible_indices={visible_indices:?})",
        expected_start + 1
    );
    assert!(
        visible_indices.contains(&(expected_start + 2)),
        "expected the reconciled window to include index {} (visible_indices={visible_indices:?})",
        expected_start + 2
    );
}

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

#[test]
fn virtual_list_keyed_reuses_node_ids_across_reorder() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(30.0)),
    );
    let mut text = FakeTextService::default();

    let mut items: Vec<u64> = vec![10, 20, 30];
    let mut ids: Vec<(u64, crate::elements::GlobalElementId)> = Vec::new();

    fn build_list<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        items: &[u64],
        mut ids: Option<&mut Vec<(u64, crate::elements::GlobalElementId)>>,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
    ) -> AnyElement {
        let items_revision = items
            .iter()
            .fold(0u64, |acc, k| acc.wrapping_mul(1_000_003).wrapping_add(*k));
        let mut options = crate::element::VirtualListOptions::new(Px(10.0), 0);
        options.items_revision = items_revision;

        cx.virtual_list_keyed(
            items.len(),
            options,
            scroll_handle,
            |i| items[i],
            |cx, i| {
                let row = cx.text("row");
                if let Some(ids) = ids.as_deref_mut() {
                    ids.push((items[i], row.id));
                }
                row
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
        "mvp50-vlist-keyed",
        |cx| vec![build_list(cx, &items, None, &scroll_handle)],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    let mut prev: std::collections::HashMap<u64, (crate::elements::GlobalElementId, NodeId)> =
        std::collections::HashMap::new();

    for pass in 0..2 {
        ids.clear();
        let root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "mvp50-vlist-keyed",
            |cx| vec![build_list(cx, &items, Some(&mut ids), &scroll_handle)],
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        let cur: std::collections::HashMap<u64, (crate::elements::GlobalElementId, NodeId)> = app
            .with_global_mut(crate::elements::ElementRuntime::new, |runtime, app| {
                runtime.prepare_window_for_frame(window, app.frame_id());
                let st = runtime.for_window_mut(window);
                ids.iter()
                    .map(|(item, id)| (*item, (*id, st.node_entry(*id).unwrap().node)))
                    .collect::<std::collections::HashMap<_, _>>()
            });

        if pass == 1 {
            for item in [10u64, 20u64, 30u64] {
                let (prev_id, prev_node) = prev.get(&item).copied().unwrap();
                let (cur_id, cur_node) = cur.get(&item).copied().unwrap();
                assert_eq!(prev_id, cur_id, "element id should be stable");
                assert_eq!(prev_node, cur_node, "node id should be stable");
            }
        }

        prev = cur;
        items.reverse();
        app.advance_frame();
    }
}
