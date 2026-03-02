pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, disabled: Model<bool>) -> AnyElement {
    shadcn::Field::new([
        shadcn::Checkbox::new(disabled)
            .control_id("ui-gallery-checkbox-disabled")
            .disabled(true)
            .a11y_label("Disabled checkbox")
            .test_id("ui-gallery-checkbox-disabled")
            .into_element(cx),
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Marketing emails")
                .for_control("ui-gallery-checkbox-disabled")
                .test_id("ui-gallery-checkbox-disabled-label")
                .into_element(cx),
            shadcn::FieldDescription::new("This preference is managed by your organization.")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .disabled(true)
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-disabled-field")
}
// endregion: example
