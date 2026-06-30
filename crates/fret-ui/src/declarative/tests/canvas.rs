#![allow(clippy::arc_with_non_send_sync)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use fret_core::FontId;
use fret_runtime::Model;

#[derive(Debug, Clone)]
struct TestFragmentPlan {
    entries: usize,
}

impl crate::tree::BoundarySceneFragmentDebug for TestFragmentPlan {
    fn boundary_scene_fragment_entry_count(&self) -> usize {
        self.entries
    }
}

#[test]
fn canvas_resolves_passive_text_style_and_foreground_from_current_scope() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let captured_style: Rc<RefCell<Option<TextStyle>>> = Rc::new(RefCell::new(None));
    let captured_fg: Rc<RefCell<Option<Color>>> = Rc::new(RefCell::new(None));

    let inherited = fret_core::TextStyleRefinement {
        size: Some(Px(17.0)),
        line_height: Some(Px(25.0)),
        weight: Some(fret_core::FontWeight::SEMIBOLD),
        ..Default::default()
    };
    let inherited_fg = Color {
        r: 0.25,
        g: 0.5,
        b: 0.75,
        a: 1.0,
    };

    let paint = {
        let captured_style = Rc::clone(&captured_style);
        let captured_fg = Rc::clone(&captured_fg);
        move |p: &mut crate::canvas::CanvasPainter<'_>| {
            *captured_style.borrow_mut() = Some(p.resolved_passive_text_style(None));
            *captured_fg.borrow_mut() = p.inherited_foreground();
        }
    };

    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "canvas-resolved-passive-text-style",
        |cx| {
            vec![
                cx.canvas(crate::element::CanvasProps::default(), paint)
                    .inherit_text_style(inherited.clone())
                    .inherit_foreground(inherited_fg),
            ]
        },
    );
    ui.set_root(node);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let resolved = captured_style
        .borrow()
        .clone()
        .expect("expected canvas paint to capture a resolved style");
    assert_eq!(resolved.size, Px(17.0));
    assert_eq!(resolved.line_height, Some(Px(25.0)));
    assert_eq!(resolved.weight, fret_core::FontWeight::SEMIBOLD);
    assert_eq!(*captured_fg.borrow(), Some(inherited_fg));
}

#[test]
fn canvas_paint_observation_replays_without_runtime_empty_deps_lookup_for_empty_siblings() {
    let mut app = TestHost::new();
    let model: Model<u32> = app.models_mut().insert(0);
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.set_paint_cache_enabled(false);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let observed = model.clone();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "canvas-paint-observation-presence",
        |cx| {
            cx.observe_model(&observed, Invalidation::Paint);
            vec![cx.stack(|cx| {
                vec![
                    cx.canvas(crate::element::CanvasProps::default(), |_p| {}),
                    cx.canvas(crate::element::CanvasProps::default(), |_p| {}),
                    cx.canvas(crate::element::CanvasProps::default(), |_p| {}),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let stats = ui.debug_stats();
    let first_calls = stats.paint_host_widget_observed_deps_calls;
    assert!(first_calls > 0);
    assert_eq!(stats.paint_host_widget_observed_models_non_empty_calls, 1);
    assert_eq!(
        stats.paint_host_widget_observed_deps_empty_calls,
        first_calls - 1
    );

    app.advance_frame();
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let stats = ui.debug_stats();
    assert_eq!(stats.paint_host_widget_observed_deps_calls, first_calls);
    assert_eq!(
        stats.paint_host_widget_observed_deps_empty_calls,
        first_calls - 1
    );
    assert_eq!(stats.paint_host_widget_observed_models_non_empty_calls, 1);

    ui.test_clear_node_invalidations(root);
    let _ = model.update(&mut app, |value, _cx| *value += 1);
    let changed = app.take_changed_models();
    assert!(ui.propagate_model_changes(&mut app, &changed));
}

#[test]
fn canvas_resolved_passive_text_style_prefers_explicit_over_inherited() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let captured_style: Rc<RefCell<Option<TextStyle>>> = Rc::new(RefCell::new(None));
    let explicit = TextStyle {
        font: FontId::default(),
        size: Px(21.0),
        line_height: Some(Px(29.0)),
        weight: fret_core::FontWeight::MEDIUM,
        ..Default::default()
    };
    let inherited = fret_core::TextStyleRefinement {
        size: Some(Px(17.0)),
        line_height: Some(Px(25.0)),
        weight: Some(fret_core::FontWeight::SEMIBOLD),
        ..Default::default()
    };

    let paint = {
        let captured_style = Rc::clone(&captured_style);
        let explicit = explicit.clone();
        move |p: &mut crate::canvas::CanvasPainter<'_>| {
            *captured_style.borrow_mut() =
                Some(p.resolved_passive_text_style(Some(explicit.clone())));
        }
    };

    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "canvas-resolved-passive-text-style-explicit-wins",
        |cx| {
            vec![
                cx.canvas(crate::element::CanvasProps::default(), paint)
                    .inherit_text_style(inherited.clone()),
            ]
        },
    );
    ui.set_root(node);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let resolved = captured_style
        .borrow()
        .clone()
        .expect("expected canvas paint to capture a resolved style");
    assert_eq!(resolved.size, Px(21.0));
    assert_eq!(resolved.line_height, Some(Px(29.0)));
    assert_eq!(resolved.weight, fret_core::FontWeight::MEDIUM);
}

#[test]
fn canvas_prepaint_hook_runs_before_paint_without_view_cache_root() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let prepaints = Arc::new(AtomicUsize::new(0));
    let paints = Arc::new(AtomicUsize::new(0));

    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "canvas-prepaint-hook",
        |cx| {
            let prepaints_for_prepaint = prepaints.clone();
            let prepaints_for_paint = prepaints.clone();
            let paints = paints.clone();
            vec![cx.canvas_with_prepaint(
                crate::element::CanvasProps::default(),
                move |cx| {
                    assert_eq!(cx.bounds().size, bounds.size);
                    prepaints_for_prepaint.fetch_add(1, Ordering::SeqCst);
                },
                move |_p| {
                    assert_eq!(prepaints_for_paint.load(Ordering::SeqCst), 1);
                    paints.fetch_add(1, Ordering::SeqCst);
                },
            )]
        },
    );
    ui.set_root(node);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(prepaints.load(Ordering::SeqCst), 1);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
}

#[test]
fn canvas_prepaint_output_is_visible_to_canvas_paint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let seen = Arc::new(AtomicUsize::new(0));
    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "canvas-prepaint-output",
        |cx| {
            let seen = seen.clone();
            vec![cx.canvas_with_prepaint(
                crate::element::CanvasProps::default(),
                move |cx| {
                    let prev = cx.output::<usize>().copied().unwrap_or(0);
                    cx.set_output(prev.saturating_add(1));
                },
                move |p| {
                    seen.store(
                        p.prepaint_output::<usize>().copied().unwrap_or(0),
                        Ordering::SeqCst,
                    );
                },
            )]
        },
    );
    ui.set_root(node);
    let canvas_node = ui.children(node)[0];
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(seen.load(Ordering::SeqCst), 1);

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        seen.load(Ordering::SeqCst),
        1,
        "stable frames should preserve the previous canvas prepaint output"
    );

    app.advance_frame();
    ui.invalidate(canvas_node, crate::widget::Invalidation::Paint);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        seen.load(Ordering::SeqCst),
        2,
        "repainted frames with the same prepaint key should observe previous canvas output"
    );

    app.advance_frame();
    ui.invalidate(canvas_node, crate::widget::Invalidation::Paint);
    ui.layout_all(&mut app, &mut services, bounds, 2.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 2.0);
    assert_eq!(
        seen.load(Ordering::SeqCst),
        1,
        "changing the canvas prepaint key should reset canvas output state"
    );
}

#[test]
fn canvas_scene_fragment_is_boundary_owned_and_keyed_by_prepaint_key() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let seen = Arc::new(AtomicUsize::new(0));
    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "canvas-scene-fragment",
        |cx| {
            let seen = seen.clone();
            vec![cx.canvas_with_prepaint(
                crate::element::CanvasProps::default(),
                move |cx| {
                    let prev = cx
                        .scene_fragment::<crate::canvas::CanvasSceneFragment<TestFragmentPlan>>()
                        .map(|fragment| fragment.payload.entries)
                        .unwrap_or(0);
                    cx.set_scene_fragment_debug(crate::canvas::CanvasSceneFragment::new(
                        TestFragmentPlan {
                            entries: prev.saturating_add(1),
                        },
                        Arc::from([]),
                        crate::canvas::CanvasHostedResources::default(),
                        cx.bounds(),
                        cx.bounds().origin,
                    ));
                },
                move |p| {
                    seen.store(
                        p.scene_fragment::<crate::canvas::CanvasSceneFragment<TestFragmentPlan>>()
                            .map(|fragment| fragment.payload.entries)
                            .unwrap_or(0),
                        Ordering::SeqCst,
                    );
                    p.record_scene_fragment_used_entries(1);
                },
            )]
        },
    );
    ui.set_root(node);
    let canvas_node = ui.children(node)[0];
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(seen.load(Ordering::SeqCst), 1);
    let boundary = ui
        .debug_boundary_stats()
        .into_iter()
        .find(|boundary| boundary.scene_fragment_entries == 1)
        .expect("expected canvas boundary stats");
    assert_eq!(
        boundary.scene_fragment_owner,
        "view_boundary_scene_fragment_state"
    );
    assert_eq!(boundary.scene_fragment_slots, 1);
    assert_eq!(boundary.scene_fragment_entries, 1);
    assert_eq!(boundary.scene_fragment_used_entries, 1);
    assert_eq!(boundary.scene_fragment_rejected_entries, 0);
    assert_eq!(boundary.scene_fragment_reject_reason, None);

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        seen.load(Ordering::SeqCst),
        1,
        "stable frames should preserve the previous boundary scene fragment"
    );
    let boundary = ui
        .debug_boundary_stats()
        .into_iter()
        .find(|boundary| boundary.scene_fragment_entries == 1)
        .expect("expected canvas boundary stats after stable cache hit");
    assert_eq!(
        boundary.scene_fragment_used_entries, 1,
        "paint-cache hits should not record an additional scene-fragment use"
    );

    app.advance_frame();
    ui.invalidate(canvas_node, crate::widget::Invalidation::Paint);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        seen.load(Ordering::SeqCst),
        2,
        "repainted frames with the same key should observe the previous scene fragment"
    );
    let boundary = ui
        .debug_boundary_stats()
        .into_iter()
        .find(|boundary| boundary.scene_fragment_entries == 2)
        .expect("expected canvas boundary stats after repaint");
    assert_eq!(boundary.scene_fragment_slots, 1);
    assert_eq!(boundary.scene_fragment_entries, 2);
    assert_eq!(boundary.scene_fragment_used_entries, 2);
    assert_eq!(boundary.scene_fragment_rejected_entries, 0);

    app.advance_frame();
    ui.invalidate(canvas_node, crate::widget::Invalidation::Paint);
    ui.layout_all(&mut app, &mut services, bounds, 2.0);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 2.0);
    assert_eq!(
        seen.load(Ordering::SeqCst),
        1,
        "changing the prepaint key should reset the boundary scene fragment"
    );
}

#[test]
fn canvas_prepaint_can_prepare_text_scene_fragment_before_paint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let seen = Arc::new(AtomicUsize::new(0));
    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "canvas-prepaint-prepares-text-fragment",
        |cx| {
            let seen = seen.clone();
            vec![cx.canvas_with_prepaint(
                crate::element::CanvasProps::default(),
                move |cx| {
                    let fragment =
                        cx.prepare_scene_fragment(cx.bounds(), cx.bounds().origin, move |p| {
                            let key = p.child_key(p.key_scope(&"prepaint-text-fragment"), &0u8).0;
                            let _ = p.text_with_blob(
                                key,
                                fret_core::DrawOrder(0),
                                Point::new(Px(4.0), Px(12.0)),
                                "hello",
                                TextStyle::default(),
                                Color {
                                    r: 1.0,
                                    g: 1.0,
                                    b: 1.0,
                                    a: 1.0,
                                },
                                crate::canvas::CanvasTextConstraints::default(),
                                p.scale_factor(),
                            );
                            TestFragmentPlan { entries: 1 }
                        });
                    cx.set_scene_fragment_debug(fragment);
                },
                move |p| {
                    if let Some(fragment) = p
                        .scene_fragment::<crate::canvas::CanvasSceneFragment<TestFragmentPlan>>()
                        .cloned()
                    {
                        p.touch_hosted_resources(&fragment.hosted_resources);
                        fragment.replay_translated_into(p.scene(), Point::new(Px(0.0), Px(0.0)));
                        seen.store(fragment.payload.entries, Ordering::SeqCst);
                        p.record_scene_fragment_used_entries(1);
                    }
                },
            )]
        },
    );
    ui.set_root(node);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        services.prepare_calls, 1,
        "prepaint should prepare the text blob before paint"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(seen.load(Ordering::SeqCst), 1);
    assert_eq!(
        services.prepare_calls, 1,
        "paint should replay the prepared fragment without preparing text again"
    );
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::Text { .. })),
        "paint should replay the prepaint-prepared text op"
    );
}

#[test]
fn canvas_hosts_text_and_releases_on_cleanup() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let paint = |p: &mut crate::canvas::CanvasPainter<'_>| {
        let scope = p.key_scope(&"text");
        let key = p.child_key(scope, &1u64).0;
        p.text(
            key,
            fret_core::DrawOrder(0),
            Point::new(Px(10.0), Px(10.0)),
            "hello",
            TextStyle::default(),
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            crate::canvas::CanvasTextConstraints::default(),
            p.scale_factor(),
        );
    };

    let mut root: Option<NodeId> = None;
    for pass in 0..2 {
        let node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "canvas-hosts-text",
            |cx| vec![cx.canvas(crate::element::CanvasProps::default(), paint)],
        );
        root.get_or_insert(node);
        ui.set_root(node);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(
            scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::Text { .. }))
        );

        if pass == 0 {
            assert_eq!(services.prepare_calls, 1);
            assert_eq!(services.release_calls, 0);
        } else {
            assert_eq!(services.prepare_calls, 1, "text blob should be cached");
        }

        app.advance_frame();
    }

    ui.cleanup_subtree(&mut services, root.expect("root"));
    assert_eq!(
        services.release_calls, 1,
        "canvas should release hosted resources on cleanup"
    );
}

#[test]
fn canvas_can_prepare_text_before_emitting_scene_text() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let paint = |p: &mut crate::canvas::CanvasPainter<'_>| {
        let key = p.child_key(p.key_scope(&"text"), &1u64).0;
        let style = TextStyle::default();
        let color = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let constraints = crate::canvas::CanvasTextConstraints::default();
        let (blob, metrics) =
            p.prepare_text_with_blob(key, "hello", style.clone(), constraints, p.scale_factor());
        assert_eq!(
            p.scene()
                .ops()
                .iter()
                .filter(|op| matches!(op, SceneOp::Text { .. }))
                .count(),
            0,
            "prepare_text_with_blob must not emit text scene ops"
        );

        p.scene().push(SceneOp::Text {
            order: fret_core::DrawOrder(0),
            origin: Point::new(Px(10.0), Px(10.0) + metrics.baseline),
            text: blob,
            paint: fret_core::Paint::Solid(color).into(),
            outline: None,
            shadow: None,
        });

        let _ = p.text(
            key,
            fret_core::DrawOrder(1),
            Point::new(Px(10.0), Px(30.0)),
            "hello",
            style,
            color,
            constraints,
            p.scale_factor(),
        );
    };

    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "canvas-prepare-text-before-draw",
        |cx| vec![cx.canvas(crate::element::CanvasProps::default(), paint)],
    );
    ui.set_root(node);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(services.prepare_calls, 1);
    assert_eq!(
        scene
            .ops()
            .iter()
            .filter(|op| matches!(op, SceneOp::Text { .. }))
            .count(),
        2
    );
}

#[test]
fn canvas_hosts_path_and_releases_on_cleanup() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let paint = |p: &mut crate::canvas::CanvasPainter<'_>| {
        let commands = [
            fret_core::PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            fret_core::PathCommand::LineTo(Point::new(Px(10.0), Px(10.0))),
            fret_core::PathCommand::Close,
        ];
        p.path(
            2,
            fret_core::DrawOrder(0),
            Point::new(Px(10.0), Px(10.0)),
            &commands,
            fret_core::PathStyle::Fill(fret_core::FillStyle::default()),
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            p.scale_factor(),
        );
    };

    let mut root: Option<NodeId> = None;
    for pass in 0..2 {
        let node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "canvas-hosts-path",
            |cx| vec![cx.canvas(crate::element::CanvasProps::default(), paint)],
        );
        root.get_or_insert(node);
        ui.set_root(node);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(
            scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::Path { .. }))
        );

        if pass == 0 {
            assert_eq!(services.path_prepare_calls, 1);
            assert_eq!(services.path_release_calls, 0);
        } else {
            assert_eq!(services.path_prepare_calls, 1, "path should be cached");
        }

        app.advance_frame();
    }

    ui.cleanup_subtree(&mut services, root.expect("root"));
    assert_eq!(
        services.path_release_calls, 1,
        "canvas should release hosted path resources on cleanup"
    );
}

#[test]
fn canvas_hosts_svg_and_releases_on_cleanup() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let paint = |p: &mut crate::canvas::CanvasPainter<'_>| {
        let svg = crate::SvgSource::Static(b"<svg/>");
        p.svg_mask_icon(
            3,
            fret_core::DrawOrder(0),
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            &svg,
            fret_core::SvgFit::Contain,
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            1.0,
        );
    };

    let mut root: Option<NodeId> = None;
    for pass in 0..2 {
        let node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "canvas-hosts-svg",
            |cx| vec![cx.canvas(crate::element::CanvasProps::default(), paint)],
        );
        root.get_or_insert(node);
        ui.set_root(node);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(
            scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::SvgMaskIcon { .. }))
        );

        if pass == 0 {
            assert_eq!(services.svg_register_calls, 1);
            assert_eq!(services.svg_unregister_calls, 0);
        } else {
            assert_eq!(services.svg_register_calls, 1, "svg should be cached");
        }

        app.advance_frame();
    }

    ui.cleanup_subtree(&mut services, root.expect("root"));
    assert_eq!(
        services.svg_unregister_calls, 1,
        "canvas should release hosted svg resources on cleanup"
    );
}

#[test]
fn canvas_scoped_ops_keep_scene_stacks_balanced() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let paint = |p: &mut crate::canvas::CanvasPainter<'_>| {
        p.with_transform(
            fret_core::Transform2D::translation(Point::new(Px(5.0), Px(6.0))),
            |p| {
                p.with_clip_rrect(
                    Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(50.0))),
                    fret_core::Corners::all(Px(4.0)),
                    |p| {
                        p.with_opacity(0.5, |p| {
                            p.scene().push(SceneOp::Quad {
                                order: fret_core::DrawOrder(0),
                                rect: Rect::new(
                                    Point::new(Px(1.0), Px(2.0)),
                                    Size::new(Px(10.0), Px(10.0)),
                                ),
                                background: fret_core::Paint::Solid(Color {
                                    r: 1.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                })
                                .into(),
                                border: fret_core::Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                                corner_radii: fret_core::Corners::default(),
                            });
                        });
                    },
                );
            },
        );
    };

    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "canvas-scoped-ops",
        |cx| vec![cx.canvas(crate::element::CanvasProps::default(), paint)],
    );
    ui.set_root(node);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    scene.validate().expect("scene should validate");
}

#[test]
fn canvas_paint_only_animation_frame_keeps_view_cache_root_reusable() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();
    let mut scene = Scene::default();

    let renders = Arc::new(AtomicUsize::new(0));
    let paints = Arc::new(AtomicUsize::new(0));

    for _ in 0..2 {
        let renders = renders.clone();
        let paints = paints.clone();
        render_root_for_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "canvas-paint-only-raf-view-cache",
            move |cx| {
                vec![
                    cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        let paints = paints.clone();
                        vec![cx.canvas(crate::element::CanvasProps::default(), move |p| {
                            paints.fetch_add(1, Ordering::SeqCst);
                            p.request_animation_frame_paint_only();
                        })]
                    }),
                ]
            },
        );

        layout_frame(&mut ui, &mut app, &mut services, bounds);
        paint_frame(&mut ui, &mut app, &mut services, bounds, &mut scene);

        let effects = app.take_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
            "paint-only canvas animation frames should still schedule the runner"
        );

        app.advance_frame();
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "paint-only canvas animation should keep the view-cache render subtree reusable"
    );
    assert_eq!(
        paints.load(Ordering::SeqCst),
        2,
        "paint-only canvas animation should still repaint on the next frame"
    );
}
