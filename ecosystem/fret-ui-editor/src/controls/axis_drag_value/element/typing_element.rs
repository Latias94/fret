//! AxisDragValue typing-branch orchestration owner.

use std::sync::{Arc, Mutex};

use fret_core::{Color, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::{ElementContext, UiHost};

use crate::controls::numeric_input::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::numeric_text_entry::{
    NumericInputSelectionBehavior, NumericTextEntryFocusHandoffState,
    numeric_text_entry_focus_state,
};
use crate::primitives::{EditorDensity, NumericValueConstraints};

use super::super::model::{AxisDragValueResetAction, AxisDragValueState, OnAxisDragValueOutcome};
use super::super::session::{draft_model, error_model};
use super::input::{AxisDragValueTypingInputArgs, axis_drag_value_typing_input};
use super::typing::{AxisDragValueTypingFrameArgs, axis_drag_value_typing_field};
use super::typing_focus::{
    AxisDragValueTypingFocusArgs, axis_drag_value_clear_typing_error_when_draft_changes,
    axis_drag_value_sync_typing_focus,
};
use super::typing_keys::{
    AxisDragValueTypingKeyHandlerArgs, axis_drag_value_add_typing_key_handler,
};

pub(super) struct AxisDragValueTypingElementArgs<T> {
    pub(super) state: Arc<Mutex<AxisDragValueState>>,
    pub(super) focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>>,
    pub(super) model: Model<T>,
    pub(super) parse: NumericParseFn<T>,
    pub(super) format: NumericFormatFn<T>,
    pub(super) validate: Option<NumericValidateFn<T>>,
    pub(super) on_outcome: Option<OnAxisDragValueOutcome>,
    pub(super) value_text: Arc<str>,
    pub(super) typing: bool,
    pub(super) layout: LayoutStyle,
    pub(super) constraints: NumericValueConstraints,
    pub(super) density: EditorDensity,
    pub(super) frame_chrome: ResolvedEditorFrameChrome,
    pub(super) input_chrome: fret_ui::TextInputStyle,
    pub(super) text_style: TextStyle,
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) selection_behavior: NumericInputSelectionBehavior,
    pub(super) axis_label: Arc<str>,
    pub(super) axis_tint: Color,
    pub(super) prefix: Option<Arc<str>>,
    pub(super) suffix: Option<Arc<str>>,
    pub(super) reset_action: Option<AxisDragValueResetAction>,
    pub(super) active_typing_test_id: Option<Arc<str>>,
    pub(super) typing_axis_test_id: Option<Arc<str>>,
    pub(super) typing_input_test_id: Option<Arc<str>>,
    pub(super) typing_prefix_test_id: Option<Arc<str>>,
    pub(super) typing_suffix_test_id: Option<Arc<str>>,
    pub(super) typing_error_icon_test_id: Option<Arc<str>>,
    pub(super) typing_reset_test_id: Option<Arc<str>>,
}

pub(super) fn axis_drag_value_typing_element<T, H>(
    cx: &mut ElementContext<'_, H>,
    args: AxisDragValueTypingElementArgs<T>,
) -> AnyElement
where
    T: DragValueScalar + Default,
    H: UiHost,
{
    let AxisDragValueTypingElementArgs {
        state,
        focus_handoff,
        model,
        parse,
        format,
        validate,
        on_outcome,
        value_text,
        typing,
        layout,
        constraints,
        density,
        frame_chrome,
        input_chrome,
        text_style,
        enabled,
        focusable,
        selection_behavior,
        axis_label,
        axis_tint,
        prefix,
        suffix,
        reset_action,
        active_typing_test_id,
        typing_axis_test_id,
        typing_input_test_id,
        typing_prefix_test_id,
        typing_suffix_test_id,
        typing_error_icon_test_id,
        typing_reset_test_id,
    } = args;

    let draft = draft_model(cx);
    let error = error_model(cx);
    let focus_state = numeric_text_entry_focus_state(cx);
    let last_draft_text = cx.slot_state(|| Arc::new(Mutex::new(String::new())), |st| st.clone());

    let input_group_layout = layout;
    let has_error = cx
        .get_model_cloned(&error, fret_ui::Invalidation::Paint)
        .unwrap_or(None)
        .is_some();

    let typing_input = axis_drag_value_typing_input(
        cx,
        AxisDragValueTypingInputArgs {
            draft: draft.clone(),
            density,
            layout,
            input_chrome,
            text_style: text_style.clone(),
            enabled,
            focusable,
            typing,
            typing_input_test_id: typing_input_test_id.clone(),
            has_error,
        },
    );
    let input = typing_input.input;
    let input_id = typing_input.input_id;
    let is_focused = typing_input.is_focused;

    if !typing {
        return input;
    }

    axis_drag_value_sync_typing_focus(
        cx,
        AxisDragValueTypingFocusArgs {
            state: state.clone(),
            focus_state: focus_state.clone(),
            focus_handoff: focus_handoff.clone(),
            draft: draft.clone(),
            error: error.clone(),
            last_draft_text: last_draft_text.clone(),
            value_text: value_text.clone(),
            typing,
            input_id,
            is_focused,
            selection_behavior,
        },
    );

    axis_drag_value_add_typing_key_handler(
        cx,
        AxisDragValueTypingKeyHandlerArgs {
            input_id,
            focus_state,
            draft: draft.clone(),
            error: error.clone(),
            last_draft_text: last_draft_text.clone(),
            state,
            model,
            parse,
            format,
            validate,
            constraints,
            on_outcome,
        },
    );

    axis_drag_value_clear_typing_error_when_draft_changes(
        cx,
        is_focused,
        &draft,
        &error,
        &last_draft_text,
    );

    axis_drag_value_typing_field(
        cx,
        AxisDragValueTypingFrameArgs {
            layout: input_group_layout,
            density,
            frame_chrome,
            is_focused,
            has_error,
            input,
            axis_label,
            axis_tint,
            prefix,
            suffix,
            reset_action,
            enabled,
            active_typing_test_id,
            typing_axis_test_id,
            typing_prefix_test_id,
            typing_suffix_test_id,
            typing_error_icon_test_id,
            typing_reset_test_id,
        },
    )
}
