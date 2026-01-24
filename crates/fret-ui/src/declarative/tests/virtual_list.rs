use super::*;
use std::sync::Arc;

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

        let list_a = cx.virtual_list(100, options, scroll_handle, |cx, items| {
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
