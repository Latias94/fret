use crate::foundation::interaction::PressableInteraction;

use super::super::visual_fixture_model::Case;

pub(super) fn enabled_input(case: &Case) -> bool {
    case.input.enabled.unwrap_or(!case.input.disabled)
}

pub(super) fn pressable_interaction(
    value: Option<&str>,
    case_id: &str,
) -> Option<PressableInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(PressableInteraction::Hovered),
        "focused" => Some(PressableInteraction::Focused),
        "pressed" => Some(PressableInteraction::Pressed),
        other => panic!("{case_id}: unsupported pressable interaction {other}"),
    }
}
