use super::*;

#[test]
fn tab_bar_helper_reports_selected_change_and_trigger_edges() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let selected_model = app.models_mut().insert(Some(Arc::<str>::from("inspector")));
    let selected_id = Rc::new(RefCell::new(None::<String>));
    let selected_changed = Rc::new(Cell::new(false));
    let scene_clicked = Rc::new(Cell::new(false));
    let scene_activated = Rc::new(Cell::new(false));
    let scene_deactivated = Rc::new(Cell::new(false));
    let scene_selected = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        let selected_id = selected_id.clone();
        let selected_changed = selected_changed.clone();
        let scene_clicked = scene_clicked.clone();
        let scene_activated = scene_activated.clone();
        let scene_deactivated = scene_deactivated.clone();
        let scene_selected = scene_selected.clone();
        let selected_model = selected_model.clone();

        crate::imui_raw(cx, move |ui| {
            let tabs = ui.tab_bar_with_options(
                "workspace",
                fret_ui_kit::imui::TabBarOptions {
                    selected: Some(selected_model.clone()),
                    test_id: Some(Arc::from("imui-tab-response.root")),
                    ..Default::default()
                },
                |tabs| {
                    tabs.begin_tab_item_with_options(
                        "scene",
                        "Scene",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-response.scene")),
                            panel_test_id: Some(Arc::from("imui-tab-response.scene.panel")),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Scene Panel");
                        },
                    );
                    tabs.begin_tab_item_with_options(
                        "inspector",
                        "Inspector",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-response.inspector")),
                            panel_test_id: Some(Arc::from("imui-tab-response.inspector.panel")),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Inspector Panel");
                        },
                    );
                },
            );

            selected_id.replace(tabs.selected_id().map(str::to_owned));
            selected_changed.set(tabs.selected_changed());
            if let Some(scene) = tabs.trigger("scene") {
                scene_clicked.set(scene.clicked());
                scene_activated.set(scene.activated());
                scene_deactivated.set(scene.deactivated());
                scene_selected.set(scene.selected());
            }
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-response",
        render,
    );
    assert_eq!(selected_id.borrow().as_deref(), Some("inspector"));
    assert!(!selected_changed.get());
    assert!(!scene_clicked.get());
    assert!(!scene_activated.get());
    assert!(!scene_deactivated.get());
    assert!(!scene_selected.get());

    let scene_tab = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-response.scene",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, scene_tab);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-response",
        &render,
    );
    assert_eq!(selected_id.borrow().as_deref(), Some("inspector"));
    assert!(!selected_changed.get());
    assert!(scene_activated.get());
    assert!(!scene_clicked.get());
    assert!(!scene_deactivated.get());
    assert!(!scene_selected.get());

    pointer_up_at(&mut ui, &mut app, &mut services, scene_tab);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-response",
        &render,
    );
    assert_eq!(selected_id.borrow().as_deref(), Some("scene"));
    assert!(selected_changed.get());
    assert!(scene_clicked.get());
    assert!(scene_deactivated.get());
    assert!(scene_selected.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-response.scene.panel",
    ));

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-response",
        &render,
    );
    assert_eq!(selected_id.borrow().as_deref(), Some("scene"));
    assert!(!selected_changed.get());
    assert!(!scene_clicked.get());
    assert!(!scene_activated.get());
    assert!(!scene_deactivated.get());
    assert!(scene_selected.get());
}
