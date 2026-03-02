// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    max_w_md: LayoutRefinement,
) -> AnyElement {
    shadcn::Input::new(text_input)
        .a11y_label("Email")
        .placeholder("name@example.com")
        .refine_layout(max_w_md)
        .into_element(cx)
        .test_id("ui-gallery-form-input")
}
// endregion: example
