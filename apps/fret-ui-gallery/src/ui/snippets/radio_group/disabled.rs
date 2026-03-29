pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let disabled_id = "ui-gallery-radio-group-disabled-option-one";
    let option_two_id = "ui-gallery-radio-group-disabled-option-two";
    let option_three_id = "ui-gallery-radio-group-disabled-option-three";

    shadcn::RadioGroup::uncontrolled(Some("option2"))
        .a11y_label("Options")
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .item(
            shadcn::RadioGroupItem::new("option1", "Disabled")
                .disabled(true)
                .control_id(disabled_id),
        )
        .item(shadcn::RadioGroupItem::new("option2", "Option 2").control_id(option_two_id))
        .item(shadcn::RadioGroupItem::new("option3", "Option 3").control_id(option_three_id))
        .into_element_parts(cx, |cx, parts| {
            vec![
                shadcn::Field::new([
                    parts.control(cx, "option1"),
                    shadcn::FieldLabel::new("Disabled")
                        .for_control(disabled_id)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .disabled(true)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "option2"),
                    shadcn::FieldLabel::new("Option 2")
                        .for_control(option_two_id)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "option3"),
                    shadcn::FieldLabel::new("Option 3")
                        .for_control(option_three_id)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ]
        })
        .test_id("ui-gallery-radio-group-disabled")
}
// endregion: example
