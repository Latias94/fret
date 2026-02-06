use fret_core::{Point, Px, Rect};

use crate::core::PortDirection;
use crate::ui::style::NodeGraphStyle;

pub(crate) fn node_size_default_px(
    input_count: usize,
    output_count: usize,
    style: &NodeGraphStyle,
) -> (f32, f32) {
    // Keep a minimum chrome height even for nodes without ports so they don't collapse to a tiny
    // header-only box (XyFlow default nodes effectively have at least one source/target handle).
    let rows = input_count.max(output_count).max(1) as f32;
    let base = style.node_header_height + 2.0 * style.node_padding;
    let pin_area = rows * style.pin_row_height;
    (style.node_width, base + pin_area)
}

pub(crate) fn port_center(
    node_rect: Rect,
    dir: PortDirection,
    row: usize,
    style: &NodeGraphStyle,
    zoom: f32,
) -> Point {
    let x = match dir {
        PortDirection::In => node_rect.origin.x.0,
        PortDirection::Out => node_rect.origin.x.0 + node_rect.size.width.0,
    };
    let y = node_rect.origin.y.0
        + (style.node_header_height + style.node_padding) / zoom
        + (row as f32 + 0.5) * (style.pin_row_height / zoom);

    Point::new(Px(x), Px(y))
}
