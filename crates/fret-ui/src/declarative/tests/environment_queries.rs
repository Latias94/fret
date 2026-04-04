use super::*;

use fret_core::Edges;
use fret_runtime::GlobalsHost;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn environment_query_change_invalidates_view_cache_subtree() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders_env = Arc::new(AtomicUsize::new(0));
    let renders_plain = Arc::new(AtomicUsize::new(0));

    let mut root: Option<NodeId> = None;

    for frame in 0..4 {
        let prefers_reduced_motion = match frame {
            0 | 1 => Some(false),
            2 | 3 => Some(true),
            _ => unreachable!(),
        };

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |rt, _| {
            rt.set_window_prefers_reduced_motion(window, prefers_reduced_motion);
        });

        let renders_env = renders_env.clone();
        let renders_plain = renders_plain.clone();

        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "environment-query-view-cache",
            move |cx| {
                let cached_env =
                    cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                        renders_env.fetch_add(1, Ordering::SeqCst);
                        let _ = cx.environment_prefers_reduced_motion(Invalidation::Paint);
                        vec![cx.text("env")]
                    });

                let cached_plain = cx.view_cache(
                    crate::element::ViewCacheProps {
                        cache_key: 1,
                        ..Default::default()
                    },
                    move |cx| {
                        renders_plain.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text("plain")]
                    },
                );

                vec![cached_env, cached_plain]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();
    }

    assert_eq!(
        renders_env.load(Ordering::SeqCst),
        2,
        "expected environment query change to force a view-cache rerender"
    );
    assert_eq!(
        renders_plain.load(Ordering::SeqCst),
        1,
        "expected view-cache subtree without environment dependencies to reuse"
    );
}

#[test]
fn window_metrics_service_insets_commit_to_environment_queries() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let safe_area = Edges::all(Px(4.0));
    let occlusion = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: Px(16.0),
        left: Px(0.0),
    };
    app.with_global_mut_untracked(
        fret_core::window::WindowMetricsService::default,
        |svc, _| {
            svc.set_safe_area_insets(window, Some(safe_area));
            svc.set_occlusion_insets(window, Some(occlusion));
        },
    );

    type ObservedInsets = Arc<Mutex<Option<(Option<Edges>, Option<Edges>)>>>;
    let observed: ObservedInsets = Arc::new(Mutex::new(None));
    let observed_ref = observed.clone();
    render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "environment-window-insets",
        move |cx| {
            let safe = cx.environment_safe_area_insets(Invalidation::Layout);
            let occlusion = cx.environment_occlusion_insets(Invalidation::Layout);
            *observed_ref.lock().unwrap() = Some((safe, occlusion));
            vec![cx.text("probe")]
        },
    );

    let (seen_safe, seen_occlusion) = observed.lock().unwrap().unwrap();
    assert_eq!(seen_safe, Some(safe_area));
    assert_eq!(seen_occlusion, Some(occlusion));
}

#[test]
fn environment_viewport_width_change_invalidates_view_cache_subtree() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds_small = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let bounds_large = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders_env = Arc::new(AtomicUsize::new(0));
    let renders_plain = Arc::new(AtomicUsize::new(0));

    let mut root: Option<NodeId> = None;

    for frame in 0..4 {
        let bounds = if frame < 2 {
            bounds_small
        } else {
            bounds_large
        };

        let renders_env = renders_env.clone();
        let renders_plain = renders_plain.clone();

        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "environment-query-viewport-width-view-cache",
            move |cx| {
                let cached_env =
                    cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                        renders_env.fetch_add(1, Ordering::SeqCst);
                        let _ = cx.environment_viewport_width(Invalidation::Layout);
                        vec![cx.text("env")]
                    });

                let cached_plain = cx.view_cache(
                    crate::element::ViewCacheProps {
                        cache_key: 1,
                        ..Default::default()
                    },
                    move |cx| {
                        renders_plain.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text("plain")]
                    },
                );

                vec![cached_env, cached_plain]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();
    }

    assert_eq!(
        renders_env.load(Ordering::SeqCst),
        2,
        "expected viewport width change to force a view-cache rerender"
    );
    assert_eq!(
        renders_plain.load(Ordering::SeqCst),
        1,
        "expected view-cache subtree without environment dependencies to reuse"
    );
}

fn viewport_resize_cached_flow(
    cx: &mut crate::elements::ElementContext<'_, TestHost>,
    cache_renders: &Arc<AtomicUsize>,
    observed_viewports: &Arc<Mutex<Vec<Rect>>>,
) -> Vec<crate::element::AnyElement> {
    let cache_renders = cache_renders.clone();
    let observed_viewports = observed_viewports.clone();
    vec![cx.view_cache(
        crate::element::ViewCacheProps {
            contained_layout: true,
            ..Default::default()
        },
        move |cx| {
            cache_renders.fetch_add(1, Ordering::SeqCst);
            let viewport = cx.environment_viewport_bounds(Invalidation::Layout);
            observed_viewports.lock().unwrap().push(viewport);
            let roomy = viewport.size.width.0 >= 560.0 && viewport.size.height.0 >= 640.0;

            let mut page = crate::element::FlexProps::default();
            page.layout.size.width = crate::element::Length::Fill;
            page.layout.size.height = crate::element::Length::Fill;
            page.direction = fret_core::Axis::Vertical;
            page.align = crate::element::CrossAlign::Center;
            page.justify = if roomy {
                crate::element::MainAlign::Center
            } else {
                crate::element::MainAlign::Start
            };

            let mut card = crate::element::ContainerProps::default();
            card.layout.size.width = crate::element::Length::Fill;
            card.layout.size.max_width = Some(crate::element::Length::Px(Px(120.0)));
            if !roomy {
                card.layout.size.min_height = Some(crate::element::Length::Px(Px(120.0)));
                card.layout.size.max_height = Some(crate::element::Length::Px(Px(120.0)));
            }

            let mut body = crate::element::FlexProps::default();
            body.layout.size.width = crate::element::Length::Fill;
            body.direction = fret_core::Axis::Vertical;

            vec![cx.flex(page, |cx| {
                vec![cx.container(card, |cx| {
                    vec![cx.flex(body, |cx| vec![cx.text("header"), cx.text("footer")])]
                })]
            })]
        },
    )]
}

#[test]
fn environment_viewport_resize_rerender_updates_flow_layout_during_interactive_resize() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let roomy_bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(680.0), Px(760.0)),
    );
    let compact_bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut services = FakeTextService::default();
    let cache_renders = Arc::new(AtomicUsize::new(0));
    let observed_viewports = Arc::new(Mutex::new(Vec::new()));

    let root0 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        roomy_bounds,
        "environment-query-viewport-resize-updates-flow-layout",
        |cx| viewport_resize_cached_flow(cx, &cache_renders, &observed_viewports),
    );
    ui.set_root(root0);
    layout_frame(&mut ui, &mut app, &mut services, roomy_bounds);

    let mut cache_root = ui.children(root0)[0];
    let mut page_node = ui.children(cache_root)[0];
    let engine = ui.take_layout_engine();
    let page_style = engine
        .debug_style_for_node(page_node)
        .cloned()
        .expect("page style after roomy layout");
    ui.put_layout_engine(engine);
    assert_eq!(
        page_style.justify_content,
        Some(taffy::style::JustifyContent::Center),
        "expected roomy frame to center the card"
    );

    app.advance_frame();
    let root1 = render_root_for_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        compact_bounds,
        "environment-query-viewport-resize-updates-flow-layout",
        |cx| viewport_resize_cached_flow(cx, &cache_renders, &observed_viewports),
    );
    assert_eq!(root1, root0, "expected stable root identity");
    ui.set_root(root1);
    layout_frame(&mut ui, &mut app, &mut services, compact_bounds);

    assert_eq!(
        cache_renders.load(Ordering::SeqCst),
        2,
        "viewport change should rerun the contained view-cache closure"
    );
    assert_eq!(
        observed_viewports.lock().unwrap().as_slice(),
        &[roomy_bounds, compact_bounds],
        "contained view-cache should observe the latest committed viewport during resize rerender"
    );

    cache_root = ui.children(root1)[0];
    page_node = ui.children(cache_root)[0];
    let card_node = ui.children(page_node)[0];
    let page_instance =
        crate::declarative::frame::element_record_for_node(&mut app, window, page_node)
            .map(|r| r.instance)
            .expect("page instance after compact rerender");
    match page_instance {
        crate::declarative::frame::ElementInstance::Flex(props) => {
            assert_eq!(
                props.justify,
                crate::element::MainAlign::Start,
                "compact rerender should author justify-start"
            );
        }
        other => panic!("expected page node to remain a Flex, got {other:?}"),
    }

    let rebuilt_card_bounds = ui
        .debug_node_bounds(card_node)
        .expect("card bounds after compact interactive resize frame");
    let engine = ui.take_layout_engine();
    let page_style = engine
        .debug_style_for_node(page_node)
        .cloned()
        .expect("page style after compact interactive resize frame");
    let card_style = engine
        .debug_style_for_node(card_node)
        .cloned()
        .expect("card style after compact interactive resize frame");
    ui.put_layout_engine(engine);

    assert_eq!(
        page_style.justify_content,
        Some(taffy::style::JustifyContent::FlexStart),
        "viewport-driven view-cache rerender should update the flow subtree in the same interactive-resize frame"
    );
    assert_eq!(
        card_style.min_size.height,
        taffy::style::Dimension::length(120.0),
        "compact frame should forward the updated min-height constraint immediately"
    );
    assert!(
        rebuilt_card_bounds.origin.y.0 <= 0.5,
        "compact interactive-resize frame should pin the card to the top instead of keeping the stale centered flow; bounds={rebuilt_card_bounds:?}"
    );
}

#[test]
fn viewport_resize_after_cache_enable_keeps_cache_root_membership_complete() {
    use std::sync::Mutex;

    #[derive(Default)]
    struct CapturedIds {
        cache_root: Option<crate::elements::GlobalElementId>,
        filter: Option<crate::elements::GlobalElementId>,
        clear: Option<crate::elements::GlobalElementId>,
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(false);

    let roomy_bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(660.0)),
    );
    let compact_bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut services = FakeTextService::default();
    let ids = Arc::new(Mutex::new(CapturedIds::default()));

    let render_footer = |cx: &mut crate::elements::ElementContext<'_, TestHost>,
                         ids: &Arc<Mutex<CapturedIds>>| {
        let viewport = cx.environment_viewport_bounds(Invalidation::Layout);
        let compact = viewport.size.width.0 < 560.0;

        let filter = cx.named("filter-chip", |cx| cx.text("filter-chip"));
        ids.lock().unwrap().filter = Some(filter.id);

        let clear = cx.named("clear-chip", |cx| cx.text("clear-chip"));
        ids.lock().unwrap().clear = Some(clear.id);

        let footer = if compact {
            cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        filter,
                        cx.flex(
                            crate::element::FlexProps {
                                direction: fret_core::Axis::Horizontal,
                                ..Default::default()
                            },
                            move |_cx| vec![clear],
                        ),
                    ]
                },
            )
        } else {
            cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    ..Default::default()
                },
                move |_cx| vec![filter, clear],
            )
        };

        let cache = cx.view_cache(
            crate::element::ViewCacheProps {
                contained_layout: true,
                ..Default::default()
            },
            move |_cx| vec![footer],
        );
        ids.lock().unwrap().cache_root = Some(cache.id);
        vec![cache]
    };

    let ids_for_frame0 = ids.clone();
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        roomy_bounds,
        "viewport-resize-cache-enable-membership",
        move |cx| render_footer(cx, &ids_for_frame0),
    );
    ui.set_root(root0);
    ui.layout_all(&mut app, &mut services, roomy_bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, roomy_bounds, &mut scene, 1.0);

    ui.set_view_cache_enabled(true);
    app.advance_frame();

    let ids_for_frame1 = ids.clone();
    let root1 = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        compact_bounds,
        "viewport-resize-cache-enable-membership",
        move |cx| render_footer(cx, &ids_for_frame1),
    );
    assert_eq!(root1, root0, "expected stable root identity across resize");
    ui.set_root(root1);
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, compact_bounds, &mut scene, 1.0);

    let (cache_root, filter, clear) = {
        let ids = ids.lock().unwrap();
        (
            ids.cache_root.expect("cache root id"),
            ids.filter.expect("filter id"),
            ids.clear.expect("clear id"),
        )
    };

    app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _app| {
        let window_state = runtime.for_window_mut(window);
        let elements = window_state
            .view_cache_elements_for_root(cache_root)
            .expect("expected cache-root membership list after compact rerender");

        assert!(
            elements.contains(&filter),
            "cache-root membership should retain the leading footer subtree after cache-enable transition; elements={elements:?} filter={filter:?}"
        );
        assert!(
            elements.contains(&clear),
            "cache-root membership should retain the trailing footer subtree after cache-enable transition; elements={elements:?} clear={clear:?}"
        );
    });
}

#[test]
fn viewport_resize_after_cache_enable_keeps_roving_keyed_semantics_and_membership_complete() {
    #[derive(Default, Clone, Debug)]
    struct CapturedIds {
        cache_root: Option<crate::elements::GlobalElementId>,
        filter_all: Option<crate::elements::GlobalElementId>,
        filter_active: Option<crate::elements::GlobalElementId>,
        filter_completed: Option<crate::elements::GlobalElementId>,
        clear: Option<crate::elements::GlobalElementId>,
    }

    impl CapturedIds {
        fn set_filter(&mut self, test_id: &str, element: crate::elements::GlobalElementId) {
            match test_id {
                "roving.filter.all" => self.filter_all = Some(element),
                "roving.filter.active" => self.filter_active = Some(element),
                "roving.filter.completed" => self.filter_completed = Some(element),
                other => panic!("unexpected filter test id: {other}"),
            }
        }
    }

    fn snapshot_test_ids(snapshot: &fret_core::SemanticsSnapshot) -> Vec<String> {
        let mut ids: Vec<String> = snapshot
            .nodes
            .iter()
            .filter_map(|node| node.test_id.as_ref().map(ToString::to_string))
            .collect();
        ids.sort();
        ids
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(false);

    let roomy_bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(660.0)),
    );
    let compact_bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut services = FakeTextService::default();
    let ids = Arc::new(Mutex::new(CapturedIds::default()));

    let render_footer = |cx: &mut crate::elements::ElementContext<'_, TestHost>,
                         ids: &Arc<Mutex<CapturedIds>>| {
        let viewport = cx.environment_viewport_bounds(Invalidation::Layout);
        let compact = viewport.size.width.0 < 560.0;

        let action_root = cx.named("__test.action_root", |cx| {
            let ids_for_cache = ids.clone();
            let cache = cx.view_cache(
                crate::element::ViewCacheProps {
                    contained_layout: true,
                    ..Default::default()
                },
                move |cx| {
                    let ids_for_roving = ids_for_cache.clone();
                    let roving = cx.roving_flex(
                        crate::element::RovingFlexProps {
                            flex: crate::element::FlexProps {
                                direction: fret_core::Axis::Horizontal,
                                ..Default::default()
                            },
                            roving: crate::element::RovingFocusProps {
                                enabled: true,
                                wrap: true,
                                disabled: Arc::from([false, false, false]),
                            },
                        },
                        move |cx| {
                            cx.roving_on_active_change(Arc::new(|_host, _cx, _idx| {}));
                            cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                                use crate::action::RovingNavigateResult;
                                use fret_core::KeyCode;

                                let Some(current) = it.current else {
                                    return RovingNavigateResult::NotHandled;
                                };

                                let forward = match it.key {
                                    KeyCode::ArrowRight => true,
                                    KeyCode::ArrowLeft => false,
                                    _ => return RovingNavigateResult::NotHandled,
                                };
                                let len = it.len;
                                let next = if forward {
                                    (current + 1) % len
                                } else {
                                    (current + len - 1) % len
                                };
                                RovingNavigateResult::Handled { target: Some(next) }
                            }));

                            let mut out = Vec::new();
                            for (label, test_id) in [
                                ("All", "roving.filter.all"),
                                ("Active", "roving.filter.active"),
                                ("Completed", "roving.filter.completed"),
                            ] {
                                let ids_for_item = ids_for_roving.clone();
                                out.push(cx.keyed(test_id, move |cx| {
                                    let label: Arc<str> = Arc::from(label);
                                    let mut props = crate::element::PressableProps::default();
                                    props.a11y = crate::element::PressableA11y {
                                        role: Some(fret_core::SemanticsRole::Button),
                                        label: Some(label.clone()),
                                        test_id: Some(Arc::from(test_id)),
                                        ..Default::default()
                                    };
                                    let pressable = cx.pressable(props, move |cx, _st| {
                                        vec![cx.text(label.clone())]
                                    });
                                    ids_for_item
                                        .lock()
                                        .unwrap()
                                        .set_filter(test_id, pressable.id);
                                    cx.flex(
                                        crate::element::FlexProps {
                                            direction: fret_core::Axis::Horizontal,
                                            ..Default::default()
                                        },
                                        move |_cx| vec![pressable],
                                    )
                                }));
                            }
                            out
                        },
                    );

                    let clear = {
                        let mut props = crate::element::PressableProps::default();
                        props.a11y = crate::element::PressableA11y {
                            role: Some(fret_core::SemanticsRole::Button),
                            label: Some(Arc::from("Clear")),
                            test_id: Some(Arc::from("roving.clear")),
                            ..Default::default()
                        };
                        let clear = cx.pressable(props, |cx, _st| vec![cx.text("Clear")]);
                        ids_for_cache.lock().unwrap().clear = Some(clear.id);
                        clear
                    };

                    let footer = if compact {
                        cx.flex(
                            crate::element::FlexProps {
                                direction: fret_core::Axis::Vertical,
                                ..Default::default()
                            },
                            move |_cx| vec![roving, clear],
                        )
                    } else {
                        cx.flex(
                            crate::element::FlexProps {
                                direction: fret_core::Axis::Horizontal,
                                ..Default::default()
                            },
                            move |_cx| vec![roving, clear],
                        )
                    };

                    vec![footer]
                },
            );
            ids.lock().unwrap().cache_root = Some(cache.id);
            cache
        });

        vec![action_root]
    };

    let ids_for_frame0 = ids.clone();
    let root0 = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        roomy_bounds,
        "viewport-resize-cache-enable-roving-keyed-membership",
        move |cx| render_footer(cx, &ids_for_frame0),
    );
    ui.set_root(root0);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, roomy_bounds, 1.0);
    let _roomy_snapshot = ui
        .semantics_snapshot()
        .expect("roomy semantics snapshot after initial frame");

    ui.set_view_cache_enabled(true);
    app.advance_frame();

    let mut failures: Vec<String> = Vec::new();
    for frame in 0..5 {
        let frame_no = frame + 2;
        let ids_for_frame = ids.clone();
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            compact_bounds,
            "viewport-resize-cache-enable-roving-keyed-membership",
            move |cx| render_footer(cx, &ids_for_frame),
        );
        assert_eq!(
            root, root0,
            "expected stable root identity across compact frames"
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);

        let snapshot = ui
            .semantics_snapshot()
            .expect("compact semantics snapshot after cache-enable transition");
        let test_ids = snapshot_test_ids(snapshot);
        for expected in [
            "roving.filter.all",
            "roving.filter.active",
            "roving.filter.completed",
            "roving.clear",
        ] {
            if !test_ids.iter().any(|id| id == expected) {
                failures.push(format!(
                    "frame{frame_no} should keep {expected} in semantics snapshot after cache-enable transition; ids={test_ids:?}"
                ));
            }
        }

        let captured = ids.lock().unwrap().clone();
        let cache_root = captured.cache_root.expect("cache root id");
        let filter_all = captured.filter_all.expect("all filter element id");
        let filter_active = captured.filter_active.expect("active filter element id");
        let filter_completed = captured
            .filter_completed
            .expect("completed filter element id");
        let clear = captured.clear.expect("clear button element id");

        app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _app| {
            let window_state = runtime.for_window_mut(window);
            let elements = window_state
                .view_cache_elements_for_root(cache_root)
                .expect("expected cache-root membership list for roving/keyed subtree");

            for (label, element) in [
                ("roving.filter.all", filter_all),
                ("roving.filter.active", filter_active),
                ("roving.filter.completed", filter_completed),
                ("roving.clear", clear),
            ] {
                if !elements.contains(&element) {
                    failures.push(format!(
                        "frame{frame_no} cache-root membership should retain {label}; elements={elements:?} missing={element:?}"
                    ));
                }
            }
        });

        app.advance_frame();
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n"));
    }
}

#[test]
fn environment_safe_area_insets_change_invalidates_view_cache_subtree() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders_env = Arc::new(AtomicUsize::new(0));
    let renders_plain = Arc::new(AtomicUsize::new(0));

    let mut root: Option<NodeId> = None;

    for frame in 0..4 {
        let insets = match frame {
            0 | 1 => None,
            2 | 3 => Some(fret_core::Edges::all(Px(12.0))),
            _ => unreachable!(),
        };

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |rt, _| {
            rt.set_window_safe_area_insets(window, insets);
        });

        let renders_env = renders_env.clone();
        let renders_plain = renders_plain.clone();

        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "environment-query-safe-area-view-cache",
            move |cx| {
                let cached_env =
                    cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                        renders_env.fetch_add(1, Ordering::SeqCst);
                        let _ = cx.environment_safe_area_insets(Invalidation::Layout);
                        vec![cx.text("env")]
                    });

                let cached_plain = cx.view_cache(
                    crate::element::ViewCacheProps {
                        cache_key: 1,
                        ..Default::default()
                    },
                    move |cx| {
                        renders_plain.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text("plain")]
                    },
                );

                vec![cached_env, cached_plain]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();
    }

    assert_eq!(
        renders_env.load(Ordering::SeqCst),
        2,
        "expected safe-area insets change to force a view-cache rerender"
    );
    assert_eq!(
        renders_plain.load(Ordering::SeqCst),
        1,
        "expected view-cache subtree without environment dependencies to reuse"
    );
}

#[test]
fn environment_occlusion_insets_change_invalidates_view_cache_subtree() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders_env = Arc::new(AtomicUsize::new(0));
    let renders_plain = Arc::new(AtomicUsize::new(0));

    let mut root: Option<NodeId> = None;

    for frame in 0..4 {
        let insets = match frame {
            0 | 1 => None,
            2 | 3 => Some(fret_core::Edges::all(Px(48.0))),
            _ => unreachable!(),
        };

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |rt, _| {
            rt.set_window_occlusion_insets(window, insets);
        });

        let renders_env = renders_env.clone();
        let renders_plain = renders_plain.clone();

        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "environment-query-occlusion-insets-view-cache",
            move |cx| {
                let cached_env =
                    cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                        renders_env.fetch_add(1, Ordering::SeqCst);
                        let _ = cx.environment_occlusion_insets(Invalidation::Layout);
                        vec![cx.text("env")]
                    });

                let cached_plain = cx.view_cache(
                    crate::element::ViewCacheProps {
                        cache_key: 1,
                        ..Default::default()
                    },
                    move |cx| {
                        renders_plain.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text("plain")]
                    },
                );

                vec![cached_env, cached_plain]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();
    }

    assert_eq!(
        renders_env.load(Ordering::SeqCst),
        2,
        "expected occlusion insets change to force a view-cache rerender"
    );
    assert_eq!(
        renders_plain.load(Ordering::SeqCst),
        1,
        "expected view-cache subtree without environment dependencies to reuse"
    );
}

#[test]
fn environment_color_scheme_change_invalidates_view_cache_subtree() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders_env = Arc::new(AtomicUsize::new(0));
    let renders_plain = Arc::new(AtomicUsize::new(0));

    let mut root: Option<NodeId> = None;

    for frame in 0..4 {
        let scheme = match frame {
            0 | 1 => Some(fret_core::ColorScheme::Light),
            2 | 3 => Some(fret_core::ColorScheme::Dark),
            _ => unreachable!(),
        };

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |rt, _| {
            rt.set_window_color_scheme(window, scheme);
        });

        let renders_env = renders_env.clone();
        let renders_plain = renders_plain.clone();

        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "environment-query-color-scheme-view-cache",
            move |cx| {
                let cached_env =
                    cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                        renders_env.fetch_add(1, Ordering::SeqCst);
                        let _ = cx.environment_color_scheme(Invalidation::Paint);
                        vec![cx.text("env")]
                    });

                let cached_plain = cx.view_cache(
                    crate::element::ViewCacheProps {
                        cache_key: 1,
                        ..Default::default()
                    },
                    move |cx| {
                        renders_plain.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text("plain")]
                    },
                );

                vec![cached_env, cached_plain]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();
    }

    assert_eq!(
        renders_env.load(Ordering::SeqCst),
        2,
        "expected color scheme change to force a view-cache rerender"
    );
    assert_eq!(
        renders_plain.load(Ordering::SeqCst),
        1,
        "expected view-cache subtree without environment dependencies to reuse"
    );
}

#[test]
fn environment_contrast_preference_change_invalidates_view_cache_subtree() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders_env = Arc::new(AtomicUsize::new(0));
    let renders_plain = Arc::new(AtomicUsize::new(0));

    let mut root: Option<NodeId> = None;

    for frame in 0..4 {
        let pref = match frame {
            0 | 1 => Some(fret_core::ContrastPreference::NoPreference),
            2 | 3 => Some(fret_core::ContrastPreference::More),
            _ => unreachable!(),
        };

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |rt, _| {
            rt.set_window_contrast_preference(window, pref);
        });

        let renders_env = renders_env.clone();
        let renders_plain = renders_plain.clone();

        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "environment-query-contrast-preference-view-cache",
            move |cx| {
                let cached_env =
                    cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                        renders_env.fetch_add(1, Ordering::SeqCst);
                        let _ = cx.environment_prefers_contrast(Invalidation::Paint);
                        vec![cx.text("env")]
                    });

                let cached_plain = cx.view_cache(
                    crate::element::ViewCacheProps {
                        cache_key: 1,
                        ..Default::default()
                    },
                    move |cx| {
                        renders_plain.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text("plain")]
                    },
                );

                vec![cached_env, cached_plain]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();
    }

    assert_eq!(
        renders_env.load(Ordering::SeqCst),
        2,
        "expected contrast preference change to force a view-cache rerender"
    );
    assert_eq!(
        renders_plain.load(Ordering::SeqCst),
        1,
        "expected view-cache subtree without environment dependencies to reuse"
    );
}

#[test]
fn environment_forced_colors_mode_change_invalidates_view_cache_subtree() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let renders_env = Arc::new(AtomicUsize::new(0));
    let renders_plain = Arc::new(AtomicUsize::new(0));

    let mut root: Option<NodeId> = None;

    for frame in 0..4 {
        let mode = match frame {
            0 | 1 => Some(fret_core::ForcedColorsMode::None),
            2 | 3 => Some(fret_core::ForcedColorsMode::Active),
            _ => unreachable!(),
        };

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |rt, _| {
            rt.set_window_forced_colors_mode(window, mode);
        });

        let renders_env = renders_env.clone();
        let renders_plain = renders_plain.clone();

        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "environment-query-forced-colors-mode-view-cache",
            move |cx| {
                let cached_env =
                    cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                        renders_env.fetch_add(1, Ordering::SeqCst);
                        let _ = cx.environment_forced_colors_mode(Invalidation::Paint);
                        vec![cx.text("env")]
                    });

                let cached_plain = cx.view_cache(
                    crate::element::ViewCacheProps {
                        cache_key: 1,
                        ..Default::default()
                    },
                    move |cx| {
                        renders_plain.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text("plain")]
                    },
                );

                vec![cached_env, cached_plain]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();
    }

    assert_eq!(
        renders_env.load(Ordering::SeqCst),
        2,
        "expected forced-colors mode change to force a view-cache rerender"
    );
    assert_eq!(
        renders_plain.load(Ordering::SeqCst),
        1,
        "expected view-cache subtree without environment dependencies to reuse"
    );
}
