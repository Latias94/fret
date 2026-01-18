use std::sync::Arc;

use fret_core::{AppWindowId, Point, PointerId, Rect};
use fret_runtime::DragKindId;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTabInsertionSide {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceTabHitRect {
    pub id: Arc<str>,
    pub rect: Rect,
}

pub fn compute_tab_drop_target(
    pointer: Point,
    dragged_tab: &str,
    rects: &[WorkspaceTabHitRect],
) -> Option<(Arc<str>, WorkspaceTabInsertionSide)> {
    let mut filtered: Vec<&WorkspaceTabHitRect> = rects
        .iter()
        .filter(|r| r.id.as_ref() != dragged_tab)
        .collect();

    if filtered.is_empty() {
        return None;
    }

    filtered.sort_by(|a, b| a.rect.origin.x.0.total_cmp(&b.rect.origin.x.0));

    for r in &filtered {
        let mid_x = r.rect.origin.x.0 + (r.rect.size.width.0 * 0.5);
        if pointer.x.0 < mid_x {
            return Some((r.id.clone(), WorkspaceTabInsertionSide::Before));
        }
    }

    let last = filtered.last()?;
    Some((last.id.clone(), WorkspaceTabInsertionSide::After))
}

#[derive(Debug, Default, Clone)]
pub struct WorkspaceTabDragState {
    pub pointer: Option<PointerId>,
    pub source_window: Option<AppWindowId>,
    pub source_pane: Option<Arc<str>>,
    pub dragged_tab: Option<Arc<str>>,
    pub hovered_pane: Option<Arc<str>>,
    pub hovered_zone: Option<WorkspaceTabDropZone>,
    pub hovered_tab: Option<Arc<str>>,
    pub hovered_tab_side: Option<WorkspaceTabInsertionSide>,
    pub hovered_pane_tab_rects: Vec<WorkspaceTabHitRect>,
}
