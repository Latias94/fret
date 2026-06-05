use super::*;

#[test]
fn tree_node_activate_shortcut_preserves_shift_f10_context_menu_request() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(260.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let requested = Rc::new(Cell::new(false));
    let requested_out = requested.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tree-node-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.tree_node_with_options(
                        "node",
                        "Node",
                        fret_ui_kit::imui::TreeNodeOptions {
                            leaf: true,
                            test_id: Some(Arc::from("imui-tree-node-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |_ui| {},
                    )
                    .response()
                    .context_menu_requested(),
                );
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
        "imui-tree-node-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.tree_node_with_options(
                        "node",
                        "Node",
                        fret_ui_kit::imui::TreeNodeOptions {
                            leaf: true,
                            test_id: Some(Arc::from("imui-tree-node-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |_ui| {},
                    )
                    .response()
                    .context_menu_requested(),
                );
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
        "imui-tree-node-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.tree_node_with_options(
                        "node",
                        "Node",
                        fret_ui_kit::imui::TreeNodeOptions {
                            leaf: true,
                            test_id: Some(Arc::from("imui-tree-node-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |_ui| {},
                    )
                    .response()
                    .context_menu_requested(),
                );
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
        "imui-tree-node-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.tree_node_with_options(
                        "node",
                        "Node",
                        fret_ui_kit::imui::TreeNodeOptions {
                            leaf: true,
                            test_id: Some(Arc::from("imui-tree-node-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |_ui| {},
                    )
                    .response()
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(!requested.get());
}
