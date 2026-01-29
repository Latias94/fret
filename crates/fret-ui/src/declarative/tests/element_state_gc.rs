use super::*;

#[test]
fn element_state_survives_gc_lag_frames_when_element_is_temporarily_missing() {
    let mut app = TestHost::new();
    app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime.set_gc_lag_frames(2);
    });

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let observed = Arc::new(std::sync::Mutex::new(Vec::<u64>::new()));
    let mut root: Option<NodeId> = None;

    for frame in 0..4 {
        let observed = observed.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "element-state-gc-lag-survive",
            move |cx| {
                if frame == 0 || frame == 3 {
                    let leaf = cx.named("leaf", |cx| cx.text("leaf"));
                    let value = cx.with_state_for(
                        leaf.id,
                        || 0u64,
                        |v| {
                            *v = v.saturating_add(1);
                            *v
                        },
                    );
                    observed.lock().unwrap().push(value);
                    vec![leaf]
                } else {
                    vec![]
                }
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
        *observed.lock().unwrap(),
        vec![1, 2],
        "expected element state to survive exactly gc_lag_frames missed renders"
    );
}

#[test]
fn element_state_is_dropped_after_exceeding_gc_lag_frames() {
    let mut app = TestHost::new();
    app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime.set_gc_lag_frames(2);
    });

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let observed = Arc::new(std::sync::Mutex::new(Vec::<u64>::new()));
    let mut root: Option<NodeId> = None;

    for frame in 0..5 {
        let observed = observed.clone();
        let root_node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "element-state-gc-lag-drop",
            move |cx| {
                if frame == 0 || frame == 4 {
                    let leaf = cx.named("leaf", |cx| cx.text("leaf"));
                    let value = cx.with_state_for(
                        leaf.id,
                        || 0u64,
                        |v| {
                            *v = v.saturating_add(1);
                            *v
                        },
                    );
                    observed.lock().unwrap().push(value);
                    vec![leaf]
                } else {
                    vec![]
                }
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
        *observed.lock().unwrap(),
        vec![1, 1],
        "expected element state to be dropped after gc_lag_frames+1 missed renders"
    );
}
