use super::*;
use fret_core::Px;

#[test]
fn canvas_point_from_pointer_position_transfers_axes() {
    let position = Point::new(Px(18.0), Px(-4.5));
    assert_eq!(
        canvas_point_from_pointer_position(position),
        crate::core::CanvasPoint { x: 18.0, y: -4.5 }
    );
}
