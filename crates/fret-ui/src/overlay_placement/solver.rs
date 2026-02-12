use fret_core::{Point, Px, Rect, Size};

use super::types::*;
use super::util::{inset_rect, intersect_rect};

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
    let outer = apply_collision_options(outer, options.collision);
    let content = Size::new(Px(content.width.0.max(0.0)), Px(content.height.0.max(0.0)));
    let gap = Px((side_offset.0 + options.offset.main_axis.0).max(0.0));

    let preferred_origin = anchored_origin_ex(anchor, content, gap, preferred_side, align, options);
    let preferred = Rect::new(preferred_origin, content);
    if side_fits_without_clamp(outer, preferred, preferred_side) {
        return finalize_layout(outer, anchor, preferred, preferred_side, align, options);
    }

    let flipped_side = opposite_side(preferred_side);
    let flipped_origin = anchored_origin_ex(anchor, content, gap, flipped_side, align, options);
    let flipped = Rect::new(flipped_origin, content);
    if side_fits_without_clamp(outer, flipped, flipped_side) {
        return finalize_layout(outer, anchor, flipped, flipped_side, align, options);
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

    finalize_layout(outer, anchor, chosen, chosen_side, align, options)
}

/// Like [`anchored_panel_layout_ex`], but also returns a debug trace describing solver decisions.
///
/// This is intended for diagnostics evidence and MUST NOT be used as a normative contract surface.
pub fn anchored_panel_layout_ex_with_trace(
    outer: Rect,
    anchor: Rect,
    content: Size,
    side_offset: Px,
    preferred_side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> (AnchoredPanelLayout, AnchoredPanelLayoutTrace) {
    let outer_input = outer;
    let outer = apply_collision_options(outer, options.collision);
    let desired = Size::new(Px(content.width.0.max(0.0)), Px(content.height.0.max(0.0)));
    let gap = Px((side_offset.0 + options.offset.main_axis.0).max(0.0));

    let preferred_origin = anchored_origin_ex(anchor, desired, gap, preferred_side, align, options);
    let preferred = Rect::new(preferred_origin, desired);
    let preferred_fits_without_main_clamp =
        side_fits_without_clamp(outer, preferred, preferred_side);

    let flipped_side = opposite_side(preferred_side);
    let flipped_origin = anchored_origin_ex(anchor, desired, gap, flipped_side, align, options);
    let flipped = Rect::new(flipped_origin, desired);
    let flipped_fits_without_main_clamp = side_fits_without_clamp(outer, flipped, flipped_side);

    let preferred_available_main_px = available_main_for_side(outer, anchor, gap, preferred_side);
    let flipped_available_main_px = available_main_for_side(outer, anchor, gap, flipped_side);

    let (chosen_side, chosen) = if preferred_fits_without_main_clamp {
        (preferred_side, preferred)
    } else if flipped_fits_without_main_clamp {
        (flipped_side, flipped)
    } else {
        // Neither side fits cleanly on the main axis. Choose the candidate that minimizes main-axis
        // overflow, breaking ties by total overflow, then clamp into `outer`.
        let preferred_overflow = overflow_amount(outer, preferred);
        let flipped_overflow = overflow_amount(outer, flipped);

        let preferred_main = main_axis_overflow(preferred_overflow, preferred_side);
        let flipped_main = main_axis_overflow(flipped_overflow, flipped_side);

        let preferred_total = total_overflow(preferred_overflow);
        let flipped_total = total_overflow(flipped_overflow);

        if (flipped_main, flipped_total) < (preferred_main, preferred_total) {
            (flipped_side, flipped)
        } else {
            (preferred_side, preferred)
        }
    };

    let (layout, rect_after_shift, shift_delta) =
        finalize_layout_with_trace(outer, anchor, chosen, chosen_side, align, options);

    let trace = AnchoredPanelLayoutTrace {
        outer_input,
        outer_collision: outer,
        anchor,
        desired,
        side_offset,
        preferred_side,
        align,
        options,
        gap,
        preferred_rect: preferred,
        flipped_rect: flipped,
        preferred_fits_without_main_clamp,
        flipped_fits_without_main_clamp,
        preferred_available_main_px,
        flipped_available_main_px,
        chosen_side,
        chosen_rect: chosen,
        rect_after_shift,
        shift_delta,
        layout,
    };

    (layout, trace)
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
    let outer = apply_collision_options(outer, options.collision);
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
            Rect::new(origin, size),
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
            Rect::new(origin, size),
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
        Rect::new(origin, size),
        chosen_side,
        align,
        options,
    )
}

/// Like [`anchored_panel_layout_sized_ex`], but also returns a debug trace describing solver decisions.
///
/// This is intended for diagnostics evidence and MUST NOT be used as a normative contract surface.
pub fn anchored_panel_layout_sized_ex_with_trace(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    side_offset: Px,
    preferred_side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> (AnchoredPanelLayout, AnchoredPanelLayoutTrace) {
    let outer_input = outer;
    let outer = apply_collision_options(outer, options.collision);
    let desired = Size::new(Px(desired.width.0.max(0.0)), Px(desired.height.0.max(0.0)));
    let gap = Px((side_offset.0 + options.offset.main_axis.0).max(0.0));

    let preferred_unclamped_origin =
        anchored_origin_ex(anchor, desired, gap, preferred_side, align, options);
    let preferred_unclamped = Rect::new(preferred_unclamped_origin, desired);
    let preferred_fits_without_main_clamp =
        side_fits_without_clamp(outer, preferred_unclamped, preferred_side);

    let flipped_side = opposite_side(preferred_side);
    let flipped_unclamped_origin =
        anchored_origin_ex(anchor, desired, gap, flipped_side, align, options);
    let flipped_unclamped = Rect::new(flipped_unclamped_origin, desired);
    let flipped_fits_without_main_clamp =
        side_fits_without_clamp(outer, flipped_unclamped, flipped_side);

    let preferred_available_main_px = available_main_for_side(outer, anchor, gap, preferred_side);
    let flipped_available_main_px = available_main_for_side(outer, anchor, gap, flipped_side);

    let (chosen_side, chosen_rect) = if preferred_fits_without_main_clamp {
        let size = clamp_size_for_side(outer, anchor, desired, gap, preferred_side);
        let origin = anchored_origin_ex(anchor, size, gap, preferred_side, align, options);
        (preferred_side, Rect::new(origin, size))
    } else if flipped_fits_without_main_clamp {
        let size = clamp_size_for_side(outer, anchor, desired, gap, flipped_side);
        let origin = anchored_origin_ex(anchor, size, gap, flipped_side, align, options);
        (flipped_side, Rect::new(origin, size))
    } else {
        // Neither side fits on the main axis given the desired size. Prefer the side with more
        // available main-axis space (Floating UI `flip` best-fit behavior), breaking ties by the
        // overflow heuristic used by the unsized solver.
        let chosen_side = if flipped_available_main_px > preferred_available_main_px {
            flipped_side
        } else if preferred_available_main_px > flipped_available_main_px {
            preferred_side
        } else {
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
        (chosen_side, Rect::new(origin, size))
    };

    let (layout, rect_after_shift, shift_delta) =
        finalize_layout_with_trace(outer, anchor, chosen_rect, chosen_side, align, options);

    let trace = AnchoredPanelLayoutTrace {
        outer_input,
        outer_collision: outer,
        anchor,
        desired,
        side_offset,
        preferred_side,
        align,
        options,
        gap,
        preferred_rect: preferred_unclamped,
        flipped_rect: flipped_unclamped,
        preferred_fits_without_main_clamp,
        flipped_fits_without_main_clamp,
        preferred_available_main_px,
        flipped_available_main_px,
        chosen_side,
        chosen_rect,
        rect_after_shift,
        shift_delta,
        layout,
    };

    (layout, trace)
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
    let is_vertical = matches!(side, Side::Top | Side::Bottom);
    let rtl = options.direction == LayoutDirection::Rtl;

    // Interpret Start/End as logical alignment for vertical placements (Top/Bottom).
    // This matches Radix/Floating's `dir` behavior: `start` flips under RTL.
    let physical_align = if is_vertical && rtl {
        match align {
            Align::Start => Align::End,
            Align::End => Align::Start,
            Align::Center => Align::Center,
        }
    } else {
        align
    };

    // Important: `apply_cross_axis_offset` must use the logical `align` (not the physical one),
    // because `Offset.alignment_axis` flips under RTL for vertical placements.
    let origin = anchored_origin(anchor, content, gap, side, physical_align);
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
    clamp_rect_to_outer_axes(outer, inner, true, true)
}

fn clamp_rect_to_outer_axes(outer: Rect, inner: Rect, clamp_x: bool, clamp_y: bool) -> Rect {
    let min_x = outer.origin.x.0;
    let min_y = outer.origin.y.0;
    let max_x = (outer.origin.x.0 + outer.size.width.0 - inner.size.width.0).max(min_x);
    let max_y = (outer.origin.y.0 + outer.size.height.0 - inner.size.height.0).max(min_y);

    let mut x = inner.origin.x.0;
    let mut y = inner.origin.y.0;
    if !x.is_finite() {
        x = min_x;
    }
    if !y.is_finite() {
        y = min_y;
    }

    if clamp_x {
        x = x.clamp(min_x, max_x);
    }
    if clamp_y {
        y = y.clamp(min_y, max_y);
    }
    Rect::new(Point::new(Px(x), Px(y)), inner.size)
}

fn apply_collision_options(outer: Rect, collision: CollisionOptions) -> Rect {
    let outer = if let Some(boundary) = collision.boundary {
        intersect_rect(outer, boundary)
    } else {
        outer
    };
    inset_rect(outer, collision.padding)
}

fn finalize_layout_with_trace(
    outer: Rect,
    anchor: Rect,
    rect: Rect,
    side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> (AnchoredPanelLayout, Rect, Point) {
    let rect_after_shift =
        shift_rect_with_sticky(outer, anchor, rect, side, options.sticky, options.shift);
    let shift_delta = Point::new(
        Px(rect_after_shift.origin.x.0 - rect.origin.x.0),
        Px(rect_after_shift.origin.y.0 - rect.origin.y.0),
    );

    let mut final_rect = rect_after_shift;
    let arrow = options.arrow.map(|arrow| {
        apply_arrow_layout(
            ArrowLayoutArgs {
                outer,
                anchor,
                placement_side: side,
                align,
                sticky: options.sticky,
                shift: options.shift,
                arrow,
            },
            &mut final_rect,
        )
    });

    (
        AnchoredPanelLayout {
            rect: final_rect,
            side,
            align,
            arrow,
        },
        rect_after_shift,
        shift_delta,
    )
}

fn finalize_layout(
    outer: Rect,
    anchor: Rect,
    mut rect: Rect,
    side: Side,
    align: Align,
    options: AnchoredPanelOptions,
) -> AnchoredPanelLayout {
    rect = shift_rect_with_sticky(outer, anchor, rect, side, options.sticky, options.shift);
    let arrow = options.arrow.map(|arrow| {
        apply_arrow_layout(
            ArrowLayoutArgs {
                outer,
                anchor,
                placement_side: side,
                align,
                sticky: options.sticky,
                shift: options.shift,
                arrow,
            },
            &mut rect,
        )
    });

    AnchoredPanelLayout {
        rect,
        side,
        align,
        arrow,
    }
}

fn shift_rect_with_sticky(
    outer: Rect,
    anchor: Rect,
    rect: Rect,
    side: Side,
    sticky: StickyMode,
    shift: ShiftOptions,
) -> Rect {
    let is_vertical = matches!(side, Side::Top | Side::Bottom);
    let clamp_x = if is_vertical {
        shift.cross_axis
    } else {
        shift.main_axis
    };
    let clamp_y = if is_vertical {
        shift.main_axis
    } else {
        shift.cross_axis
    };

    let mut rect = clamp_rect_to_outer_axes(outer, rect, clamp_x, clamp_y);
    if sticky == StickyMode::Always || !shift.cross_axis {
        return rect;
    }

    // Floating UI `limitShift()` (as used by Radix `sticky="partial"`) constrains the shift so the
    // floating element does not detach from the reference on the alignment axis, even if that
    // means overflowing the collision boundary.
    let anchor_len = if is_vertical {
        anchor.size.width.0.max(0.0)
    } else {
        anchor.size.height.0.max(0.0)
    };
    let rect_len = if is_vertical {
        rect.size.width.0.max(0.0)
    } else {
        rect.size.height.0.max(0.0)
    };
    let anchor_start = if is_vertical {
        anchor.origin.x.0
    } else {
        anchor.origin.y.0
    };

    let min = anchor_start - rect_len;
    let max = anchor_start + anchor_len;

    if is_vertical {
        let mut x = rect.origin.x.0;
        if !x.is_finite() {
            x = 0.0;
        }
        rect.origin.x = Px(x.clamp(min, max));
    } else {
        let mut y = rect.origin.y.0;
        if !y.is_finite() {
            y = 0.0;
        }
        rect.origin.y = Px(y.clamp(min, max));
    }

    rect
}

struct ArrowLayoutArgs {
    outer: Rect,
    anchor: Rect,
    placement_side: Side,
    align: Align,
    sticky: StickyMode,
    shift: ShiftOptions,
    arrow: ArrowOptions,
}

fn apply_arrow_layout(args: ArrowLayoutArgs, rect: &mut Rect) -> ArrowLayout {
    let ArrowLayoutArgs {
        outer,
        anchor,
        placement_side,
        align,
        sticky,
        shift,
        arrow,
    } = args;

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

            *rect = shift_rect_with_sticky(outer, anchor, *rect, placement_side, sticky, shift);

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
        center_offset: Px(desired_offset - offset),
    }
}
