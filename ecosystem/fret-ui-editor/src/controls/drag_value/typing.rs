use std::sync::{Arc, Mutex};

use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::{ElementContext, UiHost};

use super::OnDragValueOutcome;
use super::model::{DragValueMode, DragValueState};
use super::session::{drag_value_outcome_from_numeric_input, emit_drag_value_outcome};
use crate::controls::numeric_input::{
    NumericFormatFn, NumericInput, NumericInputErrorDisplay, NumericInputOptions,
    NumericInputOutcome, NumericInputSelectionBehavior, NumericParseFn, NumericValidateFn,
};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusHandoffState, sync_numeric_text_entry_focus_handoff,
};
use crate::primitives::{NumericValueConstraints, constrain_numeric_value};
use fret_ui_kit::Size;

pub(super) struct DragValueTypingInputArgs<T> {
    pub(super) model: Model<T>,
    pub(super) format: NumericFormatFn<T>,
    pub(super) parse: NumericParseFn<T>,
    pub(super) validate: Option<NumericValidateFn<T>>,
    pub(super) constraints: NumericValueConstraints,
    pub(super) input_layout: LayoutStyle,
    pub(super) typing: bool,
    pub(super) prefix: Option<Arc<str>>,
    pub(super) suffix: Option<Arc<str>>,
    pub(super) selection_behavior: NumericInputSelectionBehavior,
    pub(super) active_typing_test_id: Option<Arc<str>>,
    pub(super) state: Arc<Mutex<DragValueState>>,
    pub(super) focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>>,
    pub(super) on_outcome: Option<OnDragValueOutcome>,
}

pub(super) fn drag_value_typing_input<H, T>(
    cx: &mut ElementContext<'_, H>,
    args: DragValueTypingInputArgs<T>,
) -> AnyElement
where
    H: UiHost,
    T: DragValueScalar + Default,
{
    let DragValueTypingInputArgs {
        model,
        format,
        parse,
        validate,
        constraints,
        input_layout,
        typing,
        prefix,
        suffix,
        selection_behavior,
        active_typing_test_id,
        state,
        focus_handoff,
        on_outcome,
    } = args;

    let input_focus_target: Arc<Mutex<Option<fret_ui::GlobalElementId>>> =
        Arc::new(Mutex::new(None));
    let constrained_parse: NumericParseFn<T> =
        Arc::new(move |text| parse(text).map(|value| constrain_numeric_value(constraints, value)));
    let mut input_layout = input_layout;
    let hidden_layout = (!typing).then_some(input_layout);
    if hidden_layout.is_some() {
        input_layout = LayoutStyle::default();
    }

    let input = NumericInput::new(model, format, constrained_parse)
        .validate(validate)
        .focus_target(input_focus_target.clone())
        .options(NumericInputOptions {
            layout: input_layout,
            size: Size::Small,
            enabled: typing,
            focusable: typing,
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
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            match outcome {
                NumericInputOutcome::Committed | NumericInputOutcome::Canceled => {
                    st.mode = DragValueMode::Scrub;
                    st.scrub_revision = st.scrub_revision.wrapping_add(1);
                    if let Some(scrub_id) = st.scrub_id {
                        host.request_focus(scrub_id);
                    }
                    emit_drag_value_outcome(
                        host,
                        action_cx,
                        on_outcome.as_ref(),
                        drag_value_outcome_from_numeric_input(outcome),
                    );
                    host.request_redraw(action_cx.window);
                }
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
