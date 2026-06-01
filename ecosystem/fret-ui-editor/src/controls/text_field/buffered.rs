use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_runtime::{Model, TimerToken};
use fret_ui::action::KeyDownCx;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::TextFieldBlurBehavior;
use crate::primitives::EditSession;

mod actions;
mod controller;

pub(super) use actions::{
    cancel_buffered_text_field, cancel_buffered_text_field_from_controller,
    clear_buffered_text_field_state, commit_buffered_text_field,
    commit_buffered_text_field_from_controller,
};
pub use controller::TextFieldDraftController;

#[derive(Debug, Default)]
pub(super) struct BufferedTextFieldState {
    was_focused: bool,
    session: EditSession<String>,
    blur_timer: Option<TimerToken>,
    pending_blur: Option<TextFieldBlurBehavior>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BufferedTextFieldPendingBlurPlan {
    Keep,
    Clear,
    Arm(TextFieldBlurBehavior),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BufferedTextFieldFocusPlan {
    begin_session: bool,
    cancel_pending_blur: bool,
    pending_blur: BufferedTextFieldPendingBlurPlan,
}

pub(super) fn plan_buffered_text_field_focus_transition(
    was_focused: bool,
    session_active: bool,
    is_focused: bool,
    blur_behavior: TextFieldBlurBehavior,
    has_pending_blur: bool,
) -> BufferedTextFieldFocusPlan {
    if is_focused {
        return BufferedTextFieldFocusPlan {
            begin_session: !session_active,
            cancel_pending_blur: has_pending_blur,
            pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
        };
    }

    if was_focused && session_active {
        return BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: has_pending_blur,
            pending_blur: match blur_behavior {
                TextFieldBlurBehavior::Commit | TextFieldBlurBehavior::Cancel => {
                    BufferedTextFieldPendingBlurPlan::Arm(blur_behavior)
                }
                TextFieldBlurBehavior::PreserveDraft => BufferedTextFieldPendingBlurPlan::Clear,
            },
        };
    }

    if session_active {
        return BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: false,
            pending_blur: BufferedTextFieldPendingBlurPlan::Keep,
        };
    }

    BufferedTextFieldFocusPlan {
        begin_session: false,
        cancel_pending_blur: has_pending_blur,
        pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
    }
}

#[track_caller]
pub(super) fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}

#[track_caller]
pub(super) fn buffered_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Arc<Mutex<BufferedTextFieldState>> {
    cx.slot_state(
        || Arc::new(Mutex::new(BufferedTextFieldState::default())),
        |st| st.clone(),
    )
}

pub(super) fn sync_draft_from_model_when_session_inactive<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
    current_text: &str,
) {
    let session_active = buffered_state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .session
        .is_active();
    if session_active {
        return;
    }

    let next = current_text.to_owned();
    let _ = cx.app.models_mut().update(draft, |text| {
        if text.as_str() != next.as_str() {
            *text = next.clone();
        }
    });
}

pub(super) fn sync_buffered_text_field_session<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_id: GlobalElementId,
    is_focused: bool,
    current_text: &str,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
    blur_behavior: TextFieldBlurBehavior,
) {
    let (begin_session, cancel_blur_token, arm_blur_token) = {
        let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
        let plan = plan_buffered_text_field_focus_transition(
            state.was_focused,
            state.session.is_active(),
            is_focused,
            blur_behavior,
            state.blur_timer.is_some() || state.pending_blur.is_some(),
        );

        let cancel_blur_token = if plan.cancel_pending_blur {
            state.blur_timer.take()
        } else {
            None
        };
        let arm_blur_token = match plan.pending_blur {
            BufferedTextFieldPendingBlurPlan::Keep => None,
            BufferedTextFieldPendingBlurPlan::Clear => {
                state.blur_timer = None;
                state.pending_blur = None;
                None
            }
            BufferedTextFieldPendingBlurPlan::Arm(next_blur_behavior) => {
                let token = cx.app.next_timer_token();
                state.blur_timer = Some(token);
                state.pending_blur = Some(next_blur_behavior);
                Some(token)
            }
        };
        if plan.begin_session {
            state.session.begin(current_text.to_owned());
        }

        state.was_focused = is_focused;
        (plan.begin_session, cancel_blur_token, arm_blur_token)
    };

    if let Some(token) = cancel_blur_token {
        cx.cancel_timer(token);
    }
    if let Some(token) = arm_blur_token {
        cx.set_timer_for(input_id, token, Duration::ZERO);
    }

    if begin_session {
        let next = current_text.to_owned();
        let _ = cx.app.models_mut().update(draft, |text| {
            if text.as_str() != next.as_str() {
                *text = next.clone();
            }
        });
    }
}

pub(super) fn install_buffered_text_field_blur_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_id: GlobalElementId,
    model: Model<String>,
    draft: Model<String>,
    buffered_state: Arc<Mutex<BufferedTextFieldState>>,
    on_outcome: Option<super::OnTextFieldOutcome>,
) {
    cx.timer_add_on_timer_for(
        input_id,
        Arc::new(move |host, action_cx, token| {
            let blur_behavior = {
                let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
                if state.blur_timer != Some(token) {
                    return false;
                }
                state.blur_timer = None;
                state.pending_blur.take()
            };

            match blur_behavior {
                Some(TextFieldBlurBehavior::Commit) => commit_buffered_text_field(
                    host,
                    action_cx,
                    &model,
                    &draft,
                    &buffered_state,
                    on_outcome.as_ref(),
                    None,
                ),
                Some(TextFieldBlurBehavior::Cancel) => cancel_buffered_text_field(
                    host,
                    action_cx,
                    &model,
                    &draft,
                    &buffered_state,
                    on_outcome.as_ref(),
                ),
                Some(TextFieldBlurBehavior::PreserveDraft) | None => false,
            }
        }),
    );
}

pub(super) fn is_multiline_buffered_commit_shortcut(down: KeyDownCx) -> bool {
    (down.modifiers.ctrl || down.modifiers.meta) && !down.modifiers.alt && !down.modifiers.alt_gr
}

#[cfg(test)]
mod tests;
