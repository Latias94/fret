use super::*;

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
                    .collect()
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
                        .collect()
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
                        .collect()
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
                    .collect()
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
                    .collect()
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
                    .collect()
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
