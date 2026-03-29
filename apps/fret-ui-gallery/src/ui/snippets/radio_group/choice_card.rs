pub const SOURCE: &str = include_str!("choice_card.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));
    let starter_id = "ui-gallery-radio-group-choice-card-starter";
    let pro_id = "ui-gallery-radio-group-choice-card-pro";

    shadcn::RadioGroup::uncontrolled(Some("starter"))
        .a11y_label("Plans")
        .refine_layout(max_w_sm)
        .item(shadcn::RadioGroupItem::new("starter", "Starter Plan").control_id(starter_id))
        .item(shadcn::RadioGroupItem::new("pro", "Pro Plan").control_id(pro_id))
        .into_element_parts(cx, |cx, parts| {
            vec![
                shadcn::FieldLabel::new("Starter Plan")
                    .for_control(starter_id)
                    .wrap([shadcn::Field::new([
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Starter Plan").into_element(cx),
                            shadcn::FieldDescription::new(
                                "Perfect for small businesses getting started with our platform",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        parts.control(cx, "starter"),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .refine_style(
                        ChromeRefinement::default()
                            .border_1()
                            .rounded(Radius::Lg)
                            .p(Space::N4),
                    )
                    .into_element(cx)])
                    .into_element(cx),
                shadcn::FieldLabel::new("Pro Plan")
                    .for_control(pro_id)
                    .wrap([shadcn::Field::new([
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Pro Plan").into_element(cx),
                            shadcn::FieldDescription::new(
                                "Advanced features for growing businesses with higher demands",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        parts.control(cx, "pro"),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .refine_style(
                        ChromeRefinement::default()
                            .border_1()
                            .rounded(Radius::Lg)
                            .p(Space::N4),
                    )
                    .into_element(cx)])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-radio-group-choice-card")
}
// endregion: example
