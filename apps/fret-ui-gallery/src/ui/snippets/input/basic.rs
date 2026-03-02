// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<String>) -> AnyElement {
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::Input::new(value)
        .a11y_label("Enter text")
        .placeholder("Enter text")
        .refine_layout(max_w_xs)
        .into_element(cx)
        .test_id("ui-gallery-input-basic")
}
// endregion: example
