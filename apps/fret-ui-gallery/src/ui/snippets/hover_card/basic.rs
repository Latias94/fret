pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let trigger = shadcn::Button::new("Hover")
        .variant(shadcn::ButtonVariant::Outline)
        .test_id("ui-gallery-hover-card-basic-trigger")
        .into_element(cx);

    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    let body = ui::v_flex(|cx| {
        vec![
            ui::text("HoverCard content: multiline description with WordBreak wrapping.")
                .text_sm()
                .wrap(TextWrap::WordBreak)
                .into_element(cx)
                .test_id("ui-gallery-hover-card-basic-content-desc"),
            ui::text("Joined December 2021")
                .text_xs()
                .text_color(ColorRef::Color(muted_fg))
                .into_element(cx)
                .test_id("ui-gallery-hover-card-basic-content-joined"),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N1)
    .items_start()
    .into_element(cx);

    let content = shadcn::HoverCardContent::new(vec![body])
        .into_element(cx)
        .test_id("ui-gallery-hover-card-basic-content");

    shadcn::HoverCard::new(trigger, content)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-basic")
}
// endregion: example
