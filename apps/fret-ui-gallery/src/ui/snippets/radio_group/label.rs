pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let control_id = "ui-gallery-radio-group-label-control";

    shadcn::Field::new(vec![
        shadcn::FieldLabel::new("Density")
            .for_control(control_id)
            .test_id("ui-gallery-radio-group-label-label")
            .into_element(cx),
        shadcn::RadioGroup::uncontrolled(Some("comfortable"))
            .control_id(control_id)
            .a11y_label("Density")
            .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
            .item(shadcn::RadioGroupItem::new("default", "Default"))
            .item(shadcn::RadioGroupItem::new("comfortable", "Comfortable"))
            .item(shadcn::RadioGroupItem::new("compact", "Compact"))
            .into_element(cx)
            .test_id("ui-gallery-radio-group-label-control"),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-radio-group-label")
}
// endregion: example
