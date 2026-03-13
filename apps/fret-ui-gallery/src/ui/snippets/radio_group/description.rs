pub const SOURCE: &str = include_str!("description.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let w_fit = LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto);

    shadcn::RadioGroup::uncontrolled(Some("comfortable"))
        .a11y_label("Options")
        .refine_layout(w_fit)
        .item(
            shadcn::RadioGroupItem::new("default", "Default").child(
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Default").into_element(cx),
                    shadcn::FieldDescription::new("Standard spacing for most use cases.")
                        .into_element(cx),
                ])
                .into_element(cx),
            ),
        )
        .item(
            shadcn::RadioGroupItem::new("comfortable", "Comfortable").child(
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Comfortable").into_element(cx),
                    shadcn::FieldDescription::new("More space between elements.").into_element(cx),
                ])
                .into_element(cx),
            ),
        )
        .item(
            shadcn::RadioGroupItem::new("compact", "Compact").child(
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Compact").into_element(cx),
                    shadcn::FieldDescription::new("Minimal spacing for dense layouts.")
                        .into_element(cx),
                ])
                .into_element(cx),
            ),
        )
        .into_element(cx)
        .test_id("ui-gallery-radio-group-description")
}
// endregion: example
