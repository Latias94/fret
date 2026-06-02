use std::sync::{Arc, Mutex};

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiFocusActionHost};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::controls::numeric_input::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusState, handle_numeric_text_entry_replace_key,
};
use crate::primitives::{NumericValueConstraints, constrain_numeric_value};

use super::super::model::{AxisDragValueOutcome, AxisDragValueState, OnAxisDragValueOutcome};
use super::super::session::emit_axis_drag_value_outcome;

pub(super) struct AxisDragValueTypingKeyHandlerArgs<T> {
    pub(super) input_id: GlobalElementId,
    pub(super) focus_state: Arc<Mutex<NumericTextEntryFocusState>>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) last_draft_text: Arc<Mutex<String>>,
    pub(super) state: Arc<Mutex<AxisDragValueState>>,
    pub(super) model: Model<T>,
    pub(super) parse: NumericParseFn<T>,
    pub(super) format: NumericFormatFn<T>,
    pub(super) validate: Option<NumericValidateFn<T>>,
    pub(super) constraints: NumericValueConstraints,
    pub(super) on_outcome: Option<OnAxisDragValueOutcome>,
}

pub(super) fn axis_drag_value_add_typing_key_handler<T, H>(
    cx: &mut ElementContext<'_, H>,
    args: AxisDragValueTypingKeyHandlerArgs<T>,
) where
    T: DragValueScalar + Default,
    H: UiHost,
{
    let AxisDragValueTypingKeyHandlerArgs {
        input_id,
        focus_state,
        draft,
        error,
        last_draft_text,
        state,
        model,
        parse,
        format,
        validate,
        constraints,
        on_outcome,
    } = args;

    cx.key_add_on_key_down_capture_for(
        input_id,
        Arc::new(
            move |host: &mut dyn UiFocusActionHost, action_cx: ActionCx, down| {
                if let Some(consumed) = handle_numeric_text_entry_replace_key(
                    host,
                    action_cx,
                    down,
                    &focus_state,
                    &draft,
                    &error,
                ) && consumed
                {
                    return true;
                }

                match down.key {
                    KeyCode::Enter | KeyCode::NumpadEnter => axis_drag_value_commit_typed_text(
                        host,
                        action_cx,
                        &draft,
                        &error,
                        &last_draft_text,
                        &state,
                        &model,
                        parse.as_ref(),
                        format.as_ref(),
                        validate.as_ref(),
                        constraints,
                        on_outcome.as_ref(),
                    ),
                    KeyCode::Escape => axis_drag_value_cancel_typed_text(
                        host,
                        action_cx,
                        &draft,
                        &error,
                        &last_draft_text,
                        &state,
                        &model,
                        format.as_ref(),
                        on_outcome.as_ref(),
                    ),
                    _ => false,
                }
            },
        ),
    );
}

fn axis_drag_value_commit_typed_text<T>(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    last_draft_text: &Arc<Mutex<String>>,
    state: &Arc<Mutex<AxisDragValueState>>,
    model: &Model<T>,
    parse: &dyn Fn(&str) -> Option<T>,
    format: &dyn Fn(T) -> Arc<str>,
    validate: Option<&NumericValidateFn<T>>,
    constraints: NumericValueConstraints,
    on_outcome: Option<&OnAxisDragValueOutcome>,
) -> bool
where
    T: DragValueScalar + Default,
{
    let text = host
        .models_mut()
        .read(draft, |s| s.clone())
        .unwrap_or_default();
    if let Some(value) = parse(&text) {
        let value = constrain_numeric_value(constraints, value);
        if let Some(validate) = validate
            && let Some(msg) = validate(value)
        {
            let _ = host.models_mut().update(error, |e| *e = Some(msg));
            let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
            *last = text;
            host.request_redraw(action_cx.window);
            return true;
        }

        let _ = host.models_mut().update(model, |m| *m = value);
        let formatted = format(value);
        let _ = host
            .models_mut()
            .update(draft, |s| *s = formatted.as_ref().to_string());
        let _ = host.models_mut().update(error, |e| *e = None);
        let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
        *last = formatted.as_ref().to_string();

        return_to_scrub(host, state);
        emit_axis_drag_value_outcome(host, action_cx, on_outcome, AxisDragValueOutcome::Committed);
        host.request_redraw(action_cx.window);
        true
    } else {
        let _ = host
            .models_mut()
            .update(error, |e| *e = Some(Arc::from("Invalid number")));
        let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
        *last = text;
        host.request_redraw(action_cx.window);
        true
    }
}

fn axis_drag_value_cancel_typed_text<T>(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    last_draft_text: &Arc<Mutex<String>>,
    state: &Arc<Mutex<AxisDragValueState>>,
    model: &Model<T>,
    format: &dyn Fn(T) -> Arc<str>,
    on_outcome: Option<&OnAxisDragValueOutcome>,
) -> bool
where
    T: DragValueScalar + Default,
{
    let current = host.models_mut().get_copied(model).unwrap_or_default();
    let formatted = format(current);
    let _ = host
        .models_mut()
        .update(draft, |s| *s = formatted.as_ref().to_string());
    let _ = host.models_mut().update(error, |e| *e = None);
    let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
    *last = formatted.as_ref().to_string();

    return_to_scrub(host, state);
    emit_axis_drag_value_outcome(host, action_cx, on_outcome, AxisDragValueOutcome::Canceled);
    host.request_redraw(action_cx.window);
    true
}

fn return_to_scrub(host: &mut dyn UiFocusActionHost, state: &Arc<Mutex<AxisDragValueState>>) {
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    st.mode = super::super::model::AxisDragValueMode::Scrub;
    st.scrub_revision = st.scrub_revision.wrapping_add(1);
    if let Some(scrub_id) = st.scrub_id {
        host.request_focus(scrub_id);
    }
}
