use super::*;

#[test]
fn canvas_point_from_pointer_position_transfers_axes() {
    let position = Point::new(Px(12.5), Px(-8.0));
    assert_eq!(
        canvas_point_from_pointer_position(position),
        CanvasPoint { x: 12.5, y: -8.0 }
    );
}
