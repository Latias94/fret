use std::sync::{Arc, Mutex};

use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::{ElementContext, UiHost};

use crate::controls::numeric_input::{
    NumericFormatFn, NumericInput, NumericInputErrorDisplay, NumericInputOptions,
    NumericInputOutcome, NumericParseFn, NumericValidateFn,
};
use crate::controls::slider::model::SliderState;
use crate::controls::slider::pointer::reset_slider_interaction;
use crate::controls::slider::typing::{slider_typing_parse, slider_typing_validate};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::numeric_text_entry::{
    NumericInputSelectionBehavior, NumericTextEntryFocusHandoffState,
    sync_numeric_text_entry_focus_handoff,
};
use fret_ui_kit::Size;

pub(super) struct SliderTypingInputArgs<T> {
    pub(super) model: Model<T>,
    pub(super) format: NumericFormatFn<T>,
    pub(super) parse: NumericParseFn<T>,
    pub(super) validate: Option<NumericValidateFn<T>>,
    pub(super) state: Arc<Mutex<SliderState>>,
    pub(super) focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>>,
    pub(super) min: f64,
    pub(super) max: f64,
    pub(super) clamp: bool,
    pub(super) step: Option<f64>,
    pub(super) enabled: bool,
    pub(super) typing: bool,
    pub(super) input_layout: LayoutStyle,
    pub(super) prefix: Option<Arc<str>>,
    pub(super) suffix: Option<Arc<str>>,
    pub(super) selection_behavior: NumericInputSelectionBehavior,
    pub(super) active_typing_test_id: Option<Arc<str>>,
}

pub(super) fn slider_typing_input<T, H>(
    cx: &mut ElementContext<'_, H>,
    args: SliderTypingInputArgs<T>,
) -> AnyElement
where
    T: DragValueScalar + Default,
    H: UiHost,
{
    let SliderTypingInputArgs {
        model,
        format,
        parse,
        validate,
        state,
        focus_handoff,
        min,
        max,
        clamp,
        step,
        enabled,
        typing,
        input_layout,
        prefix,
        suffix,
        selection_behavior,
        active_typing_test_id,
    } = args;

    let parse_for_input = slider_typing_parse(parse, min, max, clamp, step);
    let validate_for_input = slider_typing_validate(validate, min, max, clamp);

    let state_for_input = state.clone();
    let input_focus_target: Arc<Mutex<Option<fret_ui::GlobalElementId>>> =
        Arc::new(Mutex::new(None));
    let mut input_layout = input_layout;
    let hidden_layout = (!typing).then_some(input_layout);
    if hidden_layout.is_some() {
        input_layout = LayoutStyle::default();
    }
    let input = NumericInput::new(model, format, parse_for_input)
        .validate(validate_for_input)
        .focus_target(input_focus_target.clone())
        .options(NumericInputOptions {
            layout: input_layout,
            size: Size::Small,
            enabled: enabled && typing,
            focusable: enabled && typing,
            prefix,
            suffix,
            selection_behavior,
            test_id: active_typing_test_id,
            // Avoid growing the row height when a commit-time validation error occurs.
            // A small trailing status icon keeps the inspector layout stable.
            error_display: NumericInputErrorDisplay::TrailingIcon,
            ..Default::default()
        })
        .on_outcome(Some(Arc::new(move |host, action_cx, outcome| {
            if matches!(
                outcome,
                NumericInputOutcome::Committed | NumericInputOutcome::Canceled
            ) {
                let mut st = state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                reset_slider_interaction(&mut st);
                if let Some(slider_id) = st.slider_id {
                    host.request_focus(slider_id);
                }
                host.request_redraw(action_cx.window);
            }
        })))
        .into_element_with_hidden_text_entry_layout(cx, hidden_layout);

    if let Some(input_id) = input_focus_target
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .copied()
    {
        let is_focused = cx.is_focused_element(input_id);
        sync_numeric_text_entry_focus_handoff(
            cx,
            input.id,
            &focus_handoff,
            typing,
            input_id,
            is_focused,
        );
    }

    input
}
