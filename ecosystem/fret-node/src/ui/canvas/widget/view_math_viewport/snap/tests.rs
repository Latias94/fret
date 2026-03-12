use super::*;

#[test]
fn snap_canvas_point_rounds_each_axis_independently() {
    let snapped = snap_canvas_point(
        CanvasPoint { x: 23.0, y: 46.0 },
        CanvasSize {
            width: 10.0,
            height: 25.0,
        },
    );
    assert_eq!(snapped.x, 20.0);
    assert_eq!(snapped.y, 50.0);
}

#[test]
fn snap_canvas_point_ignores_invalid_grid_axis() {
    let snapped = snap_canvas_point(
        CanvasPoint { x: 23.0, y: 46.0 },
        CanvasSize {
            width: 0.0,
            height: f32::NAN,
        },
    );
    assert_eq!(snapped.x, 23.0);
    assert_eq!(snapped.y, 46.0);
}
