pub const SOURCE: &str = include_str!("align_block_start.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .a11y_label("Block start addon")
        .block_start([shadcn::InputGroupText::new("Write a concise title").into_element(cx)])
        .block_start_border_bottom(true)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-align-block-start")
        .into_element(cx)
}
// endregion: example
