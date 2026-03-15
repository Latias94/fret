pub const SOURCE: &str = include_str!("inline.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::Field::new([
        shadcn::Input::new(value)
            .a11y_label("Search")
            .placeholder("Search...")
            .into_element(cx),
        shadcn::Button::new("Search").into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-inline")
}
// endregion: example
