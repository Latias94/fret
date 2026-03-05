use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Point, Px, Rect};
use fret_dnd::{AutoScrollConfig, compute_autoscroll_x_clamped};
use fret_ui_headless::tab_strip_drop_target::{
    TabStripDropTarget as HeadlessTabStripDropTarget, compute_tab_strip_drop_target_midpoint,
};

use crate::tab_drag::{WorkspaceTabHitRect, WorkspaceTabInsertionSide};

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
    // Use the shared headless helper for the "tab vs boundary vs end" drop target.
    match compute_tab_strip_drop_target_midpoint(
        position,
        tab_rects,
        |r| r.rect,
        |r| r.id.as_ref() == dragged_tab_id,
        pinned_boundary_rect,
        end_drop_target_rect,
        scroll_viewport_rect,
        overflow_control_rect,
        scroll_left_control_rect,
        scroll_right_control_rect,
    ) {
        HeadlessTabStripDropTarget::None => WorkspaceTabStripDropTarget::None,
        HeadlessTabStripDropTarget::PinnedBoundary => WorkspaceTabStripDropTarget::PinnedBoundary,
        HeadlessTabStripDropTarget::End => WorkspaceTabStripDropTarget::End,
        HeadlessTabStripDropTarget::Tab { index, side } => tab_rects
            .get(index)
            .map(|r| WorkspaceTabStripDropTarget::Tab(r.id.clone(), side))
            .unwrap_or(WorkspaceTabStripDropTarget::None),
    }
}

pub(crate) fn compute_workspace_tab_strip_drop_target_with_pinned_row(
    position: Point,
    dragged_tab_id: &str,
    tab_rects: &[WorkspaceTabHitRect],
    pinned_by_id: &HashMap<Arc<str>, bool>,
    pinned_boundary_rect: Option<Rect>,
    end_drop_target_rect: Option<Rect>,
    scroll_viewport_rect: Option<Rect>,
    overflow_control_rect: Option<Rect>,
    scroll_left_control_rect: Option<Rect>,
    scroll_right_control_rect: Option<Rect>,
    two_row_pinned: bool,
) -> WorkspaceTabStripDropTarget {
    if !two_row_pinned {
        return compute_workspace_tab_strip_drop_target(
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
    }

    let dragged_is_pinned = pinned_by_id.get(dragged_tab_id).copied().unwrap_or(false);

    // When rendering pinned tabs in a separate row, midpoint-x drop math must be row-scoped.
    //
    // We treat the end-drop surface as an explicit escape hatch: it always resolves "drop at end"
    // within the dragged tab's group.
    if end_drop_target_rect.is_none_or(|r| !r.contains(position)) {
        let mut min_y: Option<f32> = None;
        let mut max_y: Option<f32> = None;
        for r in tab_rects.iter() {
            if r.id.as_ref() == dragged_tab_id {
                continue;
            }
            let pinned = pinned_by_id.get(r.id.as_ref()).copied().unwrap_or(false);
            if pinned != dragged_is_pinned {
                continue;
            }
            let top = r.rect.origin.y.0;
            let bottom = r.rect.origin.y.0 + r.rect.size.height.0;
            min_y = Some(min_y.map_or(top, |prev| prev.min(top)));
            max_y = Some(max_y.map_or(bottom, |prev| prev.max(bottom)));
        }

        if let (Some(min_y), Some(max_y)) = (min_y, max_y) {
            if position.y.0 < min_y || position.y.0 > max_y {
                return WorkspaceTabStripDropTarget::None;
            }
        } else {
            // No other tabs in the group means there's no meaningful in-row reorder target.
            return WorkspaceTabStripDropTarget::None;
        }
    }

    let candidates: Vec<WorkspaceTabHitRect> = tab_rects
        .iter()
        .filter(|r| pinned_by_id.get(r.id.as_ref()).copied().unwrap_or(false) == dragged_is_pinned)
        .cloned()
        .collect();

    compute_workspace_tab_strip_drop_target(
        position,
        dragged_tab_id,
        &candidates,
        pinned_boundary_rect,
        end_drop_target_rect,
        scroll_viewport_rect,
        overflow_control_rect,
        scroll_left_control_rect,
        scroll_right_control_rect,
    )
}

pub(crate) fn compute_workspace_tab_strip_drop_target_scoped_to_pointer_row(
    position: Point,
    dragged_tab_id: &str,
    tab_rects: &[WorkspaceTabHitRect],
    pinned_by_id: &HashMap<Arc<str>, bool>,
    pinned_boundary_rect: Option<Rect>,
    end_drop_target_rect: Option<Rect>,
    scroll_viewport_rect: Option<Rect>,
    overflow_control_rect: Option<Rect>,
    scroll_left_control_rect: Option<Rect>,
    scroll_right_control_rect: Option<Rect>,
    two_row_pinned: bool,
) -> WorkspaceTabStripDropTarget {
    if !two_row_pinned {
        return compute_workspace_tab_strip_drop_target(
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
    }

    if end_drop_target_rect.is_some_and(|r| r.contains(position)) {
        return compute_workspace_tab_strip_drop_target(
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
    }

    #[derive(Clone, Copy)]
    enum Row {
        Pinned,
        Unpinned,
    }

    fn band_for_row(
        tab_rects: &[WorkspaceTabHitRect],
        pinned_by_id: &HashMap<Arc<str>, bool>,
        row: Row,
        dragged_tab_id: &str,
    ) -> Option<(f32, f32)> {
        let mut min_y: Option<f32> = None;
        let mut max_y: Option<f32> = None;
        for r in tab_rects {
            if r.id.as_ref() == dragged_tab_id {
                continue;
            }
            let pinned = pinned_by_id.get(r.id.as_ref()).copied().unwrap_or(false);
            let in_row = match row {
                Row::Pinned => pinned,
                Row::Unpinned => !pinned,
            };
            if !in_row {
                continue;
            }
            let top = r.rect.origin.y.0;
            let bottom = r.rect.origin.y.0 + r.rect.size.height.0;
            min_y = Some(min_y.map_or(top, |prev| prev.min(top)));
            max_y = Some(max_y.map_or(bottom, |prev| prev.max(bottom)));
        }
        Some((min_y?, max_y?))
    }

    let pinned_band = band_for_row(tab_rects, pinned_by_id, Row::Pinned, dragged_tab_id);
    let unpinned_band = band_for_row(tab_rects, pinned_by_id, Row::Unpinned, dragged_tab_id);

    let row = if pinned_band
        .is_some_and(|(min_y, max_y)| position.y.0 >= min_y && position.y.0 <= max_y)
    {
        Some(Row::Pinned)
    } else if unpinned_band
        .is_some_and(|(min_y, max_y)| position.y.0 >= min_y && position.y.0 <= max_y)
    {
        Some(Row::Unpinned)
    } else {
        None
    };

    let Some(row) = row else {
        return WorkspaceTabStripDropTarget::None;
    };

    let candidates: Vec<WorkspaceTabHitRect> = tab_rects
        .iter()
        .filter(|r| {
            let pinned = pinned_by_id.get(r.id.as_ref()).copied().unwrap_or(false);
            match row {
                Row::Pinned => pinned,
                Row::Unpinned => !pinned,
            }
        })
        .cloned()
        .collect();

    compute_workspace_tab_strip_drop_target(
        position,
        dragged_tab_id,
        &candidates,
        pinned_boundary_rect,
        end_drop_target_rect,
        scroll_viewport_rect,
        overflow_control_rect,
        scroll_left_control_rect,
        scroll_right_control_rect,
    )
}

pub(crate) fn compute_tab_strip_edge_auto_scroll_delta_x(
    viewport: Rect,
    pointer: Point,
    current_offset_x: Px,
    max_offset_x: Px,
) -> Px {
    let cfg = AutoScrollConfig {
        margin_px: 24.0,
        min_speed_px_per_tick: 0.0,
        max_speed_px_per_tick: 18.0,
    };
    compute_autoscroll_x_clamped(cfg, viewport, pointer, current_offset_x, max_offset_x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Px, Size};
    use std::collections::HashMap;

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

    #[test]
    fn two_row_drop_target_is_scoped_to_dragged_group_row() {
        let tab_rects = vec![
            WorkspaceTabHitRect {
                id: Arc::from("p1"),
                rect: rect(0.0, 0.0, 80.0, 20.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("p2"),
                rect: rect(90.0, 0.0, 80.0, 20.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("u1"),
                rect: rect(0.0, 24.0, 80.0, 20.0),
            },
        ];
        let pinned_by_id: HashMap<Arc<str>, bool> = [
            (Arc::from("p1"), true),
            (Arc::from("p2"), true),
            (Arc::from("u1"), false),
        ]
        .into_iter()
        .collect();

        // Pointer is in the unpinned row band, so dragging a pinned tab should not resolve to a tab target.
        let pos = Point::new(Px(10.0), Px(30.0));
        let target = compute_workspace_tab_strip_drop_target_with_pinned_row(
            pos,
            "p1",
            &tab_rects,
            &pinned_by_id,
            None,
            None,
            Some(rect(0.0, 0.0, 400.0, 48.0)),
            None,
            None,
            None,
            true,
        );
        assert!(matches!(target, WorkspaceTabStripDropTarget::None));

        // Pointer is in the pinned row band, so it resolves against pinned tabs.
        let pos = Point::new(Px(10.0), Px(10.0));
        let target = compute_workspace_tab_strip_drop_target_with_pinned_row(
            pos,
            "p1",
            &tab_rects,
            &pinned_by_id,
            None,
            None,
            Some(rect(0.0, 0.0, 400.0, 48.0)),
            None,
            None,
            None,
            true,
        );
        assert!(matches!(
            target,
            WorkspaceTabStripDropTarget::Tab(id, _) if id.as_ref() == "p2"
        ));
    }
}
