use super::*;

pub(super) fn next_power_of_two_at_least(min: u32, value: f32) -> u32 {
    let target = value.ceil().max(1.0) as u32;
    let pow2 = target.checked_next_power_of_two().unwrap_or(0x8000_0000);
    pow2.max(min)
}

pub(super) fn centered_single_tile_rect(
    viewport_rect: Rect,
    tile_size_canvas: f32,
) -> Option<Rect> {
    let center_x = viewport_rect.origin.x.0 + 0.5 * viewport_rect.size.width.0;
    let center_y = viewport_rect.origin.y.0 + 0.5 * viewport_rect.size.height.0;
    if !center_x.is_finite() || !center_y.is_finite() {
        return None;
    }

    let tile_x = (center_x / tile_size_canvas).floor() as i32;
    let tile_y = (center_y / tile_size_canvas).floor() as i32;
    let origin = Point::new(
        Px(tile_x as f32 * tile_size_canvas),
        Px(tile_y as f32 * tile_size_canvas),
    );
    Some(Rect::new(
        origin,
        Size::new(Px(tile_size_canvas), Px(tile_size_canvas)),
    ))
}
