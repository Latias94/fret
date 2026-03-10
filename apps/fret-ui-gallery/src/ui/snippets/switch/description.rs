pub const SOURCE: &str = include_str!("description.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
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
    let control_id = ControlId::from("ui-gallery-switch-description");

    shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Share across devices")
                .for_control(control_id.clone())
                .into_element(cx),
            shadcn::FieldDescription::new(
                "Focus is shared across devices, and turns off when you leave the app.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Switch::new(checked)
            .control_id(control_id)
            .a11y_label("Share across devices")
            .test_id("ui-gallery-switch-description-control")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-switch-description")
}
// endregion: example
