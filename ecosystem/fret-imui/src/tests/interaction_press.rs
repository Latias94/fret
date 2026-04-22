use super::*;

#[test]
fn click_sets_clicked_true_once() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let clicked = Rc::new(Cell::new(false));
    let clicked_out = clicked.clone();
    let button_id_frame1 = Rc::new(Cell::new(None));
    let button_id_frame1_out = button_id_frame1.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-click-once",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                button_id_frame1_out.set(resp.id);
                clicked_out.set(resp.clicked());
            })
        },
    );
    assert!(!clicked.get());

    let at = first_child_point(&ui, root);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    app.advance_frame();
    let clicked_out = clicked.clone();
    let button_id_frame2 = Rc::new(Cell::new(None));
    let button_id_frame2_out = button_id_frame2.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-click-once",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                button_id_frame2_out.set(resp.id);
                clicked_out.set(resp.clicked());
            })
        },
    );
    if std::env::var_os("FRET_DEBUG_IMUI_CLICK_ONCE").is_some() {
        eprintln!(
            "click_once: button_id_frame1={:?} button_id_frame2={:?}",
            button_id_frame1.get(),
            button_id_frame2.get()
        );
    }
    assert!(clicked.get());

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-click-once",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());
}
#[test]
fn button_lifecycle_edges_follow_press_session() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-lifecycle-edges",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
            })
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());

    let at = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-lifecycle-edges",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
            })
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());

    pointer_up_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-lifecycle-edges",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
            })
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
}
#[test]
fn menu_item_lifecycle_edges_follow_press_session() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(280.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-item-lifecycle-edges",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.menu_item_with_options(
                    "Open",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-menu-item-lifecycle-edges.item")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
            })
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());

    let item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-item-lifecycle-edges.item",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, item);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-item-lifecycle-edges",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.menu_item_with_options(
                    "Open",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-menu-item-lifecycle-edges.item")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
            })
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());

    pointer_up_at(&mut ui, &mut app, &mut services, item);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-item-lifecycle-edges",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.menu_item_with_options(
                    "Open",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-menu-item-lifecycle-edges.item")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
            })
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
}
#[test]
fn checkbox_lifecycle_reports_edit_and_deactivated_after_edit() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(false);
    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-lifecycle-edges",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.checkbox_model("Enabled", &model);
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
                after_edit_out.set(resp.deactivated_after_edit());
            })
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());

    let at = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, at);
    pointer_up_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-lifecycle-edges",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.checkbox_model("Enabled", &model);
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
                after_edit_out.set(resp.deactivated_after_edit());
            })
        },
    );
    assert!(activated.get());
    assert!(deactivated.get());
    assert!(edited.get());
    assert!(after_edit.get());
}
#[test]
fn right_click_sets_context_menu_requested_true_once() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let requested = Rc::new(Cell::new(false));
    let secondary_clicked = Rc::new(Cell::new(false));
    let requested_out = requested.clone();
    let secondary_clicked_out = secondary_clicked.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-right-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                requested_out.set(resp.context_menu_requested());
                secondary_clicked_out.set(resp.secondary_clicked());
            })
        },
    );
    assert!(!requested.get());
    assert!(!secondary_clicked.get());

    let at = first_child_point(&ui, root);
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let requested_out = requested.clone();
    let secondary_clicked_out = secondary_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-right-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                requested_out.set(resp.context_menu_requested());
                secondary_clicked_out.set(resp.secondary_clicked());
            })
        },
    );
    assert!(requested.get());
    assert!(secondary_clicked.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let secondary_clicked_out = secondary_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-right-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                requested_out.set(resp.context_menu_requested());
                secondary_clicked_out.set(resp.secondary_clicked());
            })
        },
    );
    assert!(!requested.get());
    assert!(!secondary_clicked.get());
}
#[test]
fn double_click_sets_double_clicked_true_once() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let double_clicked = Rc::new(Cell::new(false));
    let double_clicked_out = double_clicked.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-double-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                double_clicked_out.set(ui.button("OK").double_clicked());
            })
        },
    );
    assert!(!double_clicked.get());

    let at = first_child_point(&ui, root);
    double_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let double_clicked_out = double_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-double-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                double_clicked_out.set(ui.button("OK").double_clicked());
            })
        },
    );
    assert!(double_clicked.get());

    app.advance_frame();
    let double_clicked_out = double_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-double-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                double_clicked_out.set(ui.button("OK").double_clicked());
            })
        },
    );
    assert!(!double_clicked.get());
}
#[test]
fn shift_f10_sets_context_menu_requested_true_once() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let requested = Rc::new(Cell::new(false));
    let requested_out = requested.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(ui.button("OK").context_menu_requested());
            })
        },
    );
    assert!(!requested.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(ui.button("OK").context_menu_requested());
            })
        },
    );
    assert!(!requested.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::F10,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(ui.button("OK").context_menu_requested());
            })
        },
    );
    assert!(requested.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(ui.button("OK").context_menu_requested());
            })
        },
    );
    assert!(!requested.get());
}
#[allow(dead_code)]
#[test]
fn holding_press_does_not_repeat_clicked() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let clicked = Rc::new(Cell::new(false));
    let clicked_out = clicked.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hold-press",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());

    let at = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hold-press",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());

    pointer_up_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hold-press",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(clicked.get());

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hold-press",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());
}
#[test]
fn long_press_sets_long_pressed_true_once_and_reports_holding() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let long_pressed = Rc::new(Cell::new(false));
    let holding = Rc::new(Cell::new(false));

    let long_pressed_out = long_pressed.clone();
    let holding_out = holding.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-long-press-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                long_pressed_out.set(resp.long_pressed());
                holding_out.set(resp.press_holding());
            })
        },
    );
    assert!(!long_pressed.get());
    assert!(!holding.get());

    let at = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, at);
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0);

    app.advance_frame();
    let long_pressed_out = long_pressed.clone();
    let holding_out = holding.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-long-press-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                long_pressed_out.set(resp.long_pressed());
                holding_out.set(resp.press_holding());
            })
        },
    );

    assert!(long_pressed.get());
    assert!(holding.get());

    app.advance_frame();
    let long_pressed_out = long_pressed.clone();
    let holding_out = holding.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-long-press-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                long_pressed_out.set(resp.long_pressed());
                holding_out.set(resp.press_holding());
            })
        },
    );
    assert!(!long_pressed.get());
    assert!(holding.get());

    pointer_up_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let long_pressed_out = long_pressed.clone();
    let holding_out = holding.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-long-press-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                long_pressed_out.set(resp.long_pressed());
                holding_out.set(resp.press_holding());
            })
        },
    );
    assert!(!long_pressed.get());
    assert!(!holding.get());
}
