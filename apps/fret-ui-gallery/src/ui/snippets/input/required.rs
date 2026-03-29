pub const SOURCE: &str = include_str!("required.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let required_id = "ui-gallery-input-required-field";

    let label = ui::h_row(|cx| {
        vec![
            shadcn::FieldLabel::new("Required Field")
                .for_control(required_id)
                .into_element(cx),
            shadcn::raw::typography::muted("*")
                .into_element(cx)
                .attach_semantics(SemanticsDecoration::default().label("required-star")),
        ]
    })
    .gap(Space::N1)
    .items_center()
    .into_element(cx);

    shadcn::Field::new([
        label,
        shadcn::Input::new(value)
            .control_id(required_id)
            .required(true)
            .placeholder("This field is required")
            .into_element(cx),
        shadcn::FieldDescription::new("This field must be filled out.")
            .for_control(required_id)
            .into_element(cx),
    ])
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-required")
}
// endregion: example
