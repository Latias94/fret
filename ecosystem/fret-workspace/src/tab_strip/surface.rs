use fret_core::{Point, Rect};

use crate::tab_drag::WorkspaceTabHitRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceTabStripSurface {
    Outside,
    OverflowControl,
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

pub(crate) fn classify_workspace_tab_strip_surface(
    position: Point,
    dragged_tab_id: &str,
    tab_rects: &[WorkspaceTabHitRect],
    pinned_boundary_rect: Option<Rect>,
    end_drop_target_rect: Option<Rect>,
    scroll_viewport_rect: Option<Rect>,
    overflow_control_rect: Option<Rect>,
) -> WorkspaceTabStripSurface {
    // Explicit non-drop surfaces should win: they may overlap the scroll viewport in future
    // (e.g. button clusters living inside the tab bar).
    if overflow_control_rect.is_some_and(|r| rect_contains_point(r, position)) {
        return WorkspaceTabStripSurface::OverflowControl;
    }

    if pinned_boundary_rect.is_some_and(|rect| rect_contains_point(rect, position)) {
        return WorkspaceTabStripSurface::PinnedBoundary;
    }

    if end_drop_target_rect.is_some_and(|rect| rect_contains_point(rect, position)) {
        return WorkspaceTabStripSurface::HeaderSpace;
    }

    if let Some(viewport) = scroll_viewport_rect {
        if !rect_contains_point(viewport, position) {
            return WorkspaceTabStripSurface::Outside;
        }

        let mut max_right: Option<f32> = None;
        for r in tab_rects.iter().filter(|r| r.id.as_ref() != dragged_tab_id) {
            let right = r.rect.origin.x.0 + r.rect.size.width.0;
            max_right = Some(max_right.map_or(right, |prev| prev.max(right)));
        }

        if let Some(max_right) = max_right {
            if position.x.0 > max_right {
                return WorkspaceTabStripSurface::HeaderSpace;
            }
        } else {
            return WorkspaceTabStripSurface::HeaderSpace;
        }

        return WorkspaceTabStripSurface::TabsViewport;
    }

    // Fallback when the scroll viewport bounds are not available yet (first frames, synthetic
    // events): use the tab row y-band as a best-effort proxy.
    if tab_rects.is_empty() {
        return WorkspaceTabStripSurface::Outside;
    }

    let mut min_y: Option<f32> = None;
    let mut max_y: Option<f32> = None;
    for r in tab_rects {
        let top = r.rect.origin.y.0;
        let bottom = r.rect.origin.y.0 + r.rect.size.height.0;
        min_y = Some(min_y.map_or(top, |prev| prev.min(top)));
        max_y = Some(max_y.map_or(bottom, |prev| prev.max(bottom)));
    }
    let Some(min_y) = min_y else {
        return WorkspaceTabStripSurface::Outside;
    };
    let Some(max_y) = max_y else {
        return WorkspaceTabStripSurface::Outside;
    };

    if position.y.0 < min_y || position.y.0 > max_y {
        return WorkspaceTabStripSurface::Outside;
    }

    let mut max_right: Option<f32> = None;
    for r in tab_rects.iter().filter(|r| r.id.as_ref() != dragged_tab_id) {
        let right = r.rect.origin.x.0 + r.rect.size.width.0;
        max_right = Some(max_right.map_or(right, |prev| prev.max(right)));
    }
    if max_right.is_none_or(|max_right| position.x.0 > max_right) {
        return WorkspaceTabStripSurface::HeaderSpace;
    }

    WorkspaceTabStripSurface::TabsViewport
}
