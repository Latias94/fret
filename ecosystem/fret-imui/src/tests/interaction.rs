use super::*;

use fret_runtime::{
    CommandId, CommandMeta, DefaultKeybinding, Effect, KeyChord, PlatformFilter,
    WindowCommandEnabledService,
};
use fret_ui_kit::imui::ImUiMultiSelectState;
use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost_with_options,
    publish_cross_window_drag_preview_ghost_with_options, render_cross_window_drag_preview_ghosts,
};
use fret_ui_kit::recipes::imui_sortable::{
    SortableInsertionSide, reorder_vec_by_key, sortable_row,
};

#[derive(Clone)]
struct TestDragPayload {
    label: Arc<str>,
}

#[derive(Clone, PartialEq, Eq)]
struct TestSortableItem {
    id: Arc<str>,
    label: Arc<str>,
}

#[derive(Clone)]
struct TestSortablePayload {
    id: Arc<str>,
    label: Arc<str>,
}

fn test_sortable_items() -> Vec<TestSortableItem> {
    vec![
        TestSortableItem {
            id: Arc::from("camera"),
            label: Arc::from("Camera"),
        },
        TestSortableItem {
            id: Arc::from("cube"),
            label: Arc::from("Cube"),
        },
        TestSortableItem {
            id: Arc::from("key-light"),
            label: Arc::from("Key light"),
        },
    ]
}

fn test_sortable_order_line(items: &[TestSortableItem]) -> String {
    items
        .iter()
        .map(|item| item.label.as_ref())
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn render_test_sortable_rows(
    items: &Rc<RefCell<Vec<TestSortableItem>>>,
    preview_status: &Rc<RefCell<String>>,
    delivered_status: &Rc<RefCell<String>>,
    order_status: &Rc<RefCell<String>>,
    delivered_flag: &Rc<Cell<bool>>,
) -> impl FnOnce(&mut ElementContext<'_, TestHost>) -> crate::Elements + use<> {
    let items = items.clone();
    let preview_status = preview_status.clone();
    let delivered_status = delivered_status.clone();
    let order_status = order_status.clone();
    let delivered_flag = delivered_flag.clone();

    move |cx| {
        crate::imui(cx, |ui| {
            let snapshot = items.borrow().clone();
            let mut pending_reorder: Option<(
                Arc<str>,
                Arc<str>,
                Arc<str>,
                Arc<str>,
                SortableInsertionSide,
            )> = None;
            let mut preview = String::new();

            ui.vertical(|ui| {
                for item in &snapshot {
                    let row = ui.button_with_options(
                        item.label.clone(),
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from(format!("imui-sortable-row.{}", item.id))),
                            ..Default::default()
                        },
                    );
                    let payload = TestSortablePayload {
                        id: item.id.clone(),
                        label: item.label.clone(),
                    };
                    let sortable = sortable_row(ui, row, payload);

                    if let Some(signal) = sortable.delivered_reorder() {
                        let dragged = signal.payload();
                        if dragged.id != item.id {
                            pending_reorder = Some((
                                dragged.id.clone(),
                                dragged.label.clone(),
                                item.id.clone(),
                                item.label.clone(),
                                signal.side(),
                            ));
                        }
                    } else if let Some(signal) = sortable.preview_reorder() {
                        let dragged = signal.payload();
                        let side = signal.side();
                        if dragged.id != item.id {
                            preview = format!(
                                "Preview: move {} {} {}",
                                dragged.label,
                                side.label(),
                                item.label
                            );
                        }
                    }
                }
            });

            let mut delivered_message = String::new();
            let mut delivered = false;
            if let Some((active_id, active_label, over_id, over_label, side)) = pending_reorder {
                delivered = reorder_vec_by_key(
                    &mut items.borrow_mut(),
                    active_id.as_ref(),
                    over_id.as_ref(),
                    side,
                    |item| item.id.as_ref(),
                );
                if delivered {
                    delivered_message =
                        format!("Moved {} {} {}", active_label, side.label(), over_label);
                }
            }

            preview_status.replace(preview);
            delivered_status.replace(delivered_message);
            delivered_flag.set(delivered);
            order_status.replace(test_sortable_order_line(&items.borrow()));
        })
    }
}

#[test]
fn multi_selectable_supports_plain_toggle_and_range_clicks() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let items = Arc::<[Arc<str>]>::from(vec![
        Arc::from("Alpha"),
        Arc::from("Beta"),
        Arc::from("Gamma"),
        Arc::from("Delta"),
    ]);
    let selection_model = app
        .models_mut()
        .insert(ImUiMultiSelectState::<Arc<str>>::default());
    let selected = Rc::new(RefCell::new(Vec::<Arc<str>>::new()));
    let anchor = Rc::new(RefCell::new(None::<Arc<str>>));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  selected_out: &Rc<RefCell<Vec<Arc<str>>>>,
                  anchor_out: &Rc<RefCell<Option<Arc<str>>>>| {
        crate::imui(cx, |ui| {
            ui.vertical(|ui| {
                for (index, item) in items.iter().enumerate() {
                    let _ = ui.multi_selectable_with_options(
                        item.clone(),
                        &selection_model,
                        items.as_ref(),
                        item.clone(),
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from(format!("imui-multi-select.option.{index}"))),
                            ..Default::default()
                        },
                    );
                }
            });

            let state = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&selection_model)
                .unwrap_or_default();
            selected_out.replace(state.selected.clone());
            anchor_out.replace(state.anchor.clone());
        })
    };

    let selected_out = selected.clone();
    let anchor_out = anchor.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-multi-select",
        |cx| render(cx, &selected_out, &anchor_out),
    );
    assert!(selected.borrow().is_empty());
    assert!(anchor.borrow().is_none());

    let beta = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-multi-select.option.1",
    );
    click_at(&mut ui, &mut app, &mut services, beta);

    app.advance_frame();
    let selected_out = selected.clone();
    let anchor_out = anchor.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-multi-select",
        |cx| render(cx, &selected_out, &anchor_out),
    );
    assert_eq!(selected.borrow().as_slice(), &[Arc::<str>::from("Beta")]);
    assert_eq!(anchor.borrow().as_deref(), Some("Beta"));

    let delta = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-multi-select.option.3",
    );
    click_at_with_modifiers(
        &mut ui,
        &mut app,
        &mut services,
        delta,
        Modifiers {
            meta: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let selected_out = selected.clone();
    let anchor_out = anchor.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-multi-select",
        |cx| render(cx, &selected_out, &anchor_out),
    );
    assert_eq!(
        selected.borrow().as_slice(),
        &[Arc::<str>::from("Beta"), Arc::<str>::from("Delta")]
    );
    assert_eq!(anchor.borrow().as_deref(), Some("Delta"));

    let alpha = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-multi-select.option.0",
    );
    click_at_with_modifiers(
        &mut ui,
        &mut app,
        &mut services,
        alpha,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let selected_out = selected.clone();
    let anchor_out = anchor.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-multi-select",
        |cx| render(cx, &selected_out, &anchor_out),
    );
    assert_eq!(
        selected.borrow().as_slice(),
        &[
            Arc::<str>::from("Alpha"),
            Arc::<str>::from("Beta"),
            Arc::<str>::from("Gamma"),
            Arc::<str>::from("Delta"),
        ]
    );
    assert_eq!(anchor.borrow().as_deref(), Some("Delta"));
}

#[test]
fn menu_item_command_uses_command_metadata_shortcut_and_gating() {
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

    let command = CommandId::from("test.open-file");
    app.commands_mut().register(
        command.clone(),
        CommandMeta::new("Open File").with_default_keybindings([DefaultKeybinding::single(
            PlatformFilter::All,
            KeyChord::new(
                KeyCode::KeyO,
                Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
            ),
        )]),
    );

    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui(cx, |ui| {
            ui.menu_item_command_with_options(
                command.clone(),
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-menu-command")),
                    ..Default::default()
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-command",
        |cx| render(cx, &command),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-menu-command"))
        .expect("menu item semantics node");
    assert_eq!(node.label.as_deref(), Some("Open File"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-command.shortcut",
    ));

    let click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-command",
    );
    click_at(&mut ui, &mut app, &mut services, click_point);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.set_global(WindowCommandEnabledService::default());
    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        svc.set_enabled(window, command.clone(), false);
    });
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-command",
        |cx| render(cx, &command),
    );

    let disabled_click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-command",
    );
    click_at(&mut ui, &mut app, &mut services, disabled_click_point);
    assert!(!app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));
}

#[test]
fn button_command_uses_command_metadata_and_gating() {
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

    let command = CommandId::from("test.open-project");
    app.commands_mut()
        .register(command.clone(), CommandMeta::new("Open Project"));

    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui(cx, |ui| {
            ui.button_command_with_options(
                command.clone(),
                fret_ui_kit::imui::ButtonOptions {
                    test_id: Some(Arc::from("imui-button-command")),
                    ..Default::default()
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-command",
        |cx| render(cx, &command),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-button-command"))
        .expect("button semantics node");
    assert_eq!(node.label.as_deref(), Some("Open Project"));
    assert_eq!(node.role, fret_core::SemanticsRole::Button);

    let click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-command",
    );
    click_at(&mut ui, &mut app, &mut services, click_point);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.set_global(WindowCommandEnabledService::default());
    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        svc.set_enabled(window, command.clone(), false);
    });
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-command",
        |cx| render(cx, &command),
    );

    let disabled_click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-command",
    );
    click_at(&mut ui, &mut app, &mut services, disabled_click_point);
    assert!(!app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));
}

#[test]
fn button_activate_shortcut_is_scoped_to_focused_button() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.shortcut.focused");
    app.commands_mut()
        .register(command.clone(), CommandMeta::new("Focused Shortcut"));

    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui(cx, |ui| {
            ui.vertical(|ui| {
                ui.button_command_with_options(
                    command.clone(),
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-button-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                );
                ui.button_with_options(
                    "Other",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-button-shortcut.other")),
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
        "imui-button-shortcut",
        |cx| render(cx, &command),
    );

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
    assert!(
        app.effects.is_empty(),
        "expected unfocused shortcut to stay local to the button"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-shortcut.other",
    );
    click_at(&mut ui, &mut app, &mut services, other);
    app.effects.clear();

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-shortcut",
        |cx| render(cx, &command),
    );
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
    assert!(
        app.effects.is_empty(),
        "expected shortcut on another focused button to do nothing"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-shortcut.target",
    );
    click_at(&mut ui, &mut app, &mut services, target);
    app.effects.clear();

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-shortcut",
        |cx| render(cx, &command),
    );
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
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));
}

#[test]
fn selectable_activate_shortcut_is_scoped_to_focused_item() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
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
    let target_clicked = Rc::new(Cell::new(false));
    let other_clicked = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_clicked_out: &Rc<Cell<bool>>,
                  other_clicked_out: &Rc<Cell<bool>>| {
        crate::imui(cx, |ui| {
            ui.vertical(|ui| {
                target_clicked_out.set(
                    ui.selectable_with_options(
                        "Target",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut.target")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
                    .clicked(),
                );
                other_clicked_out.set(
                    ui.selectable_with_options(
                        "Other",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut.other")),
                            ..Default::default()
                        },
                    )
                    .clicked(),
                );
            });
        })
    };

    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());
    assert!(!other_clicked.get());

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
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(
        !target_clicked.get() && !other_clicked.get(),
        "expected unfocused shortcut to stay local to the selectable"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-selectable-shortcut.other",
    );
    click_at(&mut ui, &mut app, &mut services, other);

    app.advance_frame();
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());

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
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(
        !target_clicked.get() && !other_clicked.get(),
        "expected shortcut on another focused selectable to do nothing"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-selectable-shortcut.target",
    );
    click_at(&mut ui, &mut app, &mut services, target);

    app.advance_frame();
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(target_clicked.get());
    assert!(!other_clicked.get());

    app.advance_frame();
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());
    assert!(!other_clicked.get());

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
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(target_clicked.get());
    assert!(!other_clicked.get());
}

#[test]
fn selectable_activate_shortcut_preserves_popup_arrow_nav() {
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

    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let popup_id = "imui-selectable-shortcut-popup";

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui(cx, |ui| {
            let popup_open = ui.popup_open_model(popup_id);
            let is_open = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&popup_open)
                .unwrap_or(false);
            if !is_open {
                ui.open_popup_at(
                    popup_id,
                    Rect::new(Point::new(Px(48.0), Px(48.0)), Size::new(Px(1.0), Px(1.0))),
                );
            }

            assert!(ui.begin_popup_menu_with_options(
                popup_id,
                None,
                fret_ui_kit::imui::PopupMenuOptions {
                    estimated_size: Size::new(Px(160.0), Px(90.0)),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.selectable_with_options(
                        "Alpha",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut-popup.alpha")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    let _ = ui.selectable_with_options(
                        "Beta",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut-popup.beta")),
                            ..Default::default()
                        },
                    );
                },
            ));
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    let alpha = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-selectable-shortcut-popup.alpha",
    );
    click_at(&mut ui, &mut app, &mut services, alpha);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = ui
        .focus()
        .and_then(|focus| {
            snap.nodes
                .iter()
                .find(|node| node.id == focus)
                .and_then(|node| node.test_id.as_deref())
        })
        .map(str::to_owned);
    assert_eq!(
        focused_test_id.as_deref(),
        Some("imui-selectable-shortcut-popup.alpha")
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = ui
        .focus()
        .and_then(|focus| {
            snap.nodes
                .iter()
                .find(|node| node.id == focus)
                .and_then(|node| node.test_id.as_deref())
        })
        .map(str::to_owned);
    assert_eq!(
        focused_test_id.as_deref(),
        Some("imui-selectable-shortcut-popup.beta")
    );
}

#[test]
fn begin_menu_helper_toggles_popup_and_closes_after_command_activate() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.begin-menu.open");
    app.commands_mut().register(
        command.clone(),
        CommandMeta::new("Open File").with_default_keybindings([DefaultKeybinding::single(
            PlatformFilter::All,
            KeyChord::new(
                KeyCode::KeyO,
                Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
            ),
        )]),
    );

    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let open = ui.popup_open_model("file");
                            ui.menu_item_command_with_options(
                                command.clone(),
                                MenuItemOptions {
                                    close_popup: Some(open),
                                    test_id: Some(Arc::from("imui-begin-menu.file.open")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu",
        |cx| render(cx, &command),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    ));

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu",
        |cx| render(cx, &command),
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    ));

    let item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    );
    click_at(&mut ui, &mut app, &mut services, item);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu",
        |cx| render(cx, &command),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    ));
}

#[test]
fn begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.begin-submenu.open-recent");
    app.commands_mut().register(
        command.clone(),
        CommandMeta::new("Recent Project").with_default_keybindings([DefaultKeybinding::single(
            PlatformFilter::All,
            KeyChord::new(
                KeyCode::KeyR,
                Modifiers {
                    ctrl: true,
                    shift: true,
                    ..Default::default()
                },
            ),
        )]),
    );

    let file_open = Rc::new(Cell::new(false));
    let recent_open = Rc::new(Cell::new(false));
    let file_open_out = file_open.clone();
    let recent_open_out = recent_open.clone();
    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-submenu.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-submenu.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let recent_open = ui.popup_open_model("recent");
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from("imui-begin-submenu.file.recent")),
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.menu_item_command_with_options(
                                        command.clone(),
                                        MenuItemOptions {
                                            close_popup: Some(recent_open),
                                            test_id: Some(Arc::from(
                                                "imui-begin-submenu.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                },
            );

            let file_popup = ui.popup_open_model("file");
            let recent_popup = ui.popup_open_model("recent");
            file_open_out.set(
                ui.cx_mut()
                    .read_model(&file_popup, fret_ui::Invalidation::Paint, |_app, value| {
                        *value
                    })
                    .unwrap_or(false),
            );
            recent_open_out.set(
                ui.cx_mut()
                    .read_model(
                        &recent_popup,
                        fret_ui::Invalidation::Paint,
                        |_app, value| *value,
                    )
                    .unwrap_or(false),
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(!file_open.get());
    assert!(!recent_open.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    ));

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(file_open.get());
    assert!(!recent_open.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent",
    ));

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let recent_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-begin-submenu.file.recent"))
        .expect("recent submenu semantics node");
    assert_eq!(recent_node.role, fret_core::SemanticsRole::MenuItem);
    assert!(!recent_node.flags.expanded);

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent",
    );
    click_at(&mut ui, &mut app, &mut services, recent_trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(file_open.get());
    assert!(recent_open.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    ));

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(file_open.get(), "expected parent menu to remain open");
    assert!(recent_open.get(), "expected submenu to remain open");
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let recent_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-begin-submenu.file.recent"))
        .expect("recent submenu semantics node");
    assert!(recent_node.flags.expanded);

    let project_item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    );
    click_at(&mut ui, &mut app, &mut services, project_item);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    ));
}

#[test]
fn tab_bar_helper_switches_selected_panel_and_updates_selection_model() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let selected_model = app.models_mut().insert(Some(Arc::<str>::from("inspector")));
    let selected_out = Rc::new(RefCell::new(None::<Arc<str>>));
    let selected_out_render = selected_out.clone();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui(cx, |ui| {
            ui.tab_bar_with_options(
                "workspace",
                fret_ui_kit::imui::TabBarOptions {
                    selected: Some(selected_model.clone()),
                    test_id: Some(Arc::from("imui-tab-bar-interaction.root")),
                    ..Default::default()
                },
                |tabs| {
                    tabs.begin_tab_item_with_options(
                        "scene",
                        "Scene",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-bar-interaction.scene")),
                            panel_test_id: Some(Arc::from("imui-tab-bar-interaction.scene.panel")),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Scene Panel");
                        },
                    );
                    tabs.begin_tab_item_with_options(
                        "inspector",
                        "Inspector",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-bar-interaction.inspector")),
                            panel_test_id: Some(Arc::from(
                                "imui-tab-bar-interaction.inspector.panel",
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Inspector Panel");
                        },
                    );
                },
            );

            let selected = ui
                .cx_mut()
                .read_model(
                    &selected_model,
                    fret_ui::Invalidation::Paint,
                    |_app, value| value.clone(),
                )
                .unwrap_or(None);
            selected_out_render.replace(selected);
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-bar-interaction",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("inspector"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.inspector.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.scene.panel",
    ));

    let scene_tab = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.scene",
    );
    click_at(&mut ui, &mut app, &mut services, scene_tab);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-bar-interaction",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("scene"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.scene.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.inspector.panel",
    ));

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let scene_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar-interaction.scene"))
        .expect("scene tab semantics node");
    let inspector_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar-interaction.inspector"))
        .expect("inspector tab semantics node");
    assert!(scene_node.flags.selected);
    assert!(!inspector_node.flags.selected);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-bar-interaction",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("scene"));
}

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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
                requested_out.set(ui.button("OK").context_menu_requested());
            })
        },
    );
    assert!(!requested.get());
}

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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
        crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());
}

#[test]
fn drag_started_stopped_and_delta_are_consistent() {
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

    let started = Rc::new(Cell::new(false));
    let dragging = Rc::new(Cell::new(false));
    let stopped = Rc::new(Cell::new(false));
    let delta = Rc::new(Cell::new(Point::default()));

    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(!stopped.get());

    let start = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, start);

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(!stopped.get());

    // Move below the threshold.
    let p1 = Point::new(Px(start.x.0 + 2.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p1,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(!stopped.get());

    // Move past the threshold to start dragging (delta should be the frame delta, not the total).
    let p2 = Point::new(Px(start.x.0 + 6.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p2,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(started.get());
    assert!(dragging.get());
    assert!(!stopped.get());
    assert_eq!(delta.get(), Point::new(Px(4.0), Px(0.0)));

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, p2, false);

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(stopped.get());
}

#[test]
fn drag_threshold_metric_controls_drag_start() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    fret_ui::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = fret_ui::theme::ThemeConfig {
            name: "Test".to_string(),
            ..fret_ui::theme::ThemeConfig::default()
        };
        cfg.metrics
            .insert("component.imui.drag_threshold_px".to_string(), 7.0);
        theme.apply_config_patch(&cfg);
    });
    let mut services = FakeTextService::default();

    let started = Rc::new(Cell::new(false));
    let dragging = Rc::new(Cell::new(false));
    let delta = Rc::new(Cell::new(Point::default()));

    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let delta_out = delta.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-threshold-metric",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());

    let start = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, start);

    // Move below the configured threshold (7px).
    let p1 = Point::new(Px(start.x.0 + 6.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p1,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-threshold-metric",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());

    // Move past the threshold; delta should be the frame delta (8 - 6 = 2).
    let p2 = Point::new(Px(start.x.0 + 8.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p2,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-threshold-metric",
        |cx| {
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(started.get());
    assert!(dragging.get());
    assert_eq!(delta.get(), Point::new(Px(2.0), Px(0.0)));
}

#[test]
fn drag_drop_helper_previews_and_delivers_payload() {
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

    let source_active = Rc::new(Cell::new(false));
    let target_over = Rc::new(Cell::new(false));
    let delivered = Rc::new(Cell::new(false));
    let preview_label = Rc::new(RefCell::new(String::new()));
    let delivered_label = Rc::new(RefCell::new(String::new()));

    let source_active_out = source_active.clone();
    let target_over_out = target_over.clone();
    let delivered_out = delivered.clone();
    let preview_label_out = preview_label.clone();
    let delivered_label_out = delivered_label.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-drop-helper",
        |cx| {
            crate::imui(cx, |ui| {
                ui.horizontal(|ui| {
                    let source = ui.button_with_options(
                        "Asset",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-source")),
                            ..Default::default()
                        },
                    );
                    let source_state = ui.drag_source(
                        source,
                        TestDragPayload {
                            label: Arc::from("Stone"),
                        },
                    );
                    source_active_out.set(source_state.active());

                    let target = ui.button_with_options(
                        "Slot",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-target")),
                            ..Default::default()
                        },
                    );
                    let drop = ui.drop_target::<TestDragPayload>(target);
                    target_over_out.set(drop.over());
                    delivered_out.set(drop.delivered());
                    preview_label_out.replace(
                        drop.preview_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                    delivered_label_out.replace(
                        drop.delivered_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                });
            })
        },
    );

    assert!(!source_active.get());
    assert!(!target_over.get());
    assert!(!delivered.get());
    assert!(preview_label.borrow().is_empty());
    assert!(delivered_label.borrow().is_empty());

    let source_point =
        point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-dnd-source");
    let target_point =
        point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-dnd-target");

    pointer_down_at(&mut ui, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        target_point,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let source_active_out = source_active.clone();
    let target_over_out = target_over.clone();
    let delivered_out = delivered.clone();
    let preview_label_out = preview_label.clone();
    let delivered_label_out = delivered_label.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-drop-helper",
        |cx| {
            crate::imui(cx, |ui| {
                ui.horizontal(|ui| {
                    let source = ui.button_with_options(
                        "Asset",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-source")),
                            ..Default::default()
                        },
                    );
                    let source_state = ui.drag_source(
                        source,
                        TestDragPayload {
                            label: Arc::from("Stone"),
                        },
                    );
                    source_active_out.set(source_state.active());

                    let target = ui.button_with_options(
                        "Slot",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-target")),
                            ..Default::default()
                        },
                    );
                    let drop = ui.drop_target::<TestDragPayload>(target);
                    target_over_out.set(drop.over());
                    delivered_out.set(drop.delivered());
                    preview_label_out.replace(
                        drop.preview_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                    delivered_label_out.replace(
                        drop.delivered_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                });
            })
        },
    );

    assert!(source_active.get());
    assert!(target_over.get());
    assert!(!delivered.get());
    assert_eq!(preview_label.borrow().as_str(), "Stone");
    assert!(delivered_label.borrow().is_empty());

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, target_point, false);

    app.advance_frame();
    let source_active_out = source_active.clone();
    let target_over_out = target_over.clone();
    let delivered_out = delivered.clone();
    let preview_label_out = preview_label.clone();
    let delivered_label_out = delivered_label.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-drop-helper",
        |cx| {
            crate::imui(cx, |ui| {
                ui.horizontal(|ui| {
                    let source = ui.button_with_options(
                        "Asset",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-source")),
                            ..Default::default()
                        },
                    );
                    let source_state = ui.drag_source(
                        source,
                        TestDragPayload {
                            label: Arc::from("Stone"),
                        },
                    );
                    source_active_out.set(source_state.active());

                    let target = ui.button_with_options(
                        "Slot",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-target")),
                            ..Default::default()
                        },
                    );
                    let drop = ui.drop_target::<TestDragPayload>(target);
                    target_over_out.set(drop.over());
                    delivered_out.set(drop.delivered());
                    preview_label_out.replace(
                        drop.preview_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                    delivered_label_out.replace(
                        drop.delivered_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                });
            })
        },
    );

    assert!(!source_active.get());
    assert!(!target_over.get());
    assert!(delivered.get());
    assert!(preview_label.borrow().is_empty());
    assert_eq!(delivered_label.borrow().as_str(), "Stone");
}

#[test]
fn drag_preview_ghost_follows_pointer_and_clears_on_release() {
    fn subtree_contains_bounds(
        ui: &UiTree<TestHost>,
        node: fret_core::NodeId,
        expected: Rect,
    ) -> bool {
        if ui.debug_node_bounds(node) == Some(expected) {
            return true;
        }

        ui.children(node)
            .iter()
            .copied()
            .any(|child| subtree_contains_bounds(ui, child, expected))
    }

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

    let source_position = Rc::new(Cell::new(None::<Point>));
    let source_position_out = source_position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-preview-ghost",
        |cx| {
            crate::imui(cx, |ui| {
                let source = ui.button_with_options(
                    "Asset",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-drag-preview-source")),
                        ..Default::default()
                    },
                );
                let source_state = ui.drag_source(
                    source,
                    TestDragPayload {
                        label: Arc::from("Stone"),
                    },
                );
                source_position_out.set(source_state.position());

                let _ = drag_preview_ghost_with_options(
                    ui,
                    "asset-ghost",
                    source_state,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from("imui-drag-preview-ghost")),
                        ..Default::default()
                    },
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    }),
                );
            })
        },
    );

    assert!(source_position.get().is_none());
    assert_eq!(ui.layer_ids_in_paint_order().len(), 1);

    let source_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-drag-preview-source",
    );
    let drag_point = Point::new(Px(source_point.x.0 + 24.0), Px(source_point.y.0 + 18.0));

    pointer_down_at(&mut ui, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        drag_point,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let source_position_out = source_position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-preview-ghost",
        |cx| {
            crate::imui(cx, |ui| {
                let source = ui.button_with_options(
                    "Asset",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-drag-preview-source")),
                        ..Default::default()
                    },
                );
                let source_state = ui.drag_source(
                    source,
                    TestDragPayload {
                        label: Arc::from("Stone"),
                    },
                );
                source_position_out.set(source_state.position());

                let _ = drag_preview_ghost_with_options(
                    ui,
                    "asset-ghost",
                    source_state,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from("imui-drag-preview-ghost")),
                        ..Default::default()
                    },
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    }),
                );
            })
        },
    );

    assert_eq!(source_position.get(), Some(drag_point));
    let ghost_layer = *ui
        .layer_ids_in_paint_order()
        .last()
        .expect("expected ghost overlay layer");
    assert!(ui.is_layer_visible(ghost_layer));
    let ghost_root = ui.layer_root(ghost_layer).expect("ghost layer root");
    let expected_bounds = Rect::new(
        Point::new(Px(drag_point.x.0 + 12.0), Px(drag_point.y.0 + 12.0)),
        Size::new(Px(96.0), Px(28.0)),
    );
    assert!(subtree_contains_bounds(&ui, ghost_root, expected_bounds));

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, drag_point, false);

    app.advance_frame();
    let source_position_out = source_position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-preview-ghost",
        |cx| {
            crate::imui(cx, |ui| {
                let source = ui.button_with_options(
                    "Asset",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-drag-preview-source")),
                        ..Default::default()
                    },
                );
                let source_state = ui.drag_source(
                    source,
                    TestDragPayload {
                        label: Arc::from("Stone"),
                    },
                );
                source_position_out.set(source_state.position());

                let _ = drag_preview_ghost_with_options(
                    ui,
                    "asset-ghost",
                    source_state,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from("imui-drag-preview-ghost")),
                        ..Default::default()
                    },
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    }),
                );
            })
        },
    );

    assert!(source_position.get().is_none());
    assert!(!ui.is_layer_visible(ghost_layer));
}

#[test]
fn cross_window_drag_preview_ghost_transfers_between_windows() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui_a = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b = UiTree::new();
    ui_b.set_window(window_b);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render_scene = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui(cx, |ui| {
            let source = ui.button_with_options(
                "Asset",
                fret_ui_kit::imui::ButtonOptions {
                    test_id: Some(Arc::from("imui-cross-window-drag-preview-source")),
                    ..Default::default()
                },
            );
            let source_state = ui.drag_source(
                source,
                TestDragPayload {
                    label: Arc::from("Stone"),
                },
            );

            let _ = publish_cross_window_drag_preview_ghost_with_options(
                ui,
                "asset-ghost",
                source_state,
                DragPreviewGhostOptions {
                    test_id: Some(Arc::from("imui-cross-window-drag-preview-ghost")),
                    ..Default::default()
                },
                |_cx| {
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    })
                },
            );

            let _ = render_cross_window_drag_preview_ghosts(ui.cx_mut());
        })
    };

    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    assert!(!has_test_id(
        &mut ui_a,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-ghost",
    ));
    assert!(!has_test_id(
        &mut ui_b,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-ghost",
    ));

    let source_point = point_for_test_id(
        &mut ui_a,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-source",
    );
    let drag_point = Point::new(Px(source_point.x.0 + 24.0), Px(source_point.y.0 + 18.0));

    pointer_down_at(&mut ui_a, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui_a,
        &mut app,
        &mut services,
        drag_point,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );

    let ghost_layer_a = *ui_a
        .layer_ids_in_paint_order()
        .last()
        .expect("expected ghost overlay layer in window a");
    assert!(ui_a.is_layer_visible(ghost_layer_a));
    assert!(!has_test_id(
        &mut ui_b,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-ghost",
    ));

    let drag = app.drag_mut(PointerId(0)).expect("drag session");
    drag.current_window = window_b;
    drag.cross_window_hover = true;
    drag.position = Point::new(Px(120.0), Px(72.0));

    app.advance_frame();
    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );

    assert!(!ui_a.is_layer_visible(ghost_layer_a));
    let ghost_layer_b = *ui_b
        .layer_ids_in_paint_order()
        .last()
        .expect("expected ghost overlay layer in window b");
    assert!(ui_b.is_layer_visible(ghost_layer_b));

    app.cancel_drag(PointerId(0));

    app.advance_frame();
    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );

    assert!(!ui_a.is_layer_visible(ghost_layer_a));
    assert!(!ui_b.is_layer_visible(ghost_layer_b));
}

#[test]
fn sortable_rows_reorder_using_drop_positions() {
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

    let items = Rc::new(RefCell::new(test_sortable_items()));
    let preview_status = Rc::new(RefCell::new(String::new()));
    let delivered_status = Rc::new(RefCell::new(String::new()));
    let order_status = Rc::new(RefCell::new(String::new()));
    let delivered_flag = Rc::new(Cell::new(false));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-sortable-rows",
        render_test_sortable_rows(
            &items,
            &preview_status,
            &delivered_status,
            &order_status,
            &delivered_flag,
        ),
    );

    assert_eq!(
        order_status.borrow().as_str(),
        "Camera -> Cube -> Key light"
    );
    assert!(preview_status.borrow().is_empty());
    assert!(delivered_status.borrow().is_empty());
    assert!(!delivered_flag.get());

    let source_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-sortable-row.camera",
    );
    let _target_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-sortable-row.cube",
    );
    let target_bounds = bounds_for_test_id(&ui, "imui-sortable-row.cube");
    let target_lower = Point::new(
        Px(target_bounds.origin.x.0 + target_bounds.size.width.0 * 0.5),
        Px(target_bounds.origin.y.0 + target_bounds.size.height.0 * 0.75),
    );

    pointer_down_at(&mut ui, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        target_lower,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-sortable-rows",
        render_test_sortable_rows(
            &items,
            &preview_status,
            &delivered_status,
            &order_status,
            &delivered_flag,
        ),
    );

    assert_eq!(
        preview_status.borrow().as_str(),
        "Preview: move Camera after Cube"
    );
    assert!(delivered_status.borrow().is_empty());
    assert_eq!(
        order_status.borrow().as_str(),
        "Camera -> Cube -> Key light"
    );
    assert!(!delivered_flag.get());

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, target_lower, false);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-sortable-rows",
        render_test_sortable_rows(
            &items,
            &preview_status,
            &delivered_status,
            &order_status,
            &delivered_flag,
        ),
    );

    assert!(preview_status.borrow().is_empty());
    assert_eq!(
        delivered_status.borrow().as_str(),
        "Moved Camera after Cube"
    );
    assert_eq!(
        order_status.borrow().as_str(),
        "Cube -> Camera -> Key light"
    );
    assert!(delivered_flag.get());
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
                let resp = ui.button("OK");
                long_pressed_out.set(resp.long_pressed());
                holding_out.set(resp.press_holding());
            })
        },
    );
    assert!(!long_pressed.get());
    assert!(!holding.get());
}
