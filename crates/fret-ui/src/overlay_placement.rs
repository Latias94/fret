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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutDirection {
    #[default]
    Ltr,
    Rtl,
}

/// Offset configuration inspired by Floating UI's `offset()` middleware.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Offset {
    /// Distance along the placement side axis (the "gap" between anchor and panel).
    pub main_axis: Px,
    /// Distance along the alignment axis (skidding).
    pub cross_axis: Px,
    /// Optional skidding override for aligned placements (Start/End).
    ///
    /// When present and `align != Center`, this overrides `cross_axis` and flips sign for `End`.
    /// For vertical placements (Top/Bottom), the direction is also flipped under RTL.
    pub alignment_axis: Option<Px>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AnchoredPanelOptions {
    pub direction: LayoutDirection,
    pub offset: Offset,
    pub arrow: Option<ArrowOptions>,
}

/// Arrow positioning options inspired by Floating UI's `arrow()` middleware.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowOptions {
    /// Arrow element size (in the same coordinate space as `outer`/`anchor`/`content`).
    pub size: Size,
    /// Padding between the arrow and the floating element edges (useful for rounded corners).
    pub padding: Edges,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowLayout {
    /// Which side of the floating panel the arrow should attach to.
    pub side: Side,
    /// Offset along the arrow's axis inside the floating panel (x for Top/Bottom, y for Left/Right).
    pub offset: Px,
    /// The alignment-axis translation applied to the panel to keep the arrow pointing at the anchor
    /// when the anchor is too small (Radix/Floating behavior).
    pub alignment_offset: Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnchoredPanelLayout {
    pub rect: Rect,
    pub side: Side,
    pub align: Align,
    pub arrow: Option<ArrowLayout>,
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
    // IMPORTANT: `anchored_panel_bounds_sized` must still follow the same flip/overflow decision
    // rules as `anchored_panel_bounds` (ADR 0064). In particular, we must decide "fits without
    // requiring main-axis clamping" based on the *desired* size, not the clamped viewport size.
    let desired = Size::new(Px(desired.width.0.max(0.0)), Px(desired.height.0.max(0.0)));

    let preferred_unclamped_origin =
        anchored_origin(anchor, desired, side_offset, preferred_side, align);
    let preferred_unclamped = Rect::new(preferred_unclamped_origin, desired);
    if side_fits_without_clamp(outer, preferred_unclamped, preferred_side) {
        let size = clamp_size_for_side(outer, anchor, desired, side_offset, preferred_side);
        let origin = anchored_origin(anchor, size, side_offset, preferred_side, align);
        return clamp_rect_to_outer(outer, Rect::new(origin, size));
    }

    let flipped_side = opposite_side(preferred_side);
    let flipped_unclamped_origin =
        anchored_origin(anchor, desired, side_offset, flipped_side, align);
    let flipped_unclamped = Rect::new(flipped_unclamped_origin, desired);
    if side_fits_without_clamp(outer, flipped_unclamped, flipped_side) {
        let size = clamp_size_for_side(outer, anchor, desired, side_offset, flipped_side);
        let origin = anchored_origin(anchor, size, side_offset, flipped_side, align);
        return clamp_rect_to_outer(outer, Rect::new(origin, size));
    }

    // Neither side fits on the main axis given the desired size, so we will clamp the panel size
    // to the available space on whichever side we pick.
    //
    // Prefer the side with more available main-axis space (Floating UI `flip` best-fit behavior)
    // so "greedy" content measured with a large probe doesn't get stuck to the preferred side when
    // the opposite side would yield a significantly larger viewport.
    let preferred_available = available_main_for_side(outer, anchor, side_offset, preferred_side);
    let flipped_available = available_main_for_side(outer, anchor, side_offset, flipped_side);

    let chosen_side = if flipped_available > preferred_available {
        flipped_side
    } else if preferred_available > flipped_available {
        preferred_side
    } else {
        // Tie: fall back to the same overflow heuristic as the unsized solver.
        let preferred_overflow = overflow_amount(outer, preferred_unclamped);
        let flipped_overflow = overflow_amount(outer, flipped_unclamped);

        let preferred_main = main_axis_overflow(preferred_overflow, preferred_side);
        let flipped_main = main_axis_overflow(flipped_overflow, flipped_side);

        let preferred_total = total_overflow(preferred_overflow);
        let flipped_total = total_overflow(flipped_overflow);

        if (flipped_main, flipped_total) < (preferred_main, preferred_total) {
            flipped_side
        } else {
            preferred_side
        }
    };

    let size = clamp_size_for_side(outer, anchor, desired, side_offset, chosen_side);
    let origin = anchored_origin(anchor, size, side_offset, chosen_side, align);
    clamp_rect_to_outer(outer, Rect::new(origin, size))
}

/// Extended anchored panel placement that can return arrow layout data and supports Floating-like
/// offsets. Keeps the same deterministic flip/clamp behavior as [`anchored_panel_bounds`].
pub fn anchored_panel_layout_ex(
    outer: Rect,
    anchor: Rect,
    content: Size,
    side_offset: Px,
    preferred_side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> AnchoredPanelLayout {
    let content = Size::new(Px(content.width.0.max(0.0)), Px(content.height.0.max(0.0)));
    let gap = Px((side_offset.0 + options.offset.main_axis.0).max(0.0));

    let preferred_origin = anchored_origin_ex(anchor, content, gap, preferred_side, align, options);
    let preferred = Rect::new(preferred_origin, content);
    if side_fits_without_clamp(outer, preferred, preferred_side) {
        return finalize_layout(
            outer,
            anchor,
            clamp_rect_to_outer(outer, preferred),
            preferred_side,
            align,
            options,
        );
    }

    let flipped_side = opposite_side(preferred_side);
    let flipped_origin = anchored_origin_ex(anchor, content, gap, flipped_side, align, options);
    let flipped = Rect::new(flipped_origin, content);
    if side_fits_without_clamp(outer, flipped, flipped_side) {
        return finalize_layout(
            outer,
            anchor,
            clamp_rect_to_outer(outer, flipped),
            flipped_side,
            align,
            options,
        );
    }

    // Neither side fits cleanly on the main axis. Choose the candidate that minimizes main-axis
    // overflow, breaking ties by total overflow, then clamp into `outer`.
    let preferred_overflow = overflow_amount(outer, preferred);
    let flipped_overflow = overflow_amount(outer, flipped);

    let preferred_main = main_axis_overflow(preferred_overflow, preferred_side);
    let flipped_main = main_axis_overflow(flipped_overflow, flipped_side);

    let preferred_total = total_overflow(preferred_overflow);
    let flipped_total = total_overflow(flipped_overflow);

    let (chosen_side, chosen) = if (flipped_main, flipped_total) < (preferred_main, preferred_total)
    {
        (flipped_side, flipped)
    } else {
        (preferred_side, preferred)
    };

    finalize_layout(
        outer,
        anchor,
        clamp_rect_to_outer(outer, chosen),
        chosen_side,
        align,
        options,
    )
}

/// Like [`anchored_panel_layout_ex`], but clamps the panel `Size` to available space (see ADR 0064).
pub fn anchored_panel_layout_sized_ex(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    side_offset: Px,
    preferred_side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> AnchoredPanelLayout {
    // IMPORTANT: must still decide flip/overflow based on the *desired* size, not the clamped size.
    let desired = Size::new(Px(desired.width.0.max(0.0)), Px(desired.height.0.max(0.0)));
    let gap = Px((side_offset.0 + options.offset.main_axis.0).max(0.0));

    let preferred_unclamped_origin =
        anchored_origin_ex(anchor, desired, gap, preferred_side, align, options);
    let preferred_unclamped = Rect::new(preferred_unclamped_origin, desired);
    if side_fits_without_clamp(outer, preferred_unclamped, preferred_side) {
        let size = clamp_size_for_side(outer, anchor, desired, gap, preferred_side);
        let origin = anchored_origin_ex(anchor, size, gap, preferred_side, align, options);
        return finalize_layout(
            outer,
            anchor,
            clamp_rect_to_outer(outer, Rect::new(origin, size)),
            preferred_side,
            align,
            options,
        );
    }

    let flipped_side = opposite_side(preferred_side);
    let flipped_unclamped_origin =
        anchored_origin_ex(anchor, desired, gap, flipped_side, align, options);
    let flipped_unclamped = Rect::new(flipped_unclamped_origin, desired);
    if side_fits_without_clamp(outer, flipped_unclamped, flipped_side) {
        let size = clamp_size_for_side(outer, anchor, desired, gap, flipped_side);
        let origin = anchored_origin_ex(anchor, size, gap, flipped_side, align, options);
        return finalize_layout(
            outer,
            anchor,
            clamp_rect_to_outer(outer, Rect::new(origin, size)),
            flipped_side,
            align,
            options,
        );
    }

    // Neither side fits on the main axis given the desired size, so clamp the panel size to the
    // available space on whichever side we pick. Prefer the side with more available main-axis
    // space (Floating UI `flip` best-fit behavior).
    let preferred_available = available_main_for_side(outer, anchor, gap, preferred_side);
    let flipped_available = available_main_for_side(outer, anchor, gap, flipped_side);

    let chosen_side = if flipped_available > preferred_available {
        flipped_side
    } else if preferred_available > flipped_available {
        preferred_side
    } else {
        // Tie: fall back to the same overflow heuristic as the unsized solver.
        let preferred_overflow = overflow_amount(outer, preferred_unclamped);
        let flipped_overflow = overflow_amount(outer, flipped_unclamped);

        let preferred_main = main_axis_overflow(preferred_overflow, preferred_side);
        let flipped_main = main_axis_overflow(flipped_overflow, flipped_side);

        let preferred_total = total_overflow(preferred_overflow);
        let flipped_total = total_overflow(flipped_overflow);

        if (flipped_main, flipped_total) < (preferred_main, preferred_total) {
            flipped_side
        } else {
            preferred_side
        }
    };

    let size = clamp_size_for_side(outer, anchor, desired, gap, chosen_side);
    let origin = anchored_origin_ex(anchor, size, gap, chosen_side, align, options);
    finalize_layout(
        outer,
        anchor,
        clamp_rect_to_outer(outer, Rect::new(origin, size)),
        chosen_side,
        align,
        options,
    )
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

fn anchored_origin_ex(
    anchor: Rect,
    content: Size,
    gap: Px,
    side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> Point {
    let origin = anchored_origin(anchor, content, gap, side, align);
    apply_cross_axis_offset(origin, side, align, options)
}

fn apply_cross_axis_offset(
    origin: Point,
    side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> Point {
    let is_vertical = matches!(side, Side::Top | Side::Bottom);
    let rtl = options.direction == LayoutDirection::Rtl;

    let mut cross = options.offset.cross_axis.0;
    if align != Align::Center
        && let Some(axis) = options.offset.alignment_axis
    {
        let mut v = axis.0;
        if align == Align::End {
            v *= -1.0;
        }
        if is_vertical && rtl {
            v *= -1.0;
        }
        cross = v;
    }

    if !cross.is_finite() || cross == 0.0 {
        return origin;
    }

    match side {
        Side::Top | Side::Bottom => Point::new(Px(origin.x.0 + cross), origin.y),
        Side::Left | Side::Right => Point::new(origin.x, Px(origin.y.0 + cross)),
    }
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

    // Additionally clamp along the chosen side's main axis (Floating-like "available height/width").
    let available_main = available_main_for_side(outer, anchor, side_offset, side);
    match side {
        Side::Top | Side::Bottom => h = h.min(available_main),
        Side::Left | Side::Right => w = w.min(available_main),
    }

    Size::new(Px(w), Px(h))
}

fn available_main_for_side(outer: Rect, anchor: Rect, side_offset: Px, side: Side) -> f32 {
    let outer_left = outer.origin.x.0;
    let outer_top = outer.origin.y.0;
    let outer_right = outer_left + outer.size.width.0.max(0.0);
    let outer_bottom = outer_top + outer.size.height.0.max(0.0);

    let anchor_left = anchor.origin.x.0;
    let anchor_top = anchor.origin.y.0;
    let anchor_right = anchor_left + anchor.size.width.0.max(0.0);
    let anchor_bottom = anchor_top + anchor.size.height.0.max(0.0);

    let off = side_offset.0.max(0.0);

    match side {
        Side::Top => (anchor_top - off - outer_top).max(0.0),
        Side::Bottom => (outer_bottom - (anchor_bottom + off)).max(0.0),
        Side::Left => (anchor_left - off - outer_left).max(0.0),
        Side::Right => (outer_right - (anchor_right + off)).max(0.0),
    }
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

fn finalize_layout(
    outer: Rect,
    anchor: Rect,
    mut rect: Rect,
    side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> AnchoredPanelLayout {
    let arrow = options
        .arrow
        .map(|arrow| apply_arrow_layout(outer, anchor, &mut rect, side, align, arrow));

    AnchoredPanelLayout {
        rect,
        side,
        align,
        arrow,
    }
}

fn apply_arrow_layout(
    outer: Rect,
    anchor: Rect,
    rect: &mut Rect,
    placement_side: Side,
    align: Align,
    arrow: ArrowOptions,
) -> ArrowLayout {
    let is_vertical = matches!(placement_side, Side::Top | Side::Bottom);

    let (anchor_start, anchor_len, rect_start, rect_len, arrow_len, pad_start, pad_end) =
        if is_vertical {
            (
                anchor.origin.x.0,
                anchor.size.width.0.max(0.0),
                rect.origin.x.0,
                rect.size.width.0.max(0.0),
                arrow.size.width.0.max(0.0),
                arrow.padding.left.0.max(0.0),
                arrow.padding.right.0.max(0.0),
            )
        } else {
            (
                anchor.origin.y.0,
                anchor.size.height.0.max(0.0),
                rect.origin.y.0,
                rect.size.height.0.max(0.0),
                arrow.size.height.0.max(0.0),
                arrow.padding.top.0.max(0.0),
                arrow.padding.bottom.0.max(0.0),
            )
        };

    let rect_len = if rect_len.is_finite() { rect_len } else { 0.0 };
    let arrow_len = if arrow_len.is_finite() {
        arrow_len
    } else {
        0.0
    };

    let min = pad_start;
    let max = (rect_len - arrow_len - pad_end).max(min);

    let anchor_center = anchor_start + anchor_len * 0.5;

    let mut desired_offset = anchor_center - rect_start - arrow_len * 0.5;
    if !desired_offset.is_finite() {
        desired_offset = min;
    }

    let mut offset = desired_offset.clamp(min, max);
    let mut alignment_offset = 0.0;

    // Floating/Radix behavior: for aligned placements, if the reference is too small to keep the
    // arrow pointing at it due to padding constraints, shift the panel along the alignment axis.
    if align != Align::Center && (desired_offset - offset).abs() > 0.01 {
        let pad = if desired_offset < min {
            pad_start
        } else {
            pad_end
        };
        let should_add_offset = anchor_len * 0.5 - pad - arrow_len * 0.5 < 0.0;

        if should_add_offset {
            alignment_offset = desired_offset - offset;

            if is_vertical {
                rect.origin.x = Px(rect.origin.x.0 + alignment_offset);
            } else {
                rect.origin.y = Px(rect.origin.y.0 + alignment_offset);
            }

            *rect = clamp_rect_to_outer(outer, *rect);

            // Recompute after shifting/clamping.
            let rect_start = if is_vertical {
                rect.origin.x.0
            } else {
                rect.origin.y.0
            };

            desired_offset = anchor_center - rect_start - arrow_len * 0.5;
            if !desired_offset.is_finite() {
                desired_offset = min;
            }
            offset = desired_offset.clamp(min, max);
        }
    }

    ArrowLayout {
        side: opposite_side(placement_side),
        offset: Px(offset),
        alignment_offset: Px(alignment_offset),
    }
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
    fn sized_variant_prefers_side_with_less_main_axis_overflow() {
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
        // Available space above = 150 - 8 = 142
        // Neither side fits the desired height (180), so the solver should prefer the side with
        // less main-axis overflow (top in this case) and then clamp to the available space.
        assert_eq!(placed.size.height, Px(142.0));
        assert!(
            placed.origin.y.0 + placed.size.height.0 <= anchor.origin.y.0,
            "expected placement to be above the anchor"
        );
        assert!(outer.contains(placed.origin));
    }

    #[test]
    fn sized_variant_prefers_side_with_more_available_space_for_oversized_content() {
        let outer = r(0.0, 0.0, 200.0, 200.0);
        let anchor = r(10.0, 150.0, 40.0, 10.0);
        // Simulate a "greedy" widget measured with an unconstrained probe.
        let desired = Size::new(Px(120.0), Px(1.0e9));

        let placed = anchored_panel_bounds_sized(
            outer,
            anchor,
            desired,
            Px(8.0),
            Side::Bottom,
            Align::Start,
        );

        // Available space below = 200 - (150 + 10 + 8) = 32
        // Available space above = 150 - 8 = 142
        // We should choose the side with more available space (top) and clamp to it.
        assert_eq!(placed.size.height, Px(142.0));
        assert!(
            placed.origin.y.0 + placed.size.height.0 <= anchor.origin.y.0,
            "expected placement to be above the anchor"
        );
        assert!(outer.contains(placed.origin));
    }

    #[test]
    fn offset_applies_cross_axis_skidding() {
        let outer = r(0.0, 0.0, 400.0, 400.0);
        let anchor = r(100.0, 100.0, 40.0, 10.0);
        let content = Size::new(Px(120.0), Px(80.0));

        let base = anchored_panel_layout_ex(
            outer,
            anchor,
            content,
            Px(8.0),
            Side::Bottom,
            Align::Start,
            AnchoredPanelOptions::default(),
        );

        let skidded = anchored_panel_layout_ex(
            outer,
            anchor,
            content,
            Px(8.0),
            Side::Bottom,
            Align::Start,
            AnchoredPanelOptions {
                offset: Offset {
                    cross_axis: Px(12.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        assert_eq!(skidded.rect.origin.x, Px(base.rect.origin.x.0 + 12.0));
        assert_eq!(skidded.rect.origin.y, base.rect.origin.y);
    }

    #[test]
    fn alignment_axis_inverts_for_end_and_rtl_vertical() {
        let outer = r(0.0, 0.0, 400.0, 400.0);
        let anchor = r(100.0, 100.0, 40.0, 10.0);
        let content = Size::new(Px(120.0), Px(80.0));

        let ltr = anchored_panel_layout_ex(
            outer,
            anchor,
            content,
            Px(8.0),
            Side::Bottom,
            Align::End,
            AnchoredPanelOptions {
                direction: LayoutDirection::Ltr,
                offset: Offset {
                    alignment_axis: Some(Px(10.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        let rtl = anchored_panel_layout_ex(
            outer,
            anchor,
            content,
            Px(8.0),
            Side::Bottom,
            Align::End,
            AnchoredPanelOptions {
                direction: LayoutDirection::Rtl,
                offset: Offset {
                    alignment_axis: Some(Px(10.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        // For vertical placements, the `alignment_axis` flips for End and flips again under RTL.
        assert_eq!(rtl.rect.origin.x, Px(ltr.rect.origin.x.0 + 20.0));
    }

    #[test]
    fn arrow_centers_when_possible() {
        let outer = r(0.0, 0.0, 800.0, 800.0);
        let anchor = r(300.0, 200.0, 100.0, 20.0);
        let content = Size::new(Px(200.0), Px(120.0));

        let layout = anchored_panel_layout_ex(
            outer,
            anchor,
            content,
            Px(8.0),
            Side::Bottom,
            Align::Center,
            AnchoredPanelOptions {
                arrow: Some(ArrowOptions {
                    size: Size::new(Px(12.0), Px(12.0)),
                    padding: Edges::all(Px(8.0)),
                }),
                ..Default::default()
            },
        );

        let arrow = layout.arrow.expect("arrow layout");
        assert_eq!(arrow.side, Side::Top);
        assert!((arrow.offset.0 - 94.0).abs() < 0.1);
        assert_eq!(arrow.alignment_offset, Px(0.0));
    }

    #[test]
    fn arrow_clamps_to_padding_near_edge() {
        let outer = r(0.0, 0.0, 220.0, 200.0);
        let anchor = r(0.0, 50.0, 10.0, 10.0);
        let content = Size::new(Px(200.0), Px(80.0));

        let layout = anchored_panel_layout_ex(
            outer,
            anchor,
            content,
            Px(4.0),
            Side::Bottom,
            Align::Start,
            AnchoredPanelOptions {
                arrow: Some(ArrowOptions {
                    size: Size::new(Px(12.0), Px(12.0)),
                    padding: Edges::all(Px(16.0)),
                }),
                ..Default::default()
            },
        );

        let arrow = layout.arrow.expect("arrow layout");
        assert!(arrow.offset.0 >= 16.0 - 0.01);
    }
}
