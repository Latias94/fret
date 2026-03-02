pub const SOURCE: &str = include_str!("positioning.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn side_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    muted_fg: fret_core::Color,
    side_label: &'static str,
    test_id: &'static str,
) -> AnyElement {
    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N1)
            .items_start(),
        move |cx| {
            vec![
                ui::text(cx, "Hover Card").font_medium().into_element(cx),
                ui::text(
                    cx,
                    format!("This hover card appears on the {side_label} side of the trigger."),
                )
                .wrap(TextWrap::WordBreak)
                .into_element(cx),
                ui::text(cx, "Positioning is controlled by `side` and `align`.")
                    .text_xs()
                    .text_color(ColorRef::Color(muted_fg))
                    .mt(Space::N1)
                    .into_element(cx),
            ]
        },
    );

    shadcn::HoverCardContent::new(vec![body])
        .into_element(cx)
        .test_id(test_id)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    let top_start = shadcn::HoverCard::new(
        shadcn::Button::new("Top / Start")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-pos-top-start-trigger")
            .into_element(cx),
        side_content(
            cx,
            muted_fg,
            "top (align=start)",
            "ui-gallery-hover-card-pos-top-start-content",
        ),
    )
    .side(shadcn::HoverCardSide::Top)
    .align(shadcn::HoverCardAlign::Start)
    .open_delay_frames(8)
    .close_delay_frames(8)
    .into_element(cx)
    .test_id("ui-gallery-hover-card-pos-top-start");

    let right_end = shadcn::HoverCard::new(
        shadcn::Button::new("Right / End")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-pos-right-end-trigger")
            .into_element(cx),
        side_content(
            cx,
            muted_fg,
            "right (align=end)",
            "ui-gallery-hover-card-pos-right-end-content",
        ),
    )
    .side(shadcn::HoverCardSide::Right)
    .align(shadcn::HoverCardAlign::End)
    .open_delay_frames(8)
    .close_delay_frames(8)
    .into_element(cx)
    .test_id("ui-gallery-hover-card-pos-right-end");

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N3).items_center(),
        |_cx| vec![top_start, right_end],
    )
}
// endregion: example
