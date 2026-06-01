use fret_core::{SemanticsRole, Size};
use fret_ui::element::{Length, PressableA11y, PressableKeyActivation, PressableProps};

use super::super::{ImageItemOptions, ImageItemVariant};

pub(in crate::imui::image_item_controls) fn image_item_pressable_props(
    size: Size,
    options: &ImageItemOptions,
    enabled: bool,
    focusable: bool,
    variant: ImageItemVariant,
) -> PressableProps {
    let item_size = super::visual::sanitize_item_size(size);

    let mut props = PressableProps::default();
    props.enabled = enabled;
    props.focusable = focusable;
    props.layout.size.width = Length::Px(item_size.width);
    props.layout.size.height = Length::Px(item_size.height);
    if matches!(variant, ImageItemVariant::Image) {
        props.key_activation = PressableKeyActivation::None;
    }
    props.a11y = PressableA11y {
        role: Some(match variant {
            ImageItemVariant::Image => SemanticsRole::Image,
            ImageItemVariant::Button => SemanticsRole::Button,
        }),
        label: options.a11y_label.clone(),
        test_id: options.test_id.clone(),
        ..Default::default()
    };
    props
}
