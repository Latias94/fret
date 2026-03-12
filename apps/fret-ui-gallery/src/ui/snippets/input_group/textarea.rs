pub const SOURCE: &str = include_str!("textarea.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .textarea()
        .a11y_label("Textarea example")
        .block_end([
            shadcn::InputGroupText::new("Shift+Enter for newline").into_element(cx),
            shadcn::InputGroupButton::new("Send")
                .size(shadcn::InputGroupButtonSize::Sm)
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ])
        .block_end_border_top(true)
        .textarea_min_height(Px(100.0))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-textarea")
        .into_element(cx)
}
// endregion: example
