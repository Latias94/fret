use fret_runtime::Effect;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn cancel_viewport_state<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let mut canceled = false;

    if canvas.interaction.panning {
        canvas.interaction.panning = false;
        canvas.interaction.panning_button = None;
        canvas.interaction.pan_last_screen_pos = None;
        canvas.interaction.pan_last_sample_at = None;
        canvas.emit_move_end(
            snapshot,
            ViewportMoveKind::PanDrag,
            ViewportMoveEndOutcome::Canceled,
        );
        canceled = true;
    }
    if canvas.interaction.pan_inertia.is_some() {
        canvas.stop_pan_inertia_timer(cx.app);
        canvas.emit_move_end(
            snapshot,
            ViewportMoveKind::PanInertia,
            ViewportMoveEndOutcome::Canceled,
        );
        canceled = true;
    }
    if canvas.interaction.viewport_animation.is_some() {
        canvas.stop_viewport_animation_timer(cx.app);
        canceled = true;
    }
    if let Some(state) = canvas.interaction.viewport_move_debounce.take() {
        cx.app
            .push_effect(Effect::CancelTimer { token: state.timer });
        canvas.emit_move_end(snapshot, state.kind, ViewportMoveEndOutcome::Canceled);
        canceled = true;
    }

    canceled
}
