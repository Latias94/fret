use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{Length, PressableA11y, PressableProps};

use super::super::super::CheckboxOptions;

pub(super) fn checkbox_pressable_props(
    label: Arc<str>,
    value: bool,
    options: &CheckboxOptions,
    enabled: bool,
    focusable: bool,
) -> PressableProps {
    let mut props = PressableProps::default();
    props.enabled = enabled;
    props.focusable = focusable;
    props.layout.size.width = Length::Fill;
    props.layout.size.min_height = Some(Length::Px(
        super::super::super::control_chrome::FIELD_MIN_HEIGHT,
    ));
    props.a11y = PressableA11y {
        role: Some(SemanticsRole::Checkbox),
        label: options.a11y_label.clone().or(Some(label)),
        checked: Some(value),
        test_id: options.test_id.clone(),
        ..Default::default()
    };
    props
}
