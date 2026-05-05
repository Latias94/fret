use super::*;
use fret_ui_kit::imui::{InputTextMode, InputTextOptions};

#[test]
fn input_text_read_only_blocks_text_input_and_keeps_changed_false() {
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

    let model = app.models_mut().insert(String::from("locked"));
    let changed = Rc::new(Cell::new(false));

    let changed_out = changed.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-read-only",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.input_text_model_with_options(
                        &model,
                        InputTextOptions {
                            read_only: true,
                            test_id: Some(Arc::from("imui-input-text-read-only")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );
    assert!(!changed.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "!");

    app.advance_frame();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-read-only",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.input_text_model_with_options(
                        &model,
                        InputTextOptions {
                            read_only: true,
                            test_id: Some(Arc::from("imui-input-text-read-only")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    assert!(!changed.get());
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some("locked"));
}

#[test]
fn input_text_select_all_on_focus_enables_copy() {
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

    let model = app.models_mut().insert(String::from("select me"));

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-select-all-on-focus",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        select_all_on_focus: true,
                        test_id: Some(Arc::from("imui-input-text-select-all-on-focus")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-select-all-on-focus",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        select_all_on_focus: true,
                        test_id: Some(Arc::from("imui-input-text-select-all-on-focus")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(
        dispatched > 0,
        "expected select-all-on-focus timer to dispatch"
    );
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-select-all-on-focus",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        select_all_on_focus: true,
                        test_id: Some(Arc::from("imui-input-text-select-all-on-focus")),
                        ..Default::default()
                    },
                );
            })
        },
    );
    let effects: Vec<_> = app.effects.drain(..).collect();
    let mut selected_all = false;
    for effect in effects {
        if let Effect::Command {
            window: Some(target_window),
            command,
        } = effect
            && target_window == window
            && command == fret_runtime::CommandId::from("edit.select_all")
        {
            selected_all = ui.dispatch_command(&mut app, &mut services, &command);
        }
    }
    assert!(
        selected_all,
        "expected focus-time timer to emit and dispatch edit.select_all"
    );
    assert!(
        ui.is_command_available(&mut app, &fret_runtime::CommandId::from("edit.copy")),
        "expected focus-time select_all to make copy available"
    );
}

#[test]
fn input_text_select_all_on_focus_drops_if_focus_moves_before_timer() {
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

    let first = app.models_mut().insert(String::from("first"));
    let second = app.models_mut().insert(String::from("second"));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.column(|ui| {
                let _ = ui.input_text_model_with_options(
                    &first,
                    InputTextOptions {
                        select_all_on_focus: true,
                        test_id: Some(Arc::from("imui-select-all-first")),
                        ..Default::default()
                    },
                );
                let _ = ui.input_text_model_with_options(
                    &second,
                    InputTextOptions {
                        test_id: Some(Arc::from("imui-select-all-second")),
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
        "imui-input-text-select-all-focus-move",
        |cx| render(cx),
    );

    let first_at = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-all-first",
    );
    click_at(&mut ui, &mut app, &mut services, first_at);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-select-all-focus-move",
        |cx| render(cx),
    );

    let second_at = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-all-second",
    );
    click_at(&mut ui, &mut app, &mut services, second_at);

    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(
        dispatched > 0,
        "expected select-all-on-focus timer to dispatch"
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-select-all-focus-move",
        |cx| render(cx),
    );

    assert!(
        !app.effects.iter().any(|effect| {
            matches!(
                effect,
                Effect::Command { command, .. }
                    if command == &fret_runtime::CommandId::from("edit.select_all")
            )
        }),
        "stale select-all-on-focus timer must not select text in the newly focused control"
    );
}

#[test]
fn input_text_password_mode_obscures_paint_text_without_mutating_model() {
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

    let model = app.models_mut().insert(String::from("secret"));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-password",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        mode: InputTextMode::Password,
                        test_id: Some(Arc::from("imui-input-text-password")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    services.prepared.clear();
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        services.prepared.iter().any(|text| text == "••••••"),
        "expected password mode to paint an obscured string"
    );
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some("secret"),
        "expected password mode to preserve the underlying model value"
    );
}
