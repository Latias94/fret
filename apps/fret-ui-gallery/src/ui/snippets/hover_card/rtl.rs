// region: example
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1)
                .items_start(),
            move |cx| {
                vec![
                    ui::text(cx, "نموذج RTL").font_semibold().into_element(cx),
                    ui::text(cx, "تحقق من محاذاة HoverCard تحت RTL.")
                        .wrap(TextWrap::WordBreak)
                        .text_color(ColorRef::Color(muted_fg))
                        .into_element(cx),
                ]
            },
        );

        shadcn::HoverCard::new(
            shadcn::Button::new("مرر هنا")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-rtl-trigger")
                .into_element(cx),
            shadcn::HoverCardContent::new(vec![body])
                .into_element(cx)
                .test_id("ui-gallery-hover-card-rtl-content"),
        )
        .open_delay_frames(10)
        .close_delay_frames(10)
        .side(shadcn::HoverCardSide::Left)
        .into_element(cx)
    })
    .test_id("ui-gallery-hover-card-rtl")
}
// endregion: example

