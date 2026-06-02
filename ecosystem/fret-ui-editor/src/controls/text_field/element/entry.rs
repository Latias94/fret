//! TextField entry mount and session wiring owner.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use fret_core::Px;
use fret_runtime::{CommandId, Model};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};
use fret_ui_kit::Size;

use crate::primitives::EditorDensity;
use crate::primitives::text_entry::{
    EditorTextCancelBehavior, EditorTextEntryFocusState, EditorTextSelectionBehavior,
};

use super::super::buffered::{self, BufferedTextFieldState, TextFieldDraftController};
use super::super::{
    OnTextFieldOutcome, TextFieldAssistiveSemantics, TextFieldBlurBehavior, TextFieldMode,
};
use super::buffered_keys::{
    TextFieldBufferedKeyHandlerArgs, TextFieldBufferedKeyMode,
    install_buffered_text_field_key_handler,
};
use super::entry_props::{
    TextFieldAreaPropsArgs, TextFieldInputPropsArgs, text_field_area_props, text_field_input_props,
};
use super::escape_clear::install_text_field_escape_clear_handler;
use super::focus::{TextFieldFocusSelectionArgs, sync_text_field_focus_selection};

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
) -> AnyElement {
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
        let input_model = draft.clone().unwrap_or_else(|| model.clone());
        let theme = Theme::global(&*cx.app);
        let props = text_field_area_props(
            theme,
            TextFieldAreaPropsArgs {
                input_model,
                size,
                density,
                enabled,
                focusable,
                a11y_label: a11y_label.clone(),
                test_id: test_id.clone(),
                stable_line_boxes,
                min_height,
            },
        );

        let area = cx.text_area(props);
        if let Some(out) = input_id_out.as_ref() {
            out.set(Some(area.id));
        }
        let area_id = area.id;
        let is_focused = cx.is_focused_element(area_id);
        if let (Some(draft), Some(buffered_state)) = (draft.as_ref(), buffered_state.as_ref()) {
            buffered::sync_buffered_text_field_session(
                cx,
                area_id,
                is_focused,
                &current_text,
                draft,
                buffered_state,
                blur_behavior,
            );
            if let Some(controller) = draft_controller.as_ref() {
                controller.bind(model.clone(), draft.clone(), buffered_state.clone(), None);
            }

            install_buffered_text_field_key_handler(
                cx,
                TextFieldBufferedKeyHandlerArgs {
                    entry_id: area_id,
                    mode: TextFieldBufferedKeyMode::Multiline,
                    model: model.clone(),
                    draft: draft.clone(),
                    buffered_state: buffered_state.clone(),
                    on_outcome: on_outcome.clone(),
                },
            );
        }

        sync_text_field_focus_selection(
            cx,
            TextFieldFocusSelectionArgs {
                focus_state: &focus_state,
                entry_id: area_id,
                is_focused,
                model: &model,
                draft: draft.as_ref(),
                selection_behavior,
            },
        );
        if let (Some(draft), Some(buffered_state)) = (draft.as_ref(), buffered_state.as_ref()) {
            buffered::install_buffered_text_field_blur_handler(
                cx,
                area_id,
                model.clone(),
                draft.clone(),
                buffered_state.clone(),
                on_outcome.clone(),
            );
        }
        if !buffered && matches!(cancel_behavior, EditorTextCancelBehavior::Clear) {
            install_text_field_escape_clear_handler(cx, area_id, model.clone());
        }

        area
    } else {
        let input_model = draft.clone().unwrap_or_else(|| model.clone());
        let theme = Theme::global(&*cx.app);
        let props = text_field_input_props(
            theme,
            TextFieldInputPropsArgs {
                input_model,
                size,
                density,
                enabled,
                focusable,
                placeholder: placeholder.clone(),
                a11y_label: a11y_label.clone(),
                test_id: test_id.clone(),
                mode,
                assistive_semantics,
                buffered,
                submit_command: submit_command.clone(),
                cancel_behavior,
            },
        );

        let input = cx.text_input(props);
        if let Some(out) = input_id_out.as_ref() {
            out.set(Some(input.id));
        }
        let input_id = input.id;
        let is_focused = cx.is_focused_element(input_id);

        if let (Some(draft), Some(buffered_state)) = (draft.as_ref(), buffered_state.as_ref()) {
            buffered::sync_buffered_text_field_session(
                cx,
                input_id,
                is_focused,
                &current_text,
                draft,
                buffered_state,
                blur_behavior,
            );
            if let Some(controller) = draft_controller.as_ref() {
                controller.bind(
                    model.clone(),
                    draft.clone(),
                    buffered_state.clone(),
                    submit_command.clone(),
                );
            }

            install_buffered_text_field_key_handler(
                cx,
                TextFieldBufferedKeyHandlerArgs {
                    entry_id: input_id,
                    mode: TextFieldBufferedKeyMode::SingleLine {
                        submit_command: submit_command.clone(),
                    },
                    model: model.clone(),
                    draft: draft.clone(),
                    buffered_state: buffered_state.clone(),
                    on_outcome: on_outcome.clone(),
                },
            );
        }

        sync_text_field_focus_selection(
            cx,
            TextFieldFocusSelectionArgs {
                focus_state: &focus_state,
                entry_id: input_id,
                is_focused,
                model: &model,
                draft: draft.as_ref(),
                selection_behavior,
            },
        );
        if let (Some(draft), Some(buffered_state)) = (draft.as_ref(), buffered_state.as_ref()) {
            buffered::install_buffered_text_field_blur_handler(
                cx,
                input_id,
                model.clone(),
                draft.clone(),
                buffered_state.clone(),
                on_outcome.clone(),
            );
        }
        input
    }
}
