use super::*;

#[test]
fn floating_window_closable_false_hides_close_button_and_escape_does_not_close() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let open = app.models_mut().insert(true);

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-closable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options_with_behavior(
                        &open,
                        FloatingWindowOptions {
                            closable: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.close:demo",
        ),
        "expected close button to be hidden when closable=false"
    );

    let title_bar = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    click_at(&mut ui, &mut app, &mut services, title_bar);
    assert!(ui.focus().is_some(), "expected title bar to take focus");

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-closable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options_with_behavior(
                        &open,
                        FloatingWindowOptions {
                            closable: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    assert!(
        app.models().get_copied(&open).unwrap_or(false),
        "expected Escape not to close when closable=false"
    );
}
