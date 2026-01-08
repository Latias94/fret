//! Internal helpers for retained plot canvases.

use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::Color;

use super::super::style::OverlayAnchor;

pub(in crate::retained) fn contains_point(rect: Rect, p: Point) -> bool {
    p.x.0 >= rect.origin.x.0
        && p.y.0 >= rect.origin.y.0
        && p.x.0 <= rect.origin.x.0 + rect.size.width.0
        && p.y.0 <= rect.origin.y.0 + rect.size.height.0
}

pub(super) fn offset_rect(rect: Rect, origin: Point) -> Rect {
    Rect::new(
        Point::new(
            Px(origin.x.0 + rect.origin.x.0),
            Px(origin.y.0 + rect.origin.y.0),
        ),
        rect.size,
    )
}

pub(super) fn overlay_rect_in_plot(
    plot: Rect,
    size: Size,
    anchor: OverlayAnchor,
    margin: Px,
) -> Option<Rect> {
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    if size.width.0 <= 0.0 || size.height.0 <= 0.0 {
        return None;
    }

    let w = size.width.0;
    let h = size.height.0;
    let m = margin.0.max(0.0);

    let x = match anchor {
        OverlayAnchor::TopLeft | OverlayAnchor::BottomLeft => plot.origin.x.0 + m,
        OverlayAnchor::TopRight | OverlayAnchor::BottomRight => {
            plot.origin.x.0 + plot.size.width.0 - m - w
        }
    };
    let y = match anchor {
        OverlayAnchor::TopLeft | OverlayAnchor::TopRight => plot.origin.y.0 + m,
        OverlayAnchor::BottomLeft | OverlayAnchor::BottomRight => {
            plot.origin.y.0 + plot.size.height.0 - m - h
        }
    };

    let max_x = plot.origin.x.0 + plot.size.width.0 - w;
    let max_y = plot.origin.y.0 + plot.size.height.0 - h;

    let x = x.clamp(plot.origin.x.0, max_x);
    let y = y.clamp(plot.origin.y.0, max_y);

    Some(Rect::new(Point::new(Px(x), Px(y)), size))
}

pub(super) fn dim_color(color: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0);
    Color {
        a: (color.a * factor).clamp(0.0, 1.0),
        ..color
    }
}
