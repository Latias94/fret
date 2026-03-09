use fret_core::time::Instant;

use fret_core::{MouseButton, Point};
use fret_runtime::Effect;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};
use crate::core::CanvasPoint;
use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};

pub(super) fn begin_panning<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    button: MouseButton,
) -> bool {
    cancel_previous_motion(canvas, cx, snapshot);
    clear_competing_interactions(canvas);

    canvas.interaction.panning = true;
    canvas.interaction.panning_button = Some(button);

    let viewport =
        NodeGraphCanvasWith::<M>::viewport_from_pan_zoom(cx.bounds, snapshot.pan, snapshot.zoom);
    let screen_pos = viewport.canvas_to_screen(start_pos);
    canvas.interaction.pan_last_screen_pos = Some(screen_pos);
    canvas.interaction.pan_last_sample_at = Some(Instant::now());
    canvas.interaction.pan_velocity = CanvasPoint::default();

    canvas.emit_move_start(snapshot, ViewportMoveKind::PanDrag);
    cx.capture_pointer(cx.node);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

fn cancel_previous_motion<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) {
    if canvas.interaction.pan_inertia.is_some() {
        canvas.stop_pan_inertia_timer(cx.app);
        canvas.emit_move_end(
            snapshot,
            ViewportMoveKind::PanInertia,
            ViewportMoveEndOutcome::Ended,
        );
    }
    if let Some(state) = canvas.interaction.viewport_move_debounce.take() {
        cx.app
            .push_effect(Effect::CancelTimer { token: state.timer });
        canvas.emit_move_end(snapshot, state.kind, ViewportMoveEndOutcome::Ended);
    }
}

fn clear_competing_interactions<M: NodeGraphCanvasMiddleware>(canvas: &mut NodeGraphCanvasWith<M>) {
    super::press_session::prepare_for_pan_begin(&mut canvas.interaction);
}
