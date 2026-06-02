use std::sync::Arc;

use fret_ui::element::{Length, PressableProps};

use super::super::super::SwitchOptions;

pub(super) fn switch_pressable_props(
    label: Arc<str>,
    value: bool,
    options: &SwitchOptions,
    enabled: bool,
) -> PressableProps {
    let mut props = PressableProps::default();
    props.enabled = enabled;
    props.focusable = enabled && options.focusable;
    props.layout.size.width = Length::Fill;
    props.layout.size.min_height = Some(Length::Px(
        super::super::super::control_chrome::FIELD_MIN_HEIGHT,
    ));
    props.a11y =
        crate::primitives::switch::switch_a11y(options.a11y_label.clone().or(Some(label)), value);
    props.a11y.test_id = options.test_id.clone();
    props
}
