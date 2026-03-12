use super::*;

#[test]
fn selected_group_rects_keeps_only_selected_entries() {
    let rect_a = Rect::new(Point::new(Px(1.0), Px(2.0)), Size::new(Px(3.0), Px(4.0)));
    let rect_b = Rect::new(Point::new(Px(5.0), Px(6.0)), Size::new(Px(7.0), Px(8.0)));
    let rect_c = Rect::new(Point::new(Px(9.0), Px(10.0)), Size::new(Px(11.0), Px(12.0)));
    let groups = vec![
        (rect_a, Arc::<str>::from("A"), false),
        (rect_b, Arc::<str>::from("B"), true),
        (rect_c, Arc::<str>::from("C"), true),
    ];

    assert_eq!(selected_group_rects(&groups), vec![rect_b, rect_c]);
}
