use std::sync::Arc;

use fret_ui::element::{Length, PressableProps};

use super::super::{ButtonArrowDirection, ButtonVariant};
use crate::imui::control_chrome;

pub(in crate::imui::button_controls) fn apply_button_variant_layout(
    props: &mut PressableProps,
    variant: ButtonVariant,
) {
    match variant {
        ButtonVariant::Default => {
            props.layout.size.min_height = Some(Length::Px(control_chrome::BUTTON_MIN_HEIGHT));
        }
        ButtonVariant::Small => {
            props.layout.size.min_height =
                Some(Length::Px(control_chrome::SMALL_BUTTON_MIN_HEIGHT));
        }
        ButtonVariant::Arrow(_) => {
            props.layout.size.width = Length::Px(control_chrome::ARROW_BUTTON_SIZE);
            props.layout.size.height = Length::Px(control_chrome::ARROW_BUTTON_SIZE);
        }
        ButtonVariant::Invisible { size } => {
            props.layout.size.width = Length::Px(size.width);
            props.layout.size.height = Length::Px(size.height);
        }
    }
}

pub(super) fn arrow_symbol(direction: ButtonArrowDirection) -> Arc<str> {
    Arc::from(match direction {
        ButtonArrowDirection::Left => "<",
        ButtonArrowDirection::Right => ">",
        ButtonArrowDirection::Up => "^",
        ButtonArrowDirection::Down => "v",
    })
}
