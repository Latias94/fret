use super::*;

#[test]
fn tab_item_activate_shortcut_is_scoped_to_focused_trigger() {
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
    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.tab_bar_with_options(
                "workspace",
                fret_ui_kit::imui::TabBarOptions {
                    selected: Some(selected_model.clone()),
                    test_id: Some(Arc::from("imui-tab-shortcut.root")),
                    ..Default::default()
                },
                |tabs| {
                    tabs.begin_tab_item_with_options(
                        "scene",
                        "Scene",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-shortcut.scene")),
                            panel_test_id: Some(Arc::from("imui-tab-shortcut.scene.panel")),
                            activate_shortcut: Some(shortcut),
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
                            test_id: Some(Arc::from("imui-tab-shortcut.inspector")),
                            panel_test_id: Some(Arc::from("imui-tab-shortcut.inspector.panel")),
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
        "imui-tab-shortcut",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("inspector"));

    let inspector_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.inspector",
    );
    ui.set_focus(Some(inspector_node));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-shortcut",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("inspector"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.inspector.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.scene.panel",
    ));

    let scene_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.scene",
    );
    ui.set_focus(Some(scene_node));
    assert_eq!(ui.focus(), Some(scene_node));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-shortcut",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("scene"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.scene.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.inspector.panel",
    ));
}
