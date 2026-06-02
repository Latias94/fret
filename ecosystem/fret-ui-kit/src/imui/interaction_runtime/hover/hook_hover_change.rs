use std::sync::Arc;

use fret_ui::{ElementContext, UiHost};

use super::{shared_delay, timers};

pub(super) fn install_hover_change_hook<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    shared_delay_model: fret_runtime::Model<shared_delay::ImUiSharedHoverDelayState>,
) {
    cx.pressable_add_on_hover_change(Arc::new(move |host, action_cx, hovered| {
        let stationary = timers::stationary_token_for(action_cx.target);
        let delay_short = timers::delay_short_token_for(action_cx.target);
        let delay_normal = timers::delay_normal_token_for(action_cx.target);

        if hovered {
            shared_delay::on_hover_change(host, action_cx, true, &shared_delay_model);
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

        shared_delay::on_hover_change(host, action_cx, false, &shared_delay_model);
        host.push_effect(fret_runtime::Effect::CancelTimer { token: stationary });
        host.push_effect(fret_runtime::Effect::CancelTimer { token: delay_short });
        host.push_effect(fret_runtime::Effect::CancelTimer {
            token: delay_normal,
        });
    }));
}
