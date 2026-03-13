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
mod tests;
