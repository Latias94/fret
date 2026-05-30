use std::sync::Arc;

use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{long_press, read, shared_delay, timers};

pub(in crate::imui) fn install_hover_query_hooks_for_pressable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    hovered_raw: bool,
    long_press_signal_model: Option<fret_runtime::Model<super::super::LongPressSignalState>>,
) -> read::HoverQueryDelayRead {
    let shared_delay_model = shared_delay::model_for_window(cx);
    let shared_delay_model_for_hover = shared_delay_model.clone();
    cx.pressable_add_on_hover_change(Arc::new(move |host, action_cx, hovered| {
        let stationary = timers::stationary_token_for(action_cx.target);
        let delay_short = timers::delay_short_token_for(action_cx.target);
        let delay_normal = timers::delay_normal_token_for(action_cx.target);

        if hovered {
            shared_delay::on_hover_change(host, action_cx, true, &shared_delay_model_for_hover);
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token: stationary,
                after: crate::imui::HOVER_STATIONARY_DELAY,
                repeat: None,
            });
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token: delay_short,
                after: crate::imui::HOVER_DELAY_SHORT,
                repeat: None,
            });
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token: delay_normal,
                after: crate::imui::HOVER_DELAY_NORMAL,
                repeat: None,
            });
            return;
        }

        shared_delay::on_hover_change(host, action_cx, false, &shared_delay_model_for_hover);
        host.push_effect(fret_runtime::Effect::CancelTimer { token: stationary });
        host.push_effect(fret_runtime::Effect::CancelTimer { token: delay_short });
        host.push_effect(fret_runtime::Effect::CancelTimer {
            token: delay_normal,
        });
    }));

    let long_press_signal_model_for_timer = long_press_signal_model.clone();
    let shared_delay_model_for_timer = shared_delay_model.clone();
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

            if shared_delay::on_timer(host, action_cx, token, &shared_delay_model_for_timer) {
                return true;
            }

            if let Some(model) = long_press_signal_model_for_timer.as_ref() {
                return long_press::emit_if_matching(host, action_cx, model, token);
            }

            false
        }),
    );

    read::read_hover_query_delay(cx, id, hovered_raw, &shared_delay_model)
}
