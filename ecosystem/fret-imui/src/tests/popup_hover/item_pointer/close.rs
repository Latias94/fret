use super::*;

fn hit_or_ancestor_test_id(
    snap: &fret_core::SemanticsSnapshot,
    mut id: fret_core::NodeId,
) -> Option<&str> {
    loop {
        let node = snap.nodes.iter().find(|n| n.id == id)?;
        if let Some(test_id) = node.test_id.as_deref() {
            return Some(test_id);
        }
        id = node.parent?;
    }
}

#[test]
fn context_menu_popup_item_click_closes_popup() {
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

    let open = Rc::new(Cell::new(false));
    let open_out = open.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(120.0), Px(60.0)),
                        ..Default::default()
                    },
                    |ui| {
                        let open_model = ui.popup_open_model("ctx");
                        ui.menu_item_with_options(
                            "Close",
                            MenuItemOptions {
                                close_popup: Some(open_model),
                                test_id: Some(Arc::from("imui-popup-ctx-item-close")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(!open.get());

    let at = first_child_point(&ui, root);
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    ui.request_semantics_snapshot();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(120.0), Px(60.0)),
                        ..Default::default()
                    },
                    |ui| {
                        let open_model = ui.popup_open_model("ctx");
                        ui.menu_item_with_options(
                            "Close",
                            MenuItemOptions {
                                close_popup: Some(open_model),
                                test_id: Some(Arc::from("imui-popup-ctx-item-close")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(open.get());

    let item_bounds = bounds_for_test_id(&ui, "imui-popup-ctx-item-close");
    let click_point = Point::new(
        Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
        Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
    );
    let hit = ui.debug_hit_test(click_point).hit.expect("hit node");
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let hit_test_id = hit_or_ancestor_test_id(snap, hit);
    assert_eq!(
        hit_test_id,
        Some("imui-popup-ctx-item-close"),
        "expected click to hit the menu item pressable or one of its descendants"
    );

    click_at(&mut ui, &mut app, &mut services, click_point);

    app.advance_frame();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |_ui| {}));
            })
        },
    );
    assert!(!open.get());
}
