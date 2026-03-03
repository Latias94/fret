pub const SOURCE: &str = include_str!("description.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    description: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let description = cx.with_state(Models::default, |st| st.description.clone());
    let description = description.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(Models::default, |st| st.description = Some(model.clone()));
        model
    });

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
