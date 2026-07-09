use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{Length, PressableA11y, PressableProps};

use super::super::super::RadioOptions;

pub(super) fn radio_pressable_props(
    label: Arc<str>,
    selected: bool,
    options: &RadioOptions,
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
        role: Some(SemanticsRole::RadioButton),
        label: options.a11y_label.clone().or(Some(label)),
        checked: Some(selected),
        test_id: options.test_id.clone(),
        ..Default::default()
    };
    props
}
