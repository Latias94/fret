pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::TextWrap;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let story = "Jokester began sneaking into the castle in the middle of the night and leaving jokes all over the place: under the king's pillow, in his soup, even in the royal toilet.";

    let content = ui::container(move |cx| {
        [ui::text(story)
            .text_sm()
            .wrap(TextWrap::Word)
            .into_element(cx)]
    })
    .p_4()
    .into_element(cx);

    let area = shadcn::ScrollArea::new([content])
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-scroll-area-usage");

    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().border_1().rounded(Radius::Md),
        LayoutRefinement::default()
            .w_px(Px(350.0))
            .h_px(Px(200.0))
            .overflow_hidden(),
    );

    cx.container(props, move |_cx| [area])
}
// endregion: example
