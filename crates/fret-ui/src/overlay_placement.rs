use fret_core::{Edges, Point, Px, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
}

/// Place an anchored panel near `anchor`, flipping to the opposite side if the preferred side
/// overflows the `outer` bounds.
///
/// This is a small, deterministic subset inspired by Floating UI style behavior:
///
/// - compute the preferred origin (`side`, `align`) with a `side_offset` gap,
/// - if it fits on that side without requiring clamping on the main axis, keep it,
/// - otherwise flip to the opposite side and retry,
/// - if neither fits, clamp the preferred placement into `outer`.
///
/// This function is intentionally pure and testable so higher-level overlay services can lock
/// behavior with regression tests (MVP 62).
pub fn anchored_panel_bounds(
    outer: Rect,
    anchor: Rect,
    content: Size,
    side_offset: Px,
    preferred_side: Side,
    align: Align,
) -> Rect {
    let preferred_origin = anchored_origin(anchor, content, side_offset, preferred_side, align);
    let preferred = Rect::new(preferred_origin, content);
    if side_fits_without_clamp(outer, preferred, preferred_side) {
        return clamp_rect_to_outer(outer, preferred);
    }

    let flipped_side = opposite_side(preferred_side);
    let flipped_origin = anchored_origin(anchor, content, side_offset, flipped_side, align);
    let flipped = Rect::new(flipped_origin, content);
    if side_fits_without_clamp(outer, flipped, flipped_side) {
        return clamp_rect_to_outer(outer, flipped);
    }

    // Neither side fits cleanly on the main axis. Choose the candidate that minimizes main-axis
    // overflow, breaking ties by total overflow, then clamp into `outer`.
    let preferred_overflow = overflow_amount(outer, preferred);
    let flipped_overflow = overflow_amount(outer, flipped);

    let preferred_main = main_axis_overflow(preferred_overflow, preferred_side);
    let flipped_main = main_axis_overflow(flipped_overflow, flipped_side);

    let preferred_total = total_overflow(preferred_overflow);
    let flipped_total = total_overflow(flipped_overflow);

    let chosen = if (flipped_main, flipped_total) < (preferred_main, preferred_total) {
        flipped
    } else {
        preferred
    };

    clamp_rect_to_outer(outer, chosen)
}

/// Like [`anchored_panel_bounds`], but clamps the panel `Size` to the `outer` bounds (and the
/// available space on the chosen side) before computing the final rect.
///
/// This is useful for scrollable menus/panels where content may exceed the viewport: the overlay
/// can use the returned rect as the viewport bounds and scroll its internal content.
pub fn anchored_panel_bounds_sized(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    side_offset: Px,
    preferred_side: Side,
    align: Align,
) -> Rect {
    let preferred_size = clamp_size_for_side(outer, anchor, desired, side_offset, preferred_side);
    let preferred_origin =
        anchored_origin(anchor, preferred_size, side_offset, preferred_side, align);
    let preferred = Rect::new(preferred_origin, preferred_size);
    if side_fits_without_clamp(outer, preferred, preferred_side) {
        return clamp_rect_to_outer(outer, preferred);
    }

    let flipped_side = opposite_side(preferred_side);
    let flipped_size = clamp_size_for_side(outer, anchor, desired, side_offset, flipped_side);
    let flipped_origin = anchored_origin(anchor, flipped_size, side_offset, flipped_side, align);
    let flipped = Rect::new(flipped_origin, flipped_size);
    if side_fits_without_clamp(outer, flipped, flipped_side) {
        return clamp_rect_to_outer(outer, flipped);
    }

    let preferred_overflow = overflow_amount(outer, preferred);
    let flipped_overflow = overflow_amount(outer, flipped);

    let preferred_main = main_axis_overflow(preferred_overflow, preferred_side);
    let flipped_main = main_axis_overflow(flipped_overflow, flipped_side);

    let preferred_total = total_overflow(preferred_overflow);
    let flipped_total = total_overflow(flipped_overflow);

    let chosen = if (flipped_main, flipped_total) < (preferred_main, preferred_total) {
        flipped
    } else {
        preferred
    };

    clamp_rect_to_outer(outer, chosen)
}

fn opposite_side(side: Side) -> Side {
    match side {
        Side::Top => Side::Bottom,
        Side::Bottom => Side::Top,
        Side::Left => Side::Right,
        Side::Right => Side::Left,
    }
}

fn anchored_origin(
    anchor: Rect,
    content: Size,
    side_offset: Px,
    side: Side,
    align: Align,
) -> Point {
    let w = content.width.0.max(0.0);
    let h = content.height.0.max(0.0);
    let off = side_offset.0.max(0.0);

    let anchor_left = anchor.origin.x.0;
    let anchor_top = anchor.origin.y.0;
    let anchor_right = anchor_left + anchor.size.width.0.max(0.0);
    let anchor_bottom = anchor_top + anchor.size.height.0.max(0.0);

    let mut x = match side {
        Side::Left => anchor_left - off - w,
        Side::Right => anchor_right + off,
        Side::Top | Side::Bottom => match align {
            Align::Start => anchor_left,
            Align::Center => (anchor_left + anchor_right) * 0.5 - w * 0.5,
            Align::End => anchor_right - w,
        },
    };

    let mut y = match side {
        Side::Top => anchor_top - off - h,
        Side::Bottom => anchor_bottom + off,
        Side::Left | Side::Right => match align {
            Align::Start => anchor_top,
            Align::Center => (anchor_top + anchor_bottom) * 0.5 - h * 0.5,
            Align::End => anchor_bottom - h,
        },
    };

    if !x.is_finite() {
        x = 0.0;
    }
    if !y.is_finite() {
        y = 0.0;
    }

    Point::new(Px(x), Px(y))
}

fn side_fits_without_clamp(outer: Rect, inner: Rect, side: Side) -> bool {
    match side {
        Side::Top => inner.origin.y.0 >= outer.origin.y.0,
        Side::Bottom => {
            inner.origin.y.0 + inner.size.height.0 <= outer.origin.y.0 + outer.size.height.0
        }
        Side::Left => inner.origin.x.0 >= outer.origin.x.0,
        Side::Right => {
            inner.origin.x.0 + inner.size.width.0 <= outer.origin.x.0 + outer.size.width.0
        }
    }
}

fn clamp_size_for_side(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    side_offset: Px,
    side: Side,
) -> Size {
    let max_w = outer.size.width.0.max(0.0);
    let max_h = outer.size.height.0.max(0.0);

    let mut w = desired.width.0.max(0.0).min(max_w);
    let mut h = desired.height.0.max(0.0).min(max_h);

    let outer_left = outer.origin.x.0;
    let outer_top = outer.origin.y.0;
    let outer_right = outer_left + outer.size.width.0.max(0.0);
    let outer_bottom = outer_top + outer.size.height.0.max(0.0);

    let anchor_left = anchor.origin.x.0;
    let anchor_top = anchor.origin.y.0;
    let anchor_right = anchor_left + anchor.size.width.0.max(0.0);
    let anchor_bottom = anchor_top + anchor.size.height.0.max(0.0);

    let off = side_offset.0.max(0.0);

    // Additionally clamp along the chosen side's main axis (Floating-like "available height/width").
    let available_main = match side {
        Side::Top => (anchor_top - off - outer_top).max(0.0),
        Side::Bottom => (outer_bottom - (anchor_bottom + off)).max(0.0),
        Side::Left => (anchor_left - off - outer_left).max(0.0),
        Side::Right => (outer_right - (anchor_right + off)).max(0.0),
    };
    match side {
        Side::Top | Side::Bottom => h = h.min(available_main),
        Side::Left | Side::Right => w = w.min(available_main),
    }

    Size::new(Px(w), Px(h))
}

#[derive(Debug, Clone, Copy, Default)]
struct Overflow {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

fn overflow_amount(outer: Rect, inner: Rect) -> Overflow {
    let outer_left = outer.origin.x.0;
    let outer_top = outer.origin.y.0;
    let outer_right = outer_left + outer.size.width.0;
    let outer_bottom = outer_top + outer.size.height.0;

    let inner_left = inner.origin.x.0;
    let inner_top = inner.origin.y.0;
    let inner_right = inner_left + inner.size.width.0;
    let inner_bottom = inner_top + inner.size.height.0;

    Overflow {
        left: (outer_left - inner_left).max(0.0),
        right: (inner_right - outer_right).max(0.0),
        top: (outer_top - inner_top).max(0.0),
        bottom: (inner_bottom - outer_bottom).max(0.0),
    }
}

fn main_axis_overflow(o: Overflow, side: Side) -> f32 {
    match side {
        Side::Top | Side::Bottom => o.top.max(o.bottom),
        Side::Left | Side::Right => o.left.max(o.right),
    }
}

fn total_overflow(o: Overflow) -> f32 {
    o.left + o.right + o.top + o.bottom
}

fn clamp_rect_to_outer(outer: Rect, inner: Rect) -> Rect {
    let min_x = outer.origin.x.0;
    let min_y = outer.origin.y.0;
    let max_x = (outer.origin.x.0 + outer.size.width.0 - inner.size.width.0).max(min_x);
    let max_y = (outer.origin.y.0 + outer.size.height.0 - inner.size.height.0).max(min_y);

    let x = inner.origin.x.0.clamp(min_x, max_x);
    let y = inner.origin.y.0.clamp(min_y, max_y);
    Rect::new(Point::new(Px(x), Px(y)), inner.size)
}

pub fn inset_rect(rect: Rect, margin: Edges) -> Rect {
    let w = rect.size.width.0.max(0.0);
    let h = rect.size.height.0.max(0.0);

    let l = margin.left.0.max(0.0);
    let t = margin.top.0.max(0.0);
    let r = margin.right.0.max(0.0);
    let b = margin.bottom.0.max(0.0);

    Rect::new(
        Point::new(Px(rect.origin.x.0 + l), Px(rect.origin.y.0 + t)),
        Size::new(Px((w - l - r).max(0.0)), Px((h - t - b).max(0.0))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn keeps_bottom_when_it_fits() {
        let outer = r(0.0, 0.0, 400.0, 400.0);
        let anchor = r(10.0, 10.0, 40.0, 10.0);
        let content = Size::new(Px(120.0), Px(80.0));

        let placed =
            anchored_panel_bounds(outer, anchor, content, Px(8.0), Side::Bottom, Align::Start);
        assert!(placed.origin.y.0 >= anchor.origin.y.0 + anchor.size.height.0);
    }

    #[test]
    fn flips_from_bottom_to_top_when_bottom_overflows() {
        let outer = r(0.0, 0.0, 200.0, 200.0);
        let anchor = r(10.0, 190.0, 40.0, 10.0);
        let content = Size::new(Px(120.0), Px(80.0));

        let placed =
            anchored_panel_bounds(outer, anchor, content, Px(8.0), Side::Bottom, Align::Start);
        assert!(placed.origin.y.0 + placed.size.height.0 <= anchor.origin.y.0);
        assert!(outer.contains(placed.origin));
    }

    #[test]
    fn inset_rect_shrinks_bounds() {
        let outer = r(0.0, 0.0, 100.0, 50.0);
        let inset = inset_rect(outer, Edges::all(Px(8.0)));
        assert_eq!(inset.origin, Point::new(Px(8.0), Px(8.0)));
        assert_eq!(inset.size, Size::new(Px(84.0), Px(34.0)));
    }

    #[test]
    fn flips_from_right_to_left_when_right_overflows() {
        let outer = r(0.0, 0.0, 200.0, 200.0);
        let anchor = r(190.0, 10.0, 10.0, 20.0);
        let content = Size::new(Px(120.0), Px(40.0));

        let placed =
            anchored_panel_bounds(outer, anchor, content, Px(6.0), Side::Right, Align::Start);
        assert!(
            placed.origin.x.0 + placed.size.width.0 <= anchor.origin.x.0,
            "expected right placement to flip left when overflowing"
        );
    }

    #[test]
    fn chooses_side_with_less_main_axis_overflow_when_neither_fits() {
        // Both bottom and top overflow, but bottom overflows less on the main axis.
        let outer = r(0.0, 0.0, 200.0, 200.0);
        let anchor = r(10.0, 5.0, 40.0, 10.0);
        let content = Size::new(Px(120.0), Px(180.0));

        let placed =
            anchored_panel_bounds(outer, anchor, content, Px(8.0), Side::Bottom, Align::Start);
        // With less main-axis overflow on bottom, the clamped rect should end up below (as much as possible).
        assert!(
            placed.origin.y.0 >= anchor.origin.y.0,
            "expected placement to prefer bottom when it overflows less than top"
        );
        assert!(outer.contains(placed.origin));
    }

    #[test]
    fn sized_variant_clamps_height_to_available_space_below_anchor() {
        let outer = r(0.0, 0.0, 200.0, 200.0);
        let anchor = r(10.0, 150.0, 40.0, 10.0);
        let desired = Size::new(Px(120.0), Px(180.0));

        let placed = anchored_panel_bounds_sized(
            outer,
            anchor,
            desired,
            Px(8.0),
            Side::Bottom,
            Align::Start,
        );

        // Available space below = 200 - (150 + 10 + 8) = 32
        assert_eq!(placed.size.height, Px(32.0));
        assert!(outer.contains(placed.origin));
    }
}
