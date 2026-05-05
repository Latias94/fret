use super::*;

#[test]
fn input_text_model_reports_changed_once_after_text_input() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());

    let changed = Rc::new(Cell::new(false));
    let text = Rc::new(RefCell::new(String::new()));

    let changed_out = changed.clone();
    let text_out = text.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.input_text_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(!changed.get());
    assert!(text.borrow().is_empty());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "hello");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.input_text_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(changed.get());
    assert_eq!(text.borrow().as_str(), "hello");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.input_text_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(!changed.get());
    assert_eq!(text.borrow().as_str(), "hello");
}
