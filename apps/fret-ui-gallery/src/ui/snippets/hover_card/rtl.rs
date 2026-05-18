pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let body = ui::v_flex(move |cx| {
            vec![decl_text::text_paragraph_break_words(
                cx,
                "تحقق من محاذاة HoverCard تحت RTL.",
            )]
        })
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N1)
        .items_stretch()
        .into_element(cx);

        let content = shadcn::HoverCardContent::build(cx, |_cx| [body])
            .test_id("ui-gallery-hover-card-rtl-content")
            .side(shadcn::HoverCardSide::Left);

        shadcn::HoverCard::new(
            cx,
            shadcn::Button::new("مرر هنا")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-rtl-trigger"),
            content,
        )
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
    })
    .test_id("ui-gallery-hover-card-rtl")
}
// endregion: example
