pub const SOURCE: &str = include_str!("compact_helper.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::TextWrap;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let content = ui::container(move |cx| {
        [ui::text(
            "Use `scroll_area(cx, |cx| [...])` when you want the compact Fret-first shorthand and do not need explicit scrollbar parts.",
        )
        .text_sm()
        .wrap(TextWrap::Word)
        .into_element(cx)]
    })
    .p_4()
    .into_element(cx);

    shadcn::scroll_area(cx, |_cx| [content])
        .viewport_test_id("ui-gallery-scroll-area-compact-helper-viewport")
        .refine_layout(LayoutRefinement::default().w_px(Px(350.0)).h_px(Px(140.0)))
        .into_element(cx)
        .test_id("ui-gallery-scroll-area-compact-helper")
}
// endregion: example
