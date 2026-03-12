use crate::ui::canvas::widget::*;

pub(super) fn scaled_inner_port_rect(rect: Rect, inner_scale: Option<f32>) -> Rect {
    let mut fill_rect = rect;
    if let Some(scale) = inner_scale
        && scale.is_finite()
        && scale > 0.0
    {
        let scale = scale.clamp(0.05, 1.0);
        let center_x = rect.origin.x.0 + 0.5 * rect.size.width.0;
        let center_y = rect.origin.y.0 + 0.5 * rect.size.height.0;
        let width = rect.size.width.0 * scale;
        let height = rect.size.height.0 * scale;
        fill_rect = Rect::new(
            Point::new(Px(center_x - 0.5 * width), Px(center_y - 0.5 * height)),
            Size::new(Px(width), Px(height)),
        );
    }
    fill_rect
}
