use super::*;

#[test]
fn input_text_model_uses_compact_imui_chrome_without_focus_ring() {
    let mut app = App::new();
    let model = app.models_mut().insert(String::new());

    fret_ui::elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-input-text-chrome",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };

            let response = input_text_model_with_options(
                &mut ui,
                &model,
                InputTextOptions {
                    test_id: Some(Arc::from("imui-input-text-chrome")),
                    ..Default::default()
                },
            );

            assert!(response.id().is_some());
            assert_eq!(out.len(), 1);

            let props = first_text_input(&out[0]).expect("expected text input element");
            assert!(props.chrome.focus_ring.is_none());
            assert_eq!(props.chrome.border, Edges::all(Px(1.0)));
            assert_eq!(props.chrome.padding.left, Px(8.0));
            assert_eq!(props.chrome.padding.right, Px(8.0));
            assert_eq!(props.chrome.padding.top, Px(3.0));
            assert_eq!(props.chrome.padding.bottom, Px(3.0));
            assert_eq!(
                props.chrome.corner_radii,
                Corners::all(super::super::super::control_chrome::CONTROL_RADIUS)
            );
            assert_eq!(
                props.layout.size.height,
                Length::Px(super::super::super::control_chrome::FIELD_MIN_HEIGHT)
            );
            assert_eq!(
                props.layout.size.min_height,
                Some(Length::Px(
                    super::super::super::control_chrome::FIELD_MIN_HEIGHT,
                ))
            );
            assert_eq!(
                props.layout.size.max_height,
                Some(Length::Px(
                    super::super::super::control_chrome::FIELD_MIN_HEIGHT,
                ))
            );
        },
    );
}
