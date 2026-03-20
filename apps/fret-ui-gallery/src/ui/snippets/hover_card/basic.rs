pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    let body = ui::v_flex(|cx| {
        vec![
            ui::text_block("HoverCard content: multiline description with WordBreak wrapping.")
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
    .items_stretch()
    .into_element(cx);

    let content = shadcn::HoverCardContent::build(cx, |_cx| [body])
        .test_id("ui-gallery-hover-card-basic-content");

    shadcn::HoverCard::new(
        cx,
        shadcn::Button::new("Hover")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-basic-trigger"),
        content,
    )
    .into_element(cx)
    .test_id("ui-gallery-hover-card-basic")
}
// endregion: example
