use super::*;

#[test]
fn floating_window_collapsible_false_does_not_toggle_on_title_double_click() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let collapsed = Rc::new(Cell::new(false));

    let collapsed_out = collapsed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapsible-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options_with_behavior(
                        Size::new(Px(180.0), Px(120.0)),
                        FloatingWindowOptions {
                            collapsible: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
            })
        },
    );
    assert!(!collapsed.get());

    let title_bar = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    double_click_at(&mut ui, &mut app, &mut services, title_bar);

    app.advance_frame();
    let collapsed_out = collapsed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapsible-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options_with_behavior(
                        Size::new(Px(180.0), Px(120.0)),
                        FloatingWindowOptions {
                            collapsible: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
            })
        },
    );

    assert!(
        !collapsed.get(),
        "expected title-bar double click not to toggle collapse when collapsible=false"
    );
}
