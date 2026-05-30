use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::{ActionCx, UiFocusActionHost};

use super::state::ImUiSharedHoverDelayState;

pub(in crate::imui::interaction_runtime::hover) fn on_timer(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    token: TimerToken,
    shared_model: &Model<ImUiSharedHoverDelayState>,
) -> bool {
    let prev = host
        .models_mut()
        .read(shared_model, |st| *st)
        .ok()
        .unwrap_or_default();

    if prev.short_timer == Some(token) {
        let mut next = prev;
        next.delay_short_met = true;
        next.short_timer = None;
        let _ = host.models_mut().update(shared_model, |st| *st = next);
        host.notify(action_cx);
        return true;
    }

    if prev.normal_timer == Some(token) {
        let mut next = prev;
        next.delay_normal_met = true;
        next.normal_timer = None;
        let _ = host.models_mut().update(shared_model, |st| *st = next);
        host.notify(action_cx);
        return true;
    }

    if prev.clear_timer == Some(token) {
        if let Some(token) = prev.short_timer {
            host.push_effect(Effect::CancelTimer { token });
        }
        if let Some(token) = prev.normal_timer {
            host.push_effect(Effect::CancelTimer { token });
        }
        let _ = host.models_mut().update(shared_model, |st| {
            *st = ImUiSharedHoverDelayState::default()
        });
        host.notify(action_cx);
        return true;
    }

    false
}
