use super::*;

#[test]
fn button_family_variants_and_radio_mount_with_expected_bounds() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-family-variants",
        |cx| {
            crate::imui_raw(cx, |ui| {
                use fret_ui_kit::imui::{
                    ButtonArrowDirection, ButtonOptions, RadioOptions, UiWriterImUiFacadeExt as _,
                };

                let _ = ui.small_button_with_options(
                    "Quick save",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-variants.small")),
                        ..Default::default()
                    },
                );
                let _ = ui.arrow_button_with_options(
                    "imui-variants.arrow.left",
                    ButtonArrowDirection::Left,
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-variants.arrow.left")),
                        ..Default::default()
                    },
                );
                let _ = ui.invisible_button_with_options(
                    "imui-variants.hotspot",
                    Size::new(Px(48.0), Px(24.0)),
                    ButtonOptions {
                        a11y_label: Some(Arc::from("Timeline hotspot")),
                        test_id: Some(Arc::from("imui-variants.hotspot")),
                        ..Default::default()
                    },
                );
                let _ = ui.radio_with_options(
                    "Move tool",
                    true,
                    RadioOptions {
                        test_id: Some(Arc::from("imui-variants.radio")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-variants.small",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-variants.arrow.left",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-variants.hotspot",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-variants.radio",
    ));

    let arrow_bounds = bounds_for_test_id(&ui, "imui-variants.arrow.left");
    assert_eq!(arrow_bounds.size.width, arrow_bounds.size.height);

    let hotspot_bounds = bounds_for_test_id(&ui, "imui-variants.hotspot");
    assert_eq!(hotspot_bounds.size.width, Px(48.0));
    assert_eq!(hotspot_bounds.size.height, Px(24.0));
}
