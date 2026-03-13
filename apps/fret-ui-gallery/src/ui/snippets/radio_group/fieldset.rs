pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let group = shadcn::RadioGroup::uncontrolled(Some("monthly"))
        .a11y_label("Subscription plan")
        .item(shadcn::RadioGroupItem::new(
            "monthly",
            "Monthly ($9.99/month)",
        ))
        .item(shadcn::RadioGroupItem::new(
            "yearly",
            "Yearly ($99.99/year)",
        ))
        .item(shadcn::RadioGroupItem::new(
            "lifetime",
            "Lifetime ($299.99)",
        ))
        .into_element(cx);

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
