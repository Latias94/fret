pub const SOURCE: &str = include_str!("description.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let description = cx.local_model(|| false);

    shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Enable notifications")
                .for_control("ui-gallery-checkbox-description")
                .into_element(cx),
            shadcn::FieldDescription::new(
                "Receive updates about release notes, fixes, and maintenance windows.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Checkbox::new(description)
            .control_id("ui-gallery-checkbox-description")
            .a11y_label("Enable notifications")
            .test_id("ui-gallery-checkbox-description")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-description-field")
}
// endregion: example
