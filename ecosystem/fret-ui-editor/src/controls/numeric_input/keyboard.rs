use std::sync::{Arc, Mutex};

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, KeyDownCx, OnKeyDown, UiFocusActionHost};

use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusState, handle_numeric_text_entry_replace_key,
};

use super::{
    NumericFormatFn, NumericInputOutcome, NumericParseFn, NumericValidateFn, OnNumericInputOutcome,
};

pub(super) struct NumericInputKeyHandlerArgs<T> {
    pub(super) model: Model<T>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) focus_state: Arc<Mutex<NumericTextEntryFocusState>>,
    pub(super) last_draft_text: Arc<Mutex<String>>,
    pub(super) parse: NumericParseFn<T>,
    pub(super) format: NumericFormatFn<T>,
    pub(super) validate: Option<NumericValidateFn<T>>,
    pub(super) on_outcome: Option<OnNumericInputOutcome>,
}

pub(super) fn numeric_input_key_down_handler<T>(args: NumericInputKeyHandlerArgs<T>) -> OnKeyDown
where
    T: Copy + Default + 'static,
{
    Arc::new(move |host, action_cx: ActionCx, down: KeyDownCx| {
        if let Some(consumed) = handle_numeric_text_entry_replace_key(
            host,
            action_cx,
            down,
            &args.focus_state,
            &args.draft,
            &args.error,
        ) && consumed
        {
            return true;
        }

        match down.key {
            KeyCode::Enter | KeyCode::NumpadEnter => commit_numeric_input(host, action_cx, &args),
            KeyCode::Escape => cancel_numeric_input(host, action_cx, &args),
            _ => false,
        }
    })
}

fn commit_numeric_input<T>(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    args: &NumericInputKeyHandlerArgs<T>,
) -> bool
where
    T: Copy + Default + 'static,
{
    let text = host
        .models_mut()
        .read(&args.draft, |s| s.clone())
        .unwrap_or_default();
    if let Some(value) = (args.parse)(&text) {
        if let Some(validate) = args.validate.as_ref()
            && let Some(message) = validate(value)
        {
            let _ = host
                .models_mut()
                .update(&args.error, |error| *error = Some(message));
            write_last_draft(&args.last_draft_text, text);
            host.request_redraw(action_cx.window);
            return true;
        }

        let _ = host
            .models_mut()
            .update(&args.model, |model| *model = value);
        let formatted = (args.format)(value);
        let _ = host
            .models_mut()
            .update(&args.draft, |draft| *draft = formatted.as_ref().to_string());
        let _ = host.models_mut().update(&args.error, |error| *error = None);
        write_last_draft(&args.last_draft_text, formatted.as_ref().to_string());
        if let Some(callback) = args.on_outcome.as_ref() {
            callback(host, action_cx, NumericInputOutcome::Committed);
        }
    } else {
        let _ = host.models_mut().update(&args.error, |error| {
            *error = Some(Arc::from("Invalid number"))
        });
        write_last_draft(&args.last_draft_text, text);
    }
    host.request_redraw(action_cx.window);
    true
}

fn cancel_numeric_input<T>(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    args: &NumericInputKeyHandlerArgs<T>,
) -> bool
where
    T: Copy + Default + 'static,
{
    let current = host
        .models_mut()
        .get_copied(&args.model)
        .unwrap_or_default();
    let formatted = (args.format)(current);
    let _ = host
        .models_mut()
        .update(&args.draft, |draft| *draft = formatted.as_ref().to_string());
    let _ = host.models_mut().update(&args.error, |error| *error = None);
    write_last_draft(&args.last_draft_text, formatted.as_ref().to_string());
    if let Some(callback) = args.on_outcome.as_ref() {
        callback(host, action_cx, NumericInputOutcome::Canceled);
    }
    host.request_redraw(action_cx.window);
    true
}

fn write_last_draft(last_draft_text: &Arc<Mutex<String>>, text: String) {
    let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
    *last = text;
}
