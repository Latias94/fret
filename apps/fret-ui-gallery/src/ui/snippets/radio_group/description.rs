pub const SOURCE: &str = include_str!("description.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let default_id = "ui-gallery-radio-group-description-default";
    let comfortable_id = "ui-gallery-radio-group-description-comfortable";
    let compact_id = "ui-gallery-radio-group-description-compact";

    shadcn::RadioGroup::uncontrolled(Some("comfortable"))
        .a11y_label("Options")
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .item(shadcn::RadioGroupItem::new("default", "Default").control_id(default_id))
        .item(shadcn::RadioGroupItem::new("comfortable", "Comfortable").control_id(comfortable_id))
        .item(shadcn::RadioGroupItem::new("compact", "Compact").control_id(compact_id))
        .into_element_parts(cx, |cx, parts| {
            vec![
                shadcn::Field::new([
                    parts.control(cx, "default"),
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Default")
                            .for_control(default_id)
                            .into_element(cx),
                        shadcn::FieldDescription::new("Standard spacing for most use cases.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "comfortable"),
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Comfortable")
                            .for_control(comfortable_id)
                            .into_element(cx),
                        shadcn::FieldDescription::new("More space between elements.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "compact"),
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Compact")
                            .for_control(compact_id)
                            .into_element(cx),
                        shadcn::FieldDescription::new("Minimal spacing for dense layouts.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ]
        })
        .test_id("ui-gallery-radio-group-description")
}
// endregion: example
