use fret_ui::UiHost;

use crate::ui::canvas::widget::*;

pub(super) fn stop_auto_pan_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) {
    let Some(timer) = canvas.interaction.auto_pan_timer.take() else {
        return;
    };
    host.push_effect(Effect::CancelTimer { token: timer });
}

pub(super) fn ensure_auto_pan_timer_running<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
) {
    if canvas.interaction.auto_pan_timer.is_some() {
        return;
    }
    let timer = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window,
        token: timer,
        after: NodeGraphCanvasWith::<M>::AUTO_PAN_TICK_INTERVAL,
        repeat: Some(NodeGraphCanvasWith::<M>::AUTO_PAN_TICK_INTERVAL),
    });
    canvas.interaction.auto_pan_timer = Some(timer);
}
