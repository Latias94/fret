pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let monthly_id = "ui-gallery-radio-group-fieldset-monthly";
    let yearly_id = "ui-gallery-radio-group-fieldset-yearly";
    let lifetime_id = "ui-gallery-radio-group-fieldset-lifetime";

    let group = shadcn::RadioGroup::uncontrolled(Some("monthly"))
        .a11y_label("Subscription plan")
        .item(
            shadcn::RadioGroupItem::new("monthly", "Monthly ($9.99/month)").control_id(monthly_id),
        )
        .item(shadcn::RadioGroupItem::new("yearly", "Yearly ($99.99/year)").control_id(yearly_id))
        .item(shadcn::RadioGroupItem::new("lifetime", "Lifetime ($299.99)").control_id(lifetime_id))
        .into_element_parts(cx, |cx, parts| {
            vec![
                shadcn::Field::new([
                    parts.control(cx, "monthly"),
                    shadcn::FieldLabel::new("Monthly ($9.99/month)")
                        .for_control(monthly_id)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "yearly"),
                    shadcn::FieldLabel::new("Yearly ($99.99/year)")
                        .for_control(yearly_id)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    parts.control(cx, "lifetime"),
                    shadcn::FieldLabel::new("Lifetime ($299.99)")
                        .for_control(lifetime_id)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ]
        });

    shadcn::field_set(|cx| {
        ui::children![
            cx;
            shadcn::FieldLegend::new("Subscription Plan")
                .variant(shadcn::FieldLegendVariant::Label),
            shadcn::FieldDescription::new("Yearly and lifetime plans offer significant savings."),
            group,
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-radio-group-fieldset")
}
// endregion: example
