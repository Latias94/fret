use super::super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in super::super) struct ToastLayout {
    pub pad: f32,
    pub max_width: f32,
    pub min_width: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub viewport_height: f32,
    pub margin: f32,
}

pub(in super::super) fn toast_layout(
    zoom: f32,
    viewport_origin_x: f32,
    viewport_origin_y: f32,
    viewport_h: f32,
) -> ToastLayout {
    ToastLayout {
        pad: 10.0 / zoom,
        max_width: 420.0 / zoom,
        min_width: 120.0 / zoom,
        origin_x: viewport_origin_x,
        origin_y: viewport_origin_y,
        viewport_height: viewport_h,
        margin: 12.0 / zoom,
    }
}

pub(in super::super) fn toast_text_constraints(
    scale_factor: f32,
    layout: ToastLayout,
    zoom: f32,
) -> TextConstraints {
    TextConstraints {
        max_width: Some(Px(layout.max_width - 2.0 * layout.pad)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: effective_scale_factor(scale_factor, zoom),
    }
}

pub(in super::super) fn toast_rect(layout: ToastLayout, text_width: f32, text_height: f32) -> Rect {
    let box_w = (text_width + 2.0 * layout.pad).clamp(layout.min_width, layout.max_width);
    let box_h = text_height + 2.0 * layout.pad;
    let x = layout.origin_x + layout.margin;
    let y = layout.origin_y + layout.viewport_height - box_h - layout.margin;
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(box_w), Px(box_h)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_rect_clamps_box_width_to_minimum() {
        let layout = toast_layout(1.0, 0.0, 0.0, 200.0);
        let rect = toast_rect(layout, 24.0, 18.0);
        assert_eq!(rect.size.width, Px(120.0));
    }

    #[test]
    fn toast_rect_clamps_box_width_to_maximum() {
        let layout = toast_layout(1.0, 0.0, 0.0, 200.0);
        let rect = toast_rect(layout, 500.0, 18.0);
        assert_eq!(rect.size.width, Px(420.0));
    }

    #[test]
    fn toast_rect_places_box_at_viewport_bottom_left() {
        let layout = toast_layout(2.0, 30.0, 40.0, 300.0);
        let rect = toast_rect(layout, 100.0, 20.0);
        assert_eq!(rect.origin.x, Px(36.0));
        assert_eq!(rect.origin.y, Px(304.0));
        assert_eq!(rect.size.width, Px(110.0));
        assert_eq!(rect.size.height, Px(30.0));
    }
}
