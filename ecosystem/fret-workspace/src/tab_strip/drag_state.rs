use std::sync::Arc;

use fret_core::{Point, PointerId, Rect};
use fret_runtime::{Model, ModelStore, TickId};
use fret_ui::{ElementContext, UiHost};

use crate::tab_drag::WorkspaceTabHitRect;

use super::kernel::WorkspaceTabStripDropTarget;

#[derive(Debug, Clone)]
pub(super) struct WorkspaceTabStripClosePress {
    pub(super) pointer_id: PointerId,
    pub(super) start_position: Point,
    pub(super) start_position_window: Option<Point>,
    pub(super) close_command: fret_runtime::CommandId,
    pub(super) pane_activate_cmd: Option<fret_runtime::CommandId>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct WorkspaceTabStripDragState {
    pub(super) pointer: Option<PointerId>,
    pub(super) start_tick: TickId,
    pub(super) start_position: Point,
    pub(super) start_position_window: Option<Point>,
    pub(super) dragged_tab: Option<Arc<str>>,
    pub(super) dragging: bool,
    pub(super) drop_target: WorkspaceTabStripDropTarget,
    pub(super) close_press: Option<WorkspaceTabStripClosePress>,
    pub(super) tab_rects: Vec<WorkspaceTabHitRect>,
    pub(super) pinned_boundary_rect: Option<Rect>,
    pub(super) end_drop_target_rect: Option<Rect>,
    pub(super) scroll_viewport_rect: Option<Rect>,
    pub(super) overflow_control_rect: Option<Rect>,
    pub(super) scroll_left_control_rect: Option<Rect>,
    pub(super) scroll_right_control_rect: Option<Rect>,
}

#[derive(Debug, Clone)]
pub(super) struct WorkspaceTabStripDragSnapshot {
    pub(super) start_tick: TickId,
    pub(super) start_position: Point,
    pub(super) start_position_window: Option<Point>,
    pub(super) dragging: bool,
    pub(super) dragged_tab: Option<Arc<str>>,
    pub(super) tab_rects: Vec<WorkspaceTabHitRect>,
    pub(super) pinned_boundary_rect: Option<Rect>,
    pub(super) end_drop_target_rect: Option<Rect>,
    pub(super) scroll_viewport_rect: Option<Rect>,
    pub(super) overflow_control_rect: Option<Rect>,
    pub(super) scroll_left_control_rect: Option<Rect>,
    pub(super) scroll_right_control_rect: Option<Rect>,
}

pub(super) fn read_drag_snapshot_for_pointer(
    models: &mut ModelStore,
    drag_model: &Model<WorkspaceTabStripDragState>,
    pointer_id: PointerId,
) -> Option<WorkspaceTabStripDragSnapshot> {
    let mut out: Option<WorkspaceTabStripDragSnapshot> = None;
    let _ = models.read(drag_model, |st| {
        if st.pointer != Some(pointer_id) {
            return;
        }
        out = Some(WorkspaceTabStripDragSnapshot {
            start_tick: st.start_tick,
            start_position: st.start_position,
            start_position_window: st.start_position_window,
            dragging: st.dragging,
            dragged_tab: st.dragged_tab.clone(),
            tab_rects: st.tab_rects.clone(),
            pinned_boundary_rect: st.pinned_boundary_rect,
            end_drop_target_rect: st.end_drop_target_rect,
            scroll_viewport_rect: st.scroll_viewport_rect,
            overflow_control_rect: st.overflow_control_rect,
            scroll_left_control_rect: st.scroll_left_control_rect,
            scroll_right_control_rect: st.scroll_right_control_rect,
        });
    });
    out
}

#[track_caller]
pub(super) fn get_drag_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<WorkspaceTabStripDragState> {
    cx.local_model(WorkspaceTabStripDragState::default)
}
