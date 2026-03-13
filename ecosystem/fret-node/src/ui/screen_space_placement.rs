//! Shared window-space placement helpers for compat-retained panels and overlays.

use fret_core::{Point, Px, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AxisAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdjacentPosition {
    Top,
    Right,
    Bottom,
    Left,
}

pub(crate) fn clamp_rect_to_bounds(mut rect: Rect, bounds: Rect) -> Rect {
    let w = rect.size.width.0.max(0.0);
    let h = rect.size.height.0.max(0.0);

    let min_x = bounds.origin.x.0;
    let min_y = bounds.origin.y.0;
    let max_x = bounds.origin.x.0 + (bounds.size.width.0 - w).max(0.0);
    let max_y = bounds.origin.y.0 + (bounds.size.height.0 - h).max(0.0);

    rect.origin.x.0 = rect.origin.x.0.clamp(min_x, max_x);
    rect.origin.y.0 = rect.origin.y.0.clamp(min_y, max_y);
    rect
}

fn aligned_origin_in_span(origin: f32, span: f32, child_span: f32, align: AxisAlign) -> f32 {
    match align {
        AxisAlign::Start => origin,
        AxisAlign::Center => origin + 0.5 * (span - child_span),
        AxisAlign::End => origin + (span - child_span).max(0.0),
    }
}

fn aligned_origin_about_anchor(anchor: f32, child_span: f32, align: AxisAlign) -> f32 {
    match align {
        AxisAlign::Start => anchor,
        AxisAlign::Center => anchor - 0.5 * child_span,
        AxisAlign::End => anchor - child_span,
    }
}

pub(crate) fn rect_adjacent_to_rect(
    bounds: Rect,
    anchor: Rect,
    size: Size,
    position: AdjacentPosition,
    align: AxisAlign,
    gap_px: f32,
    offset: Point,
) -> Rect {
    let gap = gap_px.max(0.0);
    let w = size.width.0.max(0.0);
    let h = size.height.0.max(0.0);

    let x = match position {
        AdjacentPosition::Top | AdjacentPosition::Bottom => {
            aligned_origin_in_span(anchor.origin.x.0, anchor.size.width.0, w, align)
        }
        AdjacentPosition::Left => anchor.origin.x.0 - gap - w,
        AdjacentPosition::Right => anchor.origin.x.0 + anchor.size.width.0 + gap,
    };
    let y = match position {
        AdjacentPosition::Top => anchor.origin.y.0 - gap - h,
        AdjacentPosition::Bottom => anchor.origin.y.0 + anchor.size.height.0 + gap,
        AdjacentPosition::Left | AdjacentPosition::Right => {
            aligned_origin_in_span(anchor.origin.y.0, anchor.size.height.0, h, align)
        }
    };

    clamp_rect_to_bounds(
        Rect::new(
            Point::new(Px(x + offset.x.0), Px(y + offset.y.0)),
            Size::new(Px(w), Px(h)),
        ),
        bounds,
    )
}

pub(crate) fn rect_anchored_at_point(
    bounds: Rect,
    anchor: Point,
    size: Size,
    align_x: AxisAlign,
    align_y: AxisAlign,
    offset: Point,
) -> Rect {
    let w = size.width.0.max(0.0);
    let h = size.height.0.max(0.0);
    let x = aligned_origin_about_anchor(anchor.x.0, w, align_x);
    let y = aligned_origin_about_anchor(anchor.y.0, h, align_y);

    clamp_rect_to_bounds(
        Rect::new(
            Point::new(Px(x + offset.x.0), Px(y + offset.y.0)),
            Size::new(Px(w), Px(h)),
        ),
        bounds,
    )
}

pub(crate) fn rect_in_bounds(
    bounds: Rect,
    size: Size,
    align_x: AxisAlign,
    align_y: AxisAlign,
    margin_px: f32,
    offset: Point,
) -> Rect {
    let margin = margin_px.max(0.0);
    let w = size.width.0.max(0.0);
    let h = size.height.0.max(0.0);

    let left = bounds.origin.x.0 + margin;
    let right = bounds.origin.x.0 + (bounds.size.width.0 - margin - w).max(0.0);
    let top = bounds.origin.y.0 + margin;
    let bottom = bounds.origin.y.0 + (bounds.size.height.0 - margin - h).max(0.0);
    let center_x =
        aligned_origin_in_span(bounds.origin.x.0, bounds.size.width.0, w, AxisAlign::Center);
    let center_y = aligned_origin_in_span(
        bounds.origin.y.0,
        bounds.size.height.0,
        h,
        AxisAlign::Center,
    );

    let x = match align_x {
        AxisAlign::Start => left,
        AxisAlign::Center => center_x,
        AxisAlign::End => right,
    };
    let y = match align_y {
        AxisAlign::Start => top,
        AxisAlign::Center => center_y,
        AxisAlign::End => bottom,
    };

    clamp_rect_to_bounds(
        Rect::new(
            Point::new(Px(x + offset.x.0), Px(y + offset.y.0)),
            Size::new(Px(w), Px(h)),
        ),
        bounds,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AdjacentPosition, AxisAlign, clamp_rect_to_bounds, rect_adjacent_to_rect,
        rect_anchored_at_point, rect_in_bounds,
    };
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn rect_adjacent_to_rect_top_center_matches_expected_math() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(50.0), Px(40.0)),
            Size::new(Px(40.0), Px(20.0)),
        );
        let size = Size::new(Px(30.0), Px(10.0));

        let rect = rect_adjacent_to_rect(
            bounds,
            anchor,
            size,
            AdjacentPosition::Top,
            AxisAlign::Center,
            8.0,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(rect.origin.x.0, 55.0);
        assert_eq!(rect.origin.y.0, 22.0);
    }

    #[test]
    fn rect_anchored_at_point_center_center_matches_expected_math() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );
        let anchor = Point::new(Px(50.0), Px(60.0));
        let size = Size::new(Px(20.0), Px(10.0));

        let rect = rect_anchored_at_point(
            bounds,
            anchor,
            size,
            AxisAlign::Center,
            AxisAlign::Center,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(rect.origin.x.0, 40.0);
        assert_eq!(rect.origin.y.0, 55.0);
    }

    #[test]
    fn rect_in_bounds_top_right_respects_margin() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );
        let size = Size::new(Px(40.0), Px(20.0));

        let rect = rect_in_bounds(
            bounds,
            size,
            AxisAlign::End,
            AxisAlign::Start,
            8.0,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(rect.origin.x.0, 152.0);
        assert_eq!(rect.origin.y.0, 8.0);
    }

    #[test]
    fn clamp_rect_to_bounds_keeps_large_child_inside_origin() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(50.0), Px(40.0)),
        );
        let rect = Rect::new(
            Point::new(Px(100.0), Px(100.0)),
            Size::new(Px(80.0), Px(60.0)),
        );

        let clamped = clamp_rect_to_bounds(rect, bounds);

        assert_eq!(clamped.origin.x.0, 10.0);
        assert_eq!(clamped.origin.y.0, 20.0);
        assert_eq!(clamped.size.width.0, 80.0);
        assert_eq!(clamped.size.height.0, 60.0);
    }
}
