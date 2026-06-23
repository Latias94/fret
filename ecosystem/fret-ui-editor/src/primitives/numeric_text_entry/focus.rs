use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_runtime::{Model, TimerToken};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumericInputSelectionBehavior {
    PreserveDraft,
    #[default]
    ReplaceAllOnFocus,
}

#[derive(Debug, Default)]
pub(crate) struct NumericTextEntryFocusState {
    pub(super) was_focused: bool,
    pub(super) replace_on_next_edit: bool,
}

#[derive(Debug, Default)]
pub(crate) struct NumericTextEntryFocusHandoffState {
    pending: bool,
    timer: Option<TimerToken>,
}

#[track_caller]
pub(crate) fn numeric_text_entry_focus_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Arc<Mutex<NumericTextEntryFocusState>> {
    cx.slot_state(
        || Arc::new(Mutex::new(NumericTextEntryFocusState::default())),
        |state| state.clone(),
    )
}

pub(crate) fn arm_numeric_text_entry_focus_handoff(
    handoff: &mut NumericTextEntryFocusHandoffState,
) {
    handoff.pending = true;
    handoff.timer = None;
}

pub(crate) fn sync_numeric_text_entry_focus_handoff<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    timer_target: GlobalElementId,
    handoff: &Arc<Mutex<NumericTextEntryFocusHandoffState>>,
    typing: bool,
    input_id: GlobalElementId,
    is_focused: bool,
) {
    let (cancel_token, arm_token) = {
        let mut state = handoff.lock().unwrap_or_else(|e| e.into_inner());
        if !typing || is_focused {
            let cancel = state.timer.take();
            state.pending = false;
            (cancel, None)
        } else if state.pending && state.timer.is_none() {
            let token = cx.app.next_timer_token();
            state.timer = Some(token);
            (None, Some(token))
        } else {
            (None, None)
        }
    };

    if let Some(token) = cancel_token {
        cx.cancel_timer(token);
    }
    if let Some(token) = arm_token {
        cx.set_timer_for(timer_target, token, Duration::ZERO);
    }

    let handoff_for_timer = handoff.clone();
    // Shared numeric-entry helpers may be layered with control-owned timer hooks.
    cx.timer_add_on_timer_for(
        timer_target,
        Arc::new(move |host, action_cx, token| {
            let mut state = handoff_for_timer.lock().unwrap_or_else(|e| e.into_inner());
            if state.timer != Some(token) {
                return false;
            }

            state.timer = None;
            if !state.pending {
                return false;
            }

            state.pending = false;
            host.request_focus(input_id);
            host.request_redraw(action_cx.window);
            false
        }),
    );
}

pub(crate) fn sync_numeric_text_entry_focus<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    focus_state: &Arc<Mutex<NumericTextEntryFocusState>>,
    is_focused: bool,
    current_text: &Arc<str>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    selection_behavior: NumericInputSelectionBehavior,
) {
    let mut state = focus_state.lock().unwrap_or_else(|e| e.into_inner());

    if is_focused && !state.was_focused {
        state.replace_on_next_edit = matches!(
            selection_behavior,
            NumericInputSelectionBehavior::ReplaceAllOnFocus
        ) && !current_text.is_empty();
    } else if !is_focused {
        let draft_changed = sync_draft_from_current_text(cx, draft, current_text.as_ref());
        let error_changed = clear_error_if_present(cx, error);
        if draft_changed || error_changed {
            cx.request_frame();
        }
        state.replace_on_next_edit = false;
    }

    state.was_focused = is_focused;
}

fn sync_draft_from_current_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    draft: &Model<String>,
    current_text: &str,
) -> bool {
    let needs_sync = cx
        .app
        .models()
        .get_cloned(draft)
        .is_none_or(|text| text != current_text);
    if needs_sync {
        let next = current_text.to_string();
        let _ = cx.app.models_mut().update(draft, |text| *text = next);
    }
    needs_sync
}

fn clear_error_if_present<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    error: &Model<Option<Arc<str>>>,
) -> bool {
    let has_error = cx.app.models().get_cloned(error).flatten().is_some();
    if has_error {
        let _ = cx.app.models_mut().update(error, |value| *value = None);
    }
    has_error
}

pub(crate) fn clear_numeric_error_when_draft_changes<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    is_focused: bool,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    last_draft_text: &Arc<Mutex<String>>,
) {
    if !is_focused {
        return;
    }

    let draft_text = cx
        .get_model_cloned(draft, Invalidation::Paint)
        .unwrap_or_default();
    let changed = {
        let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
        if *last == draft_text {
            false
        } else {
            *last = draft_text;
            true
        }
    };

    if changed {
        let _ = clear_error_if_present(cx, error);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::elements::with_element_cx;

    use super::clear_numeric_error_when_draft_changes;

    #[test]
    fn clear_numeric_error_when_draft_changes_skips_none_errors() {
        let mut app = App::new();
        let draft = app.models_mut().insert(String::from("42"));
        let error = app.models_mut().insert(None::<Arc<str>>);
        let draft_revision = draft.revision(&app);
        let error_revision = error.revision(&app);
        let last_draft_text = Arc::new(Mutex::new(String::new()));

        with_element_cx(
            &mut app,
            AppWindowId::default(),
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(1.0), Px(1.0))),
            "numeric-text-entry-focus-test",
            |cx| {
                clear_numeric_error_when_draft_changes(cx, true, &draft, &error, &last_draft_text);
            },
        );

        assert_eq!(draft_revision, draft.revision(&app));
        assert_eq!(error_revision, error.revision(&app));
    }

    #[test]
    fn clear_numeric_error_when_draft_changes_clears_present_errors() {
        let mut app = App::new();
        let draft = app.models_mut().insert(String::from("42"));
        let error = app.models_mut().insert(Some(Arc::from("Invalid number")));
        let error_revision = error.revision(&app);
        let last_draft_text = Arc::new(Mutex::new(String::new()));

        with_element_cx(
            &mut app,
            AppWindowId::default(),
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(1.0), Px(1.0))),
            "numeric-text-entry-focus-test",
            |cx| {
                clear_numeric_error_when_draft_changes(cx, true, &draft, &error, &last_draft_text);
            },
        );

        assert_ne!(error_revision, error.revision(&app));
        assert!(app.models_mut().read(&error, Option::is_none).unwrap());
    }
}
