use fret_core::MouseButton;
use fret_ui::UiHost;

use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};

use super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, low_level_adapter::CanvasPaintInvalidationCx,
    pointer_up_release_cx::PointerUpReleaseCx,
};
use crate::ui::canvas::state::ViewSnapshot;

pub(in super::super) fn handle_sticky_wire_ignored_release<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl CanvasPaintInvalidationCx<H>,
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
    cx: &mut impl PointerUpReleaseCx<H>,
    snapshot: &ViewSnapshot,
    button: MouseButton,
) -> bool {
    if !super::super::cancel_session::matches_pan_release(&canvas.interaction, button) {
        return false;
    }

    super::super::cancel_session::clear_pan_drag_state(&mut canvas.interaction);
    canvas.stop_auto_pan_timer(cx.host());
    let window = cx.window();
    let started_inertia = canvas.maybe_start_pan_inertia_timer(cx.host(), window, snapshot);
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
