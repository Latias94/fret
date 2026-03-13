use fret_ui::UiHost;

use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};
use crate::ui::canvas::state::ViewportMoveDebounceState;
use crate::ui::canvas::widget::*;

pub(super) fn bump_viewport_move_debounce<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    snapshot: &ViewSnapshot,
    kind: ViewportMoveKind,
) {
    if let Some(prev) = canvas.interaction.viewport_move_debounce.take() {
        host.push_effect(Effect::CancelTimer { token: prev.timer });
        if prev.kind != kind {
            canvas.emit_move_end(snapshot, prev.kind, ViewportMoveEndOutcome::Ended);
            canvas.emit_move_start(snapshot, kind);
        }
    } else {
        canvas.emit_move_start(snapshot, kind);
    }

    let timer = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window,
        token: timer,
        after: NodeGraphCanvasWith::<M>::VIEWPORT_MOVE_END_DEBOUNCE,
        repeat: None,
    });
    canvas.interaction.viewport_move_debounce = Some(ViewportMoveDebounceState { kind, timer });
}
