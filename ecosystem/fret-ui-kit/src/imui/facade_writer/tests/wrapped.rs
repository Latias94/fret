use super::*;

#[test]
fn imui_text_wrapped_is_explicit_wrapping_text() {
    let mut app = App::new();

    elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-text-wrapped",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };

            ui.text_wrapped("Long explanatory text can opt into wrapping explicitly");

            assert_eq!(out.len(), 1);
            let ElementKind::Text(props) = &out[0].kind else {
                panic!("expected imui wrapped text item to produce a Text element");
            };

            assert_eq!(props.layout.size.width, Length::Fill);
            assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
            assert_eq!(props.layout.flex.grow, 1.0);
            assert_eq!(props.layout.flex.shrink, 1.0);
            assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
            assert_eq!(props.wrap, TextWrap::Word);
            assert_eq!(props.overflow, TextOverflow::Clip);
            assert!(out[0].inherited_text_style.is_some());
        },
    );
}
