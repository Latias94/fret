use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::super::super::{InputTextPickerOptions, ResponseExt, UiWriterImUiFacadeExt};
use super::super::input::{
    BuiltInputTextPickerInputRoot, InputTextPickerInputRootRequest,
    prepare_text_picker_input_options, render_text_picker_input_root,
};
use super::super::keyboard::InputTextPickerKeyboardState;

pub(super) struct TextPickerInputRootInput<'a> {
    pub(super) model: Model<String>,
    pub(super) options: &'a InputTextPickerOptions,
    pub(super) popup_open: Model<bool>,
    pub(super) keyboard_state: Option<Model<InputTextPickerKeyboardState>>,
    pub(super) visible_candidates: &'a [(usize, Arc<str>)],
    pub(super) keyboard_navigation: bool,
    pub(super) keyboard_repeat: bool,
    pub(super) picker_candidate_visible: bool,
    pub(super) hide_for_exact_match: bool,
    pub(super) picker_expanded: bool,
    pub(super) active_element: Option<fret_ui::GlobalElementId>,
    pub(super) popup_panel_id: Option<fret_ui::GlobalElementId>,
}

pub(super) struct TextPickerInputRootResult {
    pub(super) input: ResponseExt,
    pub(super) item_test_id_base: Option<Arc<str>>,
}

pub(super) fn build_text_picker_input_root<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    input: TextPickerInputRootInput<'_>,
) -> TextPickerInputRootResult {
    let TextPickerInputRootInput {
        model,
        options,
        popup_open,
        keyboard_state,
        visible_candidates,
        keyboard_navigation,
        keyboard_repeat,
        picker_candidate_visible,
        hide_for_exact_match,
        picker_expanded,
        active_element,
        popup_panel_id,
    } = input;

    let prepared_input = prepare_text_picker_input_options(options);
    let item_test_id_base = prepared_input.test_id;
    let input_root: BuiltInputTextPickerInputRoot = ui.with_cx_mut(|cx| {
        render_text_picker_input_root(
            cx,
            InputTextPickerInputRootRequest {
                model,
                input_options: prepared_input.options,
                popup_open,
                keyboard_state,
                visible_candidates,
                keyboard_navigation,
                keyboard_repeat,
                picker_candidate_visible,
                hide_for_exact_match,
                picker_expanded,
                active_element,
                popup_panel_id,
            },
        )
    });
    ui.add(input_root.root);
    TextPickerInputRootResult {
        input: input_root.response,
        item_test_id_base,
    }
}
