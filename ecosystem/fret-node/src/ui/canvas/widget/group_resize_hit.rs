use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasRect;

pub(super) fn group_rect_to_px(rect: CanvasRect) -> Rect {
    Rect::new(
        Point::new(Px(rect.origin.x), Px(rect.origin.y)),
        Size::new(Px(rect.size.width), Px(rect.size.height)),
    )
}

pub(super) fn group_resize_handle_hit(
    handle: Rect,
    position: Point,
    zoom: f32,
    padding_screen: f32,
) -> bool {
    let Some(pad) = zoom_adjusted_hit_padding(zoom, padding_screen) else {
        return rect_contains(handle, position);
    };
    let expanded = Rect::new(
        Point::new(Px(handle.origin.x.0 - pad), Px(handle.origin.y.0 - pad)),
        Size::new(
            Px(handle.size.width.0 + 2.0 * pad),
            Px(handle.size.height.0 + 2.0 * pad),
        ),
    );
    rect_contains(expanded, position)
}

fn zoom_adjusted_hit_padding(zoom: f32, padding_screen: f32) -> Option<f32> {
    if !padding_screen.is_finite() || padding_screen <= 0.0 {
        return None;
    }
    Some(padding_screen / zoom.max(1.0e-6))
}

fn rect_contains(rect: Rect, pos: Point) -> bool {
    pos.x.0 >= rect.origin.x.0
        && pos.y.0 >= rect.origin.y.0
        && pos.x.0 <= rect.origin.x.0 + rect.size.width.0
        && pos.y.0 <= rect.origin.y.0 + rect.size.height.0
}
