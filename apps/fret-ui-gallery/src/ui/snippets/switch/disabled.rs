pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let control_id = ControlId::from("ui-gallery-switch-disabled");

    shadcn::Field::new([
        shadcn::Switch::from_checked(false)
            .control_id(control_id.clone())
            .disabled(true)
            .a11y_label("Disabled switch")
            .test_id("ui-gallery-switch-disabled-control")
            .into_element(cx),
        shadcn::FieldLabel::new("Disabled")
            .for_control(control_id)
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .disabled(true)
    .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
    .into_element(cx)
    .test_id("ui-gallery-switch-disabled")
}
// endregion: example
