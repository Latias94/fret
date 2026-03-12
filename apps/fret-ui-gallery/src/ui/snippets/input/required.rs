pub const SOURCE: &str = include_str!("required.rs");

// region: example
use fret_core::Px;
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let label = ui::h_row(|cx| {
        vec![
            shadcn::FieldLabel::new("Required Field").into_element(cx),
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
            .a11y_label("Required field")
            .placeholder("This field is required")
            .into_element(cx),
        shadcn::FieldDescription::new("Mark required fields clearly in labels.").into_element(cx),
    ])
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-required")
}
// endregion: example
