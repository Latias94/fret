pub const SOURCE: &str = include_str!("trigger_delays.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;
use std::time::Duration;

fn demo_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
    desc: &'static str,
    joined: &'static str,
    test_id: &'static str,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    let title_test_id: Arc<str> = Arc::from(format!("{test_id}-title"));
    let desc_test_id: Arc<str> = Arc::from(format!("{test_id}-desc"));
    let joined_test_id: Arc<str> = Arc::from(format!("{test_id}-joined"));

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N0p5)
            .items_start(),
        move |cx| {
            vec![
                ui::text(cx, title)
                    .font_semibold()
                    .into_element(cx)
                    .test_id(title_test_id.clone()),
                ui::text(cx, desc)
                    .wrap(TextWrap::WordBreak)
                    .into_element(cx)
                    .test_id(desc_test_id.clone()),
                ui::text(cx, joined)
                    .text_xs()
                    .text_color(ColorRef::Color(muted_fg))
                    .mt(Space::N1)
                    .into_element(cx)
                    .test_id(joined_test_id.clone()),
            ]
        },
    );

    shadcn::HoverCardContent::new(vec![body])
        .into_element(cx)
        .test_id(test_id)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let instant = shadcn::HoverCard::new(
        shadcn::Button::new("Instant")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-delay-instant-trigger")
            .into_element(cx),
        demo_content(
            cx,
            "Instant",
            "openDelay=0",
            "closeDelay=0",
            "ui-gallery-hover-card-delay-instant-content",
        ),
    )
    .open_delay_frames(0)
    .close_delay_frames(0)
    .into_element(cx)
    .test_id("ui-gallery-hover-card-delay-instant");

    let delayed = shadcn::HoverCard::new(
        shadcn::Button::new("Delayed")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-delay-delayed-trigger")
            .into_element(cx),
        demo_content(
            cx,
            "Delayed",
            "openDelay=700ms",
            "closeDelay=300ms",
            "ui-gallery-hover-card-delay-delayed-content",
        ),
    )
    .open_delay(Duration::from_millis(700))
    .close_delay(Duration::from_millis(300))
    .into_element(cx)
    .test_id("ui-gallery-hover-card-delay-delayed");

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N3).items_center(),
        |_cx| vec![instant, delayed],
    )
}
// endregion: example
