pub const SOURCE: &str = include_str!("trigger_delays.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
use std::time::Duration;

fn demo_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
    desc: &'static str,
    joined: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    let title_test_id: Arc<str> = Arc::from(format!("{test_id}-title"));
    let desc_test_id: Arc<str> = Arc::from(format!("{test_id}-desc"));
    let joined_test_id: Arc<str> = Arc::from(format!("{test_id}-joined"));

    let body = ui::v_flex(move |cx| {
        vec![
            ui::text(title)
                .font_semibold()
                .into_element(cx)
                .test_id(title_test_id.clone()),
            ui::text_block(desc)
                .wrap(TextWrap::WordBreak)
                .into_element(cx)
                .test_id(desc_test_id.clone()),
            ui::text(joined)
                .text_xs()
                .text_color(ColorRef::Color(muted_fg))
                .mt(Space::N1)
                .into_element(cx)
                .test_id(joined_test_id.clone()),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N0p5)
    .items_stretch()
    .into_element(cx);

    shadcn::HoverCardContent::new(vec![body]).test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let instant_content = demo_content(
        cx,
        "Instant",
        "openDelay=0",
        "closeDelay=0",
        "ui-gallery-hover-card-delay-instant-content",
    )
    .into_element(cx);
    let delayed_content = demo_content(
        cx,
        "Delayed",
        "openDelay=700ms",
        "closeDelay=300ms",
        "ui-gallery-hover-card-delay-delayed-content",
    )
    .into_element(cx);

    let instant = shadcn::HoverCard::new(
        cx,
        shadcn::Button::new("Instant")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-delay-instant-trigger"),
        instant_content,
    )
    .open_delay_frames(0)
    .close_delay_frames(0)
    .into_element(cx)
    .test_id("ui-gallery-hover-card-delay-instant");

    let delayed = shadcn::HoverCard::new(
        cx,
        shadcn::Button::new("Delayed")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-delay-delayed-trigger"),
        delayed_content,
    )
    .open_delay(Duration::from_millis(700))
    .close_delay(Duration::from_millis(300))
    .into_element(cx)
    .test_id("ui-gallery-hover-card-delay-delayed");

    ui::h_row(|_cx| vec![instant, delayed])
        .gap(Space::N3)
        .items_center()
        .into_element(cx)
}
// endregion: example
