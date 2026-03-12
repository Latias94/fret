pub const SOURCE: &str = include_str!("custom_input.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .textarea()
        .a11y_label("Custom input example")
        .block_start([shadcn::InputGroupText::new("Custom control").into_element(cx)])
        .block_start_border_bottom(true)
        .block_end([shadcn::InputGroupButton::new("Resize")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::InputGroupButtonSize::Sm)
            .into_element(cx)])
        .block_end_border_top(true)
        .textarea_min_height(Px(88.0))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-custom-input")
        .into_element(cx)
}
// endregion: example
