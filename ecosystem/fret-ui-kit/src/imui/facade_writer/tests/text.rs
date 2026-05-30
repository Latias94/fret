use super::*;

#[test]
fn imui_text_item_is_single_line_and_shrinkable() {
    let mut app = App::new();

    elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-text-item",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };

            ui.text("Long editor status text that should not wrap inside a dense row");

            assert_eq!(out.len(), 1);
            let ElementKind::Text(props) = &out[0].kind else {
                panic!("expected imui text item to produce a Text element");
            };

            assert_eq!(props.layout.flex.shrink, 1.0);
            assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
            assert_eq!(props.wrap, TextWrap::None);
            assert_eq!(props.overflow, TextOverflow::Ellipsis);
            assert!(out[0].inherited_text_style.is_some());
        },
    );
}
