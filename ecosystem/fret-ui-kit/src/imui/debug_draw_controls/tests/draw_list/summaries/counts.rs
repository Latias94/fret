use super::*;

#[test]
fn debug_draw_list_summary_counts_visible_command_classes() {
    let mut list = ImUiDebugDrawList::default();
    list.push_clip_rect(Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(16.0), Px(16.0)),
    ));
    list.add_image(
        Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(8.0), Px(8.0))),
        ImageId::default(),
    );
    list.add_svg_image(
        Rect::new(Point::new(Px(2.0), Px(2.0)), Size::new(Px(8.0), Px(8.0))),
        SvgSource::Static(b"<svg/>"),
    );
    list.add_rect_filled_multi_color(
        Rect::new(Point::new(Px(3.0), Px(3.0)), Size::new(Px(10.0), Px(10.0))),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Color::from_srgb_hex_rgb(0x00_ff_00),
        Color::from_srgb_hex_rgb(0x00_00_ff),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
    );
    list.add_text(
        Point::new(Px(4.0), Px(4.0)),
        "debug",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );
    list.pop_clip_rect();

    let summary = list.list_summary();
    assert_eq!(summary.command_count(), 6);
    assert_eq!(summary.clip_push_count(), 1);
    assert_eq!(summary.clip_pop_count(), 1);
    assert_eq!(summary.image_command_count(), 1);
    assert_eq!(summary.svg_command_count(), 1);
    assert_eq!(summary.text_command_count(), 1);
    assert_eq!(summary.vertex_count(), 4);
    assert_eq!(summary.index_count(), 6);
    assert_eq!(summary.triangle_count(), 2);
}
