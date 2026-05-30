use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{Length, PressableA11y, PressableProps};

use crate::imui::control_chrome;

pub(in crate::imui::combo_controls::trigger) struct ComboTriggerPropsInput {
    pub(in crate::imui::combo_controls::trigger) enabled: bool,
    pub(in crate::imui::combo_controls::trigger) focusable: bool,
    pub(in crate::imui::combo_controls::trigger) a11y_label: Option<Arc<str>>,
    pub(in crate::imui::combo_controls::trigger) test_id: Option<Arc<str>>,
    pub(in crate::imui::combo_controls::trigger) open: bool,
    pub(in crate::imui::combo_controls::trigger) label: Arc<str>,
    pub(in crate::imui::combo_controls::trigger) preview: Arc<str>,
}

pub(in crate::imui::combo_controls::trigger) fn combo_trigger_props(
    input: ComboTriggerPropsInput,
) -> PressableProps {
    let mut props = PressableProps::default();
    props.enabled = input.enabled;
    props.focusable = input.enabled && input.focusable;
    props.layout.size.width = Length::Fill;
    props.layout.size.min_height = Some(Length::Px(control_chrome::FIELD_MIN_HEIGHT));
    props.a11y = PressableA11y {
        role: Some(SemanticsRole::ComboBox),
        label: input
            .a11y_label
            .or_else(|| Some(combo_trigger_a11y_label(&input.label, &input.preview))),
        test_id: input.test_id,
        expanded: Some(input.open),
        ..Default::default()
    };
    props
}

pub(in crate::imui::combo_controls) fn combo_trigger_a11y_label(
    label: &str,
    preview: &str,
) -> Arc<str> {
    if label.is_empty() {
        Arc::from(preview)
    } else {
        Arc::from(format!("{label}: {preview}"))
    }
}
