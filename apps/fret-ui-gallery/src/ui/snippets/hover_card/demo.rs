// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

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
    let trigger = shadcn::Button::new("Hover Here")
        .variant(shadcn::ButtonVariant::Link)
        .test_id("ui-gallery-hover-card-demo-trigger")
        .into_element(cx);
    let content = demo_content(
        cx,
        "@nextjs",
        "The React Framework – created and maintained by @vercel.",
        "Joined December 2021",
        "ui-gallery-hover-card-demo-content",
    );

    shadcn::HoverCard::new(trigger, content)
        .open_delay_frames(10)
        .close_delay_frames(100)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo")
}
// endregion: example
