use super::*;
use fret_runtime::ModelId;

#[test]
fn named_scopes_produce_stable_element_ids_across_frames() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let leaf_ids = Arc::new(std::sync::Mutex::new(
        Vec::<crate::elements::GlobalElementId>::new(),
    ));

    let mut root: Option<NodeId> = None;
    for frame in 0..2 {
        let leaf_ids = leaf_ids.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "named-scopes-stable",
            move |cx| {
                let leaf = cx.named("sidebar", |cx| cx.text("leaf"));
                leaf_ids.lock().unwrap().push(leaf.id);
                vec![leaf]
            },
        );

        root.get_or_insert(root_node);
        if frame == 0 {
            ui.set_root(root_node);
        }
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        app.advance_frame();
    }

    let ids = leaf_ids.lock().unwrap();
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[0], ids[1]);
}

#[test]
fn keyed_list_reorder_preserves_element_identity_for_state() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let results = Arc::new(std::sync::Mutex::new(Vec::<Vec<(u64, u64)>>::new()));
    let orders: [Vec<u64>; 2] = [vec![1, 2, 3], vec![3, 2, 1]];

    let mut root: Option<NodeId> = None;
    for (frame, items) in orders.into_iter().enumerate() {
        let results = results.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "keyed-reorder-identity",
            move |cx| {
                let mut elements = Vec::new();
                let mut frame_results = Vec::new();
                for item in items {
                    let el = cx.keyed(item, |cx| cx.text("row"));
                    let remembered = cx.with_state_for(el.id, || item, |st: &mut u64| *st);
                    frame_results.push((item, remembered));
                    elements.push(el);
                }
                results.lock().unwrap().push(frame_results);
                elements
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

    let results = results.lock().unwrap();
    assert_eq!(results.len(), 2);
    for &(item, remembered) in &results[1] {
        assert_eq!(
            item, remembered,
            "expected keyed items to preserve element identity across reorder"
        );
    }
}

#[test]
fn unkeyed_list_reorder_does_not_preserve_element_identity_for_state() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let results = Arc::new(std::sync::Mutex::new(Vec::<Vec<(u64, u64)>>::new()));
    let orders: [Vec<u64>; 2] = [vec![1, 2, 3], vec![3, 2, 1]];

    let mut root: Option<NodeId> = None;
    for (frame, items) in orders.into_iter().enumerate() {
        let results = results.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "unkeyed-reorder-identity",
            move |cx| {
                let mut elements = Vec::new();
                let mut frame_results = Vec::new();
                cx.for_each_unkeyed(&items, |cx, _idx, item| {
                    let el = cx.text("row");
                    let remembered = cx.with_state_for(el.id, || *item, |st: &mut u64| *st);
                    frame_results.push((*item, remembered));
                    elements.push(el);
                });
                results.lock().unwrap().push(frame_results);
                elements
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

    let results = results.lock().unwrap();
    assert_eq!(results.len(), 2);

    let reordered = &results[1];
    assert!(
        reordered
            .iter()
            .any(|(item, remembered)| item != remembered),
        "expected unkeyed reorder to reuse element identity by index, not item key"
    );
}

#[test]
fn element_tree_duplicate_ids_detect_duplicates_in_one_frame() {
    use crate::element::{ContainerProps, ElementKind};
    use crate::elements::GlobalElementId;

    let a = AnyElement::new(
        GlobalElementId(1),
        ElementKind::Container(ContainerProps::default()),
        Vec::new(),
    );
    let b = AnyElement::new(
        GlobalElementId(1),
        ElementKind::Container(ContainerProps::default()),
        Vec::new(),
    );

    let duplicates = super::super::mount::element_tree_duplicate_ids(&[a, b]);
    assert_eq!(duplicates, vec![GlobalElementId(1)]);
}

#[test]
fn local_model_assigns_distinct_callsite_slots_and_preserves_ids_across_frames() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let observed = Arc::new(std::sync::Mutex::new(Vec::<(ModelId, ModelId)>::new()));
    let mut root: Option<NodeId> = None;

    for frame in 0..2 {
        let observed = observed.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "local-model-callsite-stability",
            move |cx| {
                let first = cx.local_model(|| 1u32);
                let second = cx.local_model(|| 2u32);
                observed.lock().unwrap().push((first.id(), second.id()));
                vec![cx.text("leaf")]
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

    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 2);
    assert_ne!(
        observed[0].0, observed[0].1,
        "distinct local_model callsites should not share a slot"
    );
    assert_eq!(
        observed[0], observed[1],
        "expected local_model callsite slots to preserve model identity across frames"
    );
}

#[test]
fn local_model_keyed_preserves_model_identity_across_reorder() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let observed = Arc::new(std::sync::Mutex::new(Vec::<Vec<(u64, ModelId)>>::new()));
    let orders: [Vec<u64>; 2] = [vec![1, 2, 3], vec![3, 2, 1]];

    let mut root: Option<NodeId> = None;
    for (frame, items) in orders.into_iter().enumerate() {
        let observed = observed.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "local-model-keyed-reorder",
            move |cx| {
                let mut frame_results = Vec::new();
                let mut elements = Vec::new();
                for item in items {
                    let model = cx.local_model_keyed(item, || item);
                    frame_results.push((item, model.id()));
                    elements.push(cx.keyed(item, |cx| cx.text("row")));
                }
                observed.lock().unwrap().push(frame_results);
                elements
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

    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 2);
    let first: std::collections::HashMap<u64, ModelId> = observed[0].iter().copied().collect();
    let second: std::collections::HashMap<u64, ModelId> = observed[1].iter().copied().collect();
    assert_eq!(
        first, second,
        "expected local_model_keyed to preserve model identity by key across reorder"
    );
}

#[test]
#[cfg(feature = "diagnostics")]
fn named_scopes_appear_in_debug_paths() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let leaf_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "named-debug-path",
        |cx| {
            let leaf = cx.named("sidebar", |cx| cx.text("leaf"));
            *leaf_id.lock().unwrap() = Some(leaf.id);
            vec![leaf]
        },
    );
    ui.set_root(root);

    let leaf = leaf_id.lock().unwrap().expect("leaf id");
    let debug_path = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime.debug_path_for_element(window, leaf)
    });

    let debug_path = debug_path.expect("debug path");
    assert!(
        debug_path.contains("name=sidebar"),
        "expected named segment in debug path: {debug_path}"
    );
}

#[test]
#[cfg(feature = "diagnostics")]
fn debug_paths_survive_gc_when_touching_only_leaf_elements() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let leaf_id = Arc::new(std::sync::Mutex::new(
        None::<crate::elements::GlobalElementId>,
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "debug-path-gc-touch-leaf",
        |cx| {
            let leaf = cx.named("a", |cx| cx.named("b", |cx| cx.text("leaf")));
            *leaf_id.lock().unwrap() = Some(leaf.id);
            vec![leaf]
        },
    );
    ui.set_root(root);

    let leaf = leaf_id.lock().unwrap().expect("leaf id");

    // Simulate a long run of cache-hit frames where only a subset of elements are touched for
    // liveness bookkeeping.
    for _ in 0..8 {
        app.advance_frame();
        let frame_id = app.frame_id();
        app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
            runtime.prepare_window_for_frame(window, frame_id);
            runtime
                .for_window_mut(window)
                .touch_debug_identity_for_element(frame_id, leaf);
        });
    }

    let debug_path = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime.debug_path_for_element(window, leaf)
    });

    let debug_path = debug_path.expect("debug path");
    assert!(
        debug_path.contains("name=a") && debug_path.contains("name=b"),
        "expected named segments in debug path after GC touches: {debug_path}"
    );
}
