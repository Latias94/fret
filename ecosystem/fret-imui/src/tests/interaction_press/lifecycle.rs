use super::*;

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
