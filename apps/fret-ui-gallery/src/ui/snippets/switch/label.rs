pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let checked = cx.local_model(|| false);

    let control_id = ControlId::from("ui-gallery-switch-label");
    let switch = shadcn::Switch::new(checked)
        .control_id(control_id.clone())
        .test_id("ui-gallery-switch-label-control")
        .a11y_label("Switch label association")
        .into_element(cx);

    shadcn::FieldGroup::new([shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Enable notifications")
                .for_control(control_id.clone())
                .test_id("ui-gallery-switch-label-label")
                .into_element(cx),
            shadcn::FieldDescription::new("Click the label to toggle the switch.")
                .for_control(control_id.clone())
                .into_element(cx),
        ])
        .into_element(cx),
        switch,
    ])
    .into_element(cx)])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
    .into_element(cx)
    .test_id("ui-gallery-switch-label")
}
// endregion: example
