use std::sync::Arc;

use fret_core::{Point, Px, Rect};

use crate::tab_drag::{WorkspaceTabHitRect, WorkspaceTabInsertionSide, compute_tab_drop_target};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) enum WorkspaceTabStripDropTarget {
    #[default]
    None,
    Tab(Arc<str>, WorkspaceTabInsertionSide),
    PinnedBoundary,
    End,
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
    scroll_viewport_rect: Option<Rect>,
) -> WorkspaceTabStripDropTarget {
    if pinned_boundary_rect.is_some_and(|rect| rect_contains_point(rect, position)) {
        return WorkspaceTabStripDropTarget::PinnedBoundary;
    }

    if let Some(viewport) = scroll_viewport_rect
        && rect_contains_point(viewport, position)
    {
        let mut max_right: Option<f32> = None;
        for r in tab_rects.iter().filter(|r| r.id.as_ref() != dragged_tab_id) {
            let right = r.rect.origin.x.0 + r.rect.size.width.0;
            max_right = Some(max_right.map_or(right, |prev| prev.max(right)));
        }

        if let Some(max_right) = max_right {
            // If the pointer is in the "header space" to the right of the last tab, prefer an
            // explicit "end of strip" target so shells can treat this as "insert at end" without
            // relying on the last tab's hit-testing.
            if position.x.0 > max_right {
                return WorkspaceTabStripDropTarget::End;
            }
        } else {
            // No measurable tabs (or only the dragged tab). When the pointer is still inside the
            // strip, treat it as an "end" target so reorders can fall back to "append".
            return WorkspaceTabStripDropTarget::End;
        }
    }

    compute_tab_drop_target(position, dragged_tab_id, tab_rects)
        .map(|(target, side)| WorkspaceTabStripDropTarget::Tab(target, side))
        .unwrap_or(WorkspaceTabStripDropTarget::None)
}

pub(crate) fn compute_tab_strip_edge_auto_scroll_delta_x(
    viewport: Rect,
    pointer: Point,
    current_offset_x: Px,
    max_offset_x: Px,
) -> Px {
    if max_offset_x.0 <= 0.5 {
        return Px(0.0);
    }

    let left = viewport.origin.x.0;
    let right = viewport.origin.x.0 + viewport.size.width.0;
    let x = pointer.x.0;

    if x < left || x > right {
        return Px(0.0);
    }

    let edge_zone = 24.0;
    let max_step = 18.0;

    let dist_left = x - left;
    let dist_right = right - x;

    if dist_left < edge_zone && current_offset_x.0 > 0.5 {
        let t = ((edge_zone - dist_left) / edge_zone).clamp(0.0, 1.0);
        return Px(-max_step * t);
    }

    if dist_right < edge_zone && (current_offset_x.0 + 0.5) < max_offset_x.0 {
        let t = ((edge_zone - dist_right) / edge_zone).clamp(0.0, 1.0);
        return Px(max_step * t);
    }

    Px(0.0)
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
            Some(rect(0.0, 0.0, 200.0, 20.0)),
        );
        assert!(
            matches!(target, WorkspaceTabStripDropTarget::PinnedBoundary),
            "expected pinned boundary to win"
        );
    }

    #[test]
    fn end_target_wins_in_header_space_right_of_last_tab() {
        let tab_rects = vec![
            WorkspaceTabHitRect {
                id: Arc::from("a"),
                rect: rect(0.0, 0.0, 100.0, 20.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("b"),
                rect: rect(110.0, 0.0, 100.0, 20.0),
            },
        ];
        let viewport = rect(0.0, 0.0, 400.0, 20.0);
        let pos = Point::new(Px(380.0), Px(10.0));

        let target = compute_workspace_tab_strip_drop_target(
            pos,
            "c",
            &tab_rects,
            None,
            Some(viewport),
        );
        assert!(matches!(target, WorkspaceTabStripDropTarget::End));
    }

    #[test]
    fn edge_auto_scroll_delta_respects_edges_and_clamps() {
        let viewport = rect(0.0, 0.0, 200.0, 20.0);
        let max = Px(300.0);

        // Near left edge scrolls left when we can.
        assert!(
            compute_tab_strip_edge_auto_scroll_delta_x(
                viewport,
                Point::new(Px(1.0), Px(10.0)),
                Px(10.0),
                max,
            )
            .0
                < 0.0
        );

        // At left edge but already at 0 doesn't scroll.
        assert_eq!(
            compute_tab_strip_edge_auto_scroll_delta_x(
                viewport,
                Point::new(Px(1.0), Px(10.0)),
                Px(0.0),
                max,
            ),
            Px(0.0)
        );

        // Near right edge scrolls right when we can.
        assert!(
            compute_tab_strip_edge_auto_scroll_delta_x(
                viewport,
                Point::new(Px(199.0), Px(10.0)),
                Px(10.0),
                max,
            )
            .0
                > 0.0
        );

        // Near right edge but already at max doesn't scroll.
        assert_eq!(
            compute_tab_strip_edge_auto_scroll_delta_x(
                viewport,
                Point::new(Px(199.0), Px(10.0)),
                Px(300.0),
                max,
            ),
            Px(0.0)
        );
    }
}
