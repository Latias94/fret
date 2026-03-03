pub const SOURCE: &str = include_str!("invalid_state.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    invalid: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let invalid = cx.with_state(Models::default, |st| st.invalid.clone());
    let invalid = invalid.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(Models::default, |st| st.invalid = Some(model.clone()));
        model
    });

    let invalid_checked = cx
        .get_model_copied(&invalid, Invalidation::Layout)
        .unwrap_or(false);

    shadcn::Field::new([
        shadcn::Checkbox::new(invalid)
            .control_id("ui-gallery-checkbox-invalid")
            .a11y_label("Invalid checkbox")
            .aria_invalid(!invalid_checked)
            .test_id("ui-gallery-checkbox-invalid")
            .into_element(cx),
        shadcn::FieldLabel::new("Accept terms and conditions")
            .for_control("ui-gallery-checkbox-invalid")
            .into_element(cx),
    ])
    .invalid(!invalid_checked)
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-invalid-field")
}
// endregion: example
