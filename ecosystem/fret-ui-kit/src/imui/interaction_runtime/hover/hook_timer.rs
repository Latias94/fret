use std::sync::Arc;

use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{long_press, shared_delay, timers};

pub(super) fn install_hover_timer_hook<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    shared_delay_model: fret_runtime::Model<shared_delay::ImUiSharedHoverDelayState>,
    long_press_signal_model: Option<fret_runtime::Model<super::super::LongPressSignalState>>,
) {
    cx.timer_add_on_timer_for(
        id,
        Arc::new(move |host, action_cx, token| {
            let stationary = timers::stationary_token_for(action_cx.target);
            if token == stationary {
                host.record_transient_event(action_cx, crate::imui::KEY_HOVER_STATIONARY_MET);
                host.notify(action_cx);
                return true;
            }
            let delay_short = timers::delay_short_token_for(action_cx.target);
            if token == delay_short {
                host.record_transient_event(action_cx, crate::imui::KEY_HOVER_DELAY_SHORT_MET);
                host.notify(action_cx);
                return true;
            }
            let delay_normal = timers::delay_normal_token_for(action_cx.target);
            if token == delay_normal {
                host.record_transient_event(action_cx, crate::imui::KEY_HOVER_DELAY_NORMAL_MET);
                host.notify(action_cx);
                return true;
            }

            if shared_delay::on_timer(host, action_cx, token, &shared_delay_model) {
                return true;
            }

            if let Some(model) = long_press_signal_model.as_ref() {
                return long_press::emit_if_matching(host, action_cx, model, token);
            }

            false
        }),
    );
}
