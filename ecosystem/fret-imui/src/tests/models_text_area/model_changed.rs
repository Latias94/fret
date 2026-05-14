use super::*;

#[test]
fn textarea_model_reports_changed_once_after_text_input() {
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
        "imui-textarea",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.textarea_model(&model).changed());
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
    text_input_event(&mut ui, &mut app, &mut services, "line-1\nline-2");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.textarea_model(&model).changed());
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
    assert_eq!(text.borrow().as_str(), "line-1\nline-2");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.textarea_model(&model).changed());
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
    assert_eq!(text.borrow().as_str(), "line-1\nline-2");
}
