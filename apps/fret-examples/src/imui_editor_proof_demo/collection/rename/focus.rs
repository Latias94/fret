use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_runtime::{Model, TimerToken};
use fret_ui::action::{UiActionHostExt as _, UiFocusActionHost};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Default)]
pub(in super::super) struct ProofCollectionInlineRenameFocusState {
    timer: Option<TimerToken>,
}

#[track_caller]
pub(in super::super) fn proof_collection_inline_rename_focus_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Arc<Mutex<ProofCollectionInlineRenameFocusState>> {
    cx.slot_state(
        || Arc::new(Mutex::new(ProofCollectionInlineRenameFocusState::default())),
        |state| state.clone(),
    )
}

pub(in super::super) fn proof_collection_sync_inline_rename_focus<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_id: GlobalElementId,
    pending_focus: bool,
    pending_focus_model: &Model<bool>,
    focus_state: &Arc<Mutex<ProofCollectionInlineRenameFocusState>>,
) {
    let (cancel_token, arm_token) = {
        let mut state = focus_state.lock().unwrap_or_else(|err| err.into_inner());
        match (pending_focus, state.timer) {
            (true, None) => {
                let token = cx.app.next_timer_token();
                state.timer = Some(token);
                (None, Some(token))
            }
            (false, Some(token)) => {
                state.timer = None;
                (Some(token), None)
            }
            _ => (None, None),
        }
    };

    if let Some(token) = cancel_token {
        cx.cancel_timer(token);
    }
    if let Some(token) = arm_token {
        cx.set_timer_for(input_id, token, Duration::ZERO);
    }

    let focus_state_for_timer = focus_state.clone();
    let pending_focus_model_for_timer = pending_focus_model.clone();
    cx.timer_add_on_timer_for(
        input_id,
        Arc::new(move |host, action_cx, token| {
            {
                let mut state = focus_state_for_timer
                    .lock()
                    .unwrap_or_else(|err| err.into_inner());
                if state.timer != Some(token) {
                    return false;
                }
                state.timer = None;
            }

            let pending = host
                .update_model(&pending_focus_model_for_timer, |value| {
                    std::mem::take(value)
                })
                .unwrap_or(false);
            if !pending {
                return false;
            }

            host.request_focus(input_id);
            host.request_redraw(action_cx.window);
            false
        }),
    );
}

pub(in super::super) fn proof_collection_restore_focus_after_inline_rename(
    host: &mut dyn UiFocusActionHost,
    action_cx: fret_ui::action::ActionCx,
    focus_target_model: &Model<Option<GlobalElementId>>,
) {
    let target = host
        .models_mut()
        .read(focus_target_model, |state| *state)
        .ok()
        .flatten();
    if let Some(target) = target {
        host.request_focus(target);
        host.request_redraw(action_cx.window);
    }
}
