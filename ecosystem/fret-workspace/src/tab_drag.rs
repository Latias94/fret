use std::sync::Arc;

use crate::layout::SplitSide;
use fret_core::{AppWindowId, Axis, Point, PointerId, Px, Rect};
use fret_dnd::{EdgeDropZone, compute_edge_drop_zone};
use fret_runtime::DragKindId;
use fret_ui_headless::tab_strip_drop_target::compute_tab_drop_target_midpoint;

pub use fret_ui_headless::tab_strip_drop_target::TabInsertionSide as WorkspaceTabInsertionSide;

/// Drag kind for cross-pane workspace tab drags.
pub const DRAG_KIND_WORKSPACE_TAB: DragKindId = DragKindId(0x57535F544142); // "WS_TAB"

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTabDropZone {
    Center,
    Left,
    Right,
    Up,
    Down,
}

fn edge_for_drop_zone(zone: WorkspaceTabDropZone) -> Option<EdgeDropZone> {
    match zone {
        WorkspaceTabDropZone::Left => Some(EdgeDropZone::Left),
        WorkspaceTabDropZone::Right => Some(EdgeDropZone::Right),
        WorkspaceTabDropZone::Up => Some(EdgeDropZone::Up),
        WorkspaceTabDropZone::Down => Some(EdgeDropZone::Down),
        WorkspaceTabDropZone::Center => None,
    }
}

fn drop_zone_for_edge(edge: EdgeDropZone) -> WorkspaceTabDropZone {
    match edge {
        EdgeDropZone::Left => WorkspaceTabDropZone::Left,
        EdgeDropZone::Right => WorkspaceTabDropZone::Right,
        EdgeDropZone::Up => WorkspaceTabDropZone::Up,
        EdgeDropZone::Down => WorkspaceTabDropZone::Down,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceTabHitRect {
    pub id: Arc<str>,
    pub rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkspacePaneDragGeometry {
    pub bounds: Rect,
    pub edge_margin: Px,
    pub edge_hysteresis: Px,
}

pub fn compute_workspace_tab_drop_zone_for_position(
    geom: WorkspacePaneDragGeometry,
    position: Point,
    previous: Option<WorkspaceTabDropZone>,
) -> WorkspaceTabDropZone {
    let prev_edge = previous.and_then(edge_for_drop_zone);
    compute_edge_drop_zone(
        geom.bounds,
        position,
        geom.edge_margin,
        prev_edge,
        geom.edge_hysteresis,
    )
    .map(drop_zone_for_edge)
    .unwrap_or(WorkspaceTabDropZone::Center)
}

pub fn workspace_tab_strip_row_contains_pointer(
    pointer: Point,
    tab_rects: &[WorkspaceTabHitRect],
) -> bool {
    if tab_rects.is_empty() {
        return false;
    }
    let mut min_y: Option<f32> = None;
    let mut max_y: Option<f32> = None;
    for r in tab_rects {
        let top = r.rect.origin.y.0;
        let bottom = r.rect.origin.y.0 + r.rect.size.height.0;
        min_y = Some(min_y.map_or(top, |prev| prev.min(top)));
        max_y = Some(max_y.map_or(bottom, |prev| prev.max(bottom)));
    }
    min_y.is_some_and(|min_y| {
        max_y.is_some_and(|max_y| pointer.y.0 >= min_y && pointer.y.0 <= max_y)
    })
}

pub fn workspace_tab_pane_id_at_position(
    state: &WorkspaceTabDragState,
    position: Point,
) -> Option<Arc<str>> {
    state
        .pane_geometry
        .iter()
        .find(|(_, geom)| geom.bounds.contains(position))
        .map(|(id, _)| id.clone())
}

pub fn workspace_tab_drop_zone_for_pane(
    state: &WorkspaceTabDragState,
    pane_id: &str,
    position: Point,
) -> WorkspaceTabDropZone {
    let mut zone = state
        .pane_geometry
        .iter()
        .find(|(id, _)| id.as_ref() == pane_id)
        .map(|(_, geom)| {
            compute_workspace_tab_drop_zone_for_position(*geom, position, state.hovered_zone)
        })
        .unwrap_or_else(|| state.hovered_zone.unwrap_or(WorkspaceTabDropZone::Center));

    if zone != WorkspaceTabDropZone::Center
        && workspace_tab_strip_row_contains_pointer(position, &state.hovered_pane_tab_rects)
    {
        zone = WorkspaceTabDropZone::Center;
    }

    zone
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceTabDropIntent {
    None,
    MoveToPane {
        source: Arc<str>,
        dragged_tab: Arc<str>,
        target: Arc<str>,
    },
    InsertToPane {
        source: Arc<str>,
        dragged_tab: Arc<str>,
        target: Arc<str>,
        target_tab: Arc<str>,
        side: WorkspaceTabInsertionSide,
    },
    SplitAndMove {
        source: Arc<str>,
        dragged_tab: Arc<str>,
        target: Arc<str>,
        axis: Axis,
        side: SplitSide,
    },
}

pub fn resolve_workspace_tab_drop_intent(
    state: &WorkspaceTabDragState,
    target_pane: &Arc<str>,
    zone: WorkspaceTabDropZone,
) -> WorkspaceTabDropIntent {
    let Some(source) = state.source_pane.clone() else {
        return WorkspaceTabDropIntent::None;
    };
    let Some(dragged_tab) = state.dragged_tab.clone() else {
        return WorkspaceTabDropIntent::None;
    };

    match zone {
        WorkspaceTabDropZone::Center => {
            if source.as_ref() == target_pane.as_ref() {
                return WorkspaceTabDropIntent::None;
            }

            match (state.hovered_tab.clone(), state.hovered_tab_side) {
                (Some(target_tab), Some(side)) => WorkspaceTabDropIntent::InsertToPane {
                    source,
                    dragged_tab,
                    target: target_pane.clone(),
                    target_tab,
                    side,
                },
                _ => WorkspaceTabDropIntent::MoveToPane {
                    source,
                    dragged_tab,
                    target: target_pane.clone(),
                },
            }
        }
        WorkspaceTabDropZone::Left => WorkspaceTabDropIntent::SplitAndMove {
            source,
            dragged_tab,
            target: target_pane.clone(),
            axis: Axis::Horizontal,
            side: SplitSide::First,
        },
        WorkspaceTabDropZone::Right => WorkspaceTabDropIntent::SplitAndMove {
            source,
            dragged_tab,
            target: target_pane.clone(),
            axis: Axis::Horizontal,
            side: SplitSide::Second,
        },
        WorkspaceTabDropZone::Up => WorkspaceTabDropIntent::SplitAndMove {
            source,
            dragged_tab,
            target: target_pane.clone(),
            axis: Axis::Vertical,
            side: SplitSide::First,
        },
        WorkspaceTabDropZone::Down => WorkspaceTabDropIntent::SplitAndMove {
            source,
            dragged_tab,
            target: target_pane.clone(),
            axis: Axis::Vertical,
            side: SplitSide::Second,
        },
    }
}

pub fn compute_tab_drop_target(
    pointer: Point,
    dragged_tab: &str,
    rects: &[WorkspaceTabHitRect],
) -> Option<(Arc<str>, WorkspaceTabInsertionSide)> {
    let (target_ix, side) = compute_tab_drop_target_midpoint(
        pointer,
        rects,
        |r| r.rect,
        |r| r.id.as_ref() == dragged_tab,
    )?;

    let target = rects.get(target_ix)?.id.clone();
    Some((target, side))
}

#[derive(Debug, Default, Clone)]
pub struct WorkspaceTabDragState {
    pub pointer: Option<PointerId>,
    pub source_window: Option<AppWindowId>,
    pub source_pane: Option<Arc<str>>,
    pub dragged_tab: Option<Arc<str>>,
    pub dragged_tab_pinned: bool,
    pub hovered_pane: Option<Arc<str>>,
    pub hovered_zone: Option<WorkspaceTabDropZone>,
    pub hovered_tab: Option<Arc<str>>,
    pub hovered_tab_side: Option<WorkspaceTabInsertionSide>,
    pub hovered_pane_tab_rects: Vec<WorkspaceTabHitRect>,
    pub pane_geometry: Vec<(Arc<str>, WorkspacePaneDragGeometry)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arc(s: &str) -> Arc<str> {
        Arc::<str>::from(s)
    }

    #[test]
    fn resolve_workspace_tab_drop_intent_prefers_cached_tab_insertion() {
        let state = WorkspaceTabDragState {
            source_pane: Some(arc("p1")),
            dragged_tab: Some(arc("t1")),
            hovered_tab: Some(arc("t2")),
            hovered_tab_side: Some(WorkspaceTabInsertionSide::Before),
            ..Default::default()
        };

        let intent =
            resolve_workspace_tab_drop_intent(&state, &arc("p2"), WorkspaceTabDropZone::Center);
        assert_eq!(
            intent,
            WorkspaceTabDropIntent::InsertToPane {
                source: arc("p1"),
                dragged_tab: arc("t1"),
                target: arc("p2"),
                target_tab: arc("t2"),
                side: WorkspaceTabInsertionSide::Before,
            }
        );
    }

    #[test]
    fn resolve_workspace_tab_drop_intent_falls_back_to_move_to_pane() {
        let state = WorkspaceTabDragState {
            source_pane: Some(arc("p1")),
            dragged_tab: Some(arc("t1")),
            hovered_tab: None,
            hovered_tab_side: None,
            ..Default::default()
        };

        let intent =
            resolve_workspace_tab_drop_intent(&state, &arc("p2"), WorkspaceTabDropZone::Center);
        assert_eq!(
            intent,
            WorkspaceTabDropIntent::MoveToPane {
                source: arc("p1"),
                dragged_tab: arc("t1"),
                target: arc("p2"),
            }
        );
    }

    #[test]
    fn resolve_workspace_tab_drop_intent_noop_when_dropping_on_same_pane_center() {
        let state = WorkspaceTabDragState {
            source_pane: Some(arc("p1")),
            dragged_tab: Some(arc("t1")),
            hovered_tab: Some(arc("t2")),
            hovered_tab_side: Some(WorkspaceTabInsertionSide::After),
            ..Default::default()
        };

        let intent =
            resolve_workspace_tab_drop_intent(&state, &arc("p1"), WorkspaceTabDropZone::Center);
        assert_eq!(intent, WorkspaceTabDropIntent::None);
    }
}
