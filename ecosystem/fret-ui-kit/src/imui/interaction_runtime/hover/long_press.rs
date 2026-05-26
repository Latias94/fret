use fret_ui::action::UiActionHostExt as _;

pub(super) fn emit_if_matching(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    model: &fret_runtime::Model<super::super::LongPressSignalState>,
    token: fret_runtime::TimerToken,
) -> bool {
    let fired = host
        .update_model(model, |state| {
            if state.timer != Some(token) {
                return false;
            }
            state.timer = None;
            state.holding = true;
            true
        })
        .unwrap_or(false);
    if fired {
        host.record_transient_event(action_cx, crate::imui::KEY_LONG_PRESSED);
        host.notify(action_cx);
    }
    fired
}
