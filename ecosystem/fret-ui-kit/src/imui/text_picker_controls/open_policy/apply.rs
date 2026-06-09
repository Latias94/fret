use fret_core::Rect;
use fret_ui::UiHost;

use super::super::super::UiWriterImUiFacadeExt;

pub(in crate::imui::text_picker_controls) struct TextPickerOpenPolicyInput {
    pub(in crate::imui::text_picker_controls) enabled: bool,
    pub(in crate::imui::text_picker_controls) visible_candidates_empty: bool,
    pub(in crate::imui::text_picker_controls) hide_for_exact_match: bool,
    pub(in crate::imui::text_picker_controls) open_on_focus: bool,
    pub(in crate::imui::text_picker_controls) input_focused: bool,
    pub(in crate::imui::text_picker_controls) picker_candidate_visible: bool,
    pub(in crate::imui::text_picker_controls) anchor: Option<Rect>,
}

pub(in crate::imui::text_picker_controls) fn apply_text_picker_open_policy<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    input: TextPickerOpenPolicyInput,
) {
    if input.enabled && (input.visible_candidates_empty || input.hide_for_exact_match) {
        ui.close_popup(id);
    }
    if input.enabled
        && input.open_on_focus
        && input.input_focused
        && input.picker_candidate_visible
        && !input.hide_for_exact_match
        && let Some(anchor) = input.anchor
    {
        ui.open_popup_at(id, anchor);
    }
}
