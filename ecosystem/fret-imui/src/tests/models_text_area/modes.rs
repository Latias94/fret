use super::*;

#[test]
fn textarea_read_only_blocks_text_input_and_keeps_changed_false() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::from("locked\narea"));
    let changed = Rc::new(Cell::new(false));

    let changed_out = changed.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-read-only",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            read_only: true,
                            test_id: Some(Arc::from("imui-textarea-read-only")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );
    assert!(!changed.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "!");

    app.advance_frame();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-read-only",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            read_only: true,
                            test_id: Some(Arc::from("imui-textarea-read-only")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    assert!(!changed.get());
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some("locked\narea")
    );
}

#[test]
fn textarea_tab_key_does_not_insert_by_default() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let changed = Rc::new(Cell::new(false));

    let changed_out = changed.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-tab-default",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            test_id: Some(Arc::from("imui-textarea-tab-default")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Tab,
        Modifiers::default(),
    );

    app.advance_frame();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-tab-default",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            test_id: Some(Arc::from("imui-textarea-tab-default")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    assert!(!changed.get());
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some(""));
}

#[test]
fn textarea_allow_tab_input_inserts_tab_and_reports_changed() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let changed = Rc::new(Cell::new(false));

    let changed_out = changed.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-tab-allowed",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            allow_tab_input: true,
                            test_id: Some(Arc::from("imui-textarea-tab-allowed")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Tab,
        Modifiers::default(),
    );

    app.advance_frame();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-tab-allowed",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            allow_tab_input: true,
                            test_id: Some(Arc::from("imui-textarea-tab-allowed")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    assert!(changed.get());
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some("\t"));
}
