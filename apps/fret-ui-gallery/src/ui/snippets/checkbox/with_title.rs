// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, with_title: Model<bool>) -> AnyElement {
    shadcn::FieldGroup::new([
        shadcn::FieldLabel::new("Enable notifications")
            .for_control("ui-gallery-checkbox-with-title")
            .test_id("ui-gallery-checkbox-with-title-label")
            .wrap([shadcn::Field::new([
                shadcn::Checkbox::new(with_title.clone())
                    .control_id("ui-gallery-checkbox-with-title")
                    .a11y_label("Enable notifications")
                    .test_id("ui-gallery-checkbox-with-title")
                    .into_element(cx),
                shadcn::FieldContent::new([
                    shadcn::FieldTitle::new("Enable notifications").into_element(cx),
                    shadcn::FieldDescription::new(
                        "You can enable or disable notifications at any time.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx)])
            .into_element(cx),
        shadcn::FieldLabel::new("Enable notifications (disabled)")
            .for_control("ui-gallery-checkbox-with-title-disabled")
            .test_id("ui-gallery-checkbox-with-title-disabled-label")
            .wrap([shadcn::Field::new([
                shadcn::Checkbox::new(with_title)
                    .control_id("ui-gallery-checkbox-with-title-disabled")
                    .disabled(true)
                    .a11y_label("Enable notifications (disabled)")
                    .test_id("ui-gallery-checkbox-with-title-disabled")
                    .into_element(cx),
                shadcn::FieldContent::new([
                    shadcn::FieldTitle::new("Enable notifications").into_element(cx),
                    shadcn::FieldDescription::new(
                        "You can enable or disable notifications at any time.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .disabled(true)
            .into_element(cx)])
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-checkbox-with-title-section")
}
// endregion: example
