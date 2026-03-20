pub const SOURCE: &str = include_str!("positioning.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn side_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    muted_fg: fret_core::Color,
    side_label: &'static str,
    test_id: &'static str,
    side: shadcn::HoverCardSide,
    align: shadcn::HoverCardAlign,
) -> shadcn::HoverCardContent {
    let body = ui::v_flex(move |cx| {
        vec![
            ui::text(side_label)
                .text_sm()
                .font_medium()
                .into_element(cx),
            ui::text_block("Positioning is controlled by `side` and `align`.")
                .text_xs()
                .text_color(ColorRef::Color(muted_fg))
                .mt(Space::N1)
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N1)
    .items_stretch()
    .into_element(cx);

    shadcn::HoverCardContent::build(cx, |_cx| [body])
        .test_id(test_id)
        .side(side)
        .align(align)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");
    let top_start_content = side_content(
        cx,
        muted_fg,
        "top (align=start)",
        "ui-gallery-hover-card-pos-top-start-content",
        shadcn::HoverCardSide::Top,
        shadcn::HoverCardAlign::Start,
    );
    let right_end_content = side_content(
        cx,
        muted_fg,
        "right (align=end)",
        "ui-gallery-hover-card-pos-right-end-content",
        shadcn::HoverCardSide::Right,
        shadcn::HoverCardAlign::End,
    );

    let top_start = shadcn::HoverCard::new(
        cx,
        shadcn::Button::new("Top / Start")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-pos-top-start-trigger"),
        top_start_content,
    )
    .open_delay_frames(8)
    .close_delay_frames(8)
    .into_element(cx)
    .test_id("ui-gallery-hover-card-pos-top-start");

    let right_end = shadcn::HoverCard::new(
        cx,
        shadcn::Button::new("Right / End")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-pos-right-end-trigger"),
        right_end_content,
    )
    .open_delay_frames(8)
    .close_delay_frames(8)
    .into_element(cx)
    .test_id("ui-gallery-hover-card-pos-right-end");

    ui::h_row(|_cx| vec![top_start, right_end])
        .gap(Space::N3)
        .items_center()
        .into_element(cx)
}
// endregion: example
