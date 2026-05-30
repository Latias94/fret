use super::*;

#[test]
fn textarea_model_uses_compact_imui_chrome_without_focus_ring() {
    let mut app = App::new();
    let model = app.models_mut().insert(String::new());

    fret_ui::elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-textarea-chrome",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };

            let response = textarea_model_with_options(
                &mut ui,
                &model,
                TextAreaOptions {
                    test_id: Some(Arc::from("imui-textarea-chrome")),
                    ..Default::default()
                },
            );

            assert!(response.id().is_some());
            assert_eq!(out.len(), 1);

            let props = first_text_area(&out[0]).expect("expected text area element");
            assert!(props.chrome.focus_ring.is_none());
            assert_eq!(props.chrome.border, Edges::all(Px(1.0)));
            assert_eq!(props.chrome.padding_x, Px(8.0));
            assert_eq!(props.chrome.padding_y, Px(3.0));
            assert_eq!(
                props.chrome.corner_radii,
                Corners::all(super::super::super::control_chrome::CONTROL_RADIUS)
            );
            assert_eq!(props.layout.size.width, Length::Fill);
        },
    );
}
