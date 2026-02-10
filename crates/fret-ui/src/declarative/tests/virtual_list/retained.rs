use super::*;

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
    let cache_node = ui.children(root)[0];
    let list_node = ui.children(cache_node)[0];
    let list_element = app
        .with_global_mut(
            crate::declarative::frame::ElementFrame::default,
            |frame, _app| {
                frame
                    .windows
                    .get(&window)
                    .and_then(|w| w.instances.get(list_node))
                    .map(|record| record.element)
            },
        )
        .expect("list instance exists");

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
            let record = window_frame.instances.get(vlist_node)?;
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
                .get(vlist_node)
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
