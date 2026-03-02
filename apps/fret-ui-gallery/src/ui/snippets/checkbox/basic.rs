pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<bool>) -> AnyElement {
    shadcn::Field::new([
        shadcn::Checkbox::new(model)
            .control_id("ui-gallery-checkbox-basic")
            .a11y_label("Basic checkbox")
            .test_id("ui-gallery-checkbox-basic")
            .into_element(cx),
        shadcn::FieldLabel::new("Accept terms and conditions")
            .for_control("ui-gallery-checkbox-basic")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-basic-field")
}
// endregion: example
