//! TextField entry mount and session wiring owner.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use fret_core::Px;
use fret_runtime::{CommandId, Model};
use fret_ui::{ElementContext, GlobalElementId, UiHost};
use fret_ui_kit::Size;

use crate::primitives::EditorDensity;
use crate::primitives::text_entry::{
    EditorTextCancelBehavior, EditorTextEntryFocusState, EditorTextSelectionBehavior,
};

use super::super::buffered::{BufferedTextFieldState, TextFieldDraftController};
use super::super::{
    OnTextFieldOutcome, TextFieldAssistiveSemantics, TextFieldBlurBehavior, TextFieldMode,
};

mod multiline;
mod single_line;

use multiline::{TextFieldMultilineEntryArgs, text_field_multiline_entry};
use single_line::{TextFieldSingleLineEntryArgs, text_field_single_line_entry};

pub(super) struct TextFieldEntryArgs {
    pub(super) model: Model<String>,
    pub(super) draft: Option<Model<String>>,
    pub(super) buffered_state: Option<Arc<Mutex<BufferedTextFieldState>>>,
    pub(super) current_text: String,
    pub(super) draft_controller: Option<TextFieldDraftController>,
    pub(super) on_outcome: Option<OnTextFieldOutcome>,
    pub(super) submit_command: Option<CommandId>,
    pub(super) focus_state: Arc<Mutex<EditorTextEntryFocusState>>,
    pub(super) size: Size,
    pub(super) density: EditorDensity,
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) placeholder: Option<Arc<str>>,
    pub(super) a11y_label: Option<Arc<str>>,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) input_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    pub(super) mode: TextFieldMode,
    pub(super) buffered: bool,
    pub(super) blur_behavior: TextFieldBlurBehavior,
    pub(super) assistive_semantics: TextFieldAssistiveSemantics,
    pub(super) selection_behavior: EditorTextSelectionBehavior,
    pub(super) cancel_behavior: EditorTextCancelBehavior,
    pub(super) multiline: bool,
    pub(super) stable_line_boxes: bool,
    pub(super) min_height: Option<Px>,
}

pub(super) fn text_field_entry<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: TextFieldEntryArgs,
) -> fret_ui::element::AnyElement {
    let TextFieldEntryArgs {
        model,
        draft,
        buffered_state,
        current_text,
        draft_controller,
        on_outcome,
        submit_command,
        focus_state,
        size,
        density,
        enabled,
        focusable,
        placeholder,
        a11y_label,
        test_id,
        input_id_out,
        mode,
        buffered,
        blur_behavior,
        assistive_semantics,
        selection_behavior,
        cancel_behavior,
        multiline,
        stable_line_boxes,
        min_height,
    } = args;

    if multiline {
        text_field_multiline_entry(
            cx,
            TextFieldMultilineEntryArgs {
                model,
                draft,
                buffered_state,
                current_text,
                draft_controller,
                on_outcome,
                focus_state,
                size,
                density,
                enabled,
                focusable,
                a11y_label,
                test_id,
                input_id_out,
                buffered,
                blur_behavior,
                selection_behavior,
                cancel_behavior,
                stable_line_boxes,
                min_height,
            },
        )
    } else {
        text_field_single_line_entry(
            cx,
            TextFieldSingleLineEntryArgs {
                model,
                draft,
                buffered_state,
                current_text,
                draft_controller,
                on_outcome,
                submit_command,
                focus_state,
                size,
                density,
                enabled,
                focusable,
                placeholder,
                a11y_label,
                test_id,
                input_id_out,
                mode,
                buffered,
                blur_behavior,
                assistive_semantics,
                selection_behavior,
                cancel_behavior,
            },
        )
    }
}
