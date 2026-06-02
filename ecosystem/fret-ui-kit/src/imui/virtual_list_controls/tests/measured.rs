use super::*;

#[test]
fn measured_virtual_list_rows_keep_content_overflow_visible_for_measurement() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let row = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "virtual-list-measured-row",
        |cx| {
            let content = oversized_content(cx);
            wrap_row(cx, 0, content, None, None)
        },
    );

    let ElementKind::Container(ContainerProps { layout, .. }) = row.kind else {
        panic!("expected measured virtual-list row to render as a container");
    };

    assert_eq!(layout.size.height, Length::Auto);
    assert_eq!(layout.overflow, LayoutStyle::default().overflow);
}
