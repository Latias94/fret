//! Declarative line-plot shared geometry owner.

use fret_core::{Point, Px, Rect, Size};

use crate::cartesian::DataRect;
use crate::style::LinePlotStyle;

pub(super) fn line_plot_view_bounds_for_y_axis(
    primary: DataRect,
    axis_bounds: DataRect,
) -> DataRect {
    DataRect {
        x_min: primary.x_min,
        x_max: primary.x_max,
        y_min: axis_bounds.y_min,
        y_max: axis_bounds.y_max,
    }
}

pub(super) fn line_plot_inner_rect(bounds: Rect, style: LinePlotStyle) -> Rect {
    let pad = style.padding.0.max(0.0);
    let axis_gap = style.axis_gap.0.max(0.0);
    Rect::new(
        Point::new(
            Px(bounds.origin.x.0 + pad + axis_gap),
            Px(bounds.origin.y.0 + pad),
        ),
        Size::new(
            Px((bounds.size.width.0 - pad * 2.0 - axis_gap).max(0.0)),
            Px((bounds.size.height.0 - pad * 2.0 - axis_gap).max(0.0)),
        ),
    )
}
