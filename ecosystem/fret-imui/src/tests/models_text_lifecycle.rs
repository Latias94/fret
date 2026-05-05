use super::*;
use fret_ui_kit::imui::{ButtonOptions, InputTextOptions};

#[test]
fn input_text_focus_keeps_control_bounds_stable() {
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

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-stable-bounds")),
                        ..Default::default()
                    },
                );
                let _ = ui.button_with_options(
                    "Sibling",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-input-text-stable-bounds.sibling")),
                        ..Default::default()
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-stable-bounds",
        |cx| render(cx),
    );
    let input_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-stable-bounds",
    );
    let before = ui.debug_node_bounds(input_node).expect("input bounds");

    let at = Point::new(
        Px(before.origin.x.0 + before.size.width.0 * 0.5),
        Px(before.origin.y.0 + before.size.height.0 * 0.5),
    );
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-stable-bounds",
        |cx| render(cx),
    );
    let input_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-stable-bounds",
    );
    let after = ui.debug_node_bounds(input_node).expect("input bounds");

    assert_eq!(after.origin, before.origin);
    assert_eq!(after.size, before.size);
}

#[test]
fn input_text_lifecycle_tracks_focus_edit_and_blur_edges() {
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
    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));
    let text = Rc::new(RefCell::new(String::new()));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  activated_out: &Rc<Cell<bool>>,
                  deactivated_out: &Rc<Cell<bool>>,
                  edited_out: &Rc<Cell<bool>>,
                  after_edit_out: &Rc<Cell<bool>>,
                  text_out: &Rc<RefCell<String>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let resp = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-lifecycle")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
                after_edit_out.set(resp.deactivated_after_edit());

                let _ = ui.button_with_options(
                    "Blur target",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-input-text-lifecycle.blur-target")),
                        ..Default::default()
                    },
                );
            });

            let current = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&model)
                .unwrap_or_default();
            text_out.replace(current);
        })
    };

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    let input = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-lifecycle",
    );
    click_at(&mut ui, &mut app, &mut services, input);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    text_input_event(&mut ui, &mut app, &mut services, "hello");

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(edited.get());
    assert!(!after_edit.get());
    assert_eq!(text.borrow().as_str(), "hello");

    let blur_target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-lifecycle.blur-target",
    );
    click_at(&mut ui, &mut app, &mut services, blur_target);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
    assert!(after_edit.get());
    assert_eq!(text.borrow().as_str(), "hello");
}
