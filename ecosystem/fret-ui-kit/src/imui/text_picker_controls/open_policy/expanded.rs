pub(in crate::imui::text_picker_controls) fn text_picker_expanded(
    popup_is_open: bool,
    input_enabled_by_scope: bool,
    picker_candidate_visible: bool,
    hide_for_exact_match: bool,
) -> bool {
    popup_is_open && input_enabled_by_scope && picker_candidate_visible && !hide_for_exact_match
}
