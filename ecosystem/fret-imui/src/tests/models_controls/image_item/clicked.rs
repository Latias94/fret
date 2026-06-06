use super::*;

#[test]
fn image_button_clicked_is_delivered_once() {
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
    let image = ImageId::default();

    let clicked_out = clicked.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-image-button-clicked",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(
                    ui.image_button("image-button.clicked", image, image_size())
                        .clicked(),
                );
            })
        },
    );
    assert!(!clicked.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-image-button-clicked",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(
                    ui.image_button("image-button.clicked", image, image_size())
                        .clicked(),
                );
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
        "imui-image-button-clicked",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(
                    ui.image_button("image-button.clicked", image, image_size())
                        .clicked(),
                );
            })
        },
    );
    assert!(!clicked.get());
}
