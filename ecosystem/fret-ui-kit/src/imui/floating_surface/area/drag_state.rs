use std::sync::Arc;

use fret_core::Point;
use fret_core::window::WindowMetricsService;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::FloatingAreaOptions;
use commit::commit_floating_area_drag_state;
pub(super) use final_state::final_floating_area_state;
use snapshot::floating_area_drag_snapshot;

mod commit;
mod final_state;
mod snapshot;

pub(super) struct PreparedFloatingAreaState {
    pub(super) position: Point,
    pub(super) test_id: Arc<str>,
    pub(super) dragging: bool,
}

pub(super) fn prepare_floating_area_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    area_id: GlobalElementId,
    id: &str,
    initial_position: Point,
    options: &FloatingAreaOptions,
    drag_kind: fret_runtime::DragKindId,
) -> PreparedFloatingAreaState {
    let drag_snapshot = floating_area_drag_snapshot(cx, drag_kind);
    let dragging = drag_snapshot
        .map(|snapshot| snapshot.dragging)
        .unwrap_or(false);

    let scale_factor = cx
        .app
        .global::<WindowMetricsService>()
        .and_then(|svc| svc.scale_factor(cx.window))
        .unwrap_or(1.0);

    let (position, test_id) = commit_floating_area_drag_state(
        cx,
        area_id,
        id,
        initial_position,
        options,
        drag_snapshot,
        scale_factor,
    );

    PreparedFloatingAreaState {
        position,
        test_id,
        dragging,
    }
}
