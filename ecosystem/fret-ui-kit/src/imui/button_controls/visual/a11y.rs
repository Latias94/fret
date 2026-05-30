use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::PressableA11y;

use super::super::{ButtonArrowDirection, ButtonOptions, ButtonVariant};

pub(in crate::imui::button_controls) fn button_a11y(
    label: &Arc<str>,
    options: &ButtonOptions,
    variant: ButtonVariant,
) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::Button),
        label: button_a11y_label(label, options, variant),
        test_id: options.test_id.clone(),
        ..Default::default()
    }
}

fn arrow_a11y_label(direction: ButtonArrowDirection) -> Arc<str> {
    Arc::from(match direction {
        ButtonArrowDirection::Left => "Left arrow button",
        ButtonArrowDirection::Right => "Right arrow button",
        ButtonArrowDirection::Up => "Up arrow button",
        ButtonArrowDirection::Down => "Down arrow button",
    })
}

fn button_a11y_label(
    label: &Arc<str>,
    options: &ButtonOptions,
    variant: ButtonVariant,
) -> Option<Arc<str>> {
    options.a11y_label.clone().or_else(|| match variant {
        ButtonVariant::Arrow(direction) => Some(arrow_a11y_label(direction)),
        ButtonVariant::Invisible { .. } if label.is_empty() => None,
        _ => Some(label.clone()),
    })
}
