use std::sync::Arc;

use fret_ui::element::PressableProps;

use super::super::visual;
use crate::imui::{ButtonOptions, ButtonVariant};

pub(super) fn button_pressable_props(
    label: &Arc<str>,
    options: &ButtonOptions,
    enabled: bool,
    variant: ButtonVariant,
) -> PressableProps {
    let mut props = PressableProps::default();
    props.enabled = enabled;
    props.focusable = enabled && options.focusable;
    visual::apply_button_variant_layout(&mut props, variant);
    props.a11y = visual::button_a11y(label, options, variant);
    props
}
