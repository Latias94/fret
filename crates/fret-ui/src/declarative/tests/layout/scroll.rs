use super::*;

#[test]
fn scroll_intrinsic_viewport_mode_does_not_measure_children() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-intrinsic-viewport-mode",
        |cx| {
            let mut props = crate::element::ScrollProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            props.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Viewport;
            vec![cx.scroll(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];

    let max_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    );
    let _measured = ui.measure_in(&mut app, &mut text, scroll, max_constraints, 1.0);

    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll),
        0,
        "expected viewport-mode scroll intrinsic measurement to avoid measuring children"
    );
}

#[test]
fn scroll_intrinsic_content_mode_measures_children() {
    use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-intrinsic-content-mode",
        |cx| {
            let mut props = crate::element::ScrollProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            props.intrinsic_measure_mode = crate::element::ScrollIntrinsicMeasureMode::Content;
            vec![cx.scroll(props, |cx| vec![cx.text("child")])]
        },
    );
    ui.set_root(root);

    let scroll = ui.children(root)[0];

    let max_constraints = LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    );
    let _measured = ui.measure_in(&mut app, &mut text, scroll, max_constraints, 1.0);

    assert!(
        ui.debug_measure_child_calls_for_parent(scroll) > 0,
        "expected content-mode scroll intrinsic measurement to measure children"
    );
}

#[test]
fn scroll_wheel_updates_offset_and_shifts_child_bounds() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-wheel",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Px(Px(20.0));
            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                )]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");
    assert!(
        ui.layout_engine_has_node(column_node),
        "expected scroll content subtree nodes to remain registered in the layout engine"
    );

    let wheel_pos = fret_core::Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after scroll");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after wheel scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_translation_does_not_force_layout_engine_solves() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-wheel-solve-stats",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Px(Px(20.0));
            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a"), cx.text("b"), cx.text("c")],
                )]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");

    let wheel_pos = fret_core::Point::new(Px(5.0), Px(5.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: wheel_pos,
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after scroll");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after wheel scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "expected scroll translation to avoid triggering layout engine solves"
    );
    assert!(
        ui.layout_engine_has_node(column_node),
        "expected translation-only scroll to keep engine nodes alive (stable identity)"
    );

    // Even when the tree is fully clean (no invalidation, no translation), the request/build phase
    // must keep barrier-mounted subtrees registered so identity remains stable across frames.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        ui.layout_engine_has_node(column_node),
        "expected steady-state frames to keep scroll content nodes registered in the engine"
    );
}

#[test]
fn scroll_axis_both_probe_unbounded_keeps_content_at_least_viewport_width() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-axis-both-min-content-width",
        |cx| {
            let mut p = crate::element::ScrollProps::default();
            p.layout.size.width = crate::element::Length::Fill;
            p.layout.size.height = crate::element::Length::Fill;
            p.axis = crate::element::ScrollAxis::Both;
            p.probe_unbounded = true;

            vec![cx.scroll(p, |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("a")],
                )]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let column_bounds = ui.debug_node_bounds(column_node).expect("column bounds");

    assert_eq!(
        column_bounds.size.width, bounds.size.width,
        "expected scroll content bounds to be at least the viewport width; got={:?} want={:?}",
        column_bounds.size.width, bounds.size.width
    );
}

#[test]
fn scroll_thumb_drag_updates_offset() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            // Ensure the scrollbar has enough room for Radix-style padding + 18px minimum thumb.
            // With very small tracks, Radix clamps the thumb to the available space and dragging
            // cannot change the scroll offset.
            stack_layout.size.height = crate::element::Length::Fill;

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.column(
                                crate::element::ColumnProps {
                                    gap: Px(0.0),
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        cx.text("a"),
                                        cx.text("b"),
                                        cx.text("c"),
                                        cx.text("d"),
                                        cx.text("e"),
                                        cx.text("f"),
                                    ]
                                },
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: None,
                        },
                        size: crate::element::SizeStyle {
                            width: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Vertical,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scroll_node = ui.children(stack_node)[0];
    let scrollbar_node = ui.children(stack_node)[1];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");

    // Click/drag the scrollbar thumb down (thumb starts at the top at offset=0).
    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let thumb = crate::declarative::paint_helpers::scrollbar_thumb_rect(
        scrollbar_bounds,
        scroll_handle.viewport_size().height,
        scroll_handle.content_size().height,
        scroll_handle.offset().y,
        crate::element::ScrollbarStyle::default().track_padding,
    )
    .expect("thumb rect");
    let down_pos = fret_core::Point::new(Px(thumb.origin.x.0 + 1.0), Px(thumb.origin.y.0 + 1.0));
    let move_pos = fret_core::Point::new(down_pos.x, Px(down_pos.y.0 + 8.0));
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.captured(),
        Some(scrollbar_node),
        "expected thumb down to capture the pointer on the scrollbar node"
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected thumb drag to update scroll handle offset, got {:?}",
        scroll_handle.offset().y
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after drag");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after thumb drag: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_handle_set_offset_triggers_visual_scroll_without_manual_invalidate() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                ..Default::default()
            },
            |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    |cx| {
                        vec![
                            cx.text("a"),
                            cx.text("b"),
                            cx.text("c"),
                            cx.text("d"),
                            cx.text("e"),
                            cx.text("f"),
                        ]
                    },
                )]
            },
        )]
    }

    // Frame 0: establish viewport and content extent.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-handle-set-offset",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    app.advance_frame();

    // Frame 1: stable mount (no intentional invalidations).
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-handle-set-offset",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds");

    // Outside the UI runtime, programmatically update the handle.
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), Px(20.0)));
    app.advance_frame();

    // Frame 2: the scroll change should invalidate bound nodes implicitly via handle bindings.
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scroll-handle-set-offset",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let after = ui
        .debug_node_visual_bounds(column_node)
        .expect("column visual bounds after offset");

    assert!(
        after.origin.y.0 < before.origin.y.0,
        "expected content to move up after programmatic scroll: before={:?} after={:?}",
        before.origin.y,
        after.origin.y
    );
}

#[test]
fn scroll_content_extent_updates_immediately_when_growing_at_scroll_end() {
    let mut app = TestHost::new();
    let show_more = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        show_more: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                ..Default::default()
            },
            move |cx| {
                cx.observe_model(&show_more, Invalidation::Layout);
                let expanded = cx.app.models().get_copied(&show_more).unwrap_or(false);
                let rows = if expanded { 24 } else { 6 };

                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    move |cx| {
                        (0..rows)
                            .map(|i| cx.text(format!("row {i}")))
                            .collect::<Vec<_>>()
                    },
                )]
            },
        )]
    }

    // Frame 0: establish content extent and scroll to the end.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let _ = app.models_mut().update(&show_more, |v| *v = true);
    assert!(
        app.models().get_copied(&show_more).unwrap_or(false),
        "expected show_more model update to commit before the next frame"
    );
    app.advance_frame();

    // Frame 1: content grows while we're at the previous max offset.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    ui.set_root(root1);

    let scroll_node = ui.children(root1)[0];
    let column_node = ui.children(scroll_node)[0];
    assert!(
        ui.node_needs_layout(column_node),
        "expected the scroll content subtree to be marked dirty when its children change"
    );
    assert!(
        scroll_handle.offset().y.0 + 0.5 >= scroll_handle.max_offset().y.0,
        "expected the scroll handle to remain at the previous max offset before relayout: offset={:?} max={:?}",
        scroll_handle.offset().y,
        scroll_handle.max_offset().y
    );

    {
        use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
        let max_constraints = LayoutConstraints::new(
            LayoutSize::new(None, None),
            LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
        );
        let measured = ui.measure_in(&mut app, &mut text, column_node, max_constraints, 1.0);
        assert!(
            measured.height.0 > 60.0,
            "expected measuring the expanded column to observe increased height, got {measured:?}"
        );
    }

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root1)[0];
    let column_node = ui.children(scroll_node)[0];
    assert_eq!(
        ui.children(column_node).len(),
        24,
        "expected render to mount the expanded column children before layout"
    );

    let max1 = scroll_handle.max_offset().y;
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected scroll extent to grow immediately when content expands at the end: before={max0:?} after={max1:?}"
    );
}

#[test]
fn scroll_at_end_reuses_cached_extent_when_clean() {
    let mut app = TestHost::new();

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    move |cx| {
                        (0..12)
                            .map(|i| cx.text(format!("row {i}")))
                            .collect::<Vec<_>>()
                    },
                )]
            },
        )]
    }

    // Frame 0: populate intrinsic measurement caches and scroll to the end.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-at-end-reuses-caches",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    app.advance_frame();

    // Frame 1: no content changes while at the scroll end; layout should avoid measuring children.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-at-end-reuses-caches",
        |cx| build_root(cx, scroll_handle.clone()),
    );
    ui.set_root(root1);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let scroll_node = ui.children(root1)[0];
    assert_eq!(
        ui.debug_measure_child_calls_for_parent(scroll_node),
        0,
        "expected scroll layout to reuse intrinsic caches when at the extent edge and clean"
    );
    assert!(
        (scroll_handle.max_offset().y.0 - max0.0).abs() <= 0.5,
        "expected max offset to remain stable: before={max0:?} after={:?}",
        scroll_handle.max_offset().y
    );
}

#[test]
fn scroll_offset_clamps_when_content_shrinks_below_end() {
    let mut app = TestHost::new();
    let expanded = app.models_mut().insert(true);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn row(cx: &mut ElementContext<'_, TestHost>, i: usize) -> AnyElement {
        let mut props = crate::element::ContainerProps::default();
        props.layout.size.height = crate::element::Length::Px(Px(10.0));
        props.layout.size.width = crate::element::Length::Fill;
        cx.container(props, move |cx| vec![cx.text(format!("row {i}"))])
    }

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        expanded: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                cx.observe_model(&expanded, Invalidation::Layout);
                let expanded = cx.app.models().get_copied(&expanded).unwrap_or(false);
                let rows = if expanded { 30 } else { 6 };

                vec![cx.column(
                    crate::element::ColumnProps {
                        gap: Px(0.0),
                        ..Default::default()
                    },
                    move |cx| (0..rows).map(|i| row(cx, i)).collect::<Vec<_>>(),
                )]
            },
        )]
    }

    // Frame 0: scroll to the end with the expanded content.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-shrink-clamps-offset",
        |cx| build_root(cx, scroll_handle.clone(), expanded.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let _ = app.models_mut().update(&expanded, |v| *v = false);
    app.advance_frame();

    // Frame 1: content shrinks while we're beyond the new max; offset must clamp.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-shrink-clamps-offset",
        |cx| build_root(cx, scroll_handle.clone(), expanded.clone()),
    );
    ui.set_root(root1);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max1 = scroll_handle.max_offset().y;
    let off1 = scroll_handle.offset().y;
    assert!(
        max1.0 + 0.5 < max0.0,
        "expected shrink to reduce max offset: before={max0:?} after={max1:?}"
    );
    assert!(
        off1.0 <= max1.0 + 0.5,
        "expected offset to clamp to the new max: offset={off1:?} max={max1:?}"
    );
}

#[test]
fn scroll_axis_both_updates_extent_for_axis_growing_at_end() {
    let mut app = TestHost::new();
    let wide = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        wide: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                axis: crate::element::ScrollAxis::Both,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                cx.observe_model(&wide, Invalidation::Layout);
                let wide = cx.app.models().get_copied(&wide).unwrap_or(false);

                let mut content = crate::element::ContainerProps::default();
                content.layout.size.width =
                    crate::element::Length::Px(if wide { Px(260.0) } else { Px(140.0) });
                content.layout.size.height = crate::element::Length::Px(Px(40.0));

                vec![cx.container(content, |_cx| Vec::new())]
            },
        )]
    }

    // Frame 0: establish max offset and scroll to x end.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-grow-at-x-end",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max0 = scroll_handle.max_offset().x;
    scroll_handle.set_offset(fret_core::Point::new(max0, Px(0.0)));
    let _ = app.models_mut().update(&wide, |v| *v = true);
    app.advance_frame();

    // Frame 1: content grows in x while we're at the previous x max.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-axis-both-grow-at-x-end",
        |cx| build_root(cx, scroll_handle.clone(), wide.clone()),
    );
    ui.set_root(root1);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max1 = scroll_handle.max_offset().x;
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected x extent to grow immediately when content expands at x end: before={max0:?} after={max1:?}"
    );
}

#[test]
fn scroll_extent_updates_under_view_cache_reconciliation_when_growing_at_end() {
    let mut app = TestHost::new();
    let show_more = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();
    let mut scene = Scene::default();

    fn build_root(
        cx: &mut ElementContext<'_, TestHost>,
        scroll_handle: crate::scroll::ScrollHandle,
        show_more: fret_runtime::Model<bool>,
    ) -> Vec<AnyElement> {
        let mut scroll_layout = crate::element::LayoutStyle::default();
        scroll_layout.size.width = crate::element::Length::Fill;
        scroll_layout.size.height = crate::element::Length::Fill;
        scroll_layout.overflow = crate::element::Overflow::Clip;

        let mut cache = crate::element::ViewCacheProps::default();
        cache.layout.size.width = crate::element::Length::Fill;
        cache.layout.size.height = crate::element::Length::Auto;
        cache.cache_key = 1;

        vec![cx.scroll(
            crate::element::ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(scroll_handle),
                probe_unbounded: true,
                ..Default::default()
            },
            move |cx| {
                vec![cx.view_cache(cache, move |cx| {
                    cx.observe_model(&show_more, Invalidation::Layout);
                    let expanded = cx.app.models().get_copied(&show_more).unwrap_or(false);
                    let rows = if expanded { 24 } else { 6 };

                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0),
                            ..Default::default()
                        },
                        move |cx| {
                            (0..rows)
                                .map(|i| cx.text(format!("row {i}")))
                                .collect::<Vec<_>>()
                        },
                    )]
                })]
            },
        )]
    }

    // Frame 0: establish content extent and scroll to the end.
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end-view-cache",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let max0 = scroll_handle.max_offset().y;
    scroll_handle.set_offset(fret_core::Point::new(Px(0.0), max0));
    let _ = show_more.update(&mut app, |v, _cx| *v = true);
    let changed = app.take_changed_models();
    assert!(
        ui.propagate_model_changes(&mut app, &changed),
        "expected model change to invalidate the view-cache subtree"
    );
    app.advance_frame();

    // Frame 1: content grows while we're at the previous max offset.
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-grow-at-end-view-cache",
        |cx| build_root(cx, scroll_handle.clone(), show_more.clone()),
    );
    ui.set_root(root1);

    let scroll_node = ui.children(root1)[0];
    let cache_node = ui.children(scroll_node)[0];
    assert!(
        ui.node_needs_layout(cache_node),
        "expected view-cache scroll content node to be dirty when its children change"
    );
    assert!(
        scroll_handle.offset().y.0 + 0.5 >= scroll_handle.max_offset().y.0,
        "expected scroll handle to remain at the previous max offset before relayout: offset={:?} max={:?}",
        scroll_handle.offset().y,
        scroll_handle.max_offset().y
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    scene.clear();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let max1 = scroll_handle.max_offset().y;
    assert!(
        max1.0 > max0.0 + 0.5,
        "expected scroll extent to grow immediately under view-cache reconciliation: before={max0:?} after={max1:?}"
    );
}

#[test]
fn scroll_thumb_drag_updates_offset_horizontal() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp-scrollbar-drag-x",
        |cx| {
            let scroll_handle = scroll_handle.clone();

            let mut stack_layout = crate::element::LayoutStyle::default();
            stack_layout.size.width = crate::element::Length::Fill;
            // Ensure the scrollbar track has enough room for Radix-aligned padding + min thumb.
            stack_layout.size.height = crate::element::Length::Px(Px(30.0));

            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: stack_layout,
                },
                |cx| {
                    let mut scroll_layout = crate::element::LayoutStyle::default();
                    scroll_layout.size.width = crate::element::Length::Fill;
                    scroll_layout.size.height = crate::element::Length::Fill;
                    scroll_layout.overflow = crate::element::Overflow::Clip;

                    let scroll = cx.scroll(
                        crate::element::ScrollProps {
                            layout: scroll_layout,
                            axis: crate::element::ScrollAxis::X,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |cx| {
                            let mut content_layout = crate::element::LayoutStyle::default();
                            content_layout.size.width = crate::element::Length::Px(Px(300.0));
                            content_layout.size.height = crate::element::Length::Fill;

                            vec![cx.container(
                                crate::element::ContainerProps {
                                    layout: content_layout,
                                    ..Default::default()
                                },
                                |cx| vec![cx.text("abc")],
                            )]
                        },
                    );

                    let scrollbar_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: None,
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: Some(Px(0.0)),
                        },
                        size: crate::element::SizeStyle {
                            height: crate::element::Length::Px(Px(10.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Horizontal,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![scroll, scrollbar]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack_node = ui.children(root)[0];
    let scroll_node = ui.children(stack_node)[0];
    let scrollbar_node = ui.children(stack_node)[1];
    let content_node = ui.children(scroll_node)[0];
    let before = ui
        .debug_node_visual_bounds(content_node)
        .expect("content visual bounds");

    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    let down_pos = fret_core::Point::new(
        Px(scrollbar_bounds.origin.x.0 + 2.0),
        Px(scrollbar_bounds.origin.y.0 + 1.0),
    );
    let move_pos = fret_core::Point::new(Px(down_pos.x.0 + 12.0), down_pos.y);
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        scroll_handle.offset().x.0 > 0.01,
        "expected thumb drag to update scroll handle offset, got {:?}",
        scroll_handle.offset().x
    );

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let after = ui
        .debug_node_visual_bounds(content_node)
        .expect("content visual bounds after drag");

    assert!(
        after.origin.x.0 < before.origin.x.0,
        "expected content to move left after thumb drag: before={:?} after={:?}",
        before.origin.x,
        after.origin.x
    );
}

#[test]
fn scroll_rounds_scrollable_extent_up_to_next_pixel() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(50.0)),
    );
    let mut text = FakeTextService::default();

    let handle = crate::scroll::ScrollHandle::default();
    let handle_for_root = handle.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "scroll-rounding",
        move |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;

            let mut child_layout = crate::element::LayoutStyle::default();
            child_layout.size.width = Length::Fill;
            child_layout.size.height = Length::Px(Px(100.2));

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    scroll_handle: Some(handle_for_root.clone()),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.container(
                        crate::element::ContainerProps {
                            layout: child_layout,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("content")],
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let max = handle.max_offset();
    assert!((max.y.0 - 51.0).abs() < 0.01, "max_offset.y={:?}", max.y);

    handle.scroll_to_offset(Point::new(Px(0.0), Px(60.0)));
    assert!(
        (handle.offset().y.0 - 51.0).abs() < 0.01,
        "offset.y={:?}",
        handle.offset().y
    );
}
