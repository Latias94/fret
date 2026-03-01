use fret_core::geometry::{Point, Rect};

/// Classifies which part of a tab strip / tab bar the pointer is currently over.
///
/// This is a pure geometry helper intended to keep "drop surface" semantics stable across refactors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabStripSurface {
    Outside,
    OverflowControl,
    ScrollControls,
    PinnedBoundary,
    TabsViewport,
    HeaderSpace,
}

fn rect_contains_point(rect: Rect, point: Point) -> bool {
    let left = rect.origin.x.0;
    let top = rect.origin.y.0;
    let right = rect.origin.x.0 + rect.size.width.0;
    let bottom = rect.origin.y.0 + rect.size.height.0;
    point.x.0 >= left && point.x.0 <= right && point.y.0 >= top && point.y.0 <= bottom
}

/// Classifies a tab strip surface using a "best effort" viewport-based hit-test.
///
/// Notes:
/// - "Explicit non-drop surfaces" (overflow controls, scroll controls) always win, even if they
///   overlap the scroll viewport.
/// - When the scroll viewport bounds are missing (first frames, synthetic events), this falls
///   back to a y-band heuristic based on the tab rects.
pub fn classify_tab_strip_surface<T>(
    position: Point,
    tab_rects: &[T],
    mut tab_rect: impl FnMut(&T) -> Rect,
    mut tab_is_dragged: impl FnMut(&T) -> bool,
    pinned_boundary_rect: Option<Rect>,
    header_space_rect: Option<Rect>,
    scroll_viewport_rect: Option<Rect>,
    overflow_control_rect: Option<Rect>,
    scroll_left_control_rect: Option<Rect>,
    scroll_right_control_rect: Option<Rect>,
) -> TabStripSurface {
    // Explicit non-drop surfaces should win: they may overlap the scroll viewport in future
    // (e.g. button clusters living inside the tab bar).
    if overflow_control_rect.is_some_and(|r| rect_contains_point(r, position)) {
        return TabStripSurface::OverflowControl;
    }

    if scroll_left_control_rect.is_some_and(|r| rect_contains_point(r, position))
        || scroll_right_control_rect.is_some_and(|r| rect_contains_point(r, position))
    {
        return TabStripSurface::ScrollControls;
    }

    if pinned_boundary_rect.is_some_and(|rect| rect_contains_point(rect, position)) {
        return TabStripSurface::PinnedBoundary;
    }

    if header_space_rect.is_some_and(|rect| rect_contains_point(rect, position)) {
        return TabStripSurface::HeaderSpace;
    }

    if let Some(viewport) = scroll_viewport_rect {
        if !rect_contains_point(viewport, position) {
            return TabStripSurface::Outside;
        }

        let mut max_right: Option<f32> = None;
        for r in tab_rects.iter().filter(|r| !tab_is_dragged(r)) {
            let rect = tab_rect(r);
            let right = rect.origin.x.0 + rect.size.width.0;
            max_right = Some(max_right.map_or(right, |prev| prev.max(right)));
        }

        if let Some(max_right) = max_right {
            if position.x.0 > max_right {
                return TabStripSurface::HeaderSpace;
            }
        } else {
            return TabStripSurface::HeaderSpace;
        }

        return TabStripSurface::TabsViewport;
    }

    // Fallback when the scroll viewport bounds are not available yet (first frames, synthetic
    // events): use the tab row y-band as a best-effort proxy.
    if tab_rects.is_empty() {
        return TabStripSurface::Outside;
    }

    let mut min_y: Option<f32> = None;
    let mut max_y: Option<f32> = None;
    for r in tab_rects {
        let rect = tab_rect(r);
        let top = rect.origin.y.0;
        let bottom = rect.origin.y.0 + rect.size.height.0;
        min_y = Some(min_y.map_or(top, |prev| prev.min(top)));
        max_y = Some(max_y.map_or(bottom, |prev| prev.max(bottom)));
    }
    let Some(min_y) = min_y else {
        return TabStripSurface::Outside;
    };
    let Some(max_y) = max_y else {
        return TabStripSurface::Outside;
    };

    if position.y.0 < min_y || position.y.0 > max_y {
        return TabStripSurface::Outside;
    }

    let mut max_right: Option<f32> = None;
    for r in tab_rects.iter().filter(|r| !tab_is_dragged(r)) {
        let rect = tab_rect(r);
        let right = rect.origin.x.0 + rect.size.width.0;
        max_right = Some(max_right.map_or(right, |prev| prev.max(right)));
    }
    if max_right.is_none_or(|max_right| position.x.0 > max_right) {
        return TabStripSurface::HeaderSpace;
    }

    TabStripSurface::TabsViewport
}

/// A simpler variant of [`classify_tab_strip_surface`] for callers that do not have per-tab bounds.
///
/// This performs only explicit-rect classification (controls, pinned boundary, header space) and
/// a scroll viewport containment check.
pub fn classify_tab_strip_surface_no_tabs(
    position: Point,
    pinned_boundary_rect: Option<Rect>,
    header_space_rect: Option<Rect>,
    scroll_viewport_rect: Option<Rect>,
    overflow_control_rect: Option<Rect>,
    scroll_left_control_rect: Option<Rect>,
    scroll_right_control_rect: Option<Rect>,
) -> TabStripSurface {
    if overflow_control_rect.is_some_and(|r| rect_contains_point(r, position)) {
        return TabStripSurface::OverflowControl;
    }

    if scroll_left_control_rect.is_some_and(|r| rect_contains_point(r, position))
        || scroll_right_control_rect.is_some_and(|r| rect_contains_point(r, position))
    {
        return TabStripSurface::ScrollControls;
    }

    if pinned_boundary_rect.is_some_and(|rect| rect_contains_point(rect, position)) {
        return TabStripSurface::PinnedBoundary;
    }

    if header_space_rect.is_some_and(|rect| rect_contains_point(rect, position)) {
        return TabStripSurface::HeaderSpace;
    }

    if scroll_viewport_rect.is_some_and(|rect| rect_contains_point(rect, position)) {
        return TabStripSurface::TabsViewport;
    }

    TabStripSurface::Outside
}
