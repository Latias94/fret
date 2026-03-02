pub const SOURCE: &str = include_str!("plans.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));

    shadcn::RadioGroup::uncontrolled(Some("starter"))
        .a11y_label("Plans")
        .refine_layout(max_w_sm)
        .item(
            shadcn::RadioGroupItem::new("starter", "Starter Plan")
                .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                .child(
                    shadcn::FieldContent::new([
                        shadcn::FieldTitle::new("Starter Plan").into_element(cx),
                        shadcn::FieldDescription::new(
                            "Perfect for small businesses getting started with our platform",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ),
        )
        .item(
            shadcn::RadioGroupItem::new("pro", "Pro Plan")
                .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                .child(
                    shadcn::FieldContent::new([
                        shadcn::FieldTitle::new("Pro Plan").into_element(cx),
                        shadcn::FieldDescription::new(
                            "Advanced features for growing businesses with higher demands",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ),
        )
        .into_element(cx)
        .test_id("ui-gallery-radio-group-plans")
}
// endregion: example
