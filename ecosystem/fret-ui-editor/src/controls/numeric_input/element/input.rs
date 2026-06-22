//! NumericInput text-entry mounting owner.

use std::sync::{Arc, Mutex};

use fret_core::{SemanticsInvalid, TextStyle};
use fret_runtime::Model;
use fret_ui::TextInputStyle;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::joined_text_input_style;
use crate::primitives::numeric_text_entry::{
    NumericInputSelectionBehavior, NumericTextEntryFocusState,
    clear_numeric_error_when_draft_changes, sync_numeric_text_entry_focus,
};

use super::super::keyboard::{NumericInputKeyHandlerArgs, numeric_input_key_down_handler};
use super::super::model::{
    NumericFormatFn, NumericParseFn, NumericValidateFn, OnNumericInputOutcome,
};

pub(super) struct NumericInputTextEntryArgs<T> {
    pub(super) layout: LayoutStyle,
    pub(super) model: Model<T>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) focus_state: Arc<Mutex<NumericTextEntryFocusState>>,
    pub(super) last_draft_text: Arc<Mutex<String>>,
    pub(super) current_text: Arc<str>,
    pub(super) has_error: bool,
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) placeholder: Option<Arc<str>>,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) chrome: TextInputStyle,
    pub(super) text_style: TextStyle,
    pub(super) focus_target: Option<Arc<Mutex<Option<GlobalElementId>>>>,
    pub(super) selection_behavior: NumericInputSelectionBehavior,
    pub(super) parse: NumericParseFn<T>,
    pub(super) format: NumericFormatFn<T>,
    pub(super) validate: Option<NumericValidateFn<T>>,
    pub(super) on_outcome: Option<OnNumericInputOutcome>,
}

pub(super) fn numeric_input_text_entry<T, H>(
    cx: &mut ElementContext<'_, H>,
    args: NumericInputTextEntryArgs<T>,
) -> AnyElement
where
    T: Copy + Default + 'static,
    H: UiHost,
{
    let NumericInputTextEntryArgs {
        model,
        draft,
        error,
        focus_state,
        last_draft_text,
        current_text,
        has_error,
        enabled,
        focusable,
        placeholder,
        test_id,
        chrome,
        text_style,
        focus_target,
        selection_behavior,
        parse,
        format,
        validate,
        on_outcome,
        layout,
    } = args;

    let mut props = TextInputProps::new(draft.clone());
    props.layout = layout;
    props.enabled = enabled;
    props.focusable = focusable;
    props.placeholder = placeholder;
    props.test_id = test_id;
    props.a11y_invalid = has_error.then_some(SemanticsInvalid::True);
    props.chrome = joined_text_input_style(chrome);
    props.text_style = text_style;

    let input = cx.text_input(props);
    let input_id = input.id;
    if let Some(focus_target) = focus_target.as_ref() {
        let mut slot = focus_target.lock().unwrap_or_else(|e| e.into_inner());
        *slot = Some(input_id);
    }
    let is_focused = cx.is_focused_element(input_id);

    sync_numeric_text_entry_focus(
        cx,
        &focus_state,
        is_focused,
        &current_text,
        &draft,
        &error,
        selection_behavior,
    );

    if !is_focused {
        let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
        *last = current_text.as_ref().to_string();
    }

    cx.key_add_on_key_down_capture_for(
        input_id,
        numeric_input_key_down_handler(NumericInputKeyHandlerArgs {
            model: model.clone(),
            draft: draft.clone(),
            error: error.clone(),
            focus_state: focus_state.clone(),
            last_draft_text: last_draft_text.clone(),
            parse: parse.clone(),
            format: format.clone(),
            validate: validate.clone(),
            on_outcome: on_outcome.clone(),
        }),
    );

    clear_numeric_error_when_draft_changes(cx, is_focused, &draft, &error, &last_draft_text);

    input
}

pub(super) fn numeric_input_text_entry_fill_layout(density: EditorDensity) -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            min_height: Some(Length::Px(density.row_height)),
            ..Default::default()
        },
        ..Default::default()
    }
}
