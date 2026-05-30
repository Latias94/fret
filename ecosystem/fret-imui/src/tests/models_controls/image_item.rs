use super::*;
use fret_core::{ImageId, Px, Size};
use fret_ui_kit::imui::ImageItemOptions;

fn image_size() -> Size {
    Size::new(Px(48.0), Px(32.0))
}

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

#[test]
fn image_button_shift_f10_sets_context_menu_requested_true_once() {
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
    let image = ImageId::default();
    let render = |cx: &mut ElementContext<'_, TestHost>, requested_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            let response = ui.image_button_with_options(
                "image-button.context",
                image,
                image_size(),
                ImageItemOptions::button().with_test_id("imui-image-button-context"),
            );
            requested_out.set(response.context_menu_requested());
        })
    };

    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-image-button-context",
        |cx| render(cx, &requested_out),
    );
    assert!(!requested.get());

    let button = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-image-button-context",
    );
    click_at(&mut ui, &mut app, &mut services, button);

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-image-button-context",
        |cx| render(cx, &requested_out),
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
        "imui-image-button-context",
        |cx| render(cx, &requested_out),
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
        "imui-image-button-context",
        |cx| render(cx, &requested_out),
    );
    assert!(!requested.get());
}
