use super::*;

#[test]
fn focus_traversal_scrolls_focused_descendant_into_view() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let scroll_handle = crate::scroll::ScrollHandle::default();

    let mut first: Option<GlobalElementId> = None;
    let mut second: Option<GlobalElementId> = None;

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| {
            vec![cx.keyed(1, |cx| {
                cx.scroll(
                    crate::element::ScrollProps {
                        layout: {
                            let mut layout = crate::element::LayoutStyle::default();
                            layout.size.width = crate::element::Length::Fill;
                            layout.size.height = crate::element::Length::Px(Px(40.0));
                            layout.overflow = crate::element::Overflow::Clip;
                            layout
                        },
                        scroll_handle: Some(scroll_handle.clone()),
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.flex(
                            crate::element::FlexProps {
                                layout: crate::element::LayoutStyle::default(),
                                direction: fret_core::Axis::Vertical,
                                gap: Px(0.0).into(),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: crate::element::MainAlign::Start,
                                align: crate::element::CrossAlign::Stretch,
                                wrap: false,
                            },
                            |cx| {
                                vec![
                                    cx.keyed(10, |cx| {
                                        cx.pressable_with_id(
                                            crate::element::PressableProps {
                                                layout: {
                                                    let mut layout =
                                                        crate::element::LayoutStyle::default();
                                                    layout.size.width =
                                                        crate::element::Length::Fill;
                                                    layout.size.height =
                                                        crate::element::Length::Px(Px(20.0));
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |_cx, _st, id| {
                                                first = Some(id);
                                                Vec::new()
                                            },
                                        )
                                    }),
                                    cx.container(
                                        crate::element::ContainerProps {
                                            layout: {
                                                let mut layout =
                                                    crate::element::LayoutStyle::default();
                                                layout.size.width = crate::element::Length::Fill;
                                                layout.size.height =
                                                    crate::element::Length::Px(Px(200.0));
                                                layout
                                            },
                                            ..Default::default()
                                        },
                                        |_| Vec::new(),
                                    ),
                                    cx.keyed(11, |cx| {
                                        cx.pressable_with_id(
                                            crate::element::PressableProps {
                                                layout: {
                                                    let mut layout =
                                                        crate::element::LayoutStyle::default();
                                                    layout.size.width =
                                                        crate::element::Length::Fill;
                                                    layout.size.height =
                                                        crate::element::Length::Px(Px(20.0));
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |_cx, _st, id| {
                                                second = Some(id);
                                                Vec::new()
                                            },
                                        )
                                    }),
                                ]
                            },
                        )]
                    },
                )
            })]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let first = first.expect("first pressable id");
    let second = second.expect("second pressable id");

    let scroll_node = ui
        .nodes
        .get(root)
        .and_then(|n| n.children.first().copied())
        .expect("scroll node");
    let first_node = crate::elements::node_for_element(&mut app, window, first).expect("first");
    let second_node = crate::elements::node_for_element(&mut app, window, second).expect("second");

    ui.set_focus(Some(first_node));

    let did_handle = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert!(did_handle);
    assert_eq!(ui.focus(), Some(second_node));
    assert!(scroll_handle.offset().y.0 > 0.0);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_bounds = ui
        .debug_node_visual_bounds(scroll_node)
        .expect("scroll visual bounds");
    let second_bounds = ui
        .debug_node_visual_bounds(second_node)
        .expect("second visual bounds");

    assert!(
        UiTree::<crate::test_host::TestHost>::rects_intersect(scroll_bounds, second_bounds),
        "expected focused node to be in view after scroll-into-view"
    );
}

#[test]
fn scroll_node_into_view_does_not_scroll_scrollable_target_via_itself() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );

    fn build_list<H: UiHost>(
        cx: &mut crate::ElementContext<'_, H>,
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
            |cx, items: &[crate::virtual_list::VirtualItem]| {
                items
                    .iter()
                    .copied()
                    .map(|item| {
                        cx.keyed(item.key, |cx| {
                            let mut style = crate::element::LayoutStyle::default();
                            style.size.width = crate::element::Length::Fill;
                            style.size.height = crate::element::Length::Px(Px(28.0));
                            cx.container(
                                crate::element::ContainerProps {
                                    layout: style,
                                    ..Default::default()
                                },
                                |_| Vec::new(),
                            )
                        })
                    })
                    .collect::<Vec<_>>()
            },
        )
        .test_id("vlist-root")
    }

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| vec![build_list(cx, &scroll_handle)],
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
    let vlist_node = snapshot
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("vlist-root"))
        .map(|n| n.id)
        .expect("expected virtual list semantics node");

    // Establish a non-zero scroll offset.
    scroll_handle.set_offset(Point::new(Px(0.0), Px(999_999.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let before = scroll_handle.offset().y;
    assert!(before.0 > 0.01, "expected a non-zero scroll offset");

    // Scrolling a target into view should only scroll its ancestors, not the target itself.
    let _ = ui.scroll_node_into_view(&mut app, vlist_node);
    let after = scroll_handle.offset().y;
    assert!(
        (after.0 - before.0).abs() <= 0.01,
        "expected scroll_node_into_view to avoid mutating the target's own offset: before={:?} after={:?}",
        before,
        after
    );
}

#[test]
fn scroll_into_view_does_not_drift_virtual_list_when_descendant_is_already_visible() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();

    let mut target_id: Option<GlobalElementId> = None;
    let target_index = 5usize;

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    // Frame 0: establish content/viewport sizes for the handle.
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "virtual-list-scroll-into-view",
        |cx| {
            vec![cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(20.0), 0),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| {
                            cx.keyed(item.key, |cx| {
                                cx.pressable_with_id(
                                    crate::element::PressableProps {
                                        layout: {
                                            let mut layout = crate::element::LayoutStyle::default();
                                            layout.size.width = crate::element::Length::Fill;
                                            layout.size.height =
                                                crate::element::Length::Px(Px(20.0));
                                            layout
                                        },
                                        enabled: true,
                                        focusable: true,
                                        ..Default::default()
                                    },
                                    |_cx, _st, _id| Vec::new(),
                                )
                            })
                        })
                        .collect::<Vec<_>>()
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Scroll to show `target_index` as the first visible row.
    scroll_handle.set_offset(Point::new(Px(0.0), Px(100.0)));

    // Frame 1: materialize the scrolled rows and capture the target node id.
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "virtual-list-scroll-into-view",
        |cx| {
            vec![cx.virtual_list(
                100,
                crate::element::VirtualListOptions::new(Px(20.0), 0),
                &scroll_handle,
                |cx, items| {
                    items
                        .iter()
                        .copied()
                        .map(|item| {
                            cx.keyed(item.key, |cx| {
                                let index = item.index;
                                cx.pressable_with_id(
                                    crate::element::PressableProps {
                                        layout: {
                                            let mut layout = crate::element::LayoutStyle::default();
                                            layout.size.width = crate::element::Length::Fill;
                                            layout.size.height =
                                                crate::element::Length::Px(Px(20.0));
                                            layout
                                        },
                                        enabled: true,
                                        focusable: true,
                                        ..Default::default()
                                    },
                                    |_cx, _st, id| {
                                        if index == target_index {
                                            target_id = Some(id);
                                        }
                                        Vec::new()
                                    },
                                )
                            })
                        })
                        .collect::<Vec<_>>()
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let target_id = target_id.expect("target element id");
    let target_node = crate::elements::node_for_element(&mut app, window, target_id).expect("node");

    let before = scroll_handle.offset();
    assert!(
        (before.y.0 - 100.0).abs() < 0.01,
        "expected initial scroll offset ~=100, got={:?}",
        before
    );

    let did_scroll = ui.scroll_node_into_view(&mut app, target_node);
    assert!(
        !did_scroll,
        "expected scroll_into_view to be a no-op for already-visible virtual list content"
    );
    assert!(
        (scroll_handle.offset().y.0 - before.y.0).abs() < 0.01,
        "expected scroll offset to remain stable: before={:?} after={:?}",
        before,
        scroll_handle.offset()
    );
}

#[test]
fn scroll_into_view_does_not_drift_scroll_when_descendant_is_already_visible() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let scroll_handle = crate::scroll::ScrollHandle::default();

    let mut target_id: Option<GlobalElementId> = None;
    let target_index = 5usize;

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "scroll-scroll-into-view",
        |cx| {
            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = crate::element::Length::Fill;
                        layout.size.height = crate::element::Length::Px(Px(100.0));
                        layout.overflow = crate::element::Overflow::Clip;
                        layout
                    },
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle::default(),
                            direction: fret_core::Axis::Vertical,
                            gap: Px(0.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: crate::element::MainAlign::Start,
                            align: crate::element::CrossAlign::Stretch,
                            wrap: false,
                        },
                        |cx| {
                            (0..50)
                                .map(|index| {
                                    cx.keyed(index, |cx| {
                                        cx.pressable_with_id(
                                            crate::element::PressableProps {
                                                layout: {
                                                    let mut layout =
                                                        crate::element::LayoutStyle::default();
                                                    layout.size.width =
                                                        crate::element::Length::Fill;
                                                    layout.size.height =
                                                        crate::element::Length::Px(Px(20.0));
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |_cx, _st, id| {
                                                if index == target_index {
                                                    target_id = Some(id);
                                                }
                                                Vec::new()
                                            },
                                        )
                                    })
                                })
                                .collect::<Vec<_>>()
                        },
                    )]
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let target_id = target_id.expect("target element id");
    let target_node = crate::elements::node_for_element(&mut app, window, target_id).expect("node");

    // Scroll to show `target_index` as the first visible row.
    scroll_handle.set_offset(Point::new(Px(0.0), Px(100.0)));

    let before = scroll_handle.offset();
    assert!(
        (before.y.0 - 100.0).abs() < 0.01,
        "expected initial scroll offset ~=100, got={:?}",
        before
    );

    let did_scroll = ui.scroll_node_into_view(&mut app, target_node);
    assert!(
        !did_scroll,
        "expected scroll_into_view to be a no-op for already-visible scroll content"
    );
    assert!(
        (scroll_handle.offset().y.0 - before.y.0).abs() < 0.01,
        "expected scroll offset to remain stable: before={:?} after={:?}",
        before,
        scroll_handle.offset()
    );
}

#[test]
fn scroll_into_view_continues_past_nested_scroll_when_inner_target_is_already_visible() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let outer_handle = crate::scroll::ScrollHandle::default();
    let inner_handle = crate::scroll::ScrollHandle::default();

    let target_id = std::rc::Rc::new(std::cell::Cell::new(None::<GlobalElementId>));

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "scroll-into-view-nested-scroll-outer-continues",
        {
            let outer_handle = outer_handle.clone();
            let inner_handle = inner_handle.clone();
            let target_id = target_id.clone();
            move |cx| {
                vec![cx.scroll(
                    crate::element::ScrollProps {
                        layout: {
                            let mut layout = crate::element::LayoutStyle::default();
                            layout.size.width = crate::element::Length::Px(Px(200.0));
                            layout.size.height = crate::element::Length::Px(Px(100.0));
                            layout.overflow = crate::element::Overflow::Clip;
                            layout
                        },
                        axis: crate::element::ScrollAxis::Y,
                        scroll_handle: Some(outer_handle.clone()),
                        probe_unbounded: true,
                        ..Default::default()
                    },
                    {
                        let inner_handle = inner_handle.clone();
                        move |cx| {
                            vec![cx.flex(
                                crate::element::FlexProps {
                                    layout: crate::element::LayoutStyle::default(),
                                    direction: fret_core::Axis::Vertical,
                                    gap: Px(0.0).into(),
                                    padding: Edges::all(Px(0.0)).into(),
                                    justify: crate::element::MainAlign::Start,
                                    align: crate::element::CrossAlign::Stretch,
                                    wrap: false,
                                },
                                {
                                    let inner_handle = inner_handle.clone();
                                    move |cx| {
                                        let target_id = target_id.clone();
                                        let top = cx.container(
                                            crate::element::ContainerProps {
                                                layout: {
                                                    let mut layout = crate::element::LayoutStyle::default();
                                                    layout.size.width = crate::element::Length::Fill;
                                                    layout.size.height = crate::element::Length::Px(Px(120.0));
                                                    layout
                                                },
                                                ..Default::default()
                                            },
                                            |_cx| Vec::new(),
                                        );
                                        let inner = cx.scroll(
                                            crate::element::ScrollProps {
                                                layout: {
                                                    let mut layout = crate::element::LayoutStyle::default();
                                                    layout.size.width = crate::element::Length::Fill;
                                                    layout.size.height = crate::element::Length::Px(Px(60.0));
                                                    layout.overflow = crate::element::Overflow::Clip;
                                                    layout
                                                },
                                                axis: crate::element::ScrollAxis::Y,
                                                scroll_handle: Some(inner_handle.clone()),
                                                probe_unbounded: true,
                                                ..Default::default()
                                            },
                                            |cx| {
                                                vec![cx.flex(
                                                    crate::element::FlexProps {
                                                        layout: crate::element::LayoutStyle::default(),
                                                        direction: fret_core::Axis::Vertical,
                                                        gap: Px(0.0).into(),
                                                        padding: Edges::all(Px(0.0)).into(),
                                                        justify: crate::element::MainAlign::Start,
                                                        align: crate::element::CrossAlign::Stretch,
                                                        wrap: false,
                                                    },
                                                    |cx| {
                                                        (0..6)
                                                            .map(|index| {
                                                                cx.keyed(index, |cx| {
                                                                    cx.pressable_with_id(
                                                                        crate::element::PressableProps {
                                                                            layout: {
                                                                                let mut layout = crate::element::LayoutStyle::default();
                                                                                layout.size.width = crate::element::Length::Fill;
                                                                                layout.size.height = crate::element::Length::Px(Px(40.0));
                                                                                layout
                                                                            },
                                                                            enabled: true,
                                                                            focusable: true,
                                                                            ..Default::default()
                                                                        },
                                                                        |_cx, _st, id| {
                                                                            if index == 0 {
                                                                                target_id.set(Some(id));
                                                                            }
                                                                            Vec::new()
                                                                        },
                                                                    )
                                                                })
                                                            })
                                                            .collect::<Vec<_>>()
                                                    },
                                                )]
                                            },
                                        );
                                        let bottom = cx.container(
                                            crate::element::ContainerProps {
                                                layout: {
                                                    let mut layout = crate::element::LayoutStyle::default();
                                                    layout.size.width = crate::element::Length::Fill;
                                                    layout.size.height = crate::element::Length::Px(Px(40.0));
                                                    layout
                                                },
                                                ..Default::default()
                                            },
                                            |_cx| Vec::new(),
                                        );
                                        vec![top, inner, bottom]
                                    }
                                },
                            )]
                        }
                    },
                )]
            }
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let outer_scroll_node = ui.children(root)[0];
    let target_id = target_id.get().expect("target element id");
    let target_node = crate::elements::node_for_element(&mut app, window, target_id).expect("node");

    let bound = crate::declarative::frame::bound_elements_for_scroll_handle(
        &mut app,
        window,
        inner_handle.binding_key(),
    );
    let inner_scroll_element = bound
        .first()
        .copied()
        .expect("expected inner scroll to be registered for its handle");
    let inner_scroll_node = crate::declarative::node_for_element_in_window_frame(
        &mut app,
        window,
        inner_scroll_element,
    )
    .expect("expected inner scroll node");
    let inner_scroll_bounds = ui
        .debug_node_bounds(inner_scroll_node)
        .expect("inner scroll bounds");
    assert!(
        inner_scroll_bounds.size.height.0 > 0.5,
        "expected inner scroll node to have a non-zero viewport bounds; bounds={inner_scroll_bounds:?} viewport={:?}",
        inner_handle.viewport_size(),
    );

    assert!(
        inner_handle.max_offset().y.0 > 0.5,
        "expected nested inner scroll to have a real scroll range before scroll_into_view: viewport={:?} content={:?} max={:?}",
        inner_handle.viewport_size(),
        inner_handle.content_size(),
        inner_handle.max_offset(),
    );

    let did_scroll = ui.scroll_node_into_view(&mut app, target_node);
    assert!(
        did_scroll,
        "expected nested scroll_into_view to continue to the outer scroll ancestor"
    );
    assert!(
        inner_handle.offset().y.0.abs() <= 0.01,
        "expected inner scroll to remain stable when the target is already visible inside it: offset={:?}",
        inner_handle.offset(),
    );
    assert!(
        outer_handle.offset().y.0 > 0.5,
        "expected outer scroll to move once the target lives below the outer viewport: offset={:?}",
        outer_handle.offset(),
    );

    let _ = outer_scroll_node;
    let _ = target_node;
}

#[test]
fn scroll_into_view_scrolls_inner_and_outer_nested_scroll_ancestors() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let outer_handle = crate::scroll::ScrollHandle::default();
    let inner_handle = crate::scroll::ScrollHandle::default();

    let target_id = std::rc::Rc::new(std::cell::Cell::new(None::<GlobalElementId>));
    let target_index = 5usize;

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "scroll-into-view-nested-scroll-both-ancestors",
        {
            let outer_handle = outer_handle.clone();
            let inner_handle = inner_handle.clone();
            let target_id = target_id.clone();
            move |cx| {
                vec![cx.scroll(
                    crate::element::ScrollProps {
                        layout: {
                            let mut layout = crate::element::LayoutStyle::default();
                            layout.size.width = crate::element::Length::Px(Px(200.0));
                            layout.size.height = crate::element::Length::Px(Px(100.0));
                            layout.overflow = crate::element::Overflow::Clip;
                            layout
                        },
                        axis: crate::element::ScrollAxis::Y,
                        scroll_handle: Some(outer_handle.clone()),
                        probe_unbounded: true,
                        ..Default::default()
                    },
                    {
                        let inner_handle = inner_handle.clone();
                        move |cx| {
                            vec![cx.flex(
                                crate::element::FlexProps {
                                    layout: crate::element::LayoutStyle::default(),
                                    direction: fret_core::Axis::Vertical,
                                    gap: Px(0.0).into(),
                                    padding: Edges::all(Px(0.0)).into(),
                                    justify: crate::element::MainAlign::Start,
                                    align: crate::element::CrossAlign::Stretch,
                                    wrap: false,
                                },
                                {
                                    let inner_handle = inner_handle.clone();
                                    move |cx| {
                                        let target_id = target_id.clone();
                                        let top = cx.container(
                                            crate::element::ContainerProps {
                                                layout: {
                                                    let mut layout = crate::element::LayoutStyle::default();
                                                    layout.size.width = crate::element::Length::Fill;
                                                    layout.size.height = crate::element::Length::Px(Px(120.0));
                                                    layout
                                                },
                                                ..Default::default()
                                            },
                                            |_cx| Vec::new(),
                                        );
                                        let inner = cx.scroll(
                                            crate::element::ScrollProps {
                                                layout: {
                                                    let mut layout = crate::element::LayoutStyle::default();
                                                    layout.size.width = crate::element::Length::Fill;
                                                    layout.size.height = crate::element::Length::Px(Px(60.0));
                                                    layout.overflow = crate::element::Overflow::Clip;
                                                    layout
                                                },
                                                axis: crate::element::ScrollAxis::Y,
                                                scroll_handle: Some(inner_handle.clone()),
                                                probe_unbounded: true,
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                vec![cx.flex(
                                                    crate::element::FlexProps {
                                                        layout: crate::element::LayoutStyle::default(),
                                                        direction: fret_core::Axis::Vertical,
                                                        gap: Px(0.0).into(),
                                                        padding: Edges::all(Px(0.0)).into(),
                                                        justify: crate::element::MainAlign::Start,
                                                        align: crate::element::CrossAlign::Stretch,
                                                        wrap: false,
                                                    },
                                                    |cx| {
                                                        (0..8)
                                                            .map(|index| {
                                                                cx.keyed(index, |cx| {
                                                                    cx.pressable_with_id(
                                                                        crate::element::PressableProps {
                                                                            layout: {
                                                                                let mut layout = crate::element::LayoutStyle::default();
                                                                                layout.size.width = crate::element::Length::Fill;
                                                                                layout.size.height = crate::element::Length::Px(Px(40.0));
                                                                                layout
                                                                            },
                                                                            enabled: true,
                                                                            focusable: true,
                                                                            ..Default::default()
                                                                        },
                                                                        |_cx, _st, id| {
                                                                            if index == target_index {
                                                                                target_id.set(Some(id));
                                                                            }
                                                                            Vec::new()
                                                                        },
                                                                    )
                                                                })
                                                            })
                                                            .collect::<Vec<_>>()
                                                    },
                                                )]
                                            },
                                        );
                                        let bottom = cx.container(
                                            crate::element::ContainerProps {
                                                layout: {
                                                    let mut layout = crate::element::LayoutStyle::default();
                                                    layout.size.width = crate::element::Length::Fill;
                                                    layout.size.height = crate::element::Length::Px(Px(40.0));
                                                    layout
                                                },
                                                ..Default::default()
                                            },
                                            |_cx| Vec::new(),
                                        );
                                        vec![top, inner, bottom]
                                    }
                                },
                            )]
                        }
                    },
                )]
            }
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let outer_scroll_node = ui.children(root)[0];
    let target_id = target_id.get().expect("target element id");
    let target_node = crate::elements::node_for_element(&mut app, window, target_id).expect("node");

    assert!(
        inner_handle.max_offset().y.0 > 0.5,
        "expected nested inner scroll to have a real scroll range before scroll_into_view: viewport={:?} content={:?} max={:?}",
        inner_handle.viewport_size(),
        inner_handle.content_size(),
        inner_handle.max_offset(),
    );

    let did_scroll = ui.scroll_node_into_view(&mut app, target_node);
    assert!(
        did_scroll,
        "expected nested scroll_into_view to report scrolling"
    );
    assert!(
        inner_handle.offset().y.0 > 0.5,
        "expected inner scroll to move to reveal the target within the nested viewport: offset={:?}",
        inner_handle.offset(),
    );
    assert!(
        outer_handle.offset().y.0 > 0.5,
        "expected outer scroll to also move after the nested viewport remains below the outer viewport: offset={:?}",
        outer_handle.offset(),
    );

    let _ = outer_scroll_node;
    let _ = target_node;
}

#[test]
fn scroll_into_view_scrolls_horizontal_scroll_container_when_descendant_is_offscreen() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let scroll_handle = crate::scroll::ScrollHandle::default();

    let mut target_id: Option<GlobalElementId> = None;
    let target_index = 4usize;

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(80.0)));

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "scroll-into-view-horizontal",
        |cx| {
            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = crate::element::Length::Px(Px(100.0));
                        layout.size.height = crate::element::Length::Px(Px(80.0));
                        layout.overflow = crate::element::Overflow::Clip;
                        layout
                    },
                    axis: crate::element::ScrollAxis::X,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(0.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: crate::element::MainAlign::Start,
                            align: crate::element::CrossAlign::Stretch,
                            wrap: false,
                        },
                        |cx| {
                            (0..8)
                                .map(|index| {
                                    cx.keyed(index, |cx| {
                                        cx.pressable_with_id(
                                            crate::element::PressableProps {
                                                layout: {
                                                    let mut layout =
                                                        crate::element::LayoutStyle::default();
                                                    layout.size.width =
                                                        crate::element::Length::Px(Px(60.0));
                                                    layout.size.height =
                                                        crate::element::Length::Px(Px(40.0));
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |_cx, _st, id| {
                                                if index == target_index {
                                                    target_id = Some(id);
                                                }
                                                Vec::new()
                                            },
                                        )
                                    })
                                })
                                .collect::<Vec<_>>()
                        },
                    )]
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let target_id = target_id.expect("target element id");
    let target_node = crate::elements::node_for_element(&mut app, window, target_id).expect("node");

    assert!(
        scroll_handle.max_offset().x.0 > 0.01,
        "expected a horizontal scroll range, got viewport={:?} content={:?} max={:?}",
        scroll_handle.viewport_size(),
        scroll_handle.content_size(),
        scroll_handle.max_offset()
    );

    let before = scroll_handle.offset();
    let did_scroll = ui.scroll_node_into_view(&mut app, target_node);
    let after = scroll_handle.offset();

    assert!(
        did_scroll,
        "expected horizontal scroll_into_view to report scrolling"
    );
    assert!(
        after.x.0 > before.x.0 + 0.5,
        "expected horizontal scroll_into_view to increase x offset: before={:?} after={:?}",
        before,
        after
    );
}
