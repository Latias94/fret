pub const SOURCE: &str = include_str!("radio.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));
    let monthly_id = "ui-gallery-field-radio-monthly";
    let yearly_id = "ui-gallery-field-radio-yearly";
    let lifetime_id = "ui-gallery-field-radio-lifetime";

    shadcn::FieldSet::new([
        shadcn::FieldLegend::new("Subscription Plan")
            .variant(shadcn::FieldLegendVariant::Label)
            .into_element(cx),
        shadcn::FieldDescription::new("Yearly and lifetime plans offer significant savings.")
            .into_element(cx),
        shadcn::RadioGroup::uncontrolled(Some("monthly"))
            .a11y_label("Subscription Plan")
            .item(
                shadcn::RadioGroupItem::new("monthly", "Monthly ($9.99/month)")
                    .control_id(monthly_id)
                    .children([shadcn::FieldLabel::new("Monthly ($9.99/month)")
                        .for_control(monthly_id)
                        .into_element(cx)]),
            )
            .item(
                shadcn::RadioGroupItem::new("yearly", "Yearly ($99.99/year)")
                    .control_id(yearly_id)
                    .children([shadcn::FieldLabel::new("Yearly ($99.99/year)")
                        .for_control(yearly_id)
                        .into_element(cx)]),
            )
            .item(
                shadcn::RadioGroupItem::new("lifetime", "Lifetime ($299.99)")
                    .control_id(lifetime_id)
                    .children([shadcn::FieldLabel::new("Lifetime ($299.99)")
                        .for_control(lifetime_id)
                        .into_element(cx)]),
            )
            .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-radio")
}
// endregion: example
