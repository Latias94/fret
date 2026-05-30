use super::*;

#[test]
fn fixed_virtual_list_rows_clip_content_to_row_height() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let row = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "virtual-list-fixed-row",
        |cx| {
            let content = oversized_content(cx);
            wrap_row(cx, 0, content, None, Some(Px(20.0)))
        },
    );

    let ElementKind::Container(ContainerProps { layout, .. }) = row.kind else {
        panic!("expected fixed virtual-list row to render as a container");
    };

    assert_eq!(layout.size.height, Length::Px(Px(20.0)));
    assert_eq!(layout.overflow, Overflow::Clip);
}

#[test]
fn known_virtual_list_rows_clip_content_to_known_row_height() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let height_at: Arc<dyn Fn(usize) -> Px + Send + Sync> =
        Arc::new(|index| if index == 1 { Px(36.0) } else { Px(20.0) });
    let fixed_height =
        row_height_for_index(1, VirtualListMeasureMode::Known, Px(20.0), Some(&height_at));

    let row = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "virtual-list-known-row",
        |cx| {
            let content = oversized_content(cx);
            wrap_row(cx, 1, content, None, fixed_height)
        },
    );

    let ElementKind::Container(ContainerProps { layout, .. }) = row.kind else {
        panic!("expected known-height virtual-list row to render as a container");
    };

    assert_eq!(layout.size.height, Length::Px(Px(36.0)));
    assert_eq!(layout.overflow, Overflow::Clip);
}
