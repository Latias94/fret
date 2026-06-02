use super::*;

#[test]
fn debug_draw_command_summaries_track_effective_clip_stack() {
    let outer = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(32.0)));
    let inner = Rect::new(Point::new(Px(4.0), Px(4.0)), Size::new(Px(12.0), Px(12.0)));

    let mut list = ImUiDebugDrawList::default();
    list.add_line(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(8.0), Px(8.0)),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(1.0),
    );
    list.push_clip_rect(outer);
    list.add_rect_filled(
        Rect::new(Point::new(Px(2.0), Px(2.0)), Size::new(Px(6.0), Px(6.0))),
        Color::from_srgb_hex_rgb(0xff_00_00),
    );
    list.push_clip_rect(inner);
    list.add_text(
        Point::new(Px(6.0), Px(6.0)),
        "clipped",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );
    list.pop_clip_rect();
    list.add_image(
        Rect::new(Point::new(Px(8.0), Px(8.0)), Size::new(Px(6.0), Px(6.0))),
        ImageId::default(),
    );
    list.pop_clip_rect();

    let summaries = list.command_summaries();
    assert_eq!(summaries[0].clip_rect(), None);
    assert_eq!(summaries[0].clip_depth(), 0);
    assert_eq!(summaries[1].clip_rect(), Some(outer));
    assert_eq!(summaries[1].clip_depth(), 1);
    assert_eq!(summaries[2].clip_rect(), Some(outer));
    assert_eq!(summaries[2].clip_depth(), 1);
    assert_eq!(summaries[3].clip_rect(), Some(inner));
    assert_eq!(summaries[3].clip_depth(), 2);
    assert_eq!(summaries[4].clip_rect(), Some(inner));
    assert_eq!(summaries[4].clip_depth(), 2);
    assert_eq!(summaries[5].clip_rect(), Some(outer));
    assert_eq!(summaries[5].clip_depth(), 1);
    assert_eq!(summaries[6].clip_rect(), Some(outer));
    assert_eq!(summaries[6].clip_depth(), 1);
    assert_eq!(summaries[7].clip_rect(), None);
    assert_eq!(summaries[7].clip_depth(), 0);

    let summary = list.list_summary();
    assert_eq!(summary.max_clip_depth(), 2);
    assert_eq!(summary.final_clip_depth(), 0);
}

#[test]
fn debug_draw_list_records_clip_stack_commands() {
    let mut list = ImUiDebugDrawList::default();
    list.push_clip_rect(Rect::new(
        Point::new(Px(2.0), Px(3.0)),
        Size::new(Px(40.0), Px(50.0)),
    ));
    list.pop_clip_rect();

    assert_eq!(list.command_count(), 2);
    assert!(matches!(
        list.commands[0],
        DebugDrawCommand::PushClipRect { .. }
    ));
    assert!(matches!(list.commands[1], DebugDrawCommand::PopClipRect));
}
