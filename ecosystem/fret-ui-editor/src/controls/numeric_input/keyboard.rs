use std::sync::{Arc, Mutex};

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, KeyDownCx, OnKeyDown, UiActionHost, UiFocusActionHost};

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
            set_numeric_input_error_if_changed(host, &args.error, message);
            write_last_draft_if_changed(&args.last_draft_text, &text);
            host.request_redraw(action_cx.window);
            return true;
        }

        let _ = host
            .models_mut()
            .update(&args.model, |model| *model = value);
        let formatted = (args.format)(value);
        set_string_model_if_changed(host, &args.draft, formatted.as_ref());
        clear_numeric_input_error_if_present(host, &args.error);
        write_last_draft_if_changed(&args.last_draft_text, formatted.as_ref());
        if let Some(callback) = args.on_outcome.as_ref() {
            callback(host, action_cx, NumericInputOutcome::Committed);
        }
    } else {
        set_numeric_input_error_if_changed(host, &args.error, Arc::from("Invalid number"));
        write_last_draft_if_changed(&args.last_draft_text, &text);
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
    set_string_model_if_changed(host, &args.draft, formatted.as_ref());
    clear_numeric_input_error_if_present(host, &args.error);
    write_last_draft_if_changed(&args.last_draft_text, formatted.as_ref());
    if let Some(callback) = args.on_outcome.as_ref() {
        callback(host, action_cx, NumericInputOutcome::Canceled);
    }
    host.request_redraw(action_cx.window);
    true
}

fn set_string_model_if_changed(
    host: &mut dyn UiActionHost,
    model: &Model<String>,
    next: &str,
) -> bool {
    let unchanged = host
        .models_mut()
        .read(model, |value| value == next)
        .unwrap_or(false);
    if unchanged {
        return false;
    }

    host.models_mut()
        .update(model, |value| {
            value.clear();
            value.push_str(next);
        })
        .is_ok()
}

fn write_last_draft_if_changed(last_draft_text: &Arc<Mutex<String>>, text: &str) {
    let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
    if last.as_str() != text {
        last.clear();
        last.push_str(text);
    }
}

fn clear_numeric_input_error_if_present(
    host: &mut dyn UiActionHost,
    error: &Model<Option<Arc<str>>>,
) -> bool {
    let has_error = host
        .models_mut()
        .read(error, |value| value.is_some())
        .unwrap_or(false);
    if !has_error {
        return false;
    }

    host.models_mut()
        .update(error, |value| *value = None)
        .is_ok()
}

fn set_numeric_input_error_if_changed(
    host: &mut dyn UiActionHost,
    error: &Model<Option<Arc<str>>>,
    message: Arc<str>,
) -> bool {
    let unchanged = host
        .models_mut()
        .read(error, |value| {
            value
                .as_ref()
                .is_some_and(|current| current.as_ref() == message.as_ref())
        })
        .unwrap_or(false);
    if unchanged {
        return false;
    }

    host.models_mut()
        .update(error, |value| *value = Some(message))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_ui::action::UiActionHostAdapter;

    use super::*;

    #[test]
    fn clear_numeric_input_error_skips_empty_error_model() {
        let mut app = App::new();
        let error = app.models_mut().insert(None::<Arc<str>>);
        let revision = error.revision(&app);
        let changed = {
            let mut host = UiActionHostAdapter { app: &mut app };
            clear_numeric_input_error_if_present(&mut host, &error)
        };

        assert!(!changed);
        assert_eq!(error.revision(&app), revision);
    }

    #[test]
    fn clear_numeric_input_error_clears_present_error_model() {
        let mut app = App::new();
        let error = app
            .models_mut()
            .insert(Some(Arc::<str>::from("Invalid number")));
        let revision = error.revision(&app);
        let changed = {
            let mut host = UiActionHostAdapter { app: &mut app };
            clear_numeric_input_error_if_present(&mut host, &error)
        };

        assert!(changed);
        assert_ne!(error.revision(&app), revision);
        assert_eq!(app.models_mut().read(&error, Clone::clone).unwrap(), None);
    }

    #[test]
    fn set_numeric_input_error_skips_unchanged_message() {
        let mut app = App::new();
        let error = app
            .models_mut()
            .insert(Some(Arc::<str>::from("Invalid number")));
        let revision = error.revision(&app);
        let changed = {
            let mut host = UiActionHostAdapter { app: &mut app };
            set_numeric_input_error_if_changed(&mut host, &error, Arc::from("Invalid number"))
        };

        assert!(!changed);
        assert_eq!(error.revision(&app), revision);
    }

    #[test]
    fn set_string_model_if_changed_skips_unchanged_text() {
        let mut app = App::new();
        let draft = app.models_mut().insert(String::from("123"));
        let revision = draft.revision(&app);
        let changed = {
            let mut host = UiActionHostAdapter { app: &mut app };
            set_string_model_if_changed(&mut host, &draft, "123")
        };

        assert!(!changed);
        assert_eq!(draft.revision(&app), revision);
    }
}
