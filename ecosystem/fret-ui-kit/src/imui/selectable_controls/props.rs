use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{Length, PressableA11y, PressableProps};

use super::super::SelectableOptions;

pub(super) fn selectable_pressable_props(
    label: &Arc<str>,
    options: &SelectableOptions,
    enabled: bool,
    focusable: bool,
    selected: bool,
) -> PressableProps {
    let mut props = PressableProps::default();
    props.enabled = enabled;
    props.focusable = focusable;
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    props.a11y = PressableA11y {
        role: options.a11y_role.or(Some(SemanticsRole::ListBoxOption)),
        label: options.a11y_label.clone().or_else(|| Some(label.clone())),
        test_id: options.test_id.clone(),
        selected,
        ..Default::default()
    };
    props
}
