pub const SOURCE: &str = include_str!("required.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let required_id = "ui-gallery-textarea-required-control";

    let label = ui::h_row(|cx| {
        vec![
            shadcn::FieldLabel::new("Required Message")
                .for_control(required_id)
                .into_element(cx),
            shadcn::typography::muted("*")
                .into_element(cx)
                .attach_semantics(SemanticsDecoration::default().label("required-star")),
        ]
    })
    .gap(Space::N1)
    .items_center()
    .into_element(cx);

    shadcn::Field::new([
        label,
        shadcn::Textarea::new(value)
            .control_id(required_id)
            .required(true)
            .placeholder("This field is required")
            .test_id(required_id)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        shadcn::FieldDescription::new("This field must be filled out.")
            .for_control(required_id)
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-textarea-required")
}
// endregion: example
