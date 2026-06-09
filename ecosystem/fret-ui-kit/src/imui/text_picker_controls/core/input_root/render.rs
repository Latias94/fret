use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::super::super::super::{InputTextOptions, UiWriterImUiFacadeExt};
use super::super::super::input::{
    BuiltInputTextPickerInputRoot, InputTextPickerInputRootRequest, render_text_picker_input_root,
};
use super::super::super::keyboard::InputTextPickerKeyboardState;
use super::TextPickerInputRootResult;

pub(super) struct CoreTextPickerInputRootRenderInput<'a> {
    pub(super) model: Model<String>,
    pub(super) input_options: InputTextOptions,
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
    pub(super) item_test_id_base: Option<Arc<str>>,
}

pub(super) fn render_core_text_picker_input_root<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    input: CoreTextPickerInputRootRenderInput<'_>,
) -> TextPickerInputRootResult {
    let CoreTextPickerInputRootRenderInput {
        model,
        input_options,
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
        item_test_id_base,
    } = input;

    let input_root: BuiltInputTextPickerInputRoot = ui.with_cx_mut(|cx| {
        render_text_picker_input_root(
            cx,
            InputTextPickerInputRootRequest {
                model,
                input_options,
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
