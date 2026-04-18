pub const SOURCE: &str = include_str!("input.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let text_input = cx.local_model(String::new);

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    shadcn::Input::new(text_input)
        .a11y_label("Email")
        .placeholder("name@example.com")
        .refine_layout(max_w_md)
        .into_element(cx)
        .test_id("ui-gallery-form-input")
}
// endregion: example
