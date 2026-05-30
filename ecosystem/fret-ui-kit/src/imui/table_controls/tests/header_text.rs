use super::*;

#[test]
fn table_header_label_uses_shared_table_cell_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        header::table_header_label_text(cx, Arc::from("Very long table header"))
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected table header label to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
    assert!(el.inherited_text_style.is_some());
}

#[test]
fn table_sort_indicator_uses_shared_chrome_glyph_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        header::table_sort_indicator_text(cx, TableSortDirection::Ascending)
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected table sort indicator to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Clip);
    assert!(el.inherited_text_style.is_some());
}
