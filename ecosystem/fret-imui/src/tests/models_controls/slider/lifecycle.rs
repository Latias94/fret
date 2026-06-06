use super::*;

#[test]
fn slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(0.0_f32);
    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));
    let value = Rc::new(Cell::new(0.0_f32));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  activated_out: &Rc<Cell<bool>>,
                  deactivated_out: &Rc<Cell<bool>>,
                  edited_out: &Rc<Cell<bool>>,
                  after_edit_out: &Rc<Cell<bool>>,
                  value_out: &Rc<Cell<f32>>| {
        crate::imui_raw(cx, |ui| {
            let resp = ui.slider_f32_model_with_options(
                "Volume",
                &model,
                SliderOptions {
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    test_id: Some(Arc::from("imui-slider-lifecycle")),
                    ..Default::default()
                },
            );
            activated_out.set(resp.activated());
            deactivated_out.set(resp.deactivated());
            edited_out.set(resp.edited());
            after_edit_out.set(resp.deactivated_after_edit());
            let now = ui
                .cx_mut()
                .app
                .models()
                .get_copied(&model)
                .unwrap_or_default();
            value_out.set(now);
        })
    };

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let value_out = value.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &value_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert_eq!(value.get(), 0.0);

    let slider_node = ui.children(root)[0];
    let slider_bounds = ui.debug_node_bounds(slider_node).expect("slider bounds");
    let start = Point::new(
        Px(slider_bounds.origin.x.0 + slider_bounds.size.width.0 * 0.1),
        Px(slider_bounds.origin.y.0 + slider_bounds.size.height.0 * 0.5),
    );
    let drag = Point::new(
        Px(slider_bounds.origin.x.0 + slider_bounds.size.width.0 * 0.9),
        Px(slider_bounds.origin.y.0 + slider_bounds.size.height.0 * 0.5),
    );
    pointer_down_at(&mut ui, &mut app, &mut services, start);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let value_out = value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &value_out,
            )
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(edited.get());
    assert!(!after_edit.get());
    assert!(value.get() > 0.0);

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        drag,
        MouseButtons {
            left: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let value_out = value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &value_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(edited.get());
    assert!(!after_edit.get());
    assert!(value.get() >= 70.0);

    pointer_up_at(&mut ui, &mut app, &mut services, drag);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let value_out = value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &value_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
    assert!(after_edit.get());
    assert!(value.get() > 0.0);
}
