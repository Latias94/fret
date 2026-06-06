use super::*;

#[test]
fn tab_bar_helper_switches_selected_panel_and_updates_selection_model() {
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
    let selected_out = Rc::new(RefCell::new(None::<Arc<str>>));
    let selected_out_render = selected_out.clone();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.tab_bar_with_options(
                "workspace",
                fret_ui_kit::imui::TabBarOptions {
                    selected: Some(selected_model.clone()),
                    test_id: Some(Arc::from("imui-tab-bar-interaction.root")),
                    ..Default::default()
                },
                |tabs| {
                    tabs.begin_tab_item_with_options(
                        "scene",
                        "Scene",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-bar-interaction.scene")),
                            panel_test_id: Some(Arc::from("imui-tab-bar-interaction.scene.panel")),
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
                            test_id: Some(Arc::from("imui-tab-bar-interaction.inspector")),
                            panel_test_id: Some(Arc::from(
                                "imui-tab-bar-interaction.inspector.panel",
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Inspector Panel");
                        },
                    );
                },
            );

            let selected = ui
                .cx_mut()
                .read_model(
                    &selected_model,
                    fret_ui::Invalidation::Paint,
                    |_app, value| value.clone(),
                )
                .unwrap_or(None);
            selected_out_render.replace(selected);
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-bar-interaction",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("inspector"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.inspector.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.scene.panel",
    ));

    let scene_tab = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.scene",
    );
    click_at(&mut ui, &mut app, &mut services, scene_tab);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-bar-interaction",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("scene"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.scene.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.inspector.panel",
    ));

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let scene_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar-interaction.scene"))
        .expect("scene tab semantics node");
    let inspector_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar-interaction.inspector"))
        .expect("inspector tab semantics node");
    assert!(scene_node.flags.selected);
    assert!(!inspector_node.flags.selected);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-bar-interaction",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("scene"));
}
