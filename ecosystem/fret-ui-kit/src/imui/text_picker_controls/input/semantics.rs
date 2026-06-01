use fret_ui::GlobalElementId;

use crate::imui::text_controls::InputTextAssistiveSemantics;

pub(super) fn input_root_assistive_semantics(
    picker_expanded: bool,
    active_element: Option<GlobalElementId>,
    popup_panel_id: Option<GlobalElementId>,
) -> InputTextAssistiveSemantics {
    InputTextAssistiveSemantics {
        active_descendant: None,
        active_descendant_element: picker_expanded
            .then_some(active_element)
            .flatten()
            .map(|element| element.0),
        controls_element: picker_expanded
            .then_some(popup_panel_id)
            .flatten()
            .map(|element| element.0),
        expanded: Some(picker_expanded),
    }
}
