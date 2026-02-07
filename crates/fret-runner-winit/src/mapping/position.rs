use fret_core::{Point, Px};
use winit::dpi::{LogicalPosition, PhysicalPosition};

pub fn map_physical_position_to_point(
    window_scale_factor: f64,
    position: PhysicalPosition<f64>,
) -> Point {
    let logical: LogicalPosition<f32> = position.to_logical(window_scale_factor);
    Point::new(Px(logical.x), Px(logical.y))
}

pub fn map_optional_physical_position_to_point(
    window_scale_factor: f64,
    position: Option<PhysicalPosition<f64>>,
    fallback: Point,
) -> Point {
    position
        .map(|position| map_physical_position_to_point(window_scale_factor, position))
        .unwrap_or(fallback)
}
