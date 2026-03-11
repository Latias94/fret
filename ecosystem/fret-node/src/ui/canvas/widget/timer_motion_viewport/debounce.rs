use crate::ui::canvas::widget::*;

use super::super::timer_motion_shared::invalidate_motion;

pub(super) fn handle_viewport_move_debounce<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    token: fret_core::TimerToken,
) -> bool {
    if !canvas
        .interaction
        .viewport_move_debounce
        .as_ref()
        .is_some_and(|state| state.timer == token)
    {
        return false;
    }

    let Some(state) = canvas.interaction.viewport_move_debounce.take() else {
        return true;
    };
    let snapshot = canvas.sync_view_state(cx.app);
    canvas.emit_move_end(&snapshot, state.kind, ViewportMoveEndOutcome::Ended);
    invalidate_motion(cx);
    true
}
