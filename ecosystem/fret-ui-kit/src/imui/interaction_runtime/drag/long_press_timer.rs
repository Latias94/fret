use fret_ui::action::UiActionHostExt as _;

pub(super) fn arm_for(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    model: &fret_runtime::Model<super::super::LongPressSignalState>,
) {
    let token = host.next_timer_token();
    let previous = host
        .update_model(model, |state| {
            let previous = state.timer.take();
            state.timer = Some(token);
            state.holding = false;
            previous
        })
        .flatten();
    if let Some(previous) = previous {
        host.push_effect(fret_runtime::Effect::CancelTimer { token: previous });
    }
    host.push_effect(fret_runtime::Effect::SetTimer {
        window: Some(action_cx.window),
        token,
        after: crate::imui::LONG_PRESS_DELAY,
        repeat: None,
    });
}

pub(super) fn cancel_for(
    host: &mut dyn fret_ui::action::UiActionHost,
    model: &fret_runtime::Model<super::super::LongPressSignalState>,
) {
    let previous = host
        .update_model(model, |state| {
            let previous = state.timer.take();
            state.holding = false;
            previous
        })
        .flatten();
    if let Some(previous) = previous {
        host.push_effect(fret_runtime::Effect::CancelTimer { token: previous });
    }
}
