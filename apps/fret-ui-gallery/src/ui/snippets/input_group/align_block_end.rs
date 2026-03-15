pub const SOURCE: &str = include_str!("align_block_end.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .textarea()
        .a11y_label("Block end addon")
        .block_end([
            shadcn::InputGroupText::new("0/200").into_element(cx),
            shadcn::InputGroupButton::new("Publish")
                .size(shadcn::InputGroupButtonSize::Sm)
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ])
        .block_end_border_top(true)
        .textarea_min_height(Px(84.0))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-align-block-end")
        .into_element(cx)
}
// endregion: example
