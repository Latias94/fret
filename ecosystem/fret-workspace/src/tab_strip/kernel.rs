use std::sync::Arc;

use fret_core::{Point, Px, Rect};
use fret_dnd::{AutoScrollConfig, compute_autoscroll_x};

use crate::tab_drag::{WorkspaceTabHitRect, WorkspaceTabInsertionSide, compute_tab_drop_target};

use super::surface::{WorkspaceTabStripSurface, classify_workspace_tab_strip_surface};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) enum WorkspaceTabStripDropTarget {
    #[default]
    None,
    Tab(Arc<str>, WorkspaceTabInsertionSide),
    PinnedBoundary,
    End,
}

pub(crate) fn compute_workspace_tab_strip_drop_target(
    position: Point,
    dragged_tab_id: &str,
    tab_rects: &[WorkspaceTabHitRect],
    pinned_boundary_rect: Option<Rect>,
    end_drop_target_rect: Option<Rect>,
    scroll_viewport_rect: Option<Rect>,
    overflow_control_rect: Option<Rect>,
    scroll_left_control_rect: Option<Rect>,
    scroll_right_control_rect: Option<Rect>,
) -> WorkspaceTabStripDropTarget {
    let surface = classify_workspace_tab_strip_surface(
        position,
        dragged_tab_id,
        tab_rects,
        pinned_boundary_rect,
        end_drop_target_rect,
        scroll_viewport_rect,
        overflow_control_rect,
        scroll_left_control_rect,
        scroll_right_control_rect,
    );

    match surface {
        WorkspaceTabStripSurface::Outside
        | WorkspaceTabStripSurface::OverflowControl
        | WorkspaceTabStripSurface::ScrollControls => WorkspaceTabStripDropTarget::None,
        WorkspaceTabStripSurface::PinnedBoundary => WorkspaceTabStripDropTarget::PinnedBoundary,
        WorkspaceTabStripSurface::HeaderSpace => WorkspaceTabStripDropTarget::End,
        WorkspaceTabStripSurface::TabsViewport => {
            compute_tab_drop_target(position, dragged_tab_id, tab_rects)
                .map(|(target, side)| WorkspaceTabStripDropTarget::Tab(target, side))
                .unwrap_or(WorkspaceTabStripDropTarget::None)
        }
    }
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

    if !viewport.contains(pointer) {
        return Px(0.0);
    }

    let cfg = AutoScrollConfig {
        margin_px: 24.0,
        min_speed_px_per_tick: 0.0,
        max_speed_px_per_tick: 18.0,
    };
    let dx = compute_autoscroll_x(cfg, viewport, pointer).unwrap_or(Px(0.0));
    if dx.0 < 0.0 && current_offset_x.0 <= 0.5 {
        return Px(0.0);
    }
    if dx.0 > 0.0 && (current_offset_x.0 + 0.5) >= max_offset_x.0 {
        return Px(0.0);
    }
    dx
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Px, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
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
            None,
            Some(rect(0.0, 0.0, 200.0, 20.0)),
            None,
            None,
            None,
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
            None,
            Some(viewport),
            None,
            None,
            None,
        );
        assert!(matches!(target, WorkspaceTabStripDropTarget::End));
    }

    #[test]
    fn end_target_triggers_when_pointer_is_inside_end_drop_rect() {
        let tab_rects = vec![WorkspaceTabHitRect {
            id: Arc::from("a"),
            rect: rect(0.0, 0.0, 100.0, 20.0),
        }];
        let viewport = rect(0.0, 0.0, 200.0, 20.0);
        let end_rect = rect(120.0, 0.0, 80.0, 20.0);
        let pos = Point::new(Px(150.0), Px(10.0));

        let target = compute_workspace_tab_strip_drop_target(
            pos,
            "b",
            &tab_rects,
            None,
            Some(end_rect),
            Some(viewport),
            None,
            None,
            None,
        );
        assert!(matches!(target, WorkspaceTabStripDropTarget::End));
    }

    #[test]
    fn tab_drop_target_can_be_computed_without_scroll_viewport_bounds() {
        let tab_rects = vec![WorkspaceTabHitRect {
            id: Arc::from("a"),
            rect: rect(0.0, 0.0, 100.0, 20.0),
        }];
        let pos = Point::new(Px(10.0), Px(10.0));

        let target = compute_workspace_tab_strip_drop_target(
            pos, "b", &tab_rects, None, None, None, None, None, None,
        );
        assert!(matches!(target, WorkspaceTabStripDropTarget::Tab(id, _) if id.as_ref() == "a"));
    }

    #[test]
    fn overflow_control_is_not_treated_as_header_space() {
        let tab_rects = vec![WorkspaceTabHitRect {
            id: Arc::from("a"),
            rect: rect(0.0, 0.0, 100.0, 20.0),
        }];
        let viewport = rect(0.0, 0.0, 300.0, 20.0);
        let overflow = rect(240.0, 0.0, 20.0, 20.0);
        let pos = Point::new(Px(250.0), Px(10.0));

        let target = compute_workspace_tab_strip_drop_target(
            pos,
            "b",
            &tab_rects,
            None,
            None,
            Some(viewport),
            Some(overflow),
            None,
            None,
        );
        assert!(matches!(target, WorkspaceTabStripDropTarget::None));
    }

    #[test]
    fn scroll_controls_are_not_treated_as_header_space_without_viewport_bounds() {
        let tab_rects = vec![WorkspaceTabHitRect {
            id: Arc::from("a"),
            rect: rect(0.0, 0.0, 100.0, 20.0),
        }];
        let scroll_right = rect(200.0, 0.0, 20.0, 20.0);
        let pos = Point::new(Px(210.0), Px(10.0));

        let target = compute_workspace_tab_strip_drop_target(
            pos,
            "b",
            &tab_rects,
            None,
            None,
            None,
            None,
            None,
            Some(scroll_right),
        );
        assert!(matches!(target, WorkspaceTabStripDropTarget::None));
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
            .0 < 0.0
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
            .0 > 0.0
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
