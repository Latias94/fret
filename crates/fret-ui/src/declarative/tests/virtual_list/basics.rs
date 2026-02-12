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
        vec![0, 1, 2, 3, 4]
    );
    assert_eq!(ui.children(list_node).len(), 5);
}

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
