pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let option_one_id = "ui-gallery-radio-group-usage-option-one";
    let option_two_id = "ui-gallery-radio-group-usage-option-two";

    shadcn::RadioGroup::uncontrolled(Some("option-one"))
        .a11y_label("Choose an option")
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .item(shadcn::RadioGroupItem::new("option-one", "Option One").control_id(option_one_id))
        .item(shadcn::RadioGroupItem::new("option-two", "Option Two").control_id(option_two_id))
        .into_element_parts(cx, |cx, parts| {
            vec![
                shadcn::Field::new([
                    parts.control(cx, "option-one"),
                    shadcn::Label::new("Option One")
                        .for_control(option_one_id)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "option-two"),
                    shadcn::Label::new("Option Two")
                        .for_control(option_two_id)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ]
        })
        .test_id("ui-gallery-radio-group-usage")
}
// endregion: example
