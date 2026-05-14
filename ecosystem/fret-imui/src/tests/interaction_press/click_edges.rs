use super::*;

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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());
}
