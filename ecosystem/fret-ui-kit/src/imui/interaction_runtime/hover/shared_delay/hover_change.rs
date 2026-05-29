use std::time::Duration;

use fret_runtime::{Effect, Model};
use fret_ui::action::{ActionCx, UiActionHost};

use super::state::ImUiSharedHoverDelayState;

const SHARED_HOVER_CLEAR_DELAY: Duration = Duration::from_millis(250);

pub(in crate::imui::interaction_runtime::hover) fn on_hover_change(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    hovered: bool,
    shared_model: &Model<ImUiSharedHoverDelayState>,
) {
    let prev = host
        .models_mut()
        .read(shared_model, |st| *st)
        .ok()
        .unwrap_or_default();

    if hovered {
        if let Some(token) = prev.clear_timer {
            host.push_effect(Effect::CancelTimer { token });
        }

        let mut next = prev;
        next.clear_timer = None;

        if !prev.delay_short_met && prev.short_timer.is_none() {
            let token = host.next_timer_token();
            next.short_timer = Some(token);
            host.push_effect(Effect::SetTimer {
                window: Some(action_cx.window),
                token,
                after: crate::imui::HOVER_DELAY_SHORT,
                repeat: None,
            });
        }

        if !prev.delay_normal_met && prev.normal_timer.is_none() {
            let token = host.next_timer_token();
            next.normal_timer = Some(token);
            host.push_effect(Effect::SetTimer {
                window: Some(action_cx.window),
                token,
                after: crate::imui::HOVER_DELAY_NORMAL,
                repeat: None,
            });
        }

        let _ = host.models_mut().update(shared_model, |st| *st = next);
        return;
    }

    if prev.clear_timer.is_some() {
        return;
    }

    let token = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window: Some(action_cx.window),
        token,
        after: SHARED_HOVER_CLEAR_DELAY,
        repeat: None,
    });

    let mut next = prev;
    next.clear_timer = Some(token);
    let _ = host.models_mut().update(shared_model, |st| *st = next);
}
