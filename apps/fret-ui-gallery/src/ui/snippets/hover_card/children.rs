pub const SOURCE: &str = include_str!("children.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    let title = ui::text("Release Notes")
        .text_sm()
        .font_semibold()
        .into_element(cx)
        .test_id("ui-gallery-hover-card-children-demo-title");
    let summary =
        ui::text_block("Already-built panel nodes can be passed directly into HoverCardContent::new([...]) when the content body is caller-owned.")
            .text_sm()
            .wrap(TextWrap::WordBreak)
            .into_element(cx)
            .test_id("ui-gallery-hover-card-children-demo-summary");
    let meta = ui::h_flex(|cx| {
        vec![
            shadcn::Badge::new("Caller-owned")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            ui::text("Updated 2m ago")
                .text_xs()
                .text_color(ColorRef::Color(muted_fg))
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-hover-card-children-demo-meta");

    let content = shadcn::HoverCardContent::new([title, summary, meta])
        .test_id("ui-gallery-hover-card-children-demo-content")
        .refine_layout(LayoutRefinement::default().max_w(Px(288.0)));

    shadcn::HoverCard::new(
        cx,
        shadcn::Button::new("Composable content")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-children-demo-trigger"),
        content,
    )
    .open_delay_frames(8)
    .close_delay_frames(8)
    .into_element(cx)
    .test_id("ui-gallery-hover-card-children-demo")
}
// endregion: example
