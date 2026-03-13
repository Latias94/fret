use fret_core::MouseButton;
use fret_ui::UiHost;

use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(in super::super) fn handle_sticky_wire_ignored_release<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    button: MouseButton,
) -> bool {
    if button == MouseButton::Left
        && canvas.interaction.sticky_wire_ignore_next_up
        && canvas.interaction.wire_drag.is_some()
    {
        canvas.interaction.sticky_wire_ignore_next_up = false;
        super::super::paint_invalidation::invalidate_paint(cx);
        return true;
    }

    false
}

pub(in super::super) fn handle_pan_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    button: MouseButton,
) -> bool {
    if !super::super::cancel_session::matches_pan_release(&canvas.interaction, button) {
        return false;
    }

    super::super::cancel_session::clear_pan_drag_state(&mut canvas.interaction);
    canvas.stop_auto_pan_timer(cx.app);
    let started_inertia = canvas.maybe_start_pan_inertia_timer(cx.app, cx.window, snapshot);
    canvas.emit_move_end(
        snapshot,
        ViewportMoveKind::PanDrag,
        ViewportMoveEndOutcome::Ended,
    );
    if started_inertia {
        canvas.emit_move_start(snapshot, ViewportMoveKind::PanInertia);
    }
    cx.release_pointer_capture();
    super::super::paint_invalidation::invalidate_paint(cx);
    true
}
