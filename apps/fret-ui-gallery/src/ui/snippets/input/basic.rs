pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::Input::new(value)
        .a11y_label("Enter text")
        .placeholder("Enter text")
        .refine_layout(max_w_xs)
        .into_element(cx)
        .test_id("ui-gallery-input-basic")
}
// endregion: example
