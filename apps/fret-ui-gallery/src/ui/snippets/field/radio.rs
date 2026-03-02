pub const SOURCE: &str = include_str!("radio.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::FieldSet::new([
        shadcn::FieldLabel::new("Subscription Plan").into_element(cx),
        shadcn::FieldDescription::new("Yearly and lifetime plans offer significant savings.")
            .into_element(cx),
        shadcn::RadioGroup::uncontrolled(Some("monthly"))
            .a11y_label("Subscription Plan")
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
            .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-radio")
}
// endregion: example
