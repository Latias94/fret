mod activate;
mod checks;

use fret_core::Point;
use fret_ui::UiHost;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(in super::super) fn handle_pending_wire_drag_release<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    let Some(pending) = canvas.interaction.pending_wire_drag.take() else {
        return false;
    };

    if checks::should_promote_pending_wire_drag(
        snapshot.interaction.connect_on_click,
        &pending.kind,
    ) {
        activate::promote_pending_wire_drag(canvas, snapshot, pending, position);
    }

    super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}

#[cfg(test)]
mod tests;
