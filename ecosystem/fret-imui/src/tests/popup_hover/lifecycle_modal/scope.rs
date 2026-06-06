use super::*;

#[test]
fn drop_popup_scope_closes_and_forgets_internal_state() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(None::<Arc<str>>);
    let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];
    let popup_scope_id: Arc<str> = Arc::from("imui-drop-popup-scope");

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drop-popup-scope",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    popup_scope_id.as_ref(),
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-drop-popup-trigger")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-drop-popup-trigger",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drop-popup-scope",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    popup_scope_id.as_ref(),
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-drop-popup-trigger")),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-drop-popup-scope",
    ));

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drop-popup-scope",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.drop_popup_scope(popup_scope_id.as_ref());
                let _ = ui.combo_model_with_options(
                    popup_scope_id.as_ref(),
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-drop-popup-trigger")),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-drop-popup-scope",
    ));
}
