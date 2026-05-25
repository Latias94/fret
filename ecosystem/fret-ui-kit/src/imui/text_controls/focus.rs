use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_runtime::TimerToken;
use fret_ui::UiHost;

#[derive(Debug, Default)]
struct ImuiTextFocusSelectionState {
    was_focused: bool,
    pending_select_all: bool,
    timer: Option<TimerToken>,
}

pub(super) fn sync_select_all_on_focus<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    is_focused: bool,
    has_text: bool,
    select_all_on_focus: bool,
) {
    if !select_all_on_focus {
        return;
    }

    let state = cx.state_for(
        id,
        || Arc::new(Mutex::new(ImuiTextFocusSelectionState::default())),
        |state| state.clone(),
    );

    let (cancel_token, arm_token) = {
        let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
        let mut cancel_token = None;
        let mut arm_token = None;

        if is_focused && !state.was_focused {
            state.pending_select_all = has_text;
            if state.pending_select_all {
                let token = cx.app.next_timer_token();
                state.timer = Some(token);
                arm_token = Some(token);
            }
        } else if !is_focused {
            cancel_token = state.timer.take();
            state.pending_select_all = false;
        }

        state.was_focused = is_focused;
        (cancel_token, arm_token)
    };

    if let Some(token) = cancel_token {
        cx.cancel_timer(token);
    }
    let install_handler = arm_token.is_some();
    if let Some(token) = arm_token {
        cx.set_timer_for(id, token, Duration::ZERO);
    }

    if install_handler {
        let state_for_timer = state.clone();
        cx.timer_on_timer_for(
            id,
            Arc::new(move |host, action_cx, token| {
                let mut state = state_for_timer.lock().unwrap_or_else(|e| e.into_inner());
                if state.timer != Some(token) {
                    return false;
                }
                state.timer = None;
                if !state.pending_select_all {
                    return false;
                }
                state.pending_select_all = false;
                host.record_transient_event(action_cx, super::super::KEY_SELECT_ALL_ON_FOCUS);
                host.request_redraw(action_cx.window);
                true
            }),
        );
    }
}
