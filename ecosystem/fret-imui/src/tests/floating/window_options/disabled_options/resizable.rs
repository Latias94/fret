use super::*;

#[test]
fn floating_window_resizable_false_hides_resize_handles() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resizable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options_with_behavior(
                        Size::new(Px(180.0), Px(120.0)),
                        FloatingWindowOptions {
                            resizable: false,
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
            "imui.float_window.resize.corner:demo",
        ),
        "expected resize handles hidden when resizable=false"
    );
}
