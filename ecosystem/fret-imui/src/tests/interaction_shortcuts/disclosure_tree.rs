use super::*;

#[test]
fn checkbox_activate_shortcut_preserves_shift_f10_context_menu_request() {
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
        "imui-checkbox-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.checkbox_model_with_options(
                        "Enabled",
                        &model,
                        fret_ui_kit::imui::CheckboxOptions {
                            test_id: Some(Arc::from("imui-checkbox-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
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
        "imui-checkbox-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.checkbox_model_with_options(
                        "Enabled",
                        &model,
                        fret_ui_kit::imui::CheckboxOptions {
                            test_id: Some(Arc::from("imui-checkbox-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
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
        "imui-checkbox-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.checkbox_model_with_options(
                        "Enabled",
                        &model,
                        fret_ui_kit::imui::CheckboxOptions {
                            test_id: Some(Arc::from("imui-checkbox-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
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
        "imui-checkbox-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.checkbox_model_with_options(
                        "Enabled",
                        &model,
                        fret_ui_kit::imui::CheckboxOptions {
                            test_id: Some(Arc::from("imui-checkbox-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(!requested.get());
}
#[test]
fn collapsing_header_activate_shortcut_is_scoped_to_focused_trigger() {
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

    let target_open = app.models_mut().insert(false);
    let other_open = app.models_mut().insert(false);
    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let target_state = Rc::new(Cell::new(false));
    let other_state = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_state_out: &Rc<Cell<bool>>,
                  other_state_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.collapsing_header_with_options(
                    "target",
                    "Target",
                    fret_ui_kit::imui::CollapsingHeaderOptions {
                        open: Some(target_open.clone()),
                        header_test_id: Some(Arc::from("imui-collapsing-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                    |_ui| {},
                );
                let _ = ui.collapsing_header_with_options(
                    "other",
                    "Other",
                    fret_ui_kit::imui::CollapsingHeaderOptions {
                        open: Some(other_open.clone()),
                        header_test_id: Some(Arc::from("imui-collapsing-shortcut.other")),
                        ..Default::default()
                    },
                    |_ui| {},
                );
            });

            target_state_out.set(
                ui.cx_mut()
                    .app
                    .models()
                    .get_copied(&target_open)
                    .unwrap_or_default(),
            );
            other_state_out.set(
                ui.cx_mut()
                    .app
                    .models()
                    .get_copied(&other_open)
                    .unwrap_or_default(),
            );
        })
    };

    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(!target_state.get());
    assert!(!other_state.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(
        !target_state.get() && !other_state.get(),
        "expected unfocused disclosure shortcut to do nothing"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collapsing-shortcut.other",
    );
    click_at(&mut ui, &mut app, &mut services, other);

    app.advance_frame();
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(!target_state.get());
    assert!(other_state.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(
        !target_state.get() && other_state.get(),
        "expected shortcut on another disclosure trigger to leave target untouched"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collapsing-shortcut.target",
    );
    click_at(&mut ui, &mut app, &mut services, target);

    app.advance_frame();
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(target_state.get());
    assert!(other_state.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(!target_state.get());
    assert!(other_state.get());
}
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
                    .trigger
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
                    .trigger
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
                    .trigger
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
                    .trigger
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(!requested.get());
}
#[test]
fn tree_node_children_stack_vertically_inside_open_parents() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tree-node-vertical-stacking",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.tree_node_with_options(
                    "scene",
                    "Scene",
                    fret_ui_kit::imui::TreeNodeOptions {
                        default_open: true,
                        test_id: Some(Arc::from("imui-tree-node-stack.scene")),
                        content_test_id: Some(Arc::from("imui-tree-node-stack.scene.content")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.tree_node_with_options(
                            "geometry",
                            "Geometry",
                            fret_ui_kit::imui::TreeNodeOptions {
                                default_open: true,
                                level: 2,
                                test_id: Some(Arc::from("imui-tree-node-stack.geometry")),
                                content_test_id: Some(Arc::from(
                                    "imui-tree-node-stack.geometry.content",
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                let _ = ui.tree_node_with_options(
                                    "cube",
                                    "Cube",
                                    fret_ui_kit::imui::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from("imui-tree-node-stack.cube")),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                                let _ = ui.tree_node_with_options(
                                    "key-light",
                                    "Key light",
                                    fret_ui_kit::imui::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from("imui-tree-node-stack.key-light")),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                            },
                        );
                        let _ = ui.tree_node_with_options(
                            "postfx",
                            "Post FX",
                            fret_ui_kit::imui::TreeNodeOptions {
                                leaf: true,
                                level: 2,
                                test_id: Some(Arc::from("imui-tree-node-stack.postfx")),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let geometry_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.geometry");
    let postfx_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.postfx");
    let cube_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.cube");
    let key_light_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.key-light");

    assert!(
        postfx_bounds.origin.y.0 >= geometry_bounds.origin.y.0 + geometry_bounds.size.height.0,
        "expected Post FX to land below Geometry, got geometry={geometry_bounds:?} postfx={postfx_bounds:?}"
    );
    assert!(
        key_light_bounds.origin.y.0 >= cube_bounds.origin.y.0 + cube_bounds.size.height.0,
        "expected Key light to land below Cube, got cube={cube_bounds:?} key_light={key_light_bounds:?}"
    );
}
