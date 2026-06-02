use super::*;

#[test]
fn tooltip_body_text_uses_compact_paragraph_role() {
    let mut app = App::new();
    elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-tooltip-text-role",
        |cx| {
            let mut out = Vec::new();
            {
                let mut ui = TestWriter { cx, out: &mut out };

                let mounted = tooltip_text_with_options(
                    &mut ui,
                    "tooltip",
                    ResponseExt::default(),
                    Arc::from("Tooltip body copy may wrap when it needs to explain an action"),
                    TooltipOptions::default(),
                );

                assert!(!mounted);
            }
            assert!(out.is_empty());

            let element = tooltip_body_text(
                cx,
                "Tooltip body copy may wrap when it needs to explain an action",
            );
            let ElementKind::Text(props) = &element.kind else {
                panic!("expected tooltip text role to produce a Text element");
            };
            assert_eq!(props.layout.size.width, Length::Fill);
            assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
            assert_eq!(props.layout.flex.shrink, 1.0);
            assert_eq!(props.wrap, TextWrap::Word);
            assert_eq!(props.overflow, TextOverflow::Clip);
        },
    );
}
