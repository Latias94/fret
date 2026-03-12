pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);
    let control_id = ControlId::from("ui-gallery-input-label");

    shadcn::Field::new([
        shadcn::FieldLabel::new("Username")
            .for_control(control_id.clone())
            .test_id("ui-gallery-input-label-label")
            .into_element(cx),
        shadcn::Input::new(value)
            .a11y_label("Username")
            .placeholder("Enter your username")
            .control_id(control_id.clone())
            .test_id("ui-gallery-input-label-control")
            .into_element(cx),
        shadcn::FieldDescription::new("Click the label to focus the input control.")
            .for_control(control_id)
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-input-label")
}
// endregion: example
