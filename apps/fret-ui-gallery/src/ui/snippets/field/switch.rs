pub const SOURCE: &str = include_str!("switch.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let enabled = cx.local_model(|| false);
    let control_id = "ui-gallery-field-switch-mfa";

    shadcn::Field::new([
        shadcn::FieldLabel::new("Multi-factor authentication")
            .for_control(control_id)
            .into_element(cx),
        shadcn::Switch::new(enabled)
            .control_id(control_id)
            .a11y_label("Multi-factor authentication")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_auto())
    .into_element(cx)
    .test_id("ui-gallery-field-switch")
}
// endregion: example
