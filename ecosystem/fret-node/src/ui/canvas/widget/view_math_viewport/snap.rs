use crate::core::{CanvasPoint, CanvasSize};

pub(in super::super) fn snap_canvas_point(pos: CanvasPoint, grid: CanvasSize) -> CanvasPoint {
    CanvasPoint {
        x: snap_axis(pos.x, grid.width),
        y: snap_axis(pos.y, grid.height),
    }
}

fn snap_axis(value: f32, grid: f32) -> f32 {
    if !value.is_finite() {
        return value;
    }
    if !grid.is_finite() || grid <= 0.0 {
        return value;
    }
    (value / grid).round() * grid
}

#[cfg(test)]
mod tests {
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
}
