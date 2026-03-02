// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<String>) -> AnyElement {
    shadcn::Textarea::new(value)
        .a11y_label("Message")
        .placeholder("Type your message here.")
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-demo")
}
// endregion: example

