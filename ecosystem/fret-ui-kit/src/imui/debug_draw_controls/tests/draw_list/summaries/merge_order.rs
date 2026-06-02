use super::*;

#[test]
fn debug_draw_list_reports_command_summaries_in_merge_order() {
    let mut list = ImUiDebugDrawList::default();
    let vertices = [
        DebugDrawVertex::colored(
            Point::new(Px(0.0), Px(0.0)),
            Color::from_srgb_hex_rgb(0xff_00_00),
        ),
        DebugDrawVertex::colored(
            Point::new(Px(8.0), Px(0.0)),
            Color::from_srgb_hex_rgb(0x00_ff_00),
        ),
        DebugDrawVertex::colored(
            Point::new(Px(4.0), Px(8.0)),
            Color::from_srgb_hex_rgb(0x00_00_ff),
        ),
    ];

    list.channels_split(3);
    list.add_rect_filled(
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(4.0), Px(4.0))),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
    );
    list.channels_set_current(2);
    list.add_line(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(8.0), Px(8.0)),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Px(1.0),
    );
    list.channels_set_current(1);
    list.add_image_triangle_mesh(ImageId::default(), vertices, [0, 1, 2]);

    let summaries = list.command_summaries();
    assert_eq!(summaries.len(), 3);
    assert_eq!(
        summaries
            .iter()
            .map(|summary| (summary.channel(), summary.kind()))
            .collect::<Vec<_>>(),
        vec![
            (Some(0), DebugDrawCommandKind::RectFilled),
            (Some(1), DebugDrawCommandKind::ImageTriangleMesh),
            (Some(2), DebugDrawCommandKind::Line),
        ]
    );
    assert_eq!(summaries[1].image(), Some(ImageId::default()));
    assert_eq!(summaries[1].vertex_count(), 3);
    assert_eq!(summaries[1].index_count(), 3);
    assert_eq!(summaries[1].triangle_count(), 1);
}
