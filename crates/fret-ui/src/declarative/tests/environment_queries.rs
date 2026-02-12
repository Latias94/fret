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

    let observed: Arc<Mutex<Option<(Option<Edges>, Option<Edges>)>>> = Arc::new(Mutex::new(None));
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
