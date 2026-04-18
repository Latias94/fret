pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let default_id = "ui-gallery-radio-group-rtl-default";
    let comfortable_id = "ui-gallery-radio-group-rtl-comfortable";
    let compact_id = "ui-gallery-radio-group-rtl-compact";

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::RadioGroup::uncontrolled(Some("comfortable"))
            .a11y_label("خيارات التخطيط")
            .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
            .item(shadcn::RadioGroupItem::new("default", "افتراضي").control_id(default_id))
            .item(shadcn::RadioGroupItem::new("comfortable", "مريح").control_id(comfortable_id))
            .item(shadcn::RadioGroupItem::new("compact", "مضغوط").control_id(compact_id))
            .into_element_parts(cx, |cx, parts| {
                vec![
                    shadcn::Field::new([
                        parts.control(cx, "default"),
                        shadcn::FieldContent::new([
                            shadcn::FieldLabel::new("افتراضي")
                                .for_control(default_id)
                                .into_element(cx),
                            shadcn::FieldDescription::new("تباعد قياسي لمعظم حالات الاستخدام.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .into_element(cx),
                    shadcn::Field::new([
                        parts.control(cx, "comfortable"),
                        shadcn::FieldContent::new([
                            shadcn::FieldLabel::new("مريح")
                                .for_control(comfortable_id)
                                .into_element(cx),
                            shadcn::FieldDescription::new("مساحة أكبر بين العناصر.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .into_element(cx),
                    shadcn::Field::new([
                        parts.control(cx, "compact"),
                        shadcn::FieldContent::new([
                            shadcn::FieldLabel::new("مضغوط")
                                .for_control(compact_id)
                                .into_element(cx),
                            shadcn::FieldDescription::new("تباعد أدنى للتخطيطات الكثيفة.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .into_element(cx),
                ]
            })
    })
    .test_id("ui-gallery-radio-group-rtl")
}
// endregion: example
