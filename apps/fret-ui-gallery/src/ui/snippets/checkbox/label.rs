pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let checked = cx.local_model(|| false);

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
