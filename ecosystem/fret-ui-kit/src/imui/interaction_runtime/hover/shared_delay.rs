use std::time::Duration;

mod state;

pub(super) use state::{ImUiSharedHoverDelayState, model_for_window};

const SHARED_HOVER_CLEAR_DELAY: Duration = Duration::from_millis(250);

pub(super) fn on_hover_change(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    hovered: bool,
    shared_model: &fret_runtime::Model<ImUiSharedHoverDelayState>,
) {
    let prev = host
        .models_mut()
        .read(shared_model, |st| *st)
        .ok()
        .unwrap_or_default();

    if hovered {
        if let Some(token) = prev.clear_timer {
            host.push_effect(fret_runtime::Effect::CancelTimer { token });
        }

        let mut next = prev;
        next.clear_timer = None;

        if !prev.delay_short_met && prev.short_timer.is_none() {
            let token = host.next_timer_token();
            next.short_timer = Some(token);
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token,
                after: crate::imui::HOVER_DELAY_SHORT,
                repeat: None,
            });
        }

        if !prev.delay_normal_met && prev.normal_timer.is_none() {
            let token = host.next_timer_token();
            next.normal_timer = Some(token);
            host.push_effect(fret_runtime::Effect::SetTimer {
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
    host.push_effect(fret_runtime::Effect::SetTimer {
        window: Some(action_cx.window),
        token,
        after: SHARED_HOVER_CLEAR_DELAY,
        repeat: None,
    });

    let mut next = prev;
    next.clear_timer = Some(token);
    let _ = host.models_mut().update(shared_model, |st| *st = next);
}

pub(super) fn on_timer(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    action_cx: fret_ui::action::ActionCx,
    token: fret_runtime::TimerToken,
    shared_model: &fret_runtime::Model<ImUiSharedHoverDelayState>,
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
            host.push_effect(fret_runtime::Effect::CancelTimer { token });
        }
        if let Some(token) = prev.normal_timer {
            host.push_effect(fret_runtime::Effect::CancelTimer { token });
        }
        let _ = host.models_mut().update(shared_model, |st| {
            *st = ImUiSharedHoverDelayState::default()
        });
        host.notify(action_cx);
        return true;
    }

    false
}
