pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    checked: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let checked = cx.with_state(Models::default, |st| st.checked.clone());
    let checked = checked.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(Models::default, |st| st.checked = Some(model.clone()));
        model
    });

    shadcn::Field::new([
        shadcn::Checkbox::new(checked)
            .control_id("ui-gallery-checkbox-label-control")
            .a11y_label("Checkbox label association")
            .test_id("ui-gallery-checkbox-label-control")
            .into_element(cx),
        shadcn::FieldLabel::new("Accept terms and conditions")
            .for_control("ui-gallery-checkbox-label-control")
            .test_id("ui-gallery-checkbox-label-label")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-label")
}
// endregion: example
