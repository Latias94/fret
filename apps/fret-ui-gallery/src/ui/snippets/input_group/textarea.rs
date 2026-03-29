pub const SOURCE: &str = include_str!("textarea.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .textarea()
        .a11y_label("Code editor")
        .placeholder("console.log('Hello, world!');")
        .block_start([
            shadcn::InputGroupText::new("script.js").into_element(cx),
            shadcn::InputGroupButton::new("")
                .a11y_label("Refresh")
                .size(shadcn::InputGroupButtonSize::IconXs)
                .icon(IconId::new_static("lucide.refresh-cw"))
                .refine_layout(LayoutRefinement::default().ml_auto())
                .into_element(cx),
            shadcn::InputGroupButton::new("")
                .a11y_label("Copy")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::InputGroupButtonSize::IconXs)
                .icon(IconId::new_static("lucide.copy"))
                .into_element(cx),
        ])
        .block_start_border_bottom(true)
        .block_end([
            shadcn::InputGroupText::new("Line 1, Column 1").into_element(cx),
            shadcn::InputGroupButton::new("Run")
                .size(shadcn::InputGroupButtonSize::Sm)
                .variant(shadcn::ButtonVariant::Default)
                .trailing_icon(IconId::new_static("lucide.corner-down-left"))
                .refine_layout(LayoutRefinement::default().ml_auto())
                .into_element(cx),
        ])
        .block_end_border_top(true)
        .textarea_min_height(Px(200.0))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
        .test_id("ui-gallery-input-group-textarea")
        .into_element(cx)
}
// endregion: example
