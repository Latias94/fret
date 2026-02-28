use std::sync::Arc;

use fret_core::{Point, Rect};

use crate::tab_drag::{WorkspaceTabHitRect, WorkspaceTabInsertionSide, compute_tab_drop_target};

#[derive(Debug, Default, Clone)]
pub(crate) enum WorkspaceTabStripDropTarget {
    #[default]
    None,
    Tab(Arc<str>, WorkspaceTabInsertionSide),
    PinnedBoundary,
}

fn rect_contains_point(rect: Rect, point: Point) -> bool {
    let left = rect.origin.x.0;
    let top = rect.origin.y.0;
    let right = rect.origin.x.0 + rect.size.width.0;
    let bottom = rect.origin.y.0 + rect.size.height.0;
    point.x.0 >= left && point.x.0 <= right && point.y.0 >= top && point.y.0 <= bottom
}

pub(crate) fn compute_workspace_tab_strip_drop_target(
    position: Point,
    dragged_tab_id: &str,
    tab_rects: &[WorkspaceTabHitRect],
    pinned_boundary_rect: Option<Rect>,
) -> WorkspaceTabStripDropTarget {
    if pinned_boundary_rect.is_some_and(|rect| rect_contains_point(rect, position)) {
        return WorkspaceTabStripDropTarget::PinnedBoundary;
    }

    compute_tab_drop_target(position, dragged_tab_id, tab_rects)
        .map(|(target, side)| WorkspaceTabStripDropTarget::Tab(target, side))
        .unwrap_or(WorkspaceTabStripDropTarget::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Px, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(
            Point::new(Px(x), Px(y)),
            Size::new(Px(w), Px(h)),
        )
    }

    #[test]
    fn pinned_boundary_takes_precedence_over_tab_drop_target() {
        let tab_rects = vec![WorkspaceTabHitRect {
            id: Arc::from("a"),
            rect: rect(0.0, 0.0, 100.0, 20.0),
        }];
        let pinned_boundary = rect(10.0, 0.0, 8.0, 20.0);
        let pos = Point::new(Px(12.0), Px(10.0));

        let target = compute_workspace_tab_strip_drop_target(
            pos,
            "b",
            &tab_rects,
            Some(pinned_boundary),
        );
        assert!(
            matches!(target, WorkspaceTabStripDropTarget::PinnedBoundary),
            "expected pinned boundary to win"
        );
    }
}
