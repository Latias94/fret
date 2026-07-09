use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{Length, PressableA11y, PressableProps};

use super::super::SliderOptions;

pub(super) fn slider_pressable_props(
    enabled: bool,
    label: Arc<str>,
    options: &SliderOptions,
) -> PressableProps {
    let mut props = PressableProps::default();
    props.enabled = enabled;
    props.focusable = enabled && options.focusable;
    props.layout.size.width = Length::Fill;
    props.layout.size.min_height = Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT));

    props.a11y = PressableA11y {
        role: Some(SemanticsRole::Slider),
        label: options.a11y_label.clone().or(Some(label)),
        test_id: options.test_id.clone(),
        ..Default::default()
    };
    props
}
