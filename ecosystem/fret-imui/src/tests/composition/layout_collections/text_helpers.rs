use super::*;

#[test]
fn separator_text_helper_renders_label_with_trailing_rule() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-separator-text",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.menu_item_with_options(
                    "Above",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-separator-text.above")),
                        ..Default::default()
                    },
                );
                ui.separator_text_with_options(
                    "Section",
                    fret_ui_kit::imui::SeparatorTextOptions {
                        test_id: Some(Arc::from("imui-separator-text.section")),
                    },
                );
                ui.menu_item_with_options(
                    "Below",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-separator-text.below")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let section = bounds_for_test_id(&ui, "imui-separator-text.section");
    let label = bounds_for_test_id(&ui, "imui-separator-text.section.label");
    let line = bounds_for_test_id(&ui, "imui-separator-text.section.line");

    assert!(section.size.width.0 > 200.0);
    assert!(label.origin.x.0 >= section.origin.x.0);
    assert!(line.origin.x.0 >= label.origin.x.0 + label.size.width.0);
    assert!(line.size.width.0 > 40.0);
    assert!(line.origin.x.0 + line.size.width.0 <= section.origin.x.0 + section.size.width.0 + 1.0);
}

#[test]
fn bullet_text_helper_renders_indicator_before_wrapped_label() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(280.0), Px(180.0)),
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
        "imui-bullet-text",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.bullet_text_with_options(
                    "Bullet text keeps informational copy separate from pressable controls even when the line wraps.",
                    fret_ui_kit::imui::BulletTextOptions {
                        test_id: Some(Arc::from("imui-bullet-text.entry")),
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let entry = bounds_for_test_id(&ui, "imui-bullet-text.entry");
    let indicator = bounds_for_test_id(&ui, "imui-bullet-text.entry.indicator");
    let label = bounds_for_test_id(&ui, "imui-bullet-text.entry.label");

    assert!(entry.size.width.0 > 160.0);
    assert!(indicator.origin.x.0 >= entry.origin.x.0);
    assert!(indicator.origin.x.0 + indicator.size.width.0 <= label.origin.x.0);
    assert!(label.origin.y.0 <= indicator.origin.y.0 + Px(12.0).0);
    assert!(label.size.height.0 > indicator.size.height.0);
}
// Note: `for_each_keyed` is exercised indirectly by downstream ecosystem crates. The core
// smoke tests above focus on interaction correctness (`clicked` / `changed`).
