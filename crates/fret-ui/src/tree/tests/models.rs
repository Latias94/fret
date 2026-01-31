use super::*;

#[test]
fn model_change_invalidates_observers() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(ObservingWidget {
        model: model.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.test_clear_node_invalidations(node);

    let _ = model.update(&mut app, |v, _cx| {
        *v += 1;
    });
    let changed = app.take_changed_models();
    assert!(changed.contains(&model.id()));

    ui.propagate_model_changes(&mut app, &changed);
    let n = ui.nodes.get(node).unwrap();
    assert!(n.invalidation.layout);
    assert!(n.invalidation.paint);
}

#[test]
fn debug_invalidation_walks_record_model_change_root() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let node = ui.create_node(ObservingWidget {
        model: model.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    ui.propagate_model_changes(&mut app, &changed);

    let walks = ui.debug_invalidation_walks();
    assert!(
        walks.iter().any(|w| {
            w.root == node
                && w.source == UiDebugInvalidationSource::ModelChange
                && w.detail == UiDebugInvalidationDetail::ModelObservation
                && w.inv == Invalidation::Layout
                && w.walked_nodes > 0
        }),
        "expected a model-change layout invalidation walk rooted at the observing node; walks={walks:?}"
    );
}

#[test]
fn model_change_invalidates_observers_across_windows() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );

    let mut ui_a = UiTree::new();
    ui_a.set_window(window_a);
    let node_a = ui_a.create_node(ObservingWidget {
        model: model.clone(),
    });
    ui_a.set_root(node_a);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene_a = Scene::default();
    ui_a.paint_all(&mut app, &mut services, bounds, &mut scene_a, 1.0);
    ui_a.test_clear_node_invalidations(node_a);

    let mut ui_b = UiTree::new();
    ui_b.set_window(window_b);
    let node_b = ui_b.create_node(ObservingWidget {
        model: model.clone(),
    });
    ui_b.set_root(node_b);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene_b = Scene::default();
    ui_b.paint_all(&mut app, &mut services, bounds, &mut scene_b, 1.0);
    ui_b.test_clear_node_invalidations(node_b);

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(changed.contains(&model.id()));

    ui_a.propagate_model_changes(&mut app, &changed);
    ui_b.propagate_model_changes(&mut app, &changed);

    let na = ui_a.nodes.get(node_a).unwrap();
    assert!(na.invalidation.layout);
    assert!(na.invalidation.paint);

    let nb = ui_b.nodes.get(node_b).unwrap();
    assert!(nb.invalidation.layout);
    assert!(nb.invalidation.paint);
}

#[test]
fn paint_observation_only_invalidates_paint() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let node = ui.create_node(PaintObservingWidget {
        model: model.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.test_clear_node_invalidations(node);

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(ui.propagate_model_changes(&mut app, &changed));

    let n = ui.nodes.get(node).unwrap();
    assert!(!n.invalidation.layout);
    assert!(n.invalidation.paint);
    assert!(!n.invalidation.hit_test);
}

#[test]
fn hit_test_observation_escalates_to_layout_and_paint() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let node = ui.create_node(HitTestObservingWidget {
        model: model.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.test_clear_node_invalidations(node);

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    assert!(ui.propagate_model_changes(&mut app, &changed));

    let n = ui.nodes.get(node).unwrap();
    assert!(n.invalidation.hit_test);
    assert!(n.invalidation.layout);
    assert!(n.invalidation.paint);
}

#[test]
fn model_change_requests_redraw_for_each_invalidated_window() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );

    let mut ui_a = UiTree::new();
    ui_a.set_window(window_a);
    let node_a = ui_a.create_node(PaintObservingWidget {
        model: model.clone(),
    });
    ui_a.set_root(node_a);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene_a = Scene::default();
    ui_a.paint_all(&mut app, &mut services, bounds, &mut scene_a, 1.0);
    ui_a.test_clear_node_invalidations(node_a);

    let mut ui_b = UiTree::new();
    ui_b.set_window(window_b);
    let node_b = ui_b.create_node(PaintObservingWidget {
        model: model.clone(),
    });
    ui_b.set_root(node_b);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene_b = Scene::default();
    ui_b.paint_all(&mut app, &mut services, bounds, &mut scene_b, 1.0);
    ui_b.test_clear_node_invalidations(node_b);

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();

    ui_a.propagate_model_changes(&mut app, &changed);
    ui_b.propagate_model_changes(&mut app, &changed);

    let effects = app.flush_effects();
    let redraws: std::collections::HashSet<AppWindowId> = effects
        .into_iter()
        .filter_map(|e| match e {
            Effect::Redraw(w) => Some(w),
            _ => None,
        })
        .collect();
    let expected: std::collections::HashSet<AppWindowId> =
        [window_a, window_b].into_iter().collect();

    assert_eq!(redraws, expected);
}

#[test]
fn model_change_invalidation_dedup_stops_at_shared_ancestors() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack::default());
    let left = ui.create_node(TestStack::default());
    let right = ui.create_node(TestStack::default());
    let leaf_a = ui.create_node(PaintObservingWidget {
        model: model.clone(),
    });
    let leaf_b = ui.create_node(PaintObservingWidget {
        model: model.clone(),
    });

    ui.set_root(root);
    ui.add_child(root, left);
    ui.add_child(root, right);
    ui.add_child(left, leaf_a);
    ui.add_child(right, leaf_b);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    app.advance_frame();

    assert!(ui.propagate_model_changes(&mut app, &changed));

    let stats = ui.debug_stats();
    assert_eq!(stats.model_change_invalidation_roots, 2);
    assert_eq!(stats.invalidation_walk_calls, 2);
    assert_eq!(stats.invalidation_walk_nodes, 5);
}

#[test]
fn paint_all_sets_ime_allowed_for_focused_text_input() {
    #[derive(Default)]
    struct FakeTextInput;

    impl<H: UiHost> Widget<H> for FakeTextInput {
        fn is_text_input(&self) -> bool {
            true
        }

        fn is_focusable(&self) -> bool {
            true
        }
    }

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let node = ui.create_node(FakeTextInput);
    ui.set_root(node);
    ui.set_focus(Some(node));

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, fret_runtime::Effect::ImeAllow { enabled: true, .. }))
    );
}
