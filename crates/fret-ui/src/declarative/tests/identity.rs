use super::*;

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
